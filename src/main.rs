//! chat server
use async_std::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::Arc,
    task::{self, TaskId},
};
use futures_util::{
    stream::StreamExt,
    sink::SinkExt,
};
use futures_channel::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use std::{
    error::Error,
    env::args,
    result,
};

type Result<T> = result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
type Sender<T> = UnboundedSender<T>;
type Receiver<T> = UnboundedReceiver<T>;

const ADDR: &str = "[::1]:8000";

fn main() -> Result<()> {
    let mut args = args().skip(1);
    let addr = args
        .next()
        .map(|addr| addr.parse().unwrap_or_else(|_| ADDR.to_string()))
        .unwrap_or_else(|| ADDR.to_string());
    task::block_on(listener(addr))
}

#[derive(Debug)]
enum Event {
    Join(TaskId, Arc<TcpStream>),
    Leave(TaskId),
    Message(TaskId, String),
}

async fn listener<A: ToSocketAddrs>(addr: A) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut readers = vec![];
    let (tx, rx) = unbounded();
    let broker = task::spawn(broker(rx));

    while let Some(stream) = listener.incoming().next().await {
        match stream {
            Err(err) => {
                eprintln!("accept error: {}", err);
            }
            Ok(stream) => {
                readers.push(task::spawn(reader(tx.clone(), stream)));
            }
        }
    }
    while let Some(reader) = readers.pop() {
        reader.await?;
    }
    drop(tx);
    broker.await
}

async fn broker(mut readers: Receiver<Event>) -> Result<()> {
    while let Some(event) = readers.next().await {
        eprintln!("{:?}", event);
    }
    Ok(())
}

async fn reader(mut broker: Sender<Event>, stream: TcpStream) -> Result<()> {
    let id = task::current().id();
    let stream = Arc::new(stream);
    eprintln!("peer={:?}", stream.peer_addr().unwrap());
    broker.send(Event::Join(id, Arc::clone(&stream))).await?;
    broker.send(Event::Leave(id)).await?;
    Ok(())
}

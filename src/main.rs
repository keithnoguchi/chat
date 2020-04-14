//! Async chat server
use std::{
    env, error,
    sync::Arc,
};

use async_std::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::{self, TaskId},
};
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};

enum Event {
    Join(TaskId, Arc<TcpStream>),
    Leave(TaskId),
    Message(TaskId, String),
}

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
type Result<T> = std::result::Result<T, Box<dyn error::Error + Send + Sync + 'static>>;

static ADDR: &str = "localhost:8000";

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let addr = args
        .next()
        .map(|addr| addr.parse().unwrap_or_else(|_| ADDR.to_string()))
        .unwrap_or_else(|| ADDR.to_string());
    task::block_on(server(addr))
}

async fn server<A: ToSocketAddrs>(addr: A) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    eprintln!("listening on {:?}", listener.local_addr()?);
    let (tx, rx) = mpsc::unbounded();
    task::spawn(broker(rx));
    while let Some(s) = listener.incoming().next().await {
        match s {
            Err(err) => eprintln!("accept error: {}", err),
            Ok(s) => {
                task::spawn(reader(tx.clone(), s));
            }
        }
    }
    Ok(())
}

async fn broker(mut reader: Receiver<Event>) -> Result<()> {
    while let Some(event) = reader.next().await {
        match event {
            Event::Join(id, s) => eprintln!("[{:?}] joined: {:?}", id, s.peer_addr()?),
            Event::Leave(id) => eprintln!("[{:?}] left", id),
            Event::Message(id, msg) => eprintln!("[{:?}] {}", id, msg),
        }
    }
    Ok(())
}

async fn reader(mut broker: Sender<Event>, s: TcpStream) -> Result<()> {
    let id = task::current().id();
    let s = Arc::new(s);
    broker.send(Event::Join(id, Arc::clone(&s))).await?;
    broker.send(Event::Leave(id)).await?;
    Ok(())
}

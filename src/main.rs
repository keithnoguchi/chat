//! chat server
use async_std::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::Arc,
    task::{self, TaskId},
};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sink::SinkExt,
    stream::StreamExt,
};
use std::{collections::HashMap, env::args, error::Error, result};

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
    let mut peers = HashMap::new();
    let mut writers = vec![];
    while let Some(event) = readers.next().await {
        match event {
            Event::Join(id, stream) => {
                let (tx, rx) = unbounded();
                if let Some(old) = peers.insert(id, tx) {
                    drop(old);
                }
                writers.push(task::spawn(writer(rx, stream)));
            }
            Event::Leave(id) => {
                if let Some(tx) = peers.remove(&id) {
                    drop(tx);
                }
            }
            Event::Message(id, msg) => {
                if let Some(_) = peers.get(&id) {
                    let msg = format!("client{}> {}\n", id, msg);
                    for (peer_id, mut tx) in &peers {
                        if peer_id != &id {
                            tx.send(msg.clone()).await?;
                        }
                    }
                }
            }
        }
    }
    for writer in peers.values() {
        drop(writer);
    }
    while let Some(writer) = writers.pop() {
        writer.await?;
    }
    Ok(())
}

async fn reader(mut broker: Sender<Event>, stream: TcpStream) -> Result<()> {
    let id = task::current().id();
    let stream = Arc::new(stream);
    broker.send(Event::Join(id, Arc::clone(&stream))).await?;
    let mut reader = BufReader::new(&*stream).lines();
    while let Some(line) = reader.next().await {
        match line {
            Ok(line) => broker.send(Event::Message(id, line)).await?,
            Err(_) => break,
        }
    }
    broker.send(Event::Leave(id)).await?;
    Ok(())
}

async fn writer(mut broker: Receiver<String>, stream: Arc<TcpStream>) -> Result<()> {
    while let Some(msg) = broker.next().await {
        (&*stream).write_all(msg.as_bytes()).await?;
    }
    Ok(())
}

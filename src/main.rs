//! Async chat application
use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::{self, TaskId},
};
use futures::channel::mpsc;
use futures_util::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    sink::SinkExt,
    stream::StreamExt,
};
use std::{collections::HashMap, env::args, sync::Arc};

#[derive(Debug)]
enum Event {
    Join(TaskId, Arc<TcpStream>),
    Leave(TaskId),
    Message(TaskId, String),
}

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send + 'static>>;

const ADDR: &str = "[::1]:8000";

fn main() -> Result<()> {
    let mut args = args().skip(1);
    let addr = args
        .next()
        .map(|addr| addr.parse().unwrap_or_else(|_| ADDR.to_string()))
        .unwrap_or_else(|| ADDR.to_string());
    task::block_on(server(addr))
}

async fn server<A: ToSocketAddrs>(addr: A) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("addr={:?}", listener.local_addr());
    let mut readers = Vec::new();
    let (tx, rx) = mpsc::unbounded();
    task::spawn(broker(rx));
    while let Some(s) = listener.incoming().next().await {
        match s {
            Err(err) => {
                eprintln!("accept error: {}", err);
            }
            Ok(s) => readers.push(task::spawn(reader(tx.clone(), s))),
        }
    }
    while let Some(reader) = readers.pop() {
        reader.await?;
    }
    Ok(())
}

async fn broker(mut reader: Receiver<Event>) -> Result<()> {
    let mut peers = HashMap::new();
    eprintln!("broker started");
    while let Some(event) = reader.next().await {
        match event {
            Event::Join(id, stream) => {
                let (tx, rx) = mpsc::unbounded();
                if let Some(old) = peers.insert(id, tx) {
                    old.close_channel();
                }
                task::spawn(writer(rx, stream));
            }
            Event::Leave(id) => {
                if let Some(writer) = peers.remove(&id) {
                    writer.close_channel();
                }
            }
            Event::Message(id, msg) => {
                let msg = format!("client{}> {}\n", id, msg);
                for (peer_id, mut writer) in &peers {
                    if peer_id != &id {
                        if let Err(err) = writer.send(msg.clone()).await {
                            eprintln!("send error to {}: {}", peer_id, err);
                        }
                    }
                }
            }
        }
    }
    eprintln!("broker done");
    Ok(())
}

async fn writer(mut broker: Receiver<String>, s: Arc<TcpStream>) -> Result<()> {
    while let Some(msg) = broker.next().await {
        if let Err(err) = (&*s).write_all(msg.as_bytes()).await {
            broker.close();
            Err(err)?;
        }
    }
    broker.close();
    Ok(())
}

async fn reader(mut broker: Sender<Event>, s: TcpStream) -> Result<()> {
    let s = Arc::new(s);
    let id = task::current().id();
    broker.send(Event::Join(id, Arc::clone(&s))).await?;
    let mut lines = BufReader::new(&*s).lines();
    while let Some(line) = lines.next().await {
        let line = line?;
        broker.send(Event::Message(id, line)).await?;
    }
    broker.send(Event::Leave(id)).await?;
    Ok(())
}

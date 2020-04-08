//! Async chat server
use std::{collections::HashMap, error::Error, sync::Arc};

use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    stream::StreamExt,
    task::{self, TaskId},
};
use futures::{
    channel::mpsc,
    io::{AsyncBufReadExt, AsyncWriteExt},
    sink::SinkExt,
};

enum Event {
    Join(TaskId, Arc<TcpStream>),
    Leave(TaskId),
    Message(TaskId, String),
}

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let addr = args
        .next()
        .map(|addr| addr.parse().unwrap_or("[::1]:8000".to_string()))
        .unwrap_or("[::1]:8000".to_string());
    task::block_on(server(addr))
}

async fn server<A: ToSocketAddrs>(addr: A) -> Result<()> {
    let server = TcpListener::bind(addr).await?;
    println!("{:?}", server.local_addr().unwrap());
    let (tx, rx) = mpsc::unbounded();
    task::spawn(broker(rx));
    while let Some(s) = server.incoming().next().await {
        let s = s?;
        task::spawn(reader(tx.clone(), s));
    }
    Ok(())
}

async fn broker(mut readers: Receiver<Event>) -> Result<()> {
    let mut writers = HashMap::new();
    while let Some(event) = readers.next().await {
        match event {
            Event::Join(id, client) => {
                let (tx, rx) = mpsc::unbounded();
                writers.insert(id, tx);
                task::spawn(writer(rx, client));
            }
            Event::Leave(id) => {
                if let Some(tx) = writers.remove(&id) {
                    drop(tx);
                }
            }
            Event::Message(id, msg) => {
                let msg = format!("CLIENT{}> {}\n", id, msg);
                for (writer_id, mut writer) in &writers {
                    if writer_id != &id {
                        writer.send(msg.clone()).await?;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn reader(mut broker: Sender<Event>, s: TcpStream) -> Result<()> {
    let id = task::current().id();
    println!("{:?}: {:?}", id, s.peer_addr());
    let s = Arc::new(s);
    broker.send(Event::Join(id, Arc::clone(&s))).await?;
    let mut reader = BufReader::new(&*s).lines();
    while let Some(line) = reader.next().await {
        let msg = line?;
        broker.send(Event::Message(id, msg)).await?;
    }
    broker.send(Event::Leave(id)).await?;
    Ok(())
}

async fn writer(mut broker: Receiver<String>, s: Arc<TcpStream>) -> Result<()> {
    while let Some(msg) = broker.next().await {
        (&*s).write_all(msg.as_bytes()).await?;
    }
    Ok(())
}

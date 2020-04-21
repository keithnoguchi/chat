//! Async chat application
use std::{
    env::args,
    sync::Arc,
};

use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task,
};
use futures::channel::mpsc;
use futures_util::{
    io::AsyncBufReadExt,
    stream::StreamExt,
};

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

async fn broker(mut reader: Receiver<String>) -> Result<()> {
    eprintln!("broker started");
    while let Some(event) = reader.next().await {
    }
    eprintln!("broker done");
    Ok(())
}

async fn reader(mut broker: Sender<String>, s: TcpStream) -> Result<()> {
    eprintln!("reader: {:?}", s.peer_addr());
    let s = Arc::new(s);
    let mut lines = BufReader::new(&*s).lines();
    while let Some(line) = lines.next().await {
        eprintln!("{:?}", line);
    }
    eprintln!("reader: done");
    Ok(())
}

//! Async chat application
use std::env::args;

use async_std::{
    net::{TcpListener, ToSocketAddrs},
    task,
};
use futures::{
    channel::mpsc,
    stream::StreamExt,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send + 'static>>;
type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

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
    let (tx, rx) = mpsc::unbounded();
    task::spawn(broker(rx));
    while let Some(s) = listener.incoming().next().await {
        match s {
            Err(err) => {
                eprintln!("accept error: {}", err);
            }
            Ok(s) => {
                println!("remote addr={:?}", s.peer_addr());
            }
        }
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

//! Async chat server
use std::{env, error};

use async_std::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task,
};
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};

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

async fn broker(mut reader: Receiver<String>) -> Result<()> {
    while let Some(req) = reader.next().await {
        eprintln!("request from {}", req);
    }
    Ok(())
}

async fn reader(mut broker: Sender<String>, s: TcpStream) -> Result<()> {
    broker.send(format!("{:?}", s.peer_addr()?)).await?;
    Ok(())
}

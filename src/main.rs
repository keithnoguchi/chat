//! chat server
use async_std::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task,
};
use futures_util::stream::StreamExt;
use std::{
    error::Error,
    env::args,
    result,
};

type Result<T> = result::Result<T, Box<dyn Error + Send + Sync + 'static>>;

const ADDR: &str = "[::1]:8000";

fn main() -> Result<()> {
    let mut args = args().skip(1);
    let addr = args
        .next()
        .map(|addr| addr.parse().unwrap_or_else(|_| ADDR.to_string()))
        .unwrap_or_else(|| ADDR.to_string());
    task::block_on(listener(addr))
}

async fn listener<A: ToSocketAddrs>(addr: A) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut readers = vec![];

    while let Some(stream) = listener.incoming().next().await {
        match stream {
            Err(err) => {
                eprintln!("accept error: {}", err);
            }
            Ok(stream) => {
                readers.push(task::spawn(reader(stream)));
            }
        }
    }
    while let Some(reader) = readers.pop() {
        reader.await?;
    }
    Ok(())
}

async fn reader(stream: TcpStream) -> Result<()> {
    eprintln!("peer={:?}", stream.peer_addr().unwrap());
    Ok(())
}

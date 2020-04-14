//! Async chat server
use std::{
    env,
    error,
};

use async_std::{
    net::{TcpListener, ToSocketAddrs},
    task,
};
use futures::stream::StreamExt;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

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
    println!("{:?}", listener.local_addr()?);
    while let Some(s) = listener.incoming().next().await {
        let s = s?;
        println!("{:?}", s.peer_addr()?);
    }
    Ok(())
}

use std::error::Error;

use async_std::{
    net::{ToSocketAddrs, TcpListener, TcpStream},
    task,
    stream::StreamExt,
};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;

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
    while let Some(s) = server.incoming().next().await {
        let s = s?;
        task::spawn(client(s));
    }
    Ok(())
}

async fn client(s: TcpStream) -> Result<()> {
    println!("{:?}", s.peer_addr());
    Ok(())
}

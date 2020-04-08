use std::error::Error;

use async_std::{
    net::{ToSocketAddrs, TcpListener, TcpStream},
    task,
    stream::StreamExt,
};
use futures::channel::mpsc;

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
        task::spawn(client(tx.clone(), s));
    }
    Ok(())
}

async fn broker(_clients: Receiver<String>) -> Result<()> {
    println!("BROKER");
    Ok(())
}

async fn client(_broker: Sender<String>, s: TcpStream) -> Result<()> {
    println!("CLIENT: {:?}", s.peer_addr());
    Ok(())
}

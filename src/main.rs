use std::{
    error::Error,
    sync::Arc,
};

use async_std::{
    io::BufReader,
    net::{ToSocketAddrs, TcpListener, TcpStream},
    task,
    stream::StreamExt,
};
use futures::{
    channel::mpsc,
    io::AsyncBufReadExt,
    sink::SinkExt,
};

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

async fn broker(mut clients: Receiver<String>) -> Result<()> {
    println!("BROKER");
    while let Some(msg) = clients.next().await {
        println!("{}", msg);
    }
    Ok(())
}

async fn client(mut broker: Sender<String>, s: TcpStream) -> Result<()> {
    println!("CLIENT: {:?}", s.peer_addr());
    let s = Arc::new(s);
    let mut reader = BufReader::new(&*s).lines();
    while let Some(line) = reader.next().await {
        let msg = line?;
        broker.send(msg).await?;
    }
    Ok(())
}

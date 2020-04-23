//! chat server
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
    println!("addr={}", addr);
    Ok(())
}

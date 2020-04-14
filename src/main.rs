//! Async chat server
use std::{
    env,
    error,
};

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

static ADDR: &str = "localhost:8000";

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let addr = args
        .next()
        .map(|addr| addr.parse().unwrap_or_else(|_| ADDR.to_string()))
        .unwrap_or_else(|| ADDR.to_string());
    println!("{}", addr);
    Ok(())
}

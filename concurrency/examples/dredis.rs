use std::net::SocketAddr;

use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tracing::{info, warn};

const BUF_SIZE: usize = 4096;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:6379";
    let listener = TcpListener::bind(addr).await?;
    info!("Dredis: listening on {}", addr);

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);
        tokio::spawn(async move {
            if let Err(e) = process_redis_conn(stream).await {
                warn!("Error processing conn with {}: {:?}", raddr, e);
            }
        });
    }
    #[allow(unreachable_code)]
    Ok(())
}

async fn process_redis_conn(mut stream: tokio::net::TcpStream, raddr: SocketAddr) -> Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);
        match stream.try_read_buf(&mut buf) {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                info!("Read {} bytes", n);
                let line = String::from_utf8_lossy(&buf);
                info!("{:?}", line);
                stream.write_all(b"+OK\r\n").await?;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                info!("Error reading from socket: {:?}", e);
                return Err(e.into());
            }
        }
    }
    warn!("Closing connection with: {}", raddr);
    Ok(())
}

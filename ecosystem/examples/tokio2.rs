use std::thread;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(32);
    let handler = worker(rx);
    
    let _ = tokio::spawn(async move {
        let mut i = 0;
        loop {
            i += 1;
            println!("sending {}", i);
            tx.send(format!("task {i}")).await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    }).await.unwrap();
    
    handler.join().unwrap();
    
    Ok(())
}

fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(s) = rx.blocking_recv() {
            let ret = expensive_blocking_task(s);
            println!("result: {}", ret);
        }
    })
}

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}

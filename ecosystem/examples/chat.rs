use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use futures::{SinkExt, stream::SplitStream, StreamExt};
use tokio::{net::{TcpListener, TcpStream},
            sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, warn};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, Layer as _, layer::SubscriberExt, util::SubscriberInitExt};

const MAX_MESSAGES: usize = 128;

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr,  mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[derive(Debug)]
enum Message {
    UserJoin(String),
    UserLeave(String),
    Chat{
        sender: String,
        content: String,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    console_subscriber::init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Starting chat server on {}", addr);
    
    let state = Arc::new(State::default());
    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("accepted connection from: {}", raddr);
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) =  handle_client(state, raddr, stream).await {
                warn!("error handling connection from {:?}: {:?}", raddr, e);
            }
            Ok::<(), anyhow::Error>(())
        });
    }
    #[allow(unreachable_code)]
    Ok(())
}

async fn handle_client(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username: ").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer = state.add(addr, username, stream).await;
    let message = Arc::new(Message::user_join(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("error reading from {:?}: {:?}", addr, e);
                break;
            }
        };
        let message = Arc::new(Message::chat(&peer.username, line));
        state.broadcast(addr, message).await;
    }

    state.peers.remove(&addr);
    let message = Arc::new(Message::user_leave(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;
    Ok(())
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }
            if let Err(e) = peer.value().send(message.clone()).await {
                warn!("error sending message to {:?}: {:?}", peer.key(), e);
                // remove peer from the list
                self.peers.remove(peer.key());
            }
        }
    }

    async fn add(&self, addr: SocketAddr, username: String, stream: Framed<TcpStream, LinesCodec>) -> Peer{
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);
        self.peers.insert(addr, tx);

        let (mut stream_sender, stream_received) = stream.split();
        // receive messages from others, and send them to client
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("error sending message to {:?}: {:?}", addr, e);
                    break;
                }
            }
        });
        Peer {
            username,
            stream: stream_received,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserJoin(content) => write!(f, "{}", content),
            Self::UserLeave(content) => write!(f, "{}", content),
            Self::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}

impl Message {
    fn user_join(username: impl Into<String>) -> Self {
        let content = format!("{} joined the chat", username.into());
        Self::UserJoin(content)
    }

    fn user_leave(username: impl Into<String>) -> Self {
        let content = format!("{} left the chat", username.into());
        Self::UserLeave(content)
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}
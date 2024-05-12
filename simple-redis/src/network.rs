use anyhow::Result;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::info;

use crate::{
    Backend,
    cmd::{Command, CommandExecutor}, RespDecode, RespEncode, RespError, RespFrame, SimpleString,
};
use crate::cmd::CommandError;

#[derive(Debug)]
struct RespFrameCodec;

#[derive(Debug)]
struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}

#[derive(Debug)]
struct RedisResponse {
    frame: RespFrame,
}

pub async fn stream_handler(stream: TcpStream, backend: Backend) -> Result<()> {
    let mut framed = Framed::new(stream, RespFrameCodec);
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);
                let request = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                match requst_handler(request).await {
                    Ok(response) => {
                        info!("Sending response: {:?}", response.frame);
                        framed.send(response.frame).await?;
                    }
                    Err(e) => {
                        if let Some(CommandError::InvalidArgument(_)) =
                            e.downcast_ref::<CommandError>()
                        {
                            let response = RedisResponse {
                                frame: RespFrame::SimpleString(SimpleString::new(
                                    "Invalid argument",
                                )),
                            };
                            framed.send(response.frame).await?;
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(()),
        }
    }
}

async fn requst_handler(request: RedisRequest) -> Result<RedisResponse> {
    let (frame, backend) = (request.frame, request.backend);
    let cmd = Command::try_from(frame)?;
    info!("Executing command: {:?}", cmd);
    let ret = cmd.execute(&backend);
    Ok(RedisResponse { frame: ret })
}

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut bytes::BytesMut) -> Result<()> {
        let encoded = item.encode();
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}

impl Decoder for RespFrameCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotComplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

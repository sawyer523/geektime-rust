use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

use crate::{Backend, Array, RespError, RespFrame};

mod echo;
mod hmap;
mod map;

lazy_static! {
    static ref RESP_OK: RespFrame = RespFrame::SimpleString("OK".into());
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("{0}")]
    RespError(#[from] RespError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HMGet(HMGet),
    HMSet(HMSet),
    HGetAll(HGetAll),
    Echo(Echo),

    // unrecognized command
    Unrecognized(Unrecognized),
}

#[derive(Debug)]
pub struct Get {
    key: String,
}

#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGet {
    key: String,
    field: String,
}

#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAll {
    key: String,
    sort: bool,
}

#[derive(Debug)]
pub struct HMSet {
    key: String,
    fields: Array,
}

#[derive(Debug)]
pub struct HMGet {
    key: String,
    fields: Array,
}

#[derive(Debug)]
pub struct Echo {
    value: String,
}

#[derive(Debug)]
pub struct Unrecognized;

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(v: RespFrame) -> Result<Self, Self::Error> {
        match v {
            RespFrame::Array(array) => array.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "Command must be an Array".to_string(),
            )),
        }
    }
}

impl TryFrom<Array> for Command {
    type Error = CommandError;
    fn try_from(v: Array) -> Result<Self, Self::Error> {
        match v.0 {
            Some(ref data) => {
                if data.is_empty() {
                    return Err(CommandError::InvalidCommand(
                        "Command must have at least one argument".to_string(),
                    ));
                }

                match data.first() {
                    Some(RespFrame::BulkString(ref cmd)) => match cmd.as_ref() {
                        b"get" => Ok(Get::try_from(v)?.into()),
                        b"set" => Ok(Set::try_from(v)?.into()),
                        b"hget" => Ok(HGet::try_from(v)?.into()),
                        b"hset" => Ok(HSet::try_from(v)?.into()),
                        b"hgetall" => Ok(HGetAll::try_from(v)?.into()),
                        b"echo" => Ok(Echo::try_from(v)?.into()),
                        b"hmset" => Ok(HMSet::try_from(v)?.into()),
                        b"hmget" => Ok(HMGet::try_from(v)?.into()),
                        _ => Ok(Unrecognized.into()),
                    },
                    _ => Err(CommandError::InvalidCommand(
                        "Command requires a BulkString as the first argument".to_string(),
                    )),
                }
            }
            None => Err(CommandError::InvalidCommand(
                "Command must have at least one argument".to_string(),
            )),
        }
    }
}

impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

fn validate_command(
    value: &Array,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    match value.0.as_ref() {
        Some(data) => {
            if data.len() != n_args + names.len() {
                return Err(CommandError::InvalidArgument(format!(
                    "{} command must have exactly {} arguments",
                    names.join(" "),
                    n_args,
                )));
            }

            for (i, name) in names.iter().enumerate() {
                match data[i] {
                    RespFrame::BulkString(ref cmd) => {
                        if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
                            return Err(CommandError::InvalidCommand(format!(
                                "Invalid command: expected {}, got {}",
                                name,
                                String::from_utf8_lossy(cmd.as_ref())
                            )));
                        }
                    }
                    _ => {
                        return Err(CommandError::InvalidCommand(
                            "Command requires a BulkString as the first argument".to_string(),
                        ));
                    }
                }
            }
            return Ok(());
        }
        None => {
            return Err(CommandError::InvalidArgument(format!(
                "{} command requires {} arguments",
                names.join(" "),
                n_args
            )));
        }
    }
}

fn extract_args(value: Array, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value
        .0
        .unwrap()
        .into_iter()
        .skip(start)
        .collect::<Vec<RespFrame>>())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::{RespDecode, Null};

    use super::*;

    #[test]
    fn test_command() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = Array::decode(&mut buf)?;

        let cmd: Command = frame.try_into()?;

        let backend = Backend::new();

        let ret = cmd.execute(&backend);
        assert_eq!(ret, RespFrame::Null(Null));

        Ok(())
    }
}

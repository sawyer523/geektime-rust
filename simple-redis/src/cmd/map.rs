use crate::{Backend, cmd::Get, Array, RespFrame, Null};
use crate::cmd::{CommandError, CommandExecutor, extract_args, RESP_OK, Set, validate_command};

impl CommandExecutor for Get {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend
            .get(&self.key)
            .unwrap_or_else(|| RespFrame::Null(Null))
    }
}

impl CommandExecutor for Set {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.set(self.key.clone(), self.value.clone());
        RESP_OK.clone()
    }
}

impl TryFrom<Array> for Get {
    type Error = CommandError;
    fn try_from(value: Array) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(s)) => Ok(Get {
                key: String::from_utf8_lossy(s.as_ref()).to_string(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<Array> for Set {
    type Error = CommandError;
    fn try_from(value: Array) -> Result<Self, Self::Error> {
        validate_command(&value, &["set"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(k)), Some(v)) => Ok(Set {
                key: String::from_utf8_lossy(k.as_ref()).to_string(),
                value: v,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::cmd::Get;
    use crate::RespDecode;

    use super::*;

    #[test]
    fn test_set_get_command() -> Result<()> {
        let backend = Backend::new();
        let cmd = Set {
            key: "hello".to_string(),
            value: RespFrame::BulkString(b"world".into()),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let get = Get {
            key: "hello".to_string(),
        };
        let result = get.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"world".into()));
        Ok(())
    }

    #[test]
    fn test_get_try_from_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$3\r\nkey\r\n");
        let frame = Array::decode(&mut buf)?;
        let resutl = Get::try_from(frame)?;
        assert_eq!(resutl.key, "key");
        Ok(())
    }

    #[test]
    fn test_set_try_from_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$3\r\nkey\r\n$5\r\nvalue\r\n");
        let frame = Array::decode(&mut buf)?;
        let result = Set::try_from(frame)?;
        assert_eq!(result.key, "key");
        assert_eq!(result.value, RespFrame::BulkString(b"value".into()));
        Ok(())
    }
}

use crate::{Backend, Array, RespFrame, SimpleString};
use crate::cmd::{CommandError, CommandExecutor, Echo, extract_args, validate_command};

impl CommandExecutor for Echo {
    fn execute(self, _backend: &Backend) -> RespFrame {
        SimpleString::new(format!("\"{}\"", self.value)).into()
    }
}

impl TryFrom<Array> for Echo {
    type Error = CommandError;
    fn try_from(value: Array) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(s)) => Ok(Echo {
                value: String::from_utf8_lossy(s.as_ref()).to_string(),
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

    use crate::RespDecode;

    use super::*;

    // test the echo command
    #[test]
    fn test_echo() -> Result<()> {
        use super::*;
        use crate::RespFrame;

        let echo = Echo {
            value: "hello".to_string(),
        };

        let frame = echo.execute(&Backend::default());
        assert_eq!(
            frame,
            RespFrame::SimpleString(SimpleString::new("\"hello\""))
        );

        Ok(())
    }

    // test decode
    #[test]
    fn test_echo_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");

        let frame = Array::decode(&mut buf)?;

        let cmd: Echo = frame.try_into()?;

        assert_eq!(cmd.value, "hello");

        Ok(())
    }
}

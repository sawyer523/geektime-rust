use crate::{Backend, RespArray, RespFrame, SimpleString};
use crate::cmd::{CommandError, CommandExecutor, Echo, extract_args, validate_command};

impl CommandExecutor for Echo {
    fn execute(self, _backend: &Backend) -> RespFrame {
        SimpleString::new(format!("\"{}\"", self.value)).into()
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
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

    #[test]
    fn test_echo_encode() {
        // let frame: RespFrame = Echo::try_from("Hello, world!".to_string()).into();
        //
        // assert_eq!(frame.encode(), b"+Hello, world!\r\n");
    }

    #[test]
    fn test_echo_decode() -> Result<()> {
        // let mut buf = BytesMut::from("+Hello, world!\r\n");
        // let frame = Echo::decode(&mut buf)?;
        //
        // assert_eq!(frame.value, "Hello, world!");

        Ok(())
    }
}

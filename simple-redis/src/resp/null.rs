use bytes::BytesMut;

use crate::{RespDecode, RespEncode, RespError};

use super::extract_fixed_data;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Null;

// - null: "_\r\n"
impl RespEncode for Null {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespDecode for Null {
    const PREFIX: &'static str = "_";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "_\r\n", "Null")?;
        Ok(Null)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(3)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_null_encode() {
        let frame: RespFrame = Null.into();
        assert_eq!(frame.encode(), b"_\r\n");
    }

    #[test]
    fn test_null_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"_\r\n");

        let frame = Null::decode(&mut buf)?;
        assert_eq!(frame, Null);

        Ok(())
    }
}

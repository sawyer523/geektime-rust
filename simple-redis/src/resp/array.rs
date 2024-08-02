use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError, RespFrame};

use super::{BUF_CAP, calc_total_length, CRLF_LEN, parse_length};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Array(pub(crate) Option<Vec<RespFrame>>);

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for Array {
    fn encode(self) -> Vec<u8> {
        match self.0 {
            Some(data) => {
                let mut buf = Vec::with_capacity(BUF_CAP);
                buf.extend_from_slice(&format!("*{}\r\n", data.len()).into_bytes());
                for frame in data {
                    buf.extend_from_slice(&frame.encode());
                }
                buf
            }
            None => b"*-1\r\n".to_vec(),
        }
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
// - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
// FIXME: need to handle incomplete
impl RespDecode for Array {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.starts_with(b"*-1\r\n") {
            return Ok(Array::none());
        }

        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }

        Ok(Array::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        if buf.starts_with(b"$-1\r\n") {
            return Ok(5);
        }
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl Array {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        Array(Some(s.into()))
    }

    pub fn none() -> Self {
        Array(None)
    }
}

impl Deref for Array {
    type Target = Option<Vec<RespFrame>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<RespFrame>> for Array {
    fn from(s: Vec<RespFrame>) -> Self {
        if s.is_empty() {
            Array::none()
        } else {
            Array::new(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, RespFrame};

    use super::*;

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = Array::new(vec![
            BulkString::new("set".to_string()).into(),
            BulkString::new("hello".to_string()).into(),
            BulkString::new("world".to_string()).into(),
        ])
        .into();
        assert_eq!(
            &frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );

        let frame: RespFrame = Array::none().into();
        assert_eq!(&frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = Array::decode(&mut buf)?;
        assert_eq!(frame, Array::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = Array::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = Array::decode(&mut buf)?;
        assert_eq!(frame, Array::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*-1\r\n");
        let frame = Array::decode(&mut buf)?;
        assert_eq!(frame, Array::none());

        Ok(())
    }
}

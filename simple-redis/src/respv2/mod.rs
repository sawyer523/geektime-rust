use bytes::BytesMut;

use crate::{RespError, RespFrame};
pub use crate::respv2::parser::{parse_frame, parse_frame_length};

mod parser;

pub trait RespDecodeV2: Sized {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

impl RespDecodeV2 for RespFrame {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let len = Self::expect_length(buf)?;
        let data = buf.split_to(len);

        parse_frame(&mut data.as_ref()).map_err(|e| RespError::InvalidFrame(e.to_string()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        parse_frame_length(buf)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{Array, BulkString, Null, Set, SimpleError, SimpleString};

    use super::*;

    #[test]
    fn respv2_simple_string_length_should_work() {
        let buf = b"+OK\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, 5);
    }

    #[test]
    fn respv2_simple_string_length_should_fail() {
        let buf = b"+OK\r";
        let err = RespFrame::expect_length(buf).unwrap_err();
        assert_eq!(err, RespError::NotComplete);
    }

    #[test]
    fn respv2_simple_string_should_work() {
        let mut buf = BytesMut::from("+OK\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(
            frame,
            RespFrame::SimpleString(SimpleString::new("OK".to_string()))
        );
    }

    #[test]
    fn respv2_simple_error_length_should_work() {
        let buf = b"-ERR\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, 6);
    }

    #[test]
    fn respv2_simple_error_length_should_fail() {
        let buf = b"-ERR\r";
        let err = RespFrame::expect_length(buf).unwrap_err();
        assert_eq!(err, RespError::NotComplete);
    }

    #[test]
    fn respv2_simple_error_should_work() {
        let mut buf = BytesMut::from("-ERR\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(frame, RespFrame::Error(SimpleError::new("ERR".to_string())));
    }

    #[test]
    fn respv2_integer_length_should_work() {
        let buf = b":1234\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, 7);
    }

    #[test]
    fn respv2_integer_length_should_fail() {
        let buf = b":1234\r";
        let err = RespFrame::expect_length(buf).unwrap_err();
        assert_eq!(err, RespError::NotComplete);
    }

    #[test]
    fn respv2_integer_should_work() {
        let mut buf = BytesMut::from(":1234\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(frame, RespFrame::Integer(1234));

        let mut buf = BytesMut::from(":-1234\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(frame, RespFrame::Integer(-1234));
    }

    #[test]
    fn respv2_bulk_string_length_should_work() {
        let buf = b"$5\r\nhello\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, 11);
    }

    #[test]
    fn respv2_bulk_string_length_should_fail() {
        let buf = b"$5\r\nhello";
        let err = RespFrame::expect_length(buf).unwrap_err();
        assert_eq!(err, RespError::NotComplete);
    }

    #[test]
    fn respv2_bulk_string_should_work() {
        let mut buf = BytesMut::from("$5\r\nhello\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(
            frame,
            RespFrame::BulkString(BulkString::new(b"hello".to_vec()))
        );

        let mut buf = BytesMut::from("$0\r\n\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(frame, RespFrame::BulkString(BulkString::new(b"".to_vec())));

        let mut buf = BytesMut::from("$-1\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(frame, RespFrame::BulkString(BulkString::none()));
    }

    #[test]
    fn respv2_array_length_should_work() {
        let buf = b"*2\r\n+OK\r\n+ERR\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, 15);
    }

    #[test]
    fn respv2_array_length_should_fail() {
        let buf = b"*2\r\n+OK\r\n+ERR";
        let err = RespFrame::expect_length(buf).unwrap_err();
        assert_eq!(err, RespError::NotComplete);
    }

    #[test]
    fn respv2_array_should_work() {
        let mut buf = BytesMut::from("*2\r\n+OK\r\n+FAIL\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(
            frame,
            RespFrame::Array(Array::new(vec![
                SimpleString::new("OK".to_string()).into(),
                SimpleString::new("FAIL".to_string()).into()
            ]))
        );
    }

    #[test]
    fn respv2_null_length_should_work() {
        let buf = b"_\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, 3);
    }

    #[test]
    fn respv2_null_should_work() {
        let mut buf = BytesMut::from("_\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(frame, RespFrame::Null(Null));
    }

    #[test]
    fn respv2_boolean_length_should_work() {
        let buf = b"#t\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, 4);

        let buf = b"#f\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, 4);
    }

    #[test]
    fn respv2_double_length_should_work() {
        let buf = b",1.23\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, 7);
    }

    #[test]
    fn respv2_double_should_work() {
        let mut buf = BytesMut::from(",1.23\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(frame, RespFrame::Double(1.23));
    }

    #[test]
    fn respv2_map_length_should_work() {
        let buf = b"%1\r\n+OK\r\n-ERR\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, buf.len());
    }

    #[test]
    fn respv2_map_should_work() {
        let mut buf = BytesMut::from("%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        let items: BTreeMap<String, RespFrame> = [
            ("hello".to_string(), RespFrame::BulkString("world".into())),
            ("foo".to_string(), RespFrame::BulkString("bar".into())),
        ]
        .into_iter()
        .collect();
        assert_eq!(frame, RespFrame::Map(items.into()));
    }

    #[test]
    fn respv2_set_length_should_work() {
        let buf = BytesMut::from("~4\r\n$3\r\nset\r\n$5\r\nhello\r\n$3\r\nget\r\n$5\r\nworld\r\n");
        let len = RespFrame::expect_length(&buf).unwrap();
        assert_eq!(len, buf.len());
    }

    #[test]
    fn respv2_set_should_work() {
        let mut buf =
            BytesMut::from("~4\r\n$3\r\nset\r\n$5\r\nhello\r\n$3\r\nget\r\n$5\r\nworld\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(
            frame,
            RespFrame::Set(Set::new(vec![
                BulkString::new(b"set".to_vec()).into(),
                BulkString::new(b"hello".to_vec()).into(),
                BulkString::new(b"get".to_vec()).into(),
                BulkString::new(b"world".to_vec()).into(),
            ]))
        );
    }
}

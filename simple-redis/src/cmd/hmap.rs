use anyhow::Result;

use crate::{Backend, BulkString, RespArray, RespFrame, RespNull};
use crate::cmd::{
    CommandError, CommandExecutor, extract_args, HGet, HGetAll, HMGet, HMSet, HSet,
    RESP_OK, validate_command,
};

impl CommandExecutor for HGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend
            .hget(&self.key, &self.field)
            .unwrap_or_else(|| RespFrame::Null(RespNull))
    }
}

impl CommandExecutor for HGetAll {
    fn execute(self, backend: &Backend) -> RespFrame {
        let hmap = backend.hgetall(&self.key);
        match hmap {
            Some(hmap) => {
                let mut data = Vec::with_capacity(hmap.len());
                for v in hmap.iter() {
                    let key = v.key().to_owned();
                    data.push((key, v.value().clone()));
                }
                if self.sort {
                    data.sort_by(|a, b| a.0.cmp(&b.0));
                }
                let ret = data
                    .into_iter()
                    .flat_map(|(k, v)| vec![BulkString::from(k).into(), v])
                    .collect::<Vec<RespFrame>>();

                RespArray::new(ret).into()
            }
            None => RespArray::new([]).into(),
        }
    }
}

impl CommandExecutor for HSet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.hset(self.key.clone(), self.field.clone(), self.value.clone());
        RESP_OK.clone()
    }
}

impl CommandExecutor for HMSet {
    fn execute(self, backend: &Backend) -> RespFrame {
        let fields = self.fields.0.unwrap();
        for i in (0..fields.len()).step_by(2) {
            if let (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(value))) =
                (fields.get(i), fields.get(i + 1))
            {
                backend.hset(
                    self.key.clone(),
                    String::from_utf8_lossy(key.as_ref()).to_string(),
                    RespFrame::BulkString(value.clone()),
                );
            }
        }
        RESP_OK.clone()
    }
}

impl CommandExecutor for HMGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        let fields = self.fields.0.unwrap();
        let mut result = Vec::new();
        for field in fields {
            if let RespFrame::BulkString(field) = field {
                let value = backend.hget(
                    &self.key,
                    &String::from_utf8_lossy(field.as_ref()).to_string(),
                );
                result.push(value.unwrap_or_else(|| RespFrame::Null(RespNull)));
            }
        }
        RespFrame::Array(RespArray(Some(result)))
    }
}

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hget"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(k)), Some(RespFrame::BulkString(f))) => Ok(HGet {
                key: String::from_utf8_lossy(k.as_ref()).to_string(),
                field: String::from_utf8_lossy(f.as_ref()).to_string(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HGetAll {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hgetall"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(k)) => Ok(HGetAll {
                key: String::from_utf8_lossy(k.as_ref()).to_string(),
                sort: false,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hset"], 3)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(k)), Some(RespFrame::BulkString(f)), Some(v)) => Ok(HSet {
                key: String::from_utf8_lossy(k.as_ref()).to_string(),
                field: String::from_utf8_lossy(f.as_ref()).to_string(),
                value: v,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HMSet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> std::result::Result<Self, Self::Error> {
        validate_command(&value, &["hmset"], value.0.as_ref().unwrap().len() - 1)?;
        if value.0.as_ref().unwrap().len() % 2 != 0 {
            return Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            ));
        }
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(k)) => Ok(HMSet {
                key: String::from_utf8_lossy(k.as_ref()).to_string(),
                fields: RespArray(Some(args.collect())),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HMGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> std::result::Result<Self, Self::Error> {
        validate_command(&value, &["hmget"], value.0.as_ref().unwrap().len() - 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(k)) => Ok(HMGet {
                key: String::from_utf8_lossy(k.as_ref()).to_string(),
                fields: RespArray(Some(args.collect())),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::RespDecode;

    use super::*;

    #[test]
    fn test_hget_try_from_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$3\r\nkey\r\n$5\r\nfield\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result = HGet::try_from(frame)?;
        assert_eq!(result.key, "key");
        assert_eq!(result.field, "field");
        Ok(())
    }

    #[test]
    fn test_hgetall_try_from_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$7\r\nhgetall\r\n$3\r\nkey\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result = HGetAll::try_from(frame)?;
        assert_eq!(result.key, "key");
        Ok(())
    }

    #[test]
    fn test_hset_try_from_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nhset\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result = HSet::try_from(frame)?;
        assert_eq!(result.key, "key");
        assert_eq!(result.field, "field");
        assert_eq!(result.value, RespFrame::BulkString("value".into()));
        Ok(())
    }

    #[test]
    fn test_hset_hget_hgetall_commands() -> Result<()> {
        let backend = crate::Backend::new();
        let cmd = HSet {
            key: "map".to_string(),
            field: "hello".to_string(),
            value: RespFrame::BulkString(b"world".into()),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let cmd = HSet {
            key: "map".to_string(),
            field: "hello1".to_string(),
            value: RespFrame::BulkString(b"world1".into()),
        };
        cmd.execute(&backend);

        let cmd = HGet {
            key: "map".to_string(),
            field: "hello".to_string(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"world".into()));

        let cmd = HGetAll {
            key: "map".to_string(),
            sort: true,
        };
        let result = cmd.execute(&backend);
        let expected = RespArray::new([
            BulkString::from("hello").into(),
            BulkString::from("world").into(),
            BulkString::from("hello1").into(),
            BulkString::from("world1").into(),
        ]);
        assert_eq!(result, expected.into());
        Ok(())
    }

    #[test]
    fn test_hmset_try_from_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*12\r\n$5\r\nhmset\r\n$6\r\nmyhash\r\n$1\r\n1\r\n$1\r\n2\r\n$1\r\n3\r\n$1\r\n4\r\n$1\r\n5\r\n$1\r\n6\r\n$1\r\n7\r\n$1\r\n8\r\n$1\r\n9\r\n$2\r\n10\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result = HMSet::try_from(frame)?;
        assert_eq!(result.key, "myhash");
        let fields = result.fields.0.unwrap();
        assert_eq!(fields.len(), 10);
        assert_eq!(fields[0], RespFrame::BulkString(b"1".into()));
        assert_eq!(fields[1], RespFrame::BulkString(b"2".into()));
        assert_eq!(fields[2], RespFrame::BulkString(b"3".into()));
        assert_eq!(fields[3], RespFrame::BulkString(b"4".into()));
        assert_eq!(fields[4], RespFrame::BulkString(b"5".into()));
        assert_eq!(fields[5], RespFrame::BulkString(b"6".into()));
        assert_eq!(fields[6], RespFrame::BulkString(b"7".into()));
        assert_eq!(fields[7], RespFrame::BulkString(b"8".into()));
        assert_eq!(fields[8], RespFrame::BulkString(b"9".into()));
        assert_eq!(fields[9], RespFrame::BulkString(b"10".into()));
        Ok(())
    }

    #[test]
    fn test_hmget_try_from_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$5\r\nhmget\r\n$6\r\nmyhash\r\n$1\r\n1\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result = HMGet::try_from(frame)?;
        assert_eq!(result.key, "myhash");
        let fields = result.fields.0.unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], RespFrame::BulkString(b"1".into()));
        Ok(())
    }

    #[test]
    fn test_hmset_hmget_commands() -> Result<()> {
        let backend = crate::Backend::new();
        let cmd = HMSet {
            key: "myhash".to_string(),
            fields: RespArray(Some(vec![
                RespFrame::BulkString(b"1".into()),
                RespFrame::BulkString(b"2".into()),
                RespFrame::BulkString(b"3".into()),
                RespFrame::BulkString(b"4".into()),
                RespFrame::BulkString(b"5".into()),
                RespFrame::BulkString(b"6".into()),
                RespFrame::BulkString(b"7".into()),
                RespFrame::BulkString(b"8".into()),
                RespFrame::BulkString(b"9".into()),
                RespFrame::BulkString(b"10".into()),
            ])),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let cmd = HMGet {
            key: "myhash".to_string(),
            fields: RespArray(Some(vec![
                RespFrame::BulkString(b"1".into()),
                RespFrame::BulkString(b"3".into()),
                RespFrame::BulkString(b"5".into()),
                RespFrame::BulkString(b"7".into()),
                RespFrame::BulkString(b"9".into()),
            ])),
        };
        let result = cmd.execute(&backend);
        let expected = RespArray::new([
            RespFrame::BulkString(b"2".into()),
            RespFrame::BulkString(b"4".into()),
            RespFrame::BulkString(b"6".into()),
            RespFrame::BulkString(b"8".into()),
            RespFrame::BulkString(b"10".into()),
        ]);
        assert_eq!(result, expected.into());
        Ok(())
    }
}

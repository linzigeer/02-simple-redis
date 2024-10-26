use crate::{
    cmd::{extract_cmd_args, validate_command},
    RespArray,
    RespFrame::BulkString,
};

use super::{CommandError, HGet, HGetAll, HSet};

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "hget", 2)?;
        let mut cmd_args = extract_cmd_args(value, 1)?;
        println!("hget cmd_args.len():{}", cmd_args.len());
        match (cmd_args.pop(), cmd_args.pop()) {
            (Some(BulkString(key)), Some(BulkString(table_name))) => {
                let table_name = String::from_utf8(table_name.0)?;
                let key = String::from_utf8(key.0)?;
                Ok(HGet::new(table_name, key))
            }
            _ => Err(CommandError::InvalidArgument(
                "hget should have two RespBulkString values as arguments".into(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HGetAll {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "hgetall", 1)?;
        let mut cmd_args = extract_cmd_args(value, 1)?;
        match cmd_args.pop() {
            Some(BulkString(table_name)) => {
                let table_name = String::from_utf8(table_name.0)?;
                Ok(HGetAll::new(table_name))
            }
            _ => Err(CommandError::InvalidArgument(
                "cmd hgetall should have a RespBulkString value as argument!".into(),
            )),
        }
    }
}

///hset table1 key1 name1
///"*4\r\n$4\r\nhset\r\n$6\r\ntable1\r\n$4\r\nkey1\r\n$5\r\nname1\r\n"
impl TryFrom<RespArray> for HSet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "hset", 3)?;
        let mut cmd_args = extract_cmd_args(value, 1)?;

        match (cmd_args.pop(), cmd_args.pop(), cmd_args.pop()) {
            (Some(BulkString(value)), Some(BulkString(key)), Some(BulkString(table_name))) => {
                let table_name = String::from_utf8(table_name.0)?;
                let key = String::from_utf8(key.0)?;
                let value = value.into();
                Ok(HSet::new(table_name, key, value))
            }
            _ => Err(CommandError::InvalidArgument(
                "cmd hset should have three RespBulkString values as arguments".into(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{DecodeResp, HGet, HGetAll, HSet, RespArray, RespBulkString, RespFrame};
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_hget() -> Result<()> {
        let mut bytes_mut =
            BytesMut::from(&b"*3\r\n$4\r\nhget\r\n$6\r\ntable1\r\n$4\r\nkey1\r\n"[..]);
        let frame = RespArray::decode(&mut bytes_mut).unwrap();
        println!("frame:{frame:?}");
        let hget = HGet::try_from(frame)?;
        println!("hget:{:?}", hget);
        assert_eq!(hget.table_name, "table1");
        assert_eq!(hget.key, "key1");
        Ok(())
    }

    #[test]
    fn test_hset() -> Result<()> {
        let mut bytes_mut = BytesMut::from(
            &b"*4\r\n$4\r\nhset\r\n$6\r\ntable1\r\n$3\r\nkey\r\n$5\r\nvalue\r\n"[..],
        );
        let resp_array = RespArray::decode(&mut bytes_mut)?;
        let hset = HSet::try_from(resp_array)?;

        assert_eq!(hset.table_name, "table1");
        assert_eq!(hset.key, "key");
        assert_eq!(hset.value, RespFrame::from(RespBulkString::from("value")));

        Ok(())
    }

    #[test]
    fn test_hgetall() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"*2\r\n$7\r\nhgetall\r\n$4\r\nmap1\r\n"[..]);
        let resp_array = RespArray::decode(&mut bytes_mut)?;

        let hgetall = HGetAll::try_from(resp_array)?;
        assert_eq!(hgetall.table_name, "map1");

        Ok(())
    }
}

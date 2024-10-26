use crate::{RespArray, RespFrame, RespNull};

use super::{extract_cmd_args, validate_command, CommandError, CommandExecutor, Get, Set, RESP_OK};

impl CommandExecutor for Get {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        match backend.get(&self.key) {
            Some(resp_frame) => resp_frame,
            None => RespFrame::Null(RespNull),
        }
    }
}

impl CommandExecutor for Set {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        backend.set(self.key, self.value);
        RESP_OK.clone()
    }
}

///*2\r\n$3\r\nget\r\n$5\r\nhello\r\n
impl TryFrom<RespArray> for Get {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "get", 1)?;
        let mut args = extract_cmd_args(value, 1)?;

        match args.pop() {
            Some(RespFrame::BulkString(cmd_args)) => {
                let get = Get::new(String::from_utf8(cmd_args.0)?);
                Ok(get)
            }
            _ => Err(CommandError::InvalidArgument(
                "command get should have a BulkString type as argument!".into(),
            )),
        }
    }
}

impl TryFrom<RespArray> for Set {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, "set", 2)?;
        let mut args = extract_cmd_args(value, 1)?.into_iter();

        if let (Some(RespFrame::BulkString(key)), Some(value)) = (args.next(), args.next()) {
            let key = String::from_utf8(key.0)?;
            Ok(Set::new(key, value))
        } else {
            Err(CommandError::InvalidArgument(
                "command set must have BulkString as key and RespFrame as value!".into(),
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Backend, DecodeResp};

    use super::*;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_get_from_resp_array() -> Result<()> {
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut bytes)?;

        let get = Get::try_from(frame)?;
        // assert_eq!(get.key, "hello");
        assert_eq!(get, Get::new("hello".into()));
        Ok(())
    }

    #[test]
    fn test_set_from_resp_array() -> Result<()> {
        let mut bytes_mut =
            BytesMut::from(&b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"[..]);
        let frame = RespArray::decode(&mut bytes_mut)?;

        let key = "hello".into();
        let value = "world".into();
        let set = Set::new(key, RespFrame::BulkString(value));

        assert_eq!(Set::try_from(frame)?, set);

        Ok(())
    }

    #[test]
    fn test_set_get_cmd_execute() {
        let backend = Backend::new();
        let set_cmd = Set::new("hello".into(), RespFrame::BulkString("world".into()));
        let resp = set_cmd.execute(&backend);
        assert_eq!(resp, RESP_OK.clone());

        let get_cmd = Get::new("hello".into());
        let resp = get_cmd.execute(&backend);
        assert_eq!(resp, RespFrame::BulkString("world".into()));

        let get_cmd = Get::new("AAA".into());
        let resp = get_cmd.execute(&backend);
        assert_eq!(resp, RespFrame::Null(RespNull));
    }
}

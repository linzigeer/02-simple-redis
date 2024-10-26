use bytes::{Buf, BytesMut};
use std::iter::Peekable;
use std::slice::Iter;
use tracing::instrument;

use super::*;

/*
Simple strings: +OK\r\n
Simple errors: -Error message\r\n
                -ERR unknown command 'asdf'
                -WRONGTYPE Operation against a key holding the wrong kind of value
Integers: :[<+|->]<value>\r\n
                :0\r\n
                :1000\r\n
Bulk strings: $<length>\r\n<data>\r\n
                $5\r\nhello\r\n
                $0\r\n\r\n
Null bulk strings: $-1\r\n
Arrays: *<number-of-elements>\r\n<element-1>...<element-n>
                *0\r\n
                *2\r\n$5\r\nhello\r\n$5\r\nworld\r\n
                *3\r\n:1\r\n:2\r\n:3\r\n
Null arrays: *-1\r\n
Nulls: _\r\n
Booleans: #<t|f>\r\n
Doubles: ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
Bulk errors: !<length>\r\n<error>\r\n
Maps: %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
Sets: ~<number-of-elements>\r\n<element-1>...<element-n>
*/

impl DecodeResp for RespFrame {
    const PREFIX: u8 = 0;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter: Peekable<Iter<u8>> = buf.iter().peekable();
        let peek_first = iter.peek();

        match peek_first {
            Some(&&POSITIVE_SIGN) => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(&&NEGATIVE_SIGN) => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(&&COLON) => {
                let frame = RespInteger::decode(buf)?;
                Ok(frame.into())
            }
            Some(&&DOLLAR) => match RespBulkString::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => {
                    let frame = RespNullBulkString::decode(buf)?;
                    Ok(frame.into())
                }
            },
            Some(&&ASTERISK) => match RespArray::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => {
                    let frame = RespNullArray::decode(buf)?;
                    Ok(frame.into())
                }
            },
            Some(&&UNDERLINE) => {
                let frame = RespNull::decode(buf)?;
                Ok(frame.into())
            }
            Some(&&POND_SIGN) => {
                let frame = RespBooleans::decode(buf)?;
                Ok(frame.into())
            }
            Some(&&COMMA) => {
                let frame = RespDoubles::decode(buf)?;
                Ok(frame.into())
            }
            Some(&&EXCLAMATION_MARK) => {
                let frame = RespBulkErrors::decode(buf)?;
                Ok(frame.into())
            }
            Some(&&PERCENT_SIGN) => {
                let frame = RespMaps::decode(buf)?;
                Ok(frame.into())
            }
            Some(&&TILDE_SIGN) => {
                let frame = RespSets::decode(buf)?;
                Ok(frame.into())
            }
            None => Err(RespError::NotComplete),
            _ => Err(RespError::InvalidFrameType(format!("{peek_first:?}"))),
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let mut peekable = buf.iter().peekable();
        let peek_first = peekable.peek();
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(buf);

        match peek_first {
            Some(&&POSITIVE_SIGN) => Ok(SimpleString::expect_length(buf)?),
            Some(&&NEGATIVE_SIGN) => Ok(SimpleError::expect_length(buf)?),
            Some(&&COLON) => Ok(RespInteger::expect_length(buf)?),
            Some(&&DOLLAR) => match RespBulkString::expect_length(buf) {
                Ok(len) => Ok(len),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => Ok(RespNullBulkString::expect_length(buf)?),
            },
            Some(&&ASTERISK) => match RespArray::decode(&mut bytes) {
                Ok(_) => Ok(RespArray::expect_length(buf)?),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => match RespNullArray::decode(&mut bytes) {
                    Ok(_) => Ok(RespNullArray::expect_length(buf)?),
                    _ => Err(RespError::InvalidFrameType(
                        "neither RespArray nor RespNullArray".into(),
                    )),
                },
            },
            Some(&&UNDERLINE) => Ok(RespNull::expect_length(buf)?),
            Some(&&POND_SIGN) => Ok(RespBooleans::expect_length(buf)?),
            Some(&&COMMA) => Ok(RespDoubles::expect_length(buf)?),
            Some(&&EXCLAMATION_MARK) => Ok(RespBulkErrors::expect_length(buf)?),
            Some(&&PERCENT_SIGN) => Ok(RespMaps::expect_length(buf)?),
            Some(&&TILDE_SIGN) => Ok(RespSets::expect_length(buf)?),
            None => Err(RespError::NotComplete),
            _ => Err(RespError::InvalidFrameType(format!("{peek_first:?}"))),
        }
    }
}

///Simple strings: +OK\r\n
impl DecodeResp for SimpleString {
    const PREFIX: u8 = POSITIVE_SIGN;

    #[instrument]
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, 3)?;
        let data = buf.split_to(end + CRLF.len());
        let msg = &data[LEN_ONE..end];
        let msg = String::from_utf8_lossy(msg);

        Ok(SimpleString::from(msg))
    }
}

///Simple errors: -Error message\r\n
/// -ERR unknown command 'asdf'
///-WRONGTYPE Operation against a key holding the wrong kind of value
impl DecodeResp for SimpleError {
    const PREFIX: u8 = NEGATIVE_SIGN;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, 3)?;
        let data = buf.split_to(end + CRLF.len());
        let msg = String::from_utf8_lossy(&data[LEN_ONE..end]);

        Ok(SimpleError::from(msg))
    }
}
///Integers: :[<+|->]<value>\r\n
///:0\r\n
///:1000\r\n
impl DecodeResp for RespInteger {
    const PREFIX: u8 = COLON;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, 3)?;
        let data = buf.split_to(end + CRLF.len());
        let msg = String::from_utf8_lossy(&data[LEN_ONE..end]);

        Ok(msg.try_into()?)
    }
}

///Bulk strings: $<length>\r\n<data>\r\n
///$5\r\nhello\r\n
///$0\r\n\r\n
impl DecodeResp for RespBulkString {
    const PREFIX: u8 = DOLLAR;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        let remained = &buf[end + CRLF.len()..];
        //判断剩下的长度
        if remained.len() < len + CRLF.len() {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF.len());

        let data = buf.split_to(len + CRLF.len());

        let msg = &data[..len];

        Ok(RespBulkString::from(msg))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF.len() + len + CRLF.len())
    }
}

/// Null bulk strings: $-1\r\n
impl DecodeResp for RespNullBulkString {
    const PREFIX: u8 = DOLLAR;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "$-1\r\n", "NullBulkString")?;
        Ok(RespNullBulkString::new())
    }
}
/*
Arrays: *<number-of-elements>\r\n<element-1>...<element-n>
            *0\r\n
            *2\r\n$5\r\nhello\r\n$5\r\nworld\r\n
            *3\r\n:1\r\n:2\r\n:3\r\n
*/
impl DecodeResp for RespArray {
    const PREFIX: u8 = ASTERISK;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, element_count) = parse_length(buf, Self::PREFIX)?;
        println!("end:{end}, element_count:{element_count}");
        let total_len = calc_total_length(buf, end, element_count, Self::PREFIX)?;
        println!("total_len:{total_len}");
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + 2);
        let mut vec = Vec::with_capacity(element_count);
        for _ in 0..element_count {
            let frame = RespFrame::decode(buf)?;
            vec.push(frame);
        }

        Ok(RespArray::new(vec))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

///Null arrays: *-1\r\n
impl DecodeResp for RespNullArray {
    const PREFIX: u8 = ASTERISK;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "*-1\r\n", "RespNullArray")?;
        Ok(RespNullArray::new())
    }
}
///Nulls: _\r\n
impl DecodeResp for RespNull {
    const PREFIX: u8 = UNDERLINE;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "_\r\n", "RespNull")?;
        Ok(RespNull::new())
    }
}

///Booleans: #<t|f>\r\n
impl DecodeResp for RespBooleans {
    const PREFIX: u8 = POND_SIGN;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, 3)?;
        let data = buf.split_to(end + CRLF.len());
        let msg = &data[LEN_ONE..end];
        let msg = String::from_utf8_lossy(msg);

        Ok(msg.into())
    }
}

///Doubles: ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
impl DecodeResp for RespDoubles {
    const PREFIX: u8 = COMMA;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, 3)?;
        let data = buf.split_to(end + CRLF.len());
        let ret = &data[LEN_ONE..end];
        let ret = String::from_utf8_lossy(ret);

        Ok(ret.try_into()?)
    }
}

///Bulk errors: !<length>\r\n<error>\r\n
///              !21\r\nSYNTAX invalid syntax\r\n
impl DecodeResp for RespBulkErrors {
    const PREFIX: u8 = EXCLAMATION_MARK;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let remained = &buf[end + CRLF.len()..];
        if remained.len() < len + CRLF.len() {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF.len());

        let data = buf.split_to(len + CRLF.len());
        let msg = &data[..len];

        Ok(RespBulkErrors::from(msg))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF.len() + len + CRLF.len())
    }
}

/*
Maps: %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
        {
            "first": 1,
            "second": 2
        }
        %2\r\n+first\r\n:1\r\n+second\r\n:2\r\n
*/
impl DecodeResp for RespMaps {
    const PREFIX: u8 = PERCENT_SIGN;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = Self::expect_length(buf)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF.len());
        let mut map = BTreeMap::new();
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            map.insert(key.0, value);
        }

        Ok(RespMaps::from(map))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        Ok(total_len)
    }
}

/// Sets: ~<number-of-elements>\r\n<element-1>...<element-n>
impl DecodeResp for RespSets {
    const PREFIX: u8 = TILDE_SIGN;

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = Self::expect_length(buf)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF.len());
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            vec.push(frame);
        }

        Ok(RespSets::from(vec))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        Ok(total_len)
    }
}

fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::InvalidFrameLength(buf.len()));
    }

    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expected type:{:?}, got:{:?}",
            expect_type,
            String::from_utf8_lossy(buf)
        )));
    }

    buf.advance(expect.len());

    Ok(())
}

fn parse_length(buf: &[u8], prefix: u8) -> Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix, 3)?;
    let len = String::from_utf8_lossy(&buf[LEN_ONE..end]);

    Ok((end, len.parse()?))
}

pub fn extract_simple_frame_data(
    buf: &[u8],
    prefix: u8,
    min_len: usize,
) -> Result<usize, RespError> {
    if buf.len() < min_len {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(&[prefix]) {
        return Err(RespError::InvalidFrameType(format!(
            "expected:SimpleString({:?}), got:{:?}",
            String::from_utf8_lossy(&[prefix]).to_string(),
            buf
        )));
    }
    //ok_or方法把Option转换为Result。特别地，当Option为时，应提供默认的错误类型
    let end = find_first_crlf(buf).ok_or(RespError::NotComplete)?;

    Ok(end)
}

fn find_first_crlf(buf: &[u8]) -> Option<usize> {
    let mut first_crlf_location = None;

    for i in 1..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            first_crlf_location = Some(i);
            break;
        }
    }

    first_crlf_location
}

/*
Arrays: *<number-of-elements>\r\n<element-1>...<element-n>
                *0\r\n
                *2\r\n$5\r\nhello\r\n$5\r\nworld\r\n
                *3\r\n:1\r\n:2\r\n:3\r\n
Sets: ~<number-of-elements>\r\n<element-1>...<element-n>
Maps: %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
*/
fn calc_total_length(
    buf: &[u8],
    end: usize,
    element_count: usize,
    prefix: u8,
) -> Result<usize, RespError> {
    let mut total_len = end + CRLF.len();
    let mut data = &buf[total_len..];
    match prefix {
        ASTERISK | TILDE_SIGN => {
            for _ in 0..element_count {
                let len = RespFrame::expect_length(data)?;
                total_len += len;
                data = &buf[total_len..];
            }
            Ok(total_len)
        }
        PERCENT_SIGN => {
            for _ in 0..element_count {
                let key_len = SimpleString::expect_length(data)?;
                total_len += key_len;
                data = &buf[total_len..];

                let value_len = RespFrame::expect_length(data)?;
                total_len += value_len;
                data = &buf[total_len..];
            }
            Ok(total_len)
        }
        _ => Ok(element_count + CRLF.len()),
    }
}

// pub fn split_data_by_crlf(data: impl Into<String>) -> Vec<String> {
//     let vec = data
//         .into()
//         .split("\r\n")
//         .into_iter()
//         .filter(|element| *element != "")
//         .map(|data| data.to_owned())
//         .collect::<Vec<_>>();
//     println!("{vec:?}");
//     vec
// }

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::Result;
    use bytes::{BufMut, BytesMut};

    use crate::resp::{DecodeResp, RespArray, RespBulkString, RespError, RespFrame};

    use super::{
        RespBooleans, RespBulkErrors, RespDoubles, RespInteger, RespMaps, RespNull, RespNullArray,
        RespNullBulkString, RespSets, SimpleError, SimpleString,
    };

    ///Simple strings: +OK\r\n
    #[test]
    fn test_decode_simple_string() -> Result<()> {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.extend_from_slice(b"+OK\r\n");

        let decoded = SimpleString::decode(&mut bytes_mut)?;
        assert_eq!(decoded, SimpleString::from("OK"));

        bytes_mut.extend_from_slice(b"+hello\r");
        let decoded = SimpleString::decode(&mut bytes_mut);
        assert_eq!(decoded.unwrap_err(), RespError::NotComplete);

        bytes_mut.put_u8(b'\n');
        let decoded = SimpleString::decode(&mut bytes_mut)?;
        assert_eq!(decoded, SimpleString::from("hello"));

        Ok(())
    }

    ///Simple errors: -Error message\r\n
    #[test]
    fn test_decode_simple_error() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"-error\r\n"[..]);
        let decoded = SimpleError::decode(&mut bytes_mut)?;

        assert_eq!(decoded, SimpleError::from("error"));

        Ok(())
    }

    ///Integers: :[<+|->]<value>\r\n
    ///:0\r\n
    ///:1000\r\n
    #[test]
    fn test_decode_integer() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b":1024\r\n"[..]);
        let decoded = RespInteger::decode(&mut bytes_mut)?;

        assert_eq!(decoded, RespInteger::from(1024));

        bytes_mut.extend_from_slice(b":-4096\r\n");
        let decoded = RespInteger::decode(&mut bytes_mut)?;
        assert_eq!(decoded, RespInteger::from(-4096));

        Ok(())
    }

    ///Bulk strings: $<length>\r\n<data>\r\n
    ///$5\r\nhello\r\n
    ///$0\r\n\r\n
    #[test]
    fn test_decode_resp_bulk_string() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"$5\r\nhello\r\n"[..]);
        let decoded = RespBulkString::decode(&mut bytes_mut)?;
        assert_eq!(decoded, RespBulkString::from("hello"));

        bytes_mut.extend_from_slice(b"$5\r\nworld");
        let decoded = RespBulkString::decode(&mut bytes_mut);
        assert_eq!(decoded.unwrap_err(), RespError::NotComplete);

        bytes_mut.extend_from_slice(b"\r\n");
        let decoded = RespBulkString::decode(&mut bytes_mut)?;
        assert_eq!(decoded, RespBulkString::from("world"));

        Ok(())
    }

    /// Null bulk strings: $-1\r\n
    #[test]
    fn test_decode_resp_null_bulk_string() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"$-1\r\n"[..]);
        let decoded = RespNullBulkString::decode(&mut bytes_mut)?;

        assert_eq!(decoded, RespNullBulkString::new());

        Ok(())
    }

    /*
    Arrays: *<number-of-elements>\r\n<element-1>...<element-n>
                *0\r\n
                *2\r\n$5\r\nhello\r\n$5\r\nworld\r\n
                *3\r\n:1\r\n:2\r\n:3\r\n
    */
    #[test]
    fn test_resparray_decode() -> Result<()> {
        let mut bytesmut = BytesMut::new();
        bytesmut.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let resp_array = RespArray::decode(&mut bytesmut)?;

        let frame1: RespBulkString = "set".into();
        let frame1: RespFrame = frame1.into();
        let frame2: RespBulkString = "hello".into();
        let frame2: RespFrame = frame2.into();

        let ra = RespArray::from(vec![frame1, frame2]);
        assert_eq!(resp_array, ra);

        Ok(())
    }

    ///Null arrays: *-1\r\n
    #[test]
    fn test_decode_null_array() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"*-1\r\n"[..]);
        let decode = RespNullArray::decode(&mut bytes_mut)?;

        assert_eq!(decode, RespNullArray::new());

        Ok(())
    }

    ///Nulls: _\r\n
    #[test]
    fn test_decode_resp_null() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"_\r\n"[..]);
        let decode = RespNull::decode(&mut bytes_mut)?;

        assert_eq!(decode, RespNull::new());
        Ok(())
    }

    ///Booleans: #<t|f>\r\n
    #[test]
    fn test_decode_boolean() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"#t\r\n"[..]);
        let decode = RespBooleans::decode(&mut bytes_mut)?;
        assert_eq!(decode, RespBooleans::new(true));

        bytes_mut.extend_from_slice(b"#f\r\n");
        let decode = RespBooleans::decode(&mut bytes_mut)?;
        assert_eq!(decode, RespBooleans::new(false));

        Ok(())
    }

    ///Doubles: ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
    #[test]
    fn test_decode_double() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b",33.44\r\n"[..]);
        let decode = RespDoubles::decode(&mut bytes_mut)?;

        assert_eq!(decode, RespDoubles::new(33.44));

        bytes_mut.extend_from_slice(b",-1.96\r\n");
        let decode = RespDoubles::decode(&mut bytes_mut)?;
        assert_eq!(decode, RespDoubles::new(-1.96));

        Ok(())
    }

    ///Bulk errors: !<length>\r\n<error>\r\n
    ///              !21\r\nSYNTAX invalid syntax\r\n
    #[test]
    fn test_decode_bulk_errors() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"!21\r\nSYNTAX invalid syntax\r\n"[..]);
        let decoded = RespBulkErrors::decode(&mut bytes_mut)?;
        assert_eq!(decoded, RespBulkErrors::from("SYNTAX invalid syntax"));

        bytes_mut.extend_from_slice(b"!21\r\nSYNTAX ");
        let decoded = RespBulkErrors::decode(&mut bytes_mut);
        assert_eq!(decoded.unwrap_err(), RespError::NotComplete);

        bytes_mut.extend_from_slice(b"invalid syntax\r\n");
        let decoded = RespBulkErrors::decode(&mut bytes_mut)?;
        assert_eq!(decoded, RespBulkErrors::from("SYNTAX invalid syntax"));

        Ok(())
    }

    /*
    Maps: %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
            {
                "first": 1,
                "second": 2
            }
            %2\r\n+first\r\n:1\r\n+second\r\n:2\r\n
    */
    #[test]
    fn test_decode_resp_map() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n"[..]);
        let decoded = RespMaps::decode(&mut bytes_mut)?;

        let key1 = "first";
        let value1: RespInteger = 1.into();
        let value1: RespFrame = value1.into();

        let key2 = "second";
        let value2: RespInteger = 2.into();
        let value2: RespFrame = value2.into();

        let mut map = RespMaps::new(BTreeMap::new());
        map.insert(key1.into(), value1);
        map.insert(key2.into(), value2);

        assert_eq!(decoded, map);

        Ok(())
    }

    /// Sets: ~<number-of-elements>\r\n<element-1>...<element-n>
    #[test]
    fn test_decode_resp_set() -> Result<()> {
        let mut bytes_mut = BytesMut::from(&b"~2\r\n$3\r\nset\r\n$5\r\nhello\r\n"[..]);
        let decoded = RespSets::decode(&mut bytes_mut)?;

        assert_eq!(
            decoded,
            RespSets::new(vec![
                RespBulkString::new(b"set".to_vec()).into(),
                RespBulkString::new(b"hello".to_vec()).into()
            ])
        );

        Ok(())
    }

    #[test]
    fn test2() {
        let a = "1000".parse::<u8>().ok();
        println!("{a:?}"); //None

        //在Result上调用ok()方法转为Option时，会消耗所有权，如果是错误的话抛弃具体的错误信息，都转为None值
        let option = Ok::<i32, String>(10).ok();
        println!("{option:?}"); //Some(10)

        let ret = option.ok_or("value is None");
        println!("{ret:?}");

        let blank = Err::<i32, String>("an error occurred".into()).ok();
        println!("{blank:?}"); //None
    }
}

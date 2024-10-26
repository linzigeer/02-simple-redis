mod decode;
mod encode;

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
use crate::resp::decode::extract_simple_frame_data;
use bytes::BytesMut;
use derive_more::{AsRef, Constructor, Deref, From};
use enum_dispatch::enum_dispatch;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::num::{ParseFloatError, ParseIntError};
use std::ops::DerefMut;
use thiserror::Error;

pub const CRLF: &[u8] = b"\r\n";
pub const POSITIVE_SIGN: u8 = b'+';
pub const NEGATIVE_SIGN: u8 = b'-';
pub const ERROR: &[u8] = b"Error";
pub const COLON: u8 = b':';
pub const COMMA: u8 = b',';
pub const DOLLAR: u8 = b'$';
pub const ONE: u8 = b'1';
pub const ASTERISK: u8 = b'*';
pub const UNDERLINE: u8 = b'_';
pub const POND_SIGN: u8 = b'#';
pub const PERCENT_SIGN: u8 = b'%';
pub const TILDE_SIGN: u8 = b'~';
pub const TRUE: u8 = b't';
pub const FALSE: u8 = b'f';
pub const EXCLAMATION_MARK: u8 = b'!';
pub const MAX_BUF_SIZE: usize = 4096;
pub const WHITE_SPACE: u8 = b' ';
pub const INFINITY: &[u8] = b"inf";
pub const NAN: &[u8] = b"nan";
pub const LEN_ONE: usize = 1;

#[enum_dispatch]
pub trait EncodeResp {
    fn encode(self) -> Vec<u8>;
}

pub trait DecodeResp: Sized {
    const PREFIX: u8;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, 3)?;
        Ok(end + CRLF.len())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RespError {
    // #[error("Invalid frame {0}")]
    // InvalidFrame(String),
    #[error("Invalid frame type {0}")]
    InvalidFrameType(String),

    #[error("Invalid frame length {0}")]
    InvalidFrameLength(usize),

    #[error("Not complete")]
    NotComplete,

    #[error("Parse Error:{0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("utf8 error:{0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Parse float error:{0}")]
    ParseFloatError(#[from] ParseFloatError),
}

#[enum_dispatch(EncodeResp)]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum RespFrame {
    SimpleString(SimpleString),
    SimpleError(SimpleError),
    Integer(RespInteger),
    BulkString(RespBulkString),
    NullBulkString(RespNullBulkString),
    Arrays(RespArray),
    NullArray(RespNullArray),
    Null(RespNull),
    Booleans(RespBooleans),
    Doubles(RespDoubles),
    BulkErrors(RespBulkErrors),
    Maps(RespMaps),
    Sets(RespSets),
}

///Simple strings: +OK\r\n
#[derive(Debug, From, Deref, PartialEq, PartialOrd, Clone)]
#[from(Cow<'_, str>, String, &'static str)]
pub struct SimpleString(pub(crate) String);

#[derive(Debug, From, Deref, PartialEq, PartialOrd, Clone)]
#[from(Cow<'_, str>, String, &'static str)]
pub struct SimpleError(pub(crate) String);

#[derive(Debug, From, Deref, PartialEq, PartialOrd, Clone)]
#[from(i32, i64)]
pub struct RespInteger(pub(crate) i64);

#[derive(Debug, From, Deref, PartialEq, PartialOrd, Constructor, Clone)]
#[from(&'static str, &[u8], Vec<u8>, String)]
pub struct RespBulkString(pub(crate) Vec<u8>);

#[derive(Debug, PartialEq, PartialOrd, Constructor, Clone)]
pub struct RespNullBulkString;

#[derive(Debug, From, PartialEq, PartialOrd, Constructor, Clone)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

#[derive(Debug, PartialEq, PartialOrd, Constructor, Clone)]
pub struct RespNullArray;

#[derive(Debug, PartialEq, PartialOrd, Constructor, Clone)]
pub struct RespNull;

#[derive(Debug, From, Deref, PartialEq, PartialOrd, Constructor, Clone)]
pub struct RespBooleans(pub(crate) bool);

#[derive(Debug, From, Deref, PartialEq, PartialOrd, Constructor, Clone)]
pub struct RespDoubles(pub(crate) f64);

#[derive(Debug, From, PartialEq, PartialOrd, Clone)]
#[from(&'static str, &[u8], Vec<u8>)]
pub struct RespBulkErrors(pub(crate) Vec<u8>);

#[derive(Debug, From, PartialEq, PartialOrd, Default, Constructor, Clone)]
pub struct RespMaps(pub(crate) BTreeMap<String, RespFrame>);

#[derive(Debug, PartialEq, PartialOrd, From, Constructor, Clone)]
pub struct RespSets(pub(crate) Vec<RespFrame>);

impl From<Cow<'_, str>> for RespBooleans {
    fn from(value: Cow<'_, str>) -> Self {
        if value == "t" {
            RespBooleans::new(true)
        } else {
            RespBooleans::new(false)
        }
    }
}

impl TryFrom<Cow<'_, str>> for RespDoubles {
    type Error = ParseFloatError;

    fn try_from(value: Cow<'_, str>) -> Result<Self, Self::Error> {
        let ret = value.trim().parse::<f64>()?;
        Ok(RespDoubles::new(ret))
    }
}

impl TryFrom<String> for RespInteger {
    type Error = ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let ret = value.trim().parse::<i64>()?;
        Ok(RespInteger::from(ret))
    }
}

impl TryFrom<Cow<'_, str>> for RespInteger {
    type Error = ParseIntError;

    fn try_from(value: Cow<'_, str>) -> Result<Self, Self::Error> {
        let ret = value.trim().parse::<i64>()?;
        Ok(RespInteger::from(ret))
    }
}

// impl Deref for SimpleString {
//     type Target = String;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl SimpleString {
//     pub fn new(s: impl Into<String>) -> Self {
//         Self(s.into())
//     }
// }

// impl Deref for SimpleError {
//     type Target = String;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl Deref for RespInteger {
//     type Target = i64;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl Deref for RespBulkString {
//     type Target = Vec<u8>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

impl AsRef<[u8]> for RespBulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// impl Deref for RespNullBulkString {
//     type Target = String;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl Deref for RespBooleans {
//     type Target = bool;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl Deref for RespDoubles {
//     type Target = f64;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

impl Deref for RespBulkErrors {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespMaps {
    type Target = BTreeMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMaps {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// impl RespMaps {
//     pub fn new() -> Self {
//         Self(BTreeMap::new())
//     }
// }

impl Deref for RespSets {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

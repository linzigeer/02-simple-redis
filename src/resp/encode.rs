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

///+OK\r\n
impl EncodeResp for SimpleString {
    fn encode(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(5);
        ret.push(POSITIVE_SIGN);
        ret.extend_from_slice(self.as_bytes());
        ret.extend_from_slice(CRLF);
        ret
    }
}
///-Error message\r\n
impl EncodeResp for SimpleError {
    fn encode(self) -> Vec<u8> {
        let msg_len = self.len();
        let mut ret = Vec::with_capacity(msg_len + 9);
        ret.push(NEGATIVE_SIGN);
        ret.extend_from_slice(ERROR);
        ret.push(WHITE_SPACE);
        ret.extend_from_slice(self.as_bytes());
        ret.extend_from_slice(CRLF);

        ret
    }
}
///Integers: :[<+|->]<value>\r\n
///                 :0\r\n
///                :1000\r\n
impl EncodeResp for RespInteger {
    fn encode(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(1 + 8 + 2);
        ret.push(COLON);

        if self.is_negative() {
            ret.push(NEGATIVE_SIGN);
        } else {
            ret.push(POSITIVE_SIGN);
        }

        ret.extend_from_slice(self.abs().to_string().as_bytes());
        ret.extend_from_slice(CRLF);

        ret
    }
}

///Bulk strings: $<length>\r\n<data>\r\n
///                 $5\r\nhello\r\n
///                 $0\r\n\r\n
impl EncodeResp for RespBulkString {
    fn encode(self) -> Vec<u8> {
        let msg_len = self.len();
        let mut ret = Vec::with_capacity(1 + 8 + 2 + msg_len + 2);
        ret.push(DOLLAR);
        ret.extend_from_slice(msg_len.to_string().as_bytes());
        ret.extend_from_slice(CRLF);
        ret.extend_from_slice(self.as_slice());
        ret.extend_from_slice(CRLF);

        ret
    }
}

///Null bulk strings: $-1\r\n
impl EncodeResp for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(8);
        ret.push(DOLLAR);
        ret.push(NEGATIVE_SIGN);
        ret.push(ONE);
        ret.extend_from_slice(CRLF);

        ret
    }
}
///Arrays: *<number-of-elements>\r\n<element-1>...<element-n>
///         *0\r\n
///         *2\r\n$5\r\nhello\r\n$5\r\nworld\r\n
///         *3\r\n:1\r\n:2\r\n:3\r\n
impl EncodeResp for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(MAX_BUF_SIZE);
        let vec_len = self.len();
        ret.push(ASTERISK);
        ret.extend_from_slice(vec_len.to_string().as_bytes());
        ret.extend_from_slice(CRLF);
        for x in self.0 {
            let encoded = x.encode();
            // let len = encoded.len();
            // ret.extend_from_slice(DOLLAR);
            // ret.extend_from_slice(len.to_string().as_bytes());
            // ret.extend_from_slice(CRLF);
            ret.extend_from_slice(encoded.as_slice());
        }

        ret
    }
}

///Null arrays: *-1\r\n
impl EncodeResp for RespNullArray {
    fn encode(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(MAX_BUF_SIZE);
        ret.push(ASTERISK);
        ret.push(NEGATIVE_SIGN);
        ret.push(ONE);
        ret.extend_from_slice(CRLF);

        ret
    }
}

///Nulls: _\r\n
impl EncodeResp for RespNull {
    fn encode(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(3);
        ret.push(UNDERLINE);
        ret.extend_from_slice(CRLF);

        ret
    }
}

///Booleans: #<t|f>\r\n
impl EncodeResp for RespBooleans {
    fn encode(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(4);
        ret.push(POND_SIGN);
        if *self {
            ret.push(TRUE);
        } else {
            ret.push(FALSE);
        }
        ret.extend_from_slice(CRLF);

        ret
    }
}
///Doubles: ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
///             ,1.23\r\n
///             ,inf\r\n
///             ,-inf\r\n
///             ,nan\r\n
///
impl EncodeResp for RespDoubles {
    fn encode(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(1 + 8 + 2);
        ret.push(COMMA);

        if self.0 > 1.0e11 {
            ret.extend_from_slice(INFINITY);
        } else if self.0 < -1.0e11 {
            ret.push(NEGATIVE_SIGN);
            ret.extend_from_slice(INFINITY);
        } else if self.0.is_nan() {
            ret.extend_from_slice(NAN);
        } else {
            ret.extend_from_slice(self.to_string().as_bytes());
        }
        ret.extend_from_slice(CRLF);

        ret
    }
}
///Bulk errors: !<length>\r\n<error>\r\n
impl EncodeResp for RespBulkErrors {
    fn encode(self) -> Vec<u8> {
        let msg_len = self.len();
        let mut ret = Vec::with_capacity(1 + 8 + 2 + msg_len + 2);
        ret.push(EXCLAMATION_MARK);
        ret.extend_from_slice(msg_len.to_string().as_bytes());
        ret.extend_from_slice(CRLF);
        ret.extend_from_slice(self.as_slice());
        ret.extend_from_slice(CRLF);

        ret
    }
}
///%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
///         %2\r\n
///         +first\r\n
///         :1\r\n
///         +second\r\n
///         :2\r\n
impl EncodeResp for RespMaps {
    fn encode(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(MAX_BUF_SIZE);
        let msg_len = self.len();

        ret.push(PERCENT_SIGN);
        ret.extend_from_slice(msg_len.to_string().as_bytes());
        ret.extend_from_slice(CRLF);
        for (key, value) in self.0 {
            ret.extend_from_slice(SimpleString::from(key).encode().as_slice());
            ret.push(COLON);
            ret.extend_from_slice(value.encode().as_slice());
        }

        ret
    }
}
///Sets: ~<number-of-elements>\r\n<element-1>...<element-n>
impl EncodeResp for RespSets {
    fn encode(self) -> Vec<u8> {
        let msg_len = self.len();
        let mut ret = Vec::with_capacity(MAX_BUF_SIZE);
        ret.push(TILDE_SIGN);
        ret.extend_from_slice(msg_len.to_string().as_bytes());
        for e in self.0 {
            ret.extend_from_slice(e.encode().as_slice());
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_simple_string_should_work() {
        let ss: SimpleString = "OK".into();
        let frame: RespFrame = ss.into();
        assert_eq!(frame.encode(), b"+OK\r\n");
    }

    ///-Error message\r\n
    #[test]
    fn encode_simple_error_should_work() {
        let se: SimpleError = "Error".into();
        let frame: RespFrame = se.into();
        assert_eq!(frame.encode(), b"-Error Error\r\n");
    }

    ///Integers: :[<+|->]<value>\r\n
    ///                 :0\r\n
    ///                :1000\r\n
    #[test]
    fn encode_integer_should_work() {
        let resp_integer: RespInteger = 123.into();
        let frame: RespFrame = resp_integer.into();
        assert_eq!(frame.encode(), b":+123\r\n");

        let resp_integer: RespInteger = (-123).into();
        let frame: RespFrame = resp_integer.into();
        assert_eq!(frame.encode(), b":-123\r\n");
    }

    ///Bulk strings: $<length>\r\n<data>\r\n
    ///                 $5\r\nhello\r\n
    ///                 $0\r\n\r\n
    #[test]
    fn encode_bulk_string_should_work() {
        let bs: RespBulkString = "aaa".as_bytes().into();
        let frame: RespFrame = bs.into();
        assert_eq!(frame.encode(), b"$3\r\naaa\r\n");

        let bs: RespBulkString = "".as_bytes().into();
        let frame: RespFrame = bs.into();
        assert_eq!(frame.encode(), b"$0\r\n\r\n");
    }

    ///Null bulk strings: $-1\r\n
    #[test]
    fn encode_null_bulk_string_should_work() {
        let nbs = RespNullBulkString;
        let frame: RespFrame = nbs.into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    ///Arrays: *<number-of-elements>\r\n<element-1>...<element-n>
    ///         *0\r\n
    ///         *2\r\n$5\r\nhello\r\n$5\r\nworld\r\n
    ///         *3\r\n:1\r\n:2\r\n:3\r\n
    #[test]
    fn encode_resp_array_should_work() {
        let ss1: RespBulkString = "set".into();
        let frame1: RespFrame = ss1.into();
        let ss2: RespBulkString = "hello".into();
        let frame2: RespFrame = ss2.into();
        let ss3: RespBulkString = "world".into();
        let frame3: RespFrame = ss3.into();

        let vec = vec![frame1, frame2, frame3];
        let resp_array: RespArray = vec.into();
        let frame: RespFrame = resp_array.into();

        assert_eq!(
            frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );

        let ss1: SimpleString = "set".into();
        let frame1: RespFrame = ss1.into();
        let ss2: SimpleString = "hello".into();
        let frame2: RespFrame = ss2.into();
        let ss3: SimpleString = "world".into();
        let frame3: RespFrame = ss3.into();

        let vec = vec![frame1, frame2, frame3];
        let resp_array: RespArray = vec.into();
        let frame: RespFrame = resp_array.into();

        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "*3\r\n+set\r\n+hello\r\n+world\r\n"
        );
    }

    ///Null arrays: *-1\r\n
    #[test]
    fn encode_resp_null_array_should_work() {
        let rna = RespNullArray;
        let frame: RespFrame = rna.into();

        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    ///Nulls: _\r\n
    #[test]
    fn encode_resp_null_should_work() {
        let rn = RespNull;
        let frame: RespFrame = rn.into();

        assert_eq!(frame.encode(), b"_\r\n");
    }

    ///Booleans: #<t|f>\r\n
    #[test]
    fn encode_resp_booleans_should_work() {
        let rbt: RespBooleans = true.into();
        let frame: RespFrame = rbt.into();

        assert_eq!(frame.encode(), b"#t\r\n");

        let rbf: RespBooleans = false.into();
        let frame: RespFrame = rbf.into();

        assert_eq!(frame.encode(), b"#f\r\n");
    }

    ///Doubles: ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
    ///             ,1.23\r\n
    ///             ,inf\r\n
    ///              ,-inf\r\n
    ///             ,nan\r\n
    ///
    #[test]
    fn encode_resp_doubles_should_work() {
        let resp_double1: RespDoubles = 1.23.into();
        let frame: RespFrame = resp_double1.into();
        assert_eq!(frame.encode(), b",1.23\r\n");

        let resp_double2: RespDoubles = 1.23e11.into();
        let frame: RespFrame = resp_double2.into();
        assert_eq!(frame.encode(), b",inf\r\n");

        let resp_double3: RespDoubles = (-1.23e11).into();
        let frame: RespFrame = resp_double3.into();
        assert_eq!(frame.encode(), b",-inf\r\n");

        let resp_double4: RespDoubles = f64::NAN.into();
        let frame: RespFrame = resp_double4.into();
        assert_eq!(frame.encode(), b",nan\r\n");
    }

    ///Bulk errors: !<length>\r\n<error>\r\n
    #[test]
    fn encode_resp_bulk_errors_should_work() {
        let rbe: RespBulkErrors = "some error".into();
        let frame: RespFrame = rbe.into();

        assert_eq!(frame.encode(), b"!10\r\nsome error\r\n");
    }

    ///%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
    ///         %2\r\n
    ///         +first\r\n
    ///         :1\r\n
    ///         +second\r\n
    ///         :2\r\n
    #[test]
    fn encode_resp_maps_should_work() {
        let key1: String = "hello".into();
        let value1: SimpleString = "world".into();
        let value1: RespFrame = value1.into();
        let mut resp_map = RespMaps::default();
        resp_map.insert(key1, value1);
        let frame: RespFrame = resp_map.into();

        assert_eq!(frame.encode(), b"%1\r\n+hello\r\n:+world\r\n");

        let key2 = "A".to_string();
        let value2: RespDoubles = 1.23.into();
        let value2: RespFrame = value2.into();
        let key3 = "B".to_string();
        let value3: RespDoubles = (-1.23).into();
        let value3: RespFrame = value3.into();
        let mut resp_map = RespMaps::default();
        resp_map.insert(key2, value2);
        resp_map.insert(key3, value3);
        let frame: RespFrame = resp_map.into();

        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "%2\r\n+A\r\n:,1.23\r\n+B\r\n:,-1.23\r\n"
        );
    }

    ///Sets: ~<number-of-elements>\r\n<element-1>...<element-n>
    #[test]
    fn encode_resp_sets_should_work() {
        let element1: SimpleString = "element1".into();
        let element1: RespFrame = element1.into();
        let element2: RespInteger = 42.into();
        let element2: RespFrame = element2.into();
        let resp_set = RespSets(vec![element1, element2]);
        let frame: RespFrame = resp_set.into();

        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "~2+element1\r\n:+42\r\n"
        );

        let element3: RespDoubles = 3.33.into();
        let element3: RespFrame = element3.into();
        let element4: SimpleError = "error".into();
        let element4: RespFrame = element4.into();
        let resp_set = RespSets(vec![element3, element4]);
        let frame: RespFrame = resp_set.into();

        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "~2,3.33\r\n-Error error\r\n"
        );
    }
}

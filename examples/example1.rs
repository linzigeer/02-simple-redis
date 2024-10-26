pub fn main() {
    let s = String::from("周琳林");
    let x = &s[..3];
    println!("{}", x);
    // let y = &s[..2];
    //打印会报错，因为一个中文字符占三个字节，对s切片实际上是s的对字节序列进行切片，
    //而&s[..2]只切到了第一个中文字符的一部分，他不是一个合法有效的utf字符串
    // println!("{}", y);

    //同样地，这里调用在String上调用len()方法，计算的实际上是String的字节数组的长度
    let len = s.len();
    println!("{}", len);
}

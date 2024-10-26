pub fn main() {
    let s = "hello".to_string();
    let opt = Some(&s);
    let mut option = opt.cloned();
    println!("{:?}", option);
    let after_take = option.take();
    assert_eq!(after_take, Some("hello".into()));
    assert_eq!(option, None);
}

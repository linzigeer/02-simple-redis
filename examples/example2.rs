pub fn main() {
    let mut x = 1;
    for _ in 0..10 {
        let _y = x;
        x += 1;
    }

    println!("{x:#?}");
}

#[derive(Debug)]
pub enum Shape {
    Rectangle { width: u32, height: u32 },
    Triangle,
    Circle,
}
pub fn main() {
    // 创建实例
    let shape_a = Shape::Rectangle {
        width: 10,
        height: 20,
    };
    // 模式匹配出负载内容
    let Shape::Rectangle { width, height } = shape_a else {
        panic!("Can't extract rectangle.");
    };
    println!("width: {}, height: {}", width, height);
}

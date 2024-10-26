pub fn main() {
    let mut my_shape = Shape::Rectangle;
    let mut i = 0;

    while let Shape::Rectangle = my_shape {
        // while Shape::Rectangle == my_shape {
        // while let my_shape = Shape::Rectangle  {
        if i > 9 {
            println!("Greater than 9, quit!");
            my_shape = Shape::Circle;
        } else {
            println!("i:{i} less than 9, will try again!");
            i += 1;
        }
    }

    println!("my_shape:{my_shape:?}")
}

#[derive(Debug)]
pub enum Shape {
    Rectangle,
    Triangle,
    Circle,
}

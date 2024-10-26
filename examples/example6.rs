use anyhow::Result;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::num::ParseIntError;
use std::str::FromStr;
pub fn main() -> Result<()> {
    let rng = thread_rng();
    let scores = rng
        .sample_iter(Uniform::from(0..=100))
        .take(10)
        .collect::<Vec<_>>();
    println!("{:?}", scores);
    let diff = scores.windows(2).map(|x| x[1] - x[0]).collect::<Vec<_>>();
    println!("{:?}", diff);

    let s = " (1, 2)    ";
    let p = s.parse::<Point>()?;
    println!("{:?}", p);

    Ok(())
}

#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl FromStr for Point {
    type Err = ParseIntError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let ss = s
            .trim_matches(|x| x == '(' || x == ')' || x == ' ')
            .split(',')
            .collect::<Vec<_>>();
        let x = ss[0].trim().parse::<i32>()?;
        let y = ss[1].trim().parse::<i32>()?;

        Ok(Point::new(x, y))
    }
}

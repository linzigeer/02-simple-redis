// fn main(){
//     let a  = 3;
//     println!("{}", !a);//输出为-4
//     //1, 等价改写；2，按位取反；3，符号位不变，其余位取反；4，加1
//     //0000 0000 0000 0000 0000 0000 0000 0011
//     //1111 1111 1111 1111 1111 1111 1111 1100
//     //1000 0000 0000 0000 0000 0000 0000 0011
//     //1000 0000 0000 0000 0000 0000 0000 0100
//
//     //快速验证：!n = -n - 1
// }
// fn main() {
//     const L:i32 = 1;
//     println!("{}", L << 1 & 1);
// }
// fn main() {
//     let course = ("programming", "beginner", "course");
//     if let ("programming", c, "course") = course {
//         println!("{}", c);
//     } else {
//         println!("Value unmatched");
//     }
// }

// fn main() {
//     if (1 < 0) && (0 < -1) {
//         println!("Pass");
//     } else if (1 > 0) | false {
//         println!("Fail");
//     } else {
//         println!("Educative");
//     }
//     let ret = false | false;
//     println!("{}", ret);
// }

// fn main() {
//     let mut i = 1;
//     loop {
//         print!("{}", i);
//         if i == 5 {
//             break;
//         }
//         i = i + 1;
//     }
// }
// fn main() {
//     for i in 0..5 {
//         if i == 2 {
//             continue;
//         }
//         print!("{}", i);
//     }
// }

// use tokio_stream::{StreamExt, Stream};
// use std::time::Duration;
//
// #[tokio::main]
// async fn main() {
//     let mut stream = tokio_stream::unfold(0, |state| async move {
//         tokio::time::sleep(Duration::from_millis(500)).await;
//         Some((state, state + 1))
//     });
//
//     while let Some(value) = stream.next().await {
//         println!("Value: {}", value);
//     }
// }

use anyhow::{Context, Result};

pub fn main() {
    if let Err(e) = level1() {
        eprintln!("Error: {}", e);
        let mut source = e.source();
        while let Some(s) = source {
            eprintln!("Caused by: {}", s);
            source = s.source();
        }
    }
}

fn level1() -> Result<()> {
    level2().context("Error occurred in level1()")?;
    Ok(())
}

fn level2() -> Result<()> {
    level3().context("Error occurred in level2()")?;
    Ok(())
}

fn level3() -> Result<()> {
    std::fs::read_to_string("nonexistent_file.txt").context("Error occurred in level3()")?;
    Ok(())
}

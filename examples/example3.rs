use std::collections::HashMap;

pub fn main() {
    let mut map = HashMap::new();
    map.insert(
        "A".to_string(),
        vec!["c".to_string(), "b".to_string(), "a".to_string()],
    );
    map.insert(
        "B".to_string(),
        vec!["f".to_string(), "e".to_string(), "d".to_string()],
    );
    map.insert(
        "C".to_string(),
        vec!["g".to_string(), "i".to_string(), "h".to_string()],
    );

    show_map(&map);

    println!("{map:?}");

    sort_works(&mut map);

    println!("{map:?}");
}

fn show_map(map: &HashMap<String, Vec<String>>) {
    for (key, value) in map {
        print!("{key}-->");
        for v in value {
            print!("{v}");
        }
        println!();
    }
}

fn sort_works(map: &mut HashMap<String, Vec<String>>) {
    for works in map.values_mut() {
        works.sort();
    }
}

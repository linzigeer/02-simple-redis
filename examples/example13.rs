fn get_first(name: &(String, String)) -> &String {
    &name.0
}

pub fn main() {
    let mut name = (String::from("Ferris"), String::from("Rustacean"));
    let first = get_first(&name);
    println!("{}", first);
    name.1.push_str(", Esq.");
    println!("{}", name.1);
}

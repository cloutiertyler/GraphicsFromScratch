use std::fs;

fn main() {
    let contents = "Hello, world!";
    fs::write("./hello_world.txt", contents).unwrap();
}

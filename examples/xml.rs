use soupy::*;

const HELLO: &str = include_str!("hello.xml");

fn main() {
    let soup = Soup::xml(HELLO).expect("Failed to parse XML");

    println!("soup {:?}", soup);

    for node in soup.tag("simple") {
        println!("node {:?}", node);
    }

    for node in soup.tag("complex") {
        for node in node.tag("nested") {
            println!("nested node {:?}", node);
        }
    }
}

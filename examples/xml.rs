use soupy::*;

const HELLO: &str = include_str!("hello.xml");

fn main() {
    let soup = Soup::xml(HELLO.as_bytes()).expect("Failed to parse XML");

    println!("soup {:?}", soup);

    for node in soup.tag("simple") {
        println!("node {:?}", node);
    }

    for node in soup.tag("complex") {
        println!("Complex {:?}", node);
        println!("Complex query {:?}", node.query());

        let q = node.query();

        for node in &q {
            println!("nested {:?}", node);
        }
    }

    for node in soup.tag("complex") {
        println!("Complex 2 {:?}", node);

        for node in node.query().tag("nested") {
            println!("nested inline {:?}", node);
        }
    }
}

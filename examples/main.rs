use soupy::*;

const HELLO: &str = include_str!("hello.html");

fn main() {
    let soup = Soup::html(HELLO);

    println!("soup {:?}", soup);

    for node in soup.tag("a").attr_name("href") {
        println!("href {:?}", node.get("href"));
    }

    for node in soup.tag("p") {
        println!("paragraph {:?}", node);
    }

    if let Some(item) = soup.attr("id", "item").first() {
        println!("Found item {:?}", item);
    }
}

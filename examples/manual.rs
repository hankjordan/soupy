use soupy::{
    filter::{
        And,
        Attr,
        Tag,
    },
    parser::LenientHTMLParser,
    query::Query,
    *,
};

const HELLO: &str = include_str!("hello.html");

fn main() {
    let soup = Soup::html(HELLO);

    println!("soup {:?}", soup);

    let q1 = Query {
        filter: And(Tag { tag: "a" }, Attr {
            name: "href",
            value: true,
        }),
        soup: &soup,
        parser: std::marker::PhantomData::<LenientHTMLParser>,
    };

    let q2 = soup.tag("a");

    for node in q1 {
        println!("href {:?}", node.get("href"));
    }
}

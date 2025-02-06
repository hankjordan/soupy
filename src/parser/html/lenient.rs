use std::{
    convert::Infallible,
    marker::PhantomData,
};

use crate::parser::{
    html::HTMLNode,
    Parser,
};

/// Lenient HTML parser
///
/// Attempts to work through invalid HTML.
#[derive(Clone, Debug)]
pub struct LenientHTMLParser<S> {
    _marker: PhantomData<S>,
}

impl<S> Parser for LenientHTMLParser<S>
where
    S: AsRef<str>,
{
    type Input = S;
    type Node = HTMLNode<scraper::StrTendril>;
    type Error = Infallible;

    fn parse(text: S) -> Result<Vec<Self::Node>, Self::Error> {
        Ok(scraper::Html::parse_document(text.as_ref())
            .tree
            .root()
            .children()
            .filter_map(|n| n.try_into().ok())
            .collect())
    }
}

#[allow(clippy::mutable_key_type)]
impl<'a> TryFrom<ego_tree::NodeRef<'a, scraper::Node>> for HTMLNode<scraper::StrTendril> {
    type Error = ();

    fn try_from(node: ego_tree::NodeRef<'a, scraper::Node>) -> Result<Self, Self::Error> {
        match node.value() {
            scraper::Node::Document
            | scraper::Node::Fragment
            | scraper::Node::ProcessingInstruction(_) => Err(()),
            scraper::Node::Doctype(doctype) => Ok(HTMLNode::Doctype(doctype.name.clone())),
            scraper::Node::Comment(comment) => Ok(HTMLNode::Comment(comment.comment.clone())),
            scraper::Node::Text(text) => Ok(HTMLNode::Text(text.text.clone())),
            scraper::Node::Element(element) => {
                let name = element.name().into();
                let attrs = element.attrs().map(|(k, v)| (k.into(), v.into())).collect();

                Ok(match element.name() {
                    "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link"
                    | "meta" | "source" | "track" | "wbr" => HTMLNode::Void { name, attrs },
                    _ => HTMLNode::Element {
                        name,
                        attrs,
                        children: node.children().filter_map(|e| e.try_into().ok()).collect(),
                    },
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::HTMLNode,
        *,
    };

    const HELLO: &str = r#"
<!DOCTYPE html>
<html lang="en">

    <head>
        <meta charset="UTF-8"/>
        <title>Hello!</title>
    </head>

    <body>
        <h1>Hello World!</h1>
        <h2>Sub-heading</h2>
        <p>This is a simple paragraph.</p>

        <div class="parent">
            <div class="child">
                <div id="item">
                    <p>Nested item</p>
                    <a>Broken Link</a>
                    <a href="https://example.com">Example Link</a>
                </div>
            </div>
        </div>

        <h3>Footer heading</h3>

        <a href="https://other.com">Other Link</a>
    </body>

    <img class="self-closing"/>

    <!-- Simple comment -->

</html>"#;

    #[test]
    fn test_lenient_patterns() {
        let soup = Soup::html(HELLO);

        assert_eq!(
            soup.tag("img")
                .first()
                .and_then(|t| t.get("class").cloned()),
            Some("self-closing".into())
        );

        assert_eq!(
            soup.tag("img".to_string())
                .first()
                .and_then(|t| t.get("class").cloned()),
            Some("self-closing".into())
        );

        let regex = regex::Regex::new("^h[0-9]").expect("Failed to compile regex");

        let mut headings = soup.tag(regex).all();

        assert_eq!(
            headings.next().and_then(|h| h.name().cloned()),
            Some("h1".into())
        );
        assert_eq!(
            headings.next().and_then(|h| h.name().cloned()),
            Some("h2".into())
        );
        assert_eq!(
            headings.next().and_then(|h| h.name().cloned()),
            Some("h3".into())
        );
        assert_eq!(headings.next().and_then(|h| h.name().cloned()), None);
    }

    #[test]
    fn test_whitespace() {
        let soup = Soup::html("<a> </a>");
        assert_eq!(soup.nodes, [HTMLNode::Element {
            name: "html".into(),
            attrs: [].into(),
            children: [
                HTMLNode::Element {
                    name: "head".into(),
                    attrs: [].into(),
                    children: [].into(),
                },
                HTMLNode::Element {
                    name: "body".into(),
                    attrs: [].into(),
                    children: [HTMLNode::Element {
                        name: "a".into(),
                        attrs: [].into(),
                        children: [HTMLNode::Text(" ".into())].into(),
                    }]
                    .into(),
                }
            ]
            .into()
        }]);

        let soup = Soup::html(
            r#"
<!DOCTYPE html>
<html lang="en">

    <head>
        <meta charset="UTF-8"/>
        <title>Hello!</title>
    </head>

    <body>
        <a>Link</a>
        <span> </span>
    </body>
</html>"#,
        );
        assert_eq!(soup.nodes, [
            HTMLNode::Doctype("html".into()),
            HTMLNode::Element {
                name: "html".into(),
                attrs: [("lang".into(), "en".into())].into(),
                children: [
                    HTMLNode::Element {
                        name: "head".into(),
                        attrs: [].into(),
                        children: [
                            HTMLNode::Text("\n        ".into()),
                            HTMLNode::Void {
                                name: "meta".into(),
                                attrs: [("charset".into(), "UTF-8".into())].into()
                            },
                            HTMLNode::Text("\n        ".into()),
                            HTMLNode::Element {
                                name: "title".into(),
                                attrs: [].into(),
                                children: [HTMLNode::Text("Hello!".into())].into()
                            },
                            HTMLNode::Text("\n    ".into()),
                        ]
                        .into(),
                    },
                    HTMLNode::Text("\n\n    ".into()),
                    HTMLNode::Element {
                        name: "body".into(),
                        attrs: [].into(),
                        children: [
                            HTMLNode::Text("\n        ".into()),
                            HTMLNode::Element {
                                name: "a".into(),
                                attrs: [].into(),
                                children: [HTMLNode::Text("Link".into())].into(),
                            },
                            HTMLNode::Text("\n        ".into()),
                            HTMLNode::Element {
                                name: "span".into(),
                                attrs: [].into(),
                                children: [HTMLNode::Text(" ".into())].into(),
                            },
                            HTMLNode::Text("\n    \n".into())
                        ]
                        .into(),
                    }
                ]
                .into()
            }
        ]);
    }

    #[test]
    fn test_escaping() {
        let soup = Soup::html(r#"<a href="&#x2F;index.html">Hello &amp; Goodbye!</a>"#);
        assert_eq!(soup.nodes, [HTMLNode::Element {
            name: "html".into(),
            attrs: [].into(),
            children: [
                HTMLNode::Element {
                    name: "head".into(),
                    attrs: [].into(),
                    children: [].into(),
                },
                HTMLNode::Element {
                    name: "body".into(),
                    attrs: [].into(),
                    children: [HTMLNode::Element {
                        name: "a".into(),
                        attrs: [("href".into(), "/index.html".into())].into(),
                        children: [HTMLNode::Text("Hello & Goodbye!".into())].into(),
                    }]
                    .into(),
                }
            ]
            .into()
        }]);
    }
}

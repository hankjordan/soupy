use nom::{
    branch::alt,
    bytes::complete::{
        is_not,
        tag,
        tag_no_case,
        take_until,
    },
    character::complete::{
        alphanumeric1,
        char,
        multispace0,
    },
    combinator::map,
    multi::many0,
    sequence::{
        delimited,
        pair,
        preceded,
        separated_pair,
        terminated,
        tuple,
    },
    IResult,
    Parser,
};

use crate::HTMLNode;

/// Default HTML parser.
///
/// Errors on malformed HTML.
#[derive(Clone, Debug)]
pub struct StrictHTMLParser;

impl<'a> crate::parser::Parser<'a> for StrictHTMLParser {
    type Text = &'a str;
    type Node = HTMLNode<Self::Text>;
    type Error = nom::Err<nom::error::Error<&'a str>>;

    fn parse(text: &'a str) -> Result<Vec<Self::Node>, Self::Error> {
        nom::combinator::all_consuming(parse)(text).map(|r| r.1)
    }
}

fn attr<'a, E>(i: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: nom::error::ParseError<&'a str>,
{
    is_not(r#" "'>/="#)(i)
}

fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn take_to<'a, E: nom::error::ParseError<&'a str>>(
    i: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E> {
    terminated(take_until(i), tag(i))
}

fn comment(i: &str) -> IResult<&str, HTMLNode<&str>> {
    map(preceded(tag("<!--"), take_to("-->")), HTMLNode::Comment)(i)
}

fn doctype(i: &str) -> IResult<&str, HTMLNode<&str>> {
    map(
        preceded(tag_no_case("<!doctype "), take_to(">")),
        HTMLNode::Doctype,
    )(i)
}

fn start_tag<'a, F: 'a, E: 'a>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, (&'a str, Vec<(&'a str, &'a str)>, bool), E>
where
    F: Parser<&'a str, &'a str, E>,
    E: nom::error::ParseError<&'a str>,
{
    preceded(
        tag("<"),
        tuple((
            inner,
            many0(preceded(
                multispace0,
                alt((
                    // unquoted
                    separated_pair(attr, ws(char('=')), is_not(r#"\t\n\f\r "'=<>`"#)),
                    // quoted
                    separated_pair(
                        attr,
                        ws(char('=')),
                        alt((
                            delimited(char('\''), take_until("'"), char('\'')),
                            delimited(char('"'), take_until("\""), char('"')),
                        )),
                    ),
                    // boolean
                    pair(attr, |i| Ok((i, ""))),
                )),
            )),
            preceded(
                multispace0,
                alt((map(tag("/>"), |_| true), map(tag(">"), |_| false))),
            ),
        )),
    )
}

fn void(i: &str) -> IResult<&str, HTMLNode<&str>> {
    map(
        start_tag(alt((
            tag_no_case("area"),
            tag_no_case("base"),
            tag_no_case("br"),
            tag_no_case("col"),
            tag_no_case("embed"),
            tag_no_case("hr"),
            tag_no_case("img"),
            tag_no_case("input"),
            tag_no_case("link"),
            tag_no_case("meta"),
            tag_no_case("source"),
            tag_no_case("track"),
            tag_no_case("wbr"),
        ))),
        |(name, attrs, _)| HTMLNode::Void {
            name,
            attrs: attrs.into_iter().collect(),
        },
    )(i)
}

fn raw_element(i: &str) -> IResult<&str, HTMLNode<&str>> {
    let start = start_tag(alt((tag_no_case("script"), tag_no_case("style"))))(i)?;

    let (left, (name, attrs, closed)) = start;

    if closed {
        return Ok((left, HTMLNode::RawElement {
            name,
            attrs: attrs.into_iter().collect(),
            content: "",
        }));
    }

    let (left, content) = terminated(
        take_until(&*format!("</{name}")),
        delimited(
            tag("</"),
            tag_no_case(name),
            preceded(multispace0, char('>')),
        ),
    )(left)?;

    Ok((left, HTMLNode::RawElement {
        name,
        attrs: attrs.into_iter().collect(),
        content: content.trim(),
    }))
}

fn element(i: &str) -> IResult<&str, HTMLNode<&str>> {
    let start = start_tag(alphanumeric1)(i)?;

    let (left, (name, attrs, closed)) = start;

    if closed {
        return Ok((left, HTMLNode::Element {
            name,
            attrs: attrs.into_iter().collect(),
            children: vec![],
        }));
    }

    let (left, children) = terminated(
        parse,
        delimited(
            tag("</"),
            tag_no_case(name),
            preceded(multispace0, char('>')),
        ),
    )(left)?;

    Ok((left, HTMLNode::Element {
        name,
        attrs: attrs.into_iter().collect(),
        children,
    }))
}

fn text(i: &str) -> IResult<&str, HTMLNode<&str>> {
    map(map(is_not("<"), str::trim), HTMLNode::Text)(i)
}

fn single(i: &str) -> IResult<&str, HTMLNode<&str>> {
    alt((comment, doctype, void, raw_element, element, text))(i)
}

pub(crate) fn parse(i: &str) -> IResult<&str, Vec<HTMLNode<&str>>> {
    many0(ws(single))(i)
}

#[allow(clippy::too_many_lines)]
#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_comment() {
        assert_eq!(
            comment("<!-- Hello, world! -->"),
            Ok(("", HTMLNode::Comment(" Hello, world! ")))
        );
        assert_eq!(
            comment("<!--My favorite operators are > and <!-->"),
            Ok(("", HTMLNode::Comment("My favorite operators are > and <!")))
        );
    }

    #[test]
    fn test_doctype() {
        assert_eq!(
            doctype("<!DOCTYPE html>"),
            Ok(("", HTMLNode::Doctype("html")))
        );
        assert_eq!(
            doctype("<!doctype html>"),
            Ok(("", HTMLNode::Doctype("html")))
        );
        assert_eq!(
            doctype(r#"<!DOCTYPE html SYSTEM "about:legacy-compat">"#),
            Ok((
                "",
                HTMLNode::Doctype(r#"html SYSTEM "about:legacy-compat""#)
            ))
        );
    }

    #[test]
    fn test_void() {
        assert_eq!(
            void("<hr>"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: BTreeMap::new()
            }))
        );
        assert_eq!(
            void("<HR>"),
            Ok(("", HTMLNode::Void {
                name: "HR",
                attrs: BTreeMap::new()
            }))
        ); // TODO: convert to lowercase
        assert_eq!(
            void("<hr/>"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: BTreeMap::new()
            }))
        );
        assert_eq!(
            void("<hr >"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: BTreeMap::new()
            }))
        );
        assert_eq!(
            void("<hr />"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: BTreeMap::new()
            }))
        );

        assert_eq!(
            void("<hr value=yes>"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("value", "yes")].into()
            }))
        );
        assert_eq!(
            void("<hr value=yes >"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("value", "yes")].into()
            }))
        );
        assert_eq!(
            void("<hr value  = yes >"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("value", "yes")].into()
            }))
        );

        assert_eq!(
            void(r#"<hr value="yes">"#),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("value", "yes")].into()
            }))
        );
        assert_eq!(
            void(r#"<hr value= "yes" >"#),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("value", "yes")].into()
            }))
        );
        assert_eq!(
            void(r#"<hr value  ="yes">"#),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("value", "yes")].into()
            }))
        );

        assert_eq!(
            void("<hr value='yes'>"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("value", "yes")].into()
            }))
        );
        assert_eq!(
            void("<hr value='yes' >"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("value", "yes")].into()
            }))
        );
        assert_eq!(
            void("<hr value  = 'yes' >"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("value", "yes")].into()
            }))
        );

        assert_eq!(
            void("<hr disabled>"),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [("disabled", "")].into()
            }))
        );

        assert_eq!(
            void(r#"<hr value="yes" next='good' final=ok boolean>"#),
            Ok(("", HTMLNode::Void {
                name: "hr",
                attrs: [
                    ("value", "yes"),
                    ("next", "good"),
                    ("final", "ok"),
                    ("boolean", "")
                ]
                .into()
            }))
        );
    }

    #[test]
    fn test_element() {
        assert_eq!(
            element("<a/>"),
            Ok(("", HTMLNode::Element {
                name: "a",
                attrs: [].into(),
                children: [].into()
            }))
        );
        assert_eq!(
            element("<a></a>"),
            Ok(("", HTMLNode::Element {
                name: "a",
                attrs: [].into(),
                children: [].into()
            }))
        );
        assert_eq!(
            element(r#"<a rel=""></a>"#),
            Ok(("", HTMLNode::Element {
                name: "a",
                attrs: [("rel", "")].into(),
                children: [].into()
            }))
        );
        assert_eq!(
            element(r#"<a href="https://example.com"></a>"#),
            Ok(("", HTMLNode::Element {
                name: "a",
                attrs: [("href", "https://example.com")].into(),
                children: [].into()
            }))
        );
        assert_eq!(
            element(r#"<a href="https://example.com">Example Link</a>"#),
            Ok(("", HTMLNode::Element {
                name: "a",
                attrs: [("href", "https://example.com")].into(),
                children: [HTMLNode::Text("Example Link")].into()
            }))
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            parse("<!-- Hello --><!doctype html><!-- second -->"),
            Ok(("", vec![
                HTMLNode::Comment(" Hello "),
                HTMLNode::Doctype("html"),
                HTMLNode::Comment(" second ")
            ]))
        );

        assert_eq!(
            parse("\t\t<!-- Hello -->\n\t<!doctype html>\n<!-- second -->"),
            Ok(("", vec![
                HTMLNode::Comment(" Hello "),
                HTMLNode::Doctype("html"),
                HTMLNode::Comment(" second ")
            ]))
        );

        assert_eq!(
            parse(
                r#"
                <!--Here's a link.-->
                <a href="https://example.com"/>
                With some text.
            "#
            ),
            Ok(("", vec![
                HTMLNode::Comment("Here's a link."),
                HTMLNode::Element {
                    name: "a",
                    attrs: [("href", "https://example.com")].into(),
                    children: [].into()
                },
                HTMLNode::Text("With some text.")
            ])),
        );

        assert_eq!(
            parse(
                r#"
                <div class="outer">
                    <div class="inner">
                        <p>Hello, world!</p>
                    </div>
                </div>
            "#
            ),
            Ok(("", vec![HTMLNode::Element {
                name: "div",
                attrs: [("class", "outer")].into(),
                children: vec![HTMLNode::Element {
                    name: "div",
                    attrs: [("class", "inner")].into(),
                    children: vec![HTMLNode::Element {
                        name: "p",
                        attrs: [].into(),
                        children: vec![HTMLNode::Text("Hello, world!")],
                    }],
                }],
            }])),
        );

        assert_eq!(
            parse(
                r#"
<script type="application/javascript">
if (1 < 2) {
    console.log("Hello, world!");
}
</script>
<div class="outer">
    <div class="inner">
        <p>Hello, world!</p>
        <p>Another element...</p>
        Just some text...
    </div>
    <div>
        <p>Fancy nesting</p>
    </div>
</div>
"#
            ),
            Ok(("", vec![
                HTMLNode::RawElement {
                    name: "script",
                    attrs: [("type", "application/javascript")].into(),
                    content: "if (1 < 2) {\n    console.log(\"Hello, world!\");\n}",
                },
                HTMLNode::Element {
                    name: "div",
                    attrs: [("class", "outer")].into(),
                    children: vec![
                        HTMLNode::Element {
                            name: "div",
                            attrs: [("class", "inner")].into(),
                            children: vec![
                                HTMLNode::Element {
                                    name: "p",
                                    attrs: [].into(),
                                    children: vec![HTMLNode::Text("Hello, world!")],
                                },
                                HTMLNode::Element {
                                    name: "p",
                                    attrs: [].into(),
                                    children: vec![HTMLNode::Text("Another element...")],
                                },
                                HTMLNode::Text("Just some text...")
                            ],
                        },
                        HTMLNode::Element {
                            name: "div",
                            attrs: [].into(),
                            children: vec![HTMLNode::Element {
                                name: "p",
                                attrs: [].into(),
                                children: vec![HTMLNode::Text("Fancy nesting")],
                            }]
                        }
                    ],
                }
            ])),
        );
    }
}

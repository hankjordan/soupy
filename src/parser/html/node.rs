use std::collections::BTreeMap;

use crate::node::Node;

/// An HTML node
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HTMLNode<S> {
    /// A comment, like `<!-- ... -->`
    Comment(S),
    /// The doctype, like `<!DOCTYPE ...>`
    Doctype(S),
    /// A standard element, like `<p> ... </p>`
    Element {
        /// Name
        name: S,
        /// Attributes
        attrs: BTreeMap<S, S>,
        /// Direct children
        children: Vec<HTMLNode<S>>,
    },
    /// An element that contains code, like `<script> ... </script>`
    RawElement {
        /// Name
        name: S,
        /// Attributes
        attrs: BTreeMap<S, S>,
        /// Raw content contained by the element
        content: S,
    },
    /// A void element that is unable to contain children, like `<input>`
    Void {
        /// Name
        name: S,
        /// Attributes
        attrs: BTreeMap<S, S>,
    },
    /// Raw text
    Text(S),
}

impl<S> Node for HTMLNode<S> {
    type Text = S;

    fn name(&self) -> Option<&S> {
        match self {
            Self::Element { name, .. }
            | Self::RawElement { name, .. }
            | Self::Void { name, .. } => Some(name),
            _ => None,
        }
    }

    fn text(&self) -> Option<&S> {
        match self {
            Self::Text(t) => Some(t),
            _ => None,
        }
    }

    fn attrs(&self) -> Option<&BTreeMap<S, S>> {
        match self {
            Self::Element { attrs, .. }
            | Self::RawElement { attrs, .. }
            | Self::Void { attrs, .. } => Some(attrs),
            _ => None,
        }
    }

    fn children(&self) -> &[Self] {
        if let Self::Element { children, .. } = &self {
            children.as_slice()
        } else {
            &[]
        }
    }
}

impl<S> HTMLNode<S> {
    /// Iterate over direct children
    pub fn iter(&self) -> std::slice::Iter<Self> {
        self.children().iter()
    }
}

impl<'a, S> IntoIterator for &'a HTMLNode<S> {
    type Item = &'a HTMLNode<S>;
    type IntoIter = std::slice::Iter<'a, HTMLNode<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::*;

    const HELLO: &str = r#"
<!DOCTYPE html>
<html lang="en">

    <head>
        <meta charset="UTF-8"/>
        <title>Hello!</title>
    </head>

    <body>
        <h1>Hello World!</h1>
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

        <a href="https://other.com">Other Link</a>
    </body>

    <img class="self-closing"/>

    <!-- Simple comment -->

</html>"#;

    #[test]
    fn test_tree_iter() {
        let soup = Soup::html_strict(HELLO).expect("Failed to parse HTML");

        let body = soup
            .tag("body")
            .first()
            .expect("Could not find body tag")
            .deref()
            .clone();

        let mut nodes = body.descendants();

        assert_eq!(nodes.next().unwrap().name(), Some(&"body".into()));

        assert_eq!(nodes.next().unwrap(), &HTMLNode::Text("\n        ".into()));

        assert_eq!(nodes.next().unwrap(), &HTMLNode::Element {
            name: "h1".into(),
            attrs: [].into(),
            children: vec![HTMLNode::Text("Hello World!".into())]
        });

        assert_eq!(
            nodes.next().unwrap(),
            &HTMLNode::Text("Hello World!".into())
        );

        assert_eq!(nodes.next().unwrap(), &HTMLNode::Text("\n        ".into()));

        assert_eq!(nodes.next().unwrap(), &HTMLNode::Element {
            name: "p".into(),
            attrs: [].into(),
            children: vec![HTMLNode::Text("This is a simple paragraph.".into())]
        });
    }

    #[test]
    fn test_direct_iter() {
        let soup = Soup::html_strict(HELLO).expect("Failed to parse HTML");

        let body = soup
            .tag("body")
            .first()
            .expect("Could not find body tag")
            .deref()
            .clone();

        let mut nodes = body.into_iter();

        assert_eq!(nodes.next().unwrap(), &HTMLNode::Text("\n        ".into()));

        assert_eq!(nodes.next().unwrap(), &HTMLNode::Element {
            name: "h1".into(),
            attrs: [].into(),
            children: vec![HTMLNode::Text("Hello World!".into())]
        });

        assert_eq!(nodes.next().unwrap(), &HTMLNode::Text("\n        ".into()));

        assert_eq!(nodes.next().unwrap(), &HTMLNode::Element {
            name: "p".into(),
            attrs: [].into(),
            children: vec![HTMLNode::Text("This is a simple paragraph.".into())]
        });
    }

    #[test]
    fn test_iter_order() {
        let soup = Soup::html_strict(HELLO).expect("Failed to parse HTML");

        let soup = soup
            .tag("body")
            .first()
            .expect("Could not find body tag")
            .query();

        // By default, the data is searched recursively, depth-first.
        assert_eq!(
            soup.tag("a").first().map(|t| t.all_text()),
            Some("Broken Link".into())
        );

        // Strict queries only match direct children, no recursion.
        assert_eq!(
            soup.strict().tag("a").first().map(|t| t.all_text()),
            Some("Other Link".into())
        );
    }
}

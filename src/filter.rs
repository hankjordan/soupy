use crate::{
    HTMLNode,
    Pattern,
};

/// Applied by [`Query`](`crate::query::Query`) to find matching elements
pub trait Filter<N> {
    /// Matches the `Filter` with the [`Node`]
    fn matches(&self, node: &N) -> bool;
}

impl<N> Filter<N> for () {
    fn matches(&self, _: &N) -> bool {
        true
    }
}

/// Returns `true` if `A && B`
pub struct And<A, B>(pub A, pub B);

impl<N, A, B> Filter<N> for And<A, B>
where
    A: Filter<N>,
    B: Filter<N>,
{
    fn matches(&self, node: &N) -> bool {
        self.0.matches(node) && self.1.matches(node)
    }
}

/// Returns `true` if `A || B`
pub struct Or<A, B>(pub A, pub B);

impl<N, A, B> Filter<N> for Or<A, B>
where
    A: Filter<N>,
    B: Filter<N>,
{
    fn matches(&self, node: &N) -> bool {
        self.0.matches(node) || self.1.matches(node)
    }
}

/// Filters elements by attribute
pub struct Attr<N, V> {
    pub name: N,
    pub value: V,
}

impl<'a, N, V> Filter<HTMLNode<'a>> for Attr<N, V>
where
    N: Pattern,
    V: Pattern,
{
    fn matches(&self, node: &HTMLNode) -> bool {
        if let Some(attrs) = match node {
            HTMLNode::Element { attrs, .. }
            | HTMLNode::RawElement { attrs, .. }
            | HTMLNode::Void { attrs, .. } => Some(attrs),
            _ => None,
        } {
            if let Some(name) = self.name.as_str() {
                if let Some(value) = attrs.get(name) {
                    self.value.matches(value)
                } else {
                    false
                }
            } else {
                for (name, value) in attrs {
                    if self.name.matches(name) && self.value.matches(value) {
                        return true;
                    }
                }

                false
            }
        } else {
            false
        }
    }
}

#[cfg(feature = "lenient")]
impl<N, V> Filter<scraper::Node> for Attr<N, V>
where
    N: Pattern,
    V: Pattern,
{
    fn matches(&self, node: &scraper::Node) -> bool {
        if let Some(attrs) = match node {
            scraper::Node::Element(e) => Some(e.attrs()),
            _ => None,
        } {
            for (name, value) in attrs {
                if self.name.matches(name) && self.value.matches(value) {
                    return true;
                }
            }

            false
        } else {
            false
        }
    }
}

/// Filters elements by tag
pub struct Tag<P> {
    pub tag: P,
}

impl<'a, P> Filter<HTMLNode<'a>> for Tag<P>
where
    P: Pattern,
{
    fn matches(&self, node: &HTMLNode) -> bool {
        if let Some(pattern) = self.tag.as_bool() {
            pattern
        } else if let Some(name) = match node {
            HTMLNode::Element { name, .. }
            | HTMLNode::RawElement { name, .. }
            | HTMLNode::Void { name, .. } => Some(name),
            _ => None,
        } {
            self.tag.matches(name)
        } else {
            false
        }
    }
}

#[cfg(feature = "lenient")]
impl<P> Filter<scraper::Node> for Tag<P>
where
    P: Pattern,
{
    fn matches(&self, node: &scraper::Node) -> bool {
        if let Some(pattern) = self.tag.as_bool() {
            pattern
        } else if let Some(name) = match node {
            scraper::Node::Element(e) => Some(e.name()),
            _ => None,
        } {
            self.tag.matches(name)
        } else {
            false
        }
    }
}

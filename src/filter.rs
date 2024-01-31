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

impl<S, N, V> Filter<HTMLNode<S>> for Attr<N, V>
where
    S: Ord,
    N: Pattern<S>,
    V: Pattern<S>,
{
    fn matches(&self, node: &HTMLNode<S>) -> bool {
        if let Some(attrs) = node.attrs() {
            if let Some(name) = self.name.value() {
                if let Some(value) = attrs.get(&name) {
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

/// Filters elements by tag
pub struct Tag<P> {
    pub tag: P,
}

impl<S, P> Filter<HTMLNode<S>> for Tag<P>
where
    P: Pattern<S>,
{
    fn matches(&self, node: &HTMLNode<S>) -> bool {
        if let Some(bypass) = self.tag.bypass() {
            bypass
        } else if let Some(name) = node.name() {
            self.tag.matches(name)
        } else {
            false
        }
    }
}

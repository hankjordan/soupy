use crate::{
    Node,
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
    /// Attribute name pattern
    pub name: N,

    /// Attribute value pattern
    pub value: V,
}

impl<T, N, V> Filter<T> for Attr<N, V>
where
    T: Node,
    T::Text: Ord,
    N: Pattern<T::Text>,
    V: Pattern<T::Text>,
{
    fn matches(&self, node: &T) -> bool {
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
    /// Tag pattern
    pub tag: P,
}

impl<N, P> Filter<N> for Tag<P>
where
    N: Node,
    P: Pattern<N::Text>,
{
    fn matches(&self, node: &N) -> bool {
        if let Some(name) = node.name() {
            self.tag.matches(name)
        } else {
            false
        }
    }
}

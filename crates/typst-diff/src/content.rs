use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::slice;

use comemo::Prehashed;
use typst::foundations::Content;

#[derive(Clone, Debug)]
pub(crate) enum DiffableContent<'a> {
    Content(&'a Content),
    ContentSlice(Vec<&'a Content>),
}

impl<'a> DiffableContent<'a> {
    pub fn as_slice(&'a self) -> &'a [&'a Content] {
        match self {
            Self::Content(x) => slice::from_ref(x),
            Self::ContentSlice(vec) => vec,
        }
    }

    pub fn append(self, content: &'a Content) -> Self {
        let x = match self {
            Self::Content(x) => {
                vec![x, content]
            }
            Self::ContentSlice(mut x) => {
                x.push(content);
                x
            }
        };
        Self::ContentSlice(x)
    }
}

impl<'a> From<&'a Content> for DiffableContent<'a> {
    fn from(content: &'a Content) -> Self {
        Self::Content(content)
    }
}

impl<'a> From<&'a Prehashed<Content>> for DiffableContent<'a> {
    fn from(src: &'a Prehashed<Content>) -> Self {
        let content: &Content = src;
        Self::from(content)
    }
}

impl<'a> From<&'a [Content]> for DiffableContent<'a> {
    fn from(content: &'a [Content]) -> Self {
        let content: Vec<&'a Content> = content.iter().collect();
        Self::ContentSlice(content)
    }
}

impl<'a> From<&'a [&'a Content]> for DiffableContent<'a> {
    fn from(content: &'a [&'a Content]) -> Self {
        Self::ContentSlice(content.to_vec())
    }
}

impl<'a> Hash for DiffableContent<'a> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Content(content) => content.hash(state),
            Self::ContentSlice(content) => content.iter().for_each(|x| x.hash(state)),
        };
    }
}

impl<'a> PartialOrd for DiffableContent<'a> {
    fn partial_cmp(&self, other: &DiffableContent<'a>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for DiffableContent<'a> {
    fn cmp(&self, _other: &Self) -> Ordering {
        todo!("not needed until now")
    }
}

impl<'a> PartialEq for DiffableContent<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Content(content), Self::Content(other)) => {
                let content = content.plain_text();
                let other = other.plain_text();
                if content == other {
                    tracing::trace!("Content == Content:\ntext: {content}");
                    true
                } else {
                    tracing::trace!("Content != Content:\nleft: {content}\nright: {other}");
                    false
                }
            }
            (Self::Content(content), Self::ContentSlice(other)) => {
                let content = content.plain_text();
                let other: Vec<_> = other.iter().map(|x| x.plain_text()).collect();
                let other = other.concat();
                if content == other {
                    tracing::trace!("Content == ContentSlice:\ntext: {content}");
                    true
                } else {
                    tracing::trace!("Content != ContentSlice:\nleft: {content}\nright: {other}");
                    false
                }
            }
            (Self::ContentSlice(content), Self::Content(other)) => {
                let other = other.plain_text();
                let content: Vec<_> = content.iter().map(|x| x.plain_text()).collect();
                let content = content.concat();
                if content == other {
                    tracing::trace!("ContentSlice == Content:\ntext: {content}");
                    true
                } else {
                    tracing::trace!("ContentSlice != Content:\nleft: {content}\nright: {other}");
                    false
                }
            }
            (Self::ContentSlice(content), Self::ContentSlice(other)) => {
                let content: Vec<_> = content.iter().map(|x| x.plain_text()).collect();
                let content = content.concat();
                let other: Vec<_> = other.iter().map(|x| x.plain_text()).collect();
                let other = other.concat();
                if content == other {
                    tracing::trace!("ContentSlice == ContentSlice:\ntext: {content}");
                    true
                } else {
                    tracing::trace!(
                        "ContentSlice != ContentSlice:\nleft: {content}\nright: {other}"
                    );
                    false
                }
            }
        }
    }
}

impl<'a> Eq for DiffableContent<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    use tracing_test::traced_test;
    use typst::foundations::NativeElement;
    use typst::text::*;

    macro_rules! test_eq {
        ( $a:expr, [ $b:expr, $( $c:expr ),+ ] ) => {
            test_eq!($a, [ $b ]);
            test_eq!($a, [ $($c),+ ]);
        };
        ( $a:expr, [ $b:expr ] ) => {
            test_eq!($a, $b);
        };
        ( $a:expr, $b:expr, $($c:expr),+ ) => {
            test_eq!($a, $b);
            test_eq!($a, $($c),+);
            test_eq!($b, $($c),+);
        };
        ( $a:expr, $b:expr ) => {
            let da = DiffableContent::from($a);
            let db = DiffableContent::from($b);
            assert_eq!(da, db, "{a} == {b} not fulfilled", a = stringify!($a), b = stringify!($b));
            assert_eq!(db, da, "{b} == {a} not fulfilled", a = stringify!($a), b = stringify!($b));
            println!("{a} == {b} fulfilled", a = stringify!($a), b = stringify!($b));
        };
    }

    macro_rules! test_ne {
        ( $a:expr, [ $b:expr, $( $c:expr ),+ ] ) => {
            test_ne!($a, [ $b ]);
            test_ne!($a, [ $($c),+ ]);
        };
        ( $a:expr, [ $b:expr ] ) => {
            test_ne!($a, $b);
        };
        ( $a:expr, $b:expr, $($c:expr),+ ) => {
            test_ne!($a, $b);
            test_ne!($a, $($c),+);
            test_ne!($b, $($c),+);
        };
        ( $a:expr, $b:expr ) => {
            let da = DiffableContent::from($a);
            let db = DiffableContent::from($b);
            assert_ne!(da, db, "{a} != {b} not fulfilled", a = stringify!($a), b = stringify!($b));
            assert_ne!(db, da, "{b} != {a} not fulfilled", a = stringify!($a), b = stringify!($b));
            println!("{a} != {b} fulfilled", a = stringify!($a), b = stringify!($b));
        };
    }

    #[traced_test]
    #[test]
    fn two_empty_contents() {
        let e1: Content = Content::empty();
        let e2: Content = Content::empty();
        test_eq!(&e1, &e2);
    }

    #[traced_test]
    #[test]
    fn one_empty_content() {
        let e: Content = Content::empty();
        let a: Content = TextElem::packed("aaa");
        let b: Content = TextElem::packed("bbb");
        let c: Content = TextElem::packed("ccc");
        test_ne!(&e, [&a, &b, &c]);
    }

    #[traced_test]
    #[test]
    fn two_texts() {
        let a1: Content = Content::new(TextElem::new("aaa".into()));
        let a2: Content = TextElem::new("aaa".into()).pack();
        let a3: Content = TextElem::packed("aaa");
        let b1: Content = Content::new(TextElem::new("bbb".into()));
        let b2: Content = TextElem::new("bbb".into()).pack();
        let b3: Content = TextElem::packed("bbb");
        test_eq!(&a1, &a2, &a3);
        test_eq!(&b1, &b2, &b3);
        test_ne!(&b1, [&a1, &a2, &a3]);
        test_ne!(&a1, [&b1, &b2, &b3]);
    }

    #[traced_test]
    #[test]
    fn one_text() {
        let t: Content = TextElem::packed("aaa");
        let a: Content = UnderlineElem::new(t.clone()).pack();
        let b: Content = HighlightElem::new(t.clone()).pack();
        let c: Content = SuperElem::new(t.clone()).pack();
        let d: Content = SubElem::new(t.clone()).pack();
        let e: Content = OverlineElem::new(t.clone()).pack();
        let f: Content = StrikeElem::new(t.clone()).pack();
        test_eq!(&t, [&a, &b, &c, &d, &e, &f]);
    }

    #[traced_test]
    #[test]
    fn cascading_texts() {
        let t: Content = TextElem::packed("aaa");
        let a: Content = UnderlineElem::new(t.clone()).pack();
        let b: Content = HighlightElem::new(a.clone()).pack();
        let c: Content = SuperElem::new(b.clone()).pack();
        let d: Content = OverlineElem::new(c.clone()).pack();
        test_eq!(&t, [&a, &b, &c, &d]);
    }

    #[traced_test]
    #[test]
    fn compare_with_content_slices() {
        let a1: Content = TextElem::packed("aaabbb");
        let a2: Vec<Content> = vec![TextElem::packed("aaa"), TextElem::packed("bbb")];
        let a3: Vec<Content> = vec![TextElem::packed("aa"), TextElem::packed("abbb")];
        let a4: Vec<Content> = vec![
            TextElem::packed("aa"),
            TextElem::packed("ab"),
            TextElem::packed("bb"),
        ];
        let a5: Vec<Content> = vec![
            TextElem::packed("aa"),
            UnderlineElem::new(TextElem::packed("ab")).pack(),
            TextElem::packed("bb"),
        ];
        let a6: Content = UnderlineElem::new(a1.clone()).pack();
        test_eq!(
            &a1,
            a2.as_slice(),
            a3.as_slice(),
            a4.as_slice(),
            a5.as_slice(),
            &a6
        );
    }

    #[traced_test]
    #[test]
    fn space_element() {
        let a1: Content = TextElem::packed("aaa bbb");
        let a2: Vec<Content> = vec![
            TextElem::packed("aaa"),
            SpaceElem::new().pack(),
            TextElem::packed("bbb"),
        ];
        let a3: Vec<Content> = vec![
            TextElem::packed("aa"),
            TextElem::packed("a b"),
            TextElem::packed("bb"),
        ];
        let a4: Vec<Content> = vec![
            TextElem::packed("aa"),
            UnderlineElem::new(TextElem::packed("a b")).pack(),
            TextElem::packed("bb"),
        ];
        let a5: Vec<Content> = vec![
            TextElem::packed("aa"),
            UnderlineElem::new(TextElem::packed("a")).pack(),
            SpaceElem::new().pack(),
            UnderlineElem::new(TextElem::packed("b")).pack(),
            TextElem::packed("bb"),
        ];
        let a6: Content = UnderlineElem::new(a1.clone()).pack();
        test_eq!(
            &a1,
            a2.as_slice(),
            a3.as_slice(),
            a4.as_slice(),
            a5.as_slice(),
            &a6
        );
    }
}

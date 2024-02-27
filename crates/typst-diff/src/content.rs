use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use comemo::Prehashed;
use typst::foundations::Content;

#[derive(Clone, Debug)]
pub(crate) enum DiffableContent<'a> {
    Content(&'a Content),
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

impl<'a> Hash for DiffableContent<'a> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Content(content) => content.hash(state),
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
                content == other
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
}

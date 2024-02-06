use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use comemo::Prehashed;
use typst::foundations::Content;
use typst::text::*;

#[derive(Clone, Debug)]
pub(crate) struct DiffableContent<'a>(&'a Content);

impl<'a> DiffableContent<'a> {
    pub fn content(&self) -> &Content {
        self.0
    }

    fn dyn_eq(&self, other: &Content) -> bool {
        let content = self.0;

        /* Undicided elements:
         *  - SmartQuoteElem
         */
        if let Some(elem) = content.to::<UnderlineElem>() {
            DiffableContent::from(elem.body()).dyn_eq(other)
        } else if let Some(elem) = content.to::<OverlineElem>() {
            DiffableContent::from(elem.body()).dyn_eq(other)
        } else if let Some(elem) = content.to::<HighlightElem>() {
            DiffableContent::from(elem.body()).dyn_eq(other)
        } else if let Some(elem) = content.to::<SuperElem>() {
            DiffableContent::from(elem.body()).dyn_eq(other)
        } else if let Some(elem) = content.to::<SubElem>() {
            DiffableContent::from(elem.body()).dyn_eq(other)
        } else if let Some(elem) = content.to::<StrikeElem>() {
            DiffableContent::from(elem.body()).dyn_eq(other)
        } else {
            /* Elements to compare:
             *  - TextElem
             *  - LinebreakElem
             *  - SpaceElem
             *  - RawElem
             */
            content.eq(other)
        }
    }
}

impl<'a> From<&'a Content> for DiffableContent<'a> {
    fn from(src: &'a Content) -> Self {
        Self(src)
    }
}

impl<'a> From<&'a Prehashed<Content>> for DiffableContent<'a> {
    fn from(src: &'a Prehashed<Content>) -> Self {
        let content: &Content = src;
        Self(content)
    }
}

impl<'a> Hash for DiffableContent<'a> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
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
        self.dyn_eq(other.0) || other.dyn_eq(self.0)
    }
}

impl<'a> Eq for DiffableContent<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use typst::foundations::NativeElement;

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

    #[test]
    fn two_empty_contents() {
        let e1: Content = Content::empty();
        let e2: Content = Content::empty();
        test_eq!(&e1, &e2);
    }

    #[test]
    fn one_empty_content() {
        let e: Content = Content::empty();
        let a: Content = TextElem::packed("a");
        let b: Content = TextElem::packed("b");
        let c: Content = TextElem::packed("c");
        test_ne!(&e, [&a, &b, &c]);
    }

    #[test]
    fn two_texts() {
        let a1: Content = Content::new(TextElem::new("a".into()));
        let a2: Content = TextElem::new("a".into()).pack();
        let a3: Content = TextElem::packed("a");
        let b1: Content = Content::new(TextElem::new("b".into()));
        let b2: Content = TextElem::new("b".into()).pack();
        let b3: Content = TextElem::packed("b");
        test_eq!(&a1, &a2, &a3);
        test_eq!(&b1, &b2, &b3);
        test_ne!(&b1, [&a1, &a2, &a3]);
        test_ne!(&a1, [&b1, &b2, &b3]);
    }

    #[test]
    fn one_text() {
        let t: Content = TextElem::packed("a");
        let a: Content = UnderlineElem::new(t.clone()).pack();
        let b: Content = HighlightElem::new(t.clone()).pack();
        let c: Content = SuperElem::new(t.clone()).pack();
        let d: Content = SubElem::new(t.clone()).pack();
        let e: Content = OverlineElem::new(t.clone()).pack();
        let f: Content = StrikeElem::new(t.clone()).pack();
        test_eq!(&t, [&a, &b, &c, &d, &e, &f]);
    }

    #[test]
    fn cascading_texts() {
        let t: Content = TextElem::packed("a");
        let a: Content = UnderlineElem::new(t.clone()).pack();
        let b: Content = HighlightElem::new(a.clone()).pack();
        let c: Content = SuperElem::new(b.clone()).pack();
        let d: Content = OverlineElem::new(c.clone()).pack();
        test_eq!(&t, [&a, &b, &c, &d]);
    }
}

mod common;

use indoc::formatdoc;

#[test]
fn empty_doc() {
    test_diff!();
}

#[test]
fn empty_vs_one_line() {
    test_diff!("", "HeLLo World!");
}

#[test]
fn one_line_vs_empty() {
    test_diff!("Hello World!", "");
}

#[test]
fn one_line() {
    test_diff!("Hello World!", "HeLLo World!");
}

#[test]
fn one_vs_two_lines() {
    test_diff!(
        formatdoc!(r#"Hello World! I am here."#),
        formatdoc!(
            r#"Hello World!
               I am there."#
        )
    );
}

#[test]
fn two_lines() {
    test_diff!(
        formatdoc!(
            r#"Hello World!
               I am here."#
        ),
        formatdoc!(
            r#"Hello World!
               I am there."#
        )
    );
}

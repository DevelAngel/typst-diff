mod common;

use indoc::formatdoc;

#[test]
fn one_vs_two_lines() {
    test_diff!(
        formatdoc!(r#"Hello World! I am here."#),
        formatdoc!(
            r#"Hello World!
               I am here."#
        )
    );
}

#[test]
fn one_vs_three_lines() {
    test_diff!(
        formatdoc!(r#"Hello World! I am here. How are you?"#),
        formatdoc!(
            r#"Hello World!
               I am here.
               How are you?"#
        )
    );
}

#[test]
fn two_vs_three_lines() {
    test_diff!(
        formatdoc!(
            r#"Hello World!
               I am here. How are you?"#
        ),
        formatdoc!(
            r#"Hello World!
               I am here.
               How are you?"#
        )
    );
}

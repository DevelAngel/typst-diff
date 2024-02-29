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

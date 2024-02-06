/**
 * Compile test with:
 * env TMPDIR=$PWD/tmp TEST_PERSIST_FILES=1 cargo test -- --show-output
 */
#[macro_export]
macro_rules! test_diff {
    ( ) => {
        use crate::common::intern::diff;
        use stdext::function_name;
        diff(function_name!(), "", "");
    };
    ( $a:expr, $b:expr ) => {
        use crate::common::intern::diff;
        use stdext::function_name;
        diff(function_name!(), $a, $b);
    };
}

#[cfg(test)]
pub(crate) mod intern {
    use std::env;

    use assert_cmd::Command;
    use assert_fs::prelude::*;
    use predicates::prelude::*;

    #[allow(dead_code)]
    pub fn diff<T: AsRef<str>>(function_name: &'static str, text_a: T, text_b: T) {
        const ENV_PERSIST_FILES: &'static str = "TEST_PERSIST_FILES";
        let tmp_dir = assert_fs::TempDir::new()
            .expect("cannot create temporary directory")
            .into_persistent_if(env::var_os(ENV_PERSIST_FILES).is_some());

        let basename = function_name.replace("::", "-");
        let file_diff = format!("{basename}-dff.pdf");
        let file_a = format!("{basename}-a.typ");
        let file_b = format!("{basename}-b.typ");

        let file_diff = tmp_dir.child(file_diff);
        let file_a = tmp_dir.child(file_a);
        let file_b = tmp_dir.child(file_b);

        file_diff.assert(predicate::path::missing());
        file_a.assert(predicate::path::missing());
        file_b.assert(predicate::path::missing());

        let text_a = text_a.as_ref();
        if text_a.is_empty() {
            file_a.touch().unwrap();
        } else {
            file_a.write_str(text_a).unwrap();
        }

        let text_b = text_b.as_ref();
        if text_b.is_empty() {
            file_b.touch().unwrap();
        } else {
            file_b.write_str(text_b).unwrap();
        }

        file_diff.assert(predicate::path::missing());
        file_a.assert(predicate::path::is_file());
        file_b.assert(predicate::path::is_file());

        let mut cmd = Command::cargo_bin("typst-diff").unwrap();
        let assert = cmd
            .arg("compile")
            .arg(file_a.path())
            .arg(file_b.path())
            .arg(file_diff.path())
            .assert();
        assert.success();

        println!(
            "tmp dir of test {function_name}:\n{}",
            tmp_dir.path().display()
        );
    }
}

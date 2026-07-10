use ecci_editorconfig::Config;

use crate::Output;

pub fn check_trim_trailing_whitespace<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    if let Some(true) = config.trim_trailing_whitespace {
        let trimmed = content.trim_end_matches(['\r', '\n']);
        if trimmed.ends_with(' ') {
            output.output(
                line_number,
                trimmed.len() - 1,
                1,
                &config.path.to_string_lossy(),
                content,
                "trim_trailing_whitespace",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{check_all, MockOutput};

    #[test]
    fn check_trim_trailing_whitespace_no_error() {
        let target_path = "../../testdata/trim_trailing_whitespace/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_trim_trailing_whitespace_error() {
        let target_path = "../../testdata/trim_trailing_whitespace/error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 2
                    && *column == 2
                    && *length == 1
                    && path == target_path
                    && content == "b  \n"
                    && rule == "trim_trailing_whitespace"
            })
            .times(1)
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_trim_trailing_whitespace_error_on_whitespace_only_line() {
        let target_path = "../../testdata/trim_trailing_whitespace/empty_line/error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 2
                    && *column == 1
                    && *length == 1
                    && path == target_path
                    && content == "  \n"
                    && rule == "trim_trailing_whitespace"
            })
            .times(1)
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_trim_trailing_whitespace_false_keeps_whitespace() {
        let target_path = "../../testdata/trim_trailing_whitespace/false/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_trim_trailing_whitespace_uppercase_true() {
        let target_path = "../../testdata/trim_trailing_whitespace/uppercase/error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 1
                    && *column == 4
                    && *length == 1
                    && path == target_path
                    && content == "text \n"
                    && rule == "trim_trailing_whitespace"
            })
            .times(1)
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    #[ignore = "documents failure to handle unset in the EditorConfig parser"]
    fn check_trim_trailing_whitespace_unset_removes_inherited_effect() {
        let target_path = "../../testdata/trim_trailing_whitespace/unset/child/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    #[ignore = "documents missing support for tabs and other whitespace characters"]
    fn check_trim_trailing_whitespace_rejects_any_whitespace_before_newline() {
        let target_path =
            "../../testdata/trim_trailing_whitespace/specification_regressions/any_whitespace.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().times(4).return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    #[ignore = "documents incorrect handling of whitespace on a final line without a newline"]
    fn check_trim_trailing_whitespace_ignores_final_line_without_newline() {
        let target_path =
            "../../testdata/trim_trailing_whitespace/specification_regressions/final_line.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    #[ignore = "documents duplicate output with insert_final_newline on a final line without a newline"]
    fn check_trim_trailing_whitespace_does_not_conflict_with_insert_final_newline() {
        let target_path =
            "../../testdata/trim_trailing_whitespace/insert_final_newline_interaction/final_line.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 1
                    && *column == 31
                    && *length == 0
                    && path == target_path
                    && content == "last line has a trailing space "
                    && rule == "insert_final_newline"
            })
            .times(1)
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }
}

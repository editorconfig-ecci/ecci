use ecci_editorconfig::Config;

use crate::Output;

pub fn check_trim_trailing_whitespace<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    if let Some(true) = config.trim_trailing_whitespace {
        let newline_len = if content.ends_with("\r\n") {
            2
        } else if content.ends_with(['\r', '\n']) {
            1
        } else {
            return;
        };
        let line = &content[..content.len() - newline_len];
        let whitespace_start = line.trim_end_matches(char::is_whitespace).len();

        if whitespace_start != line.len() {
            output.output(
                line_number,
                whitespace_start,
                line.len() - whitespace_start,
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
                    && *column == 1
                    && *length == 2
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
                    && *column == 0
                    && *length == 2
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
    fn check_trim_trailing_whitespace_unset_removes_inherited_effect() {
        let target_path = "../../testdata/trim_trailing_whitespace/unset/child/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        assert_eq!(config.trim_trailing_whitespace, None);
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_trim_trailing_whitespace_rejects_any_whitespace_before_newline() {
        let target_path =
            "../../testdata/trim_trailing_whitespace/specification_regressions/any_whitespace.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        for (line_number, start, length, content) in [
            (1, 5, 1, "space \n"),
            (2, 0, 1, "\t\n"),
            (3, 0, 1, "\u{000b}\n"),
            (4, 0, 2, "\u{00a0}\n"),
        ] {
            mock.expect_output()
                .withf(
                    move |actual_line_number,
                          actual_start,
                          actual_length,
                          path,
                          actual_content,
                          rule| {
                        *actual_line_number == line_number
                            && *actual_start == start
                            && *actual_length == length
                            && path == target_path
                            && actual_content == content
                            && rule == "trim_trailing_whitespace"
                    },
                )
                .once()
                .return_const(());
        }
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
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

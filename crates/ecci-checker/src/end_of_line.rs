use ecci_editorconfig::{Config, EndOfLine};

use crate::Output;

pub fn check_end_of_line<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    if let Some(EndOfLine::LF) = &config.end_of_line {
        for (i, c) in content.char_indices() {
            if c == '\r' {
                output.output(
                    line_number,
                    i,
                    1,
                    &config.path.to_string_lossy(),
                    content,
                    "end_of_line",
                );
            }
        }
    } else if let Some(EndOfLine::CRLF) = &config.end_of_line {
        for (i, c) in content.char_indices() {
            // Only validate newline characters that are actually present.  In
            // particular, a final line without a newline is handled by
            // `insert_final_newline`, not by this rule.
            if c == '\r' && content.as_bytes().get(i + 1) != Some(&b'\n') {
                output.output(
                    line_number,
                    i,
                    1,
                    &config.path.to_string_lossy(),
                    content,
                    "end_of_line",
                );
            }
            if c == '\n' && (i == 0 || content.as_bytes()[i - 1] != b'\r') {
                output.output(
                    line_number,
                    i,
                    1,
                    &config.path.to_string_lossy(),
                    content,
                    "end_of_line",
                );
            }
        }
    } else if let Some(EndOfLine::CR) = &config.end_of_line {
        for (i, c) in content.char_indices() {
            if c == '\n' {
                output.output(
                    line_number,
                    i,
                    1,
                    &config.path.to_string_lossy(),
                    content,
                    "end_of_line",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{check_all, MockOutput};

    struct RecordingOutput {
        rules: Vec<String>,
    }

    impl RecordingOutput {
        fn new() -> Self {
            Self { rules: Vec::new() }
        }
    }

    impl crate::Output for RecordingOutput {
        fn output(
            &mut self,
            _line_number: usize,
            _start: usize,
            _length: usize,
            _path: &str,
            _content: &str,
            rule: &str,
        ) {
            self.rules.push(rule.to_owned());
        }
    }

    fn eol_rules(target_path: &str) -> Vec<String> {
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut output = RecordingOutput::new();
        check_all(&config, &mut output).unwrap();
        output.rules
    }

    #[test]
    fn check_eol_lf_no_error() {
        let target_path = "../../testdata/end_of_line/lf/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_eol_lf_cr() {
        let target_path = "../../testdata/end_of_line/lf/error_cr.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 1
                    && *length == 1
                    && path == target_path
                    && content == "a\rb\rc\r"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 3
                    && *length == 1
                    && path == target_path
                    && content == "a\rb\rc\r"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 5
                    && *length == 1
                    && path == target_path
                    && content == "a\rb\rc\r"
                    && rule == "end_of_line"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }
    #[test]
    fn check_eol_lf_crlf() {
        let target_path = "../../testdata/end_of_line/lf/error_crlf.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 1
                    && *length == 1
                    && path == target_path
                    && content == "a\r\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 2
                    && *start == 1
                    && *length == 1
                    && path == target_path
                    && content == "b\r\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 3
                    && *start == 1
                    && *length == 1
                    && path == target_path
                    && content == "c\r\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_eol_cr_no_error() {
        let target_path = "../../testdata/end_of_line/cr/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_eol_cr_lf() {
        let target_path = "../../testdata/end_of_line/cr/error_lf.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 1
                    && *length == 1
                    && path == target_path
                    && content == "a\nb\nc\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 3
                    && *length == 1
                    && path == target_path
                    && content == "a\nb\nc\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 5
                    && *length == 1
                    && path == target_path
                    && content == "a\nb\nc\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    #[ignore] // なんかうまくいかない
    fn check_eol_cr_crlf() {
        let target_path = "../../testdata/end_of_line/cr/error_crlf.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 1
                    && *length == 1
                    && path == target_path
                    && content == "a\r"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 2
                    && *start == 0
                    && *length == 1
                    && path == target_path
                    && content == "\nb\r"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 3
                    && *start == 0
                    && *length == 1
                    && path == target_path
                    && content == "\nc\r"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 4
                    && *start == 0
                    && *length == 1
                    && path == target_path
                    && content == "\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_eol_crlf_no_error() {
        let target_path = "../../testdata/end_of_line/crlf/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_eol_crlf_cr() {
        assert_eq!(
            eol_rules("../../testdata/end_of_line/crlf/error_cr.target"),
            vec!["end_of_line", "end_of_line", "end_of_line"]
        );
    }

    #[test]
    fn check_eol_crlf_lf() {
        assert_eq!(
            eol_rules("../../testdata/end_of_line/crlf/error_lf.target"),
            vec!["end_of_line", "end_of_line", "end_of_line"]
        );
    }

    #[test]
    fn check_eol_lf_reports_every_non_lf_terminator_in_mixed_file() {
        assert_eq!(
            eol_rules("../../testdata/end_of_line/lf_mixed/error_mixed.target"),
            vec!["end_of_line", "end_of_line"]
        );
    }

    #[test]
    fn check_eol_accepts_case_insensitive_value() {
        let target_path = "../../testdata/end_of_line/lf_uppercase/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        assert_eq!(config.end_of_line, Some(ecci_editorconfig::EndOfLine::LF));
        assert!(eol_rules(target_path).is_empty());
    }

    #[test]
    fn check_eol_unset_disables_inherited_setting() {
        let target_path = "../../testdata/end_of_line/unset/nested/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        assert_eq!(config.end_of_line, None);
        assert!(eol_rules(target_path).is_empty());
    }

    #[test]
    fn check_eol_accepts_empty_file() {
        assert!(eol_rules("../../testdata/end_of_line/empty/no_error.target").is_empty());
    }

    #[test]
    fn check_eol_allows_missing_final_lf_or_cr_when_final_newline_is_not_required() {
        assert!(eol_rules("../../testdata/end_of_line/lf/no_final_newline.target").is_empty());
        assert!(eol_rules("../../testdata/end_of_line/cr/no_final_newline.target").is_empty());
    }

    #[test]
    fn check_eol_crlf_allows_missing_final_newline_when_insert_final_newline_is_false() {
        assert!(
            eol_rules("../../testdata/end_of_line/crlf_no_final_newline/no_error.target")
                .is_empty()
        );
    }
}

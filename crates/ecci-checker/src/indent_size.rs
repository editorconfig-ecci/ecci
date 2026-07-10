use ecci_editorconfig::{Config, IndentStyle};

use crate::Output;

pub fn check_indent_size<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    if let Some(IndentStyle::Space) = config.indent_style {
        if let Some(size) = config.indent_size.filter(|size| *size > 0) {
            let mut indent = 0;
            for c in content.chars() {
                if c == ' ' {
                    indent += 1;
                } else {
                    break;
                }
            }
            if indent % size != 0 {
                output.output(
                    line_number,
                    0,
                    indent,
                    &config.path.to_string_lossy(),
                    content,
                    "indent_size",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{check_all, MockOutput};

    #[test]
    fn check_indent_size_2_no_error() {
        let target_path = "../../testdata/indent_size/2/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_size_2_error_3() {
        let target_path = "../../testdata/indent_size/2/error_3.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 2
                    && *column == 0
                    && *length == 3
                    && path == target_path
                    && content == "   b\n"
                    && rule == "indent_size"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_size_4_no_error() {
        let target_path = "../../testdata/indent_size/4/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_size_4_error_2() {
        let target_path = "../../testdata/indent_size/4/error_2.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 2
                    && *column == 0
                    && *length == 2
                    && path == target_path
                    && content == "  b\n"
                    && rule == "indent_size"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_size_is_case_insensitive() {
        let target_path = "../../testdata/indent_size/case_insensitive/error_3.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 2
                    && *column == 0
                    && *length == 3
                    && path == target_path
                    && content == "   b\n"
                    && rule == "indent_size"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_size_unset_disables_inherited_size() {
        let target_path = "../../testdata/indent_size/unset/unset.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        assert_eq!(config.indent_size, None);
        assert!(!config.indent_size_is_tab);

        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_size_tab_without_tab_width_uses_editor_default() {
        let target_path = "../../testdata/indent_size/tab_without_tab_width/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        assert_eq!(config.indent_size, None);
        assert!(config.indent_size_is_tab);
        assert_eq!(config.tab_width, None);

        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_size_one_accepts_every_space_indent() {
        let target_path = "../../testdata/indent_size/one/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        assert_eq!(config.indent_size, Some(1));

        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_size_tab_uses_tab_width() {
        let target_path = "../../testdata/indent_size/tab_with_tab_width/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        assert_eq!(config.indent_size, Some(4));
        assert!(!config.indent_size_is_tab);
        assert_eq!(config.tab_width, Some(4));

        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_size_tab_with_tab_width_rejects_non_multiple() {
        let target_path = "../../testdata/indent_size/tab_with_tab_width/error_2.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 2
                    && *column == 0
                    && *length == 2
                    && path == target_path
                    && content == "  b\n"
                    && rule == "indent_size"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn invalid_indent_size_returns_an_error() {
        let target_path = "../../testdata/indent_size/invalid_value/invalid.target";
        let result = ecci_editorconfig::Config::from_path(std::path::Path::new(target_path));
        assert!(result.is_err());
    }

    #[test]
    fn zero_indent_size_does_not_panic() {
        let target_path = "../../testdata/indent_size/zero/zero.target";
        let result = std::panic::catch_unwind(|| {
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path))
        });

        // libeditorconfig resolves this to tab_width = 0. That is invalid under
        // the tab_width contract, but configuration parsing must not panic.
        assert!(matches!(result, Ok(Err(_))));
    }
}

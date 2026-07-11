use ecci_editorconfig::{Config, IndentStyle};

use crate::Output;

pub fn check_indent_style<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    let invalid_indent_character = match config.indent_style {
        Some(IndentStyle::Space) => '\t',
        Some(IndentStyle::Tab) => ' ',
        None => return,
    };

    let mut invalid_column = None;
    for (column, character) in content.char_indices() {
        if !matches!(character, ' ' | '\t') {
            break;
        }
        if character == invalid_indent_character && invalid_column.is_none() {
            invalid_column = Some(column);
        }
    }

    if let Some(column) = invalid_column {
        output.output(
            line_number,
            column,
            1,
            &config.path.to_string_lossy(),
            content,
            "indent_style.invalid_value",
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{check_all, MockOutput};

    #[test]
    fn check_indent_style_space() {
        let target_path = "../../testdata/indent_style/space/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_style_space_error() {
        let target_path = "../../testdata/indent_style/space/error_tab.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 2
                    && *column == 0
                    && *length == 1
                    && path == target_path
                    && content == "\t\tb\n"
                    && rule == "indent_style.invalid_value"
            })
            .times(1)
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_style_space_unindented() {
        let target_path = "../../testdata/indent_style/space/unindented.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_style_tab() {
        let target_path = "../../testdata/indent_style/tab/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_style_tab_error() {
        let target_path = "../../testdata/indent_style/tab/error_space.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 2
                    && *column == 0
                    && *length == 1
                    && path == target_path
                    && content == "  b\n"
                    && rule == "indent_style.invalid_value"
            })
            .times(1)
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_style_tab_unindented() {
        let target_path = "../../testdata/indent_style/tab/unindented.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_style_is_case_insensitive() {
        for target_path in [
            "../../testdata/indent_style/case_insensitive/space/no_error.target",
            "../../testdata/indent_style/case_insensitive/tab/no_error.target",
        ] {
            let config =
                ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
            let mut mock = MockOutput::new();
            mock.expect_output().never();
            check_all(&config, &mut mock).unwrap();
        }
    }

    #[test]
    fn check_indent_style_unset_disables_inherited_rule() {
        let target_path = "../../testdata/indent_style/unset/nested/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        assert!(config.indent_style.is_none());
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_style_rejects_mixed_indentation() {
        for (target_path, expected_content) in [
            (
                "../../testdata/indent_style/space/error_mixed_tab.target",
                " \tb\n",
            ),
            (
                "../../testdata/indent_style/tab/error_mixed_space.target",
                "\t b\n",
            ),
        ] {
            let config =
                ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
            let mut mock = MockOutput::new();
            mock.expect_output()
                .withf(move |line_number, column, length, path, content, rule| {
                    *line_number == 2
                        && *column == 1
                        && *length == 1
                        && path == target_path
                        && content == expected_content
                        && rule == "indent_style.invalid_value"
                })
                .once()
                .return_const(());
            check_all(&config, &mut mock).unwrap();
        }
    }
}

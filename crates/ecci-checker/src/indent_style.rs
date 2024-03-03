use ecci_editorconfig::{Config, IndentStyle};

use crate::Output;

pub fn check_indent_style<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    if let Some(IndentStyle::Space) = config.indent_style {
        if content.starts_with('\t') {
            output.output(
                line_number,
                0,
                1,
                &config.path.to_string_lossy(),
                content,
                "indent_style",
            )
        }
    } else if let Some(IndentStyle::Tab) = config.indent_style {
        if content.starts_with(' ') {
            output.output(
                line_number,
                0,
                1,
                &config.path.to_string_lossy(),
                content,
                "indent_style",
            )
        }
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
                    && rule == "indent_style"
            })
            .times(1)
            .return_const(());
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
                    && rule == "indent_style"
            })
            .times(1)
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }
}

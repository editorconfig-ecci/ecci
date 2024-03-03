use ecci_editorconfig::{Config, IndentStyle};

use crate::Output;

pub fn check_indent_size<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    if let Some(IndentStyle::Space) = config.indent_style {
        if let Some(size) = config.indent_size {
            let mut indent = 0;
            for c in content.chars() {
                if c == ' ' {
                    indent += 1;
                } else {
                    break;
                }
            }
            println!("[{:?}] indent: {} size: {}", content, indent, size);
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
}

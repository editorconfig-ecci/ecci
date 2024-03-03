use ecci_editorconfig::Config;

use crate::Output;

pub fn check_max_line_length<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    if let Some(max_line_length) = config.max_line_length {
        let trimmed = content.trim_end_matches(['\r', '\n']);
        if trimmed.len() > max_line_length {
            output.output(
                line_number,
                max_line_length,
                trimmed.len() - max_line_length,
                &config.path.to_string_lossy(),
                content,
                "max_line_length",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{check_all, MockOutput};

    #[test]
    fn check_max_line_length_no_error() {
        let target_path = "../../testdata/max_line_length/10/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_indent_style_space_error() {
        let target_path = "../../testdata/max_line_length/10/error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 2
                    && *column == 10
                    && *length == 2
                    && path == target_path
                    && content == "bbbbbbbbbbbb\n"
                    && rule == "max_line_length"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }
}

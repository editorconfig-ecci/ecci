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
}

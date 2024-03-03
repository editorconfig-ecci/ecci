use ecci_editorconfig::{Config, EndOfLine};

use crate::Output;

pub fn check_insert_final_newline<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
    has_next_line: bool,
) {
    let eol = match config.end_of_line {
        Some(EndOfLine::CR) => "\r",
        Some(EndOfLine::LF) => "\n",
        Some(EndOfLine::CRLF) => "\r\n",
        None => "\n",
    };
    if let Some(true) = config.insert_final_newline {
        if !has_next_line && !content.ends_with(eol) {
            output.output(
                line_number,
                content.len(),
                0,
                &config.path.to_string_lossy(),
                content,
                "insert_final_newline",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{check_all, MockOutput};

    #[test]
    fn check_insert_final_newline_true_no_error() {
        let target_path = "../../testdata/insert_final_newline/true/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_insert_final_newline_error() {
        let target_path = "../../testdata/insert_final_newline/true/error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 3
                    && *column == 1
                    && *length == 0
                    && path == target_path
                    && content == "c"
                    && rule == "insert_final_newline"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_insert_final_newline_false_no_error() {
        let target_path = "../../testdata/insert_final_newline/false/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }
}

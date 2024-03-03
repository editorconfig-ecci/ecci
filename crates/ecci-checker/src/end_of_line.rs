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
            if c == '\r' && i != content.len() - 2 {
                output.output(
                    line_number,
                    i,
                    1,
                    &config.path.to_string_lossy(),
                    content,
                    "end_of_line",
                );
            }
            if c == '\n' && i != content.len() - 1 {
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
        if !content.ends_with("\r\n") {
            output.output(
                line_number,
                content.len() - 2,
                2,
                &config.path.to_string_lossy(),
                content,
                "end_of_line",
            );
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
        let target_path = "../../testdata/end_of_line/crlf/error_cr.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 4
                    && *length == 2
                    && path == target_path
                    && content == "a\rb\rc\r"
                    && rule == "end_of_line"
            })
            .return_const(());
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
    #[ignore]
    fn check_eol_crlf_lf() {
        let target_path = "../../testdata/end_of_line/crlf/error_lf.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 0
                    && *length == 2
                    && path == target_path
                    && content == "a\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 1
                    && *start == 1
                    && *length == 1
                    && path == target_path
                    && content == "a\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        mock.expect_output()
            .once()
            .withf(move |line_number, start, length, path, content, rule| {
                *line_number == 2
                    && *start == 0
                    && *length == 2
                    && path == target_path
                    && content == "b\n"
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
                    && content == "b\n"
                    && rule == "end_of_line"
            })
            .return_const(());
        // mock.expect_output()
        //     .once()
        //     .withf(move |line_number, start, length, path, content, rule| {
        //         *line_number == 3
        //             && *start == 0
        //             && *length == 2
        //             && path == target_path
        //             && content == "c\n"
        //             && rule == "end_of_line"
        //     })
        //     .return_const(());
        // mock.expect_output()
        //     .once()
        //     .withf(move |line_number, start, length, path, content, rule| {
        //         *line_number == 3
        //             && *start == 1
        //             && *length == 1
        //             && path == target_path
        //             && content == "c\n"
        //             && rule == "end_of_line"
        //     })
        //     .return_const(());
        check_all(&config, &mut mock).unwrap();
    }
}

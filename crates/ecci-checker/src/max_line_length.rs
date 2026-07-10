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

    #[test]
    fn check_max_line_length_one_character_boundary() {
        let target_path = "../../testdata/max_line_length/1/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_max_line_length_counts_tabs_as_characters() {
        let target_path = "../../testdata/max_line_length/tabs/error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 1
                    && *column == 4
                    && *length == 1
                    && path == target_path
                    && content == "\t\t\t\t\t\n"
                    && rule == "max_line_length"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_max_line_length_accepts_tabs_and_crlf_at_the_boundary() {
        for target_path in [
            "../../testdata/max_line_length/tabs/no_error.target",
            "../../testdata/max_line_length/crlf/no_error.target",
        ] {
            let config =
                ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
            let mut mock = MockOutput::new();
            mock.expect_output().never();
            check_all(&config, &mut mock).unwrap();
        }
    }

    #[test]
    fn check_max_line_length_excludes_crlf() {
        let target_path = "../../testdata/max_line_length/crlf/error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 1
                    && *column == 4
                    && *length == 1
                    && path == target_path
                    && content == "aaaaa\r\n"
                    && rule == "max_line_length"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    fn check_max_line_length_unset_disables_inherited_value_case_insensitively() {
        let target_path = "../../testdata/max_line_length/unset/child/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        assert_eq!(config.max_line_length, None);
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    // EditorConfig specifies a character count, while the current checker uses
    // UTF-8 byte length. Keep the desired behavior executable but ignored until
    // the tracked checker bug is fixed.
    #[test]
    #[ignore = "max_line_length currently counts UTF-8 bytes rather than characters"]
    fn check_max_line_length_accepts_multibyte_character_boundary() {
        let target_path = "../../testdata/max_line_length/multibyte/no_error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output().never();
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    #[ignore = "max_line_length currently counts UTF-8 bytes rather than characters"]
    fn check_max_line_length_counts_multibyte_characters() {
        let target_path = "../../testdata/max_line_length/multibyte/error.target";
        let config =
            ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
        let mut mock = MockOutput::new();
        mock.expect_output()
            .once()
            .withf(move |line_number, column, length, path, content, rule| {
                *line_number == 1
                    && *column == 5
                    && *length == 1
                    && path == target_path
                    && content == "あいうえおか\n"
                    && rule == "max_line_length"
            })
            .return_const(());
        check_all(&config, &mut mock).unwrap();
    }

    #[test]
    #[ignore = "invalid max_line_length values currently panic or are accepted"]
    fn invalid_max_line_length_values_are_ignored_without_panicking() {
        for value in ["0", "-1", "1.5", "invalid"] {
            let target_path =
                format!("../../testdata/max_line_length/invalid/{value}/target.target");
            let config = std::panic::catch_unwind(|| {
                ecci_editorconfig::Config::from_path(std::path::Path::new(&target_path))
            })
            .expect("invalid max_line_length must not panic")
            .expect("target file must be readable");
            assert_eq!(
                config.max_line_length, None,
                "invalid value {value} must not enable max_line_length"
            );
        }
    }
}

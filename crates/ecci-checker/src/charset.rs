use ecci_editorconfig::Config;

use crate::Output;

pub fn check_charset<T: Output>(
    _config: &Config,
    _output: &mut T,
    _line_number: usize,
    _content: &str,
) {
    // todo!();
}

#[cfg(test)]
mod tests {
    use ecci_editorconfig::{Charset, Config};
    use std::path::Path;

    #[test]
    fn parses_all_supported_charsets_and_unset() {
        let cases = [
            (
                "../../testdata/charset/latin1/no_error_non_ascii.target",
                Some(Charset::Latin1),
            ),
            (
                "../../testdata/charset/utf8/no_error_non_ascii.target",
                Some(Charset::UTF8),
            ),
            (
                "../../testdata/charset/utf8_bom/no_error_non_ascii.target",
                Some(Charset::UTF8BOM),
            ),
            (
                "../../testdata/charset/utf16be/no_error_non_ascii.target",
                Some(Charset::UTF16BE),
            ),
            (
                "../../testdata/charset/utf16le/no_error_non_ascii.target",
                Some(Charset::UTF16LE),
            ),
            (
                "../../testdata/charset/unset/nested/no_error_latin1.target",
                None,
            ),
        ];

        for (path, expected) in cases {
            assert_eq!(
                Config::from_path(Path::new(path)).unwrap().charset,
                expected,
                "{path}"
            );
        }
    }
}

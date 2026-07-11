use ecci_editorconfig::{Charset, Config};

use crate::Output;

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
const UTF16BE_BOM: &[u8] = b"\xFE\xFF";
const UTF16LE_BOM: &[u8] = b"\xFF\xFE";

pub fn check_charset<T: Output>(config: &Config, output: &mut T, bytes: &[u8]) {
    if validate(config.charset.as_ref(), bytes).is_err() {
        // Charset is a file-level property.  Report one diagnostic for the
        // complete byte sequence rather than reporting the BOM for every line.
        output.output(
            1,
            0,
            bytes.len(),
            &config.path.to_string_lossy(),
            &String::from_utf8_lossy(bytes),
            "charset.invalid_value",
        );
    }
}

pub fn decode_for_line_checks(config: &Config, bytes: &[u8]) -> Option<String> {
    let charset = config.charset.as_ref();
    validate(charset, bytes).ok()?;

    match charset {
        Some(Charset::Latin1) => Some(bytes.iter().map(|&byte| char::from(byte)).collect()),
        Some(Charset::UTF8BOM) => String::from_utf8(bytes[UTF8_BOM.len()..].to_vec()).ok(),
        Some(Charset::UTF16BE) => decode_utf16(bytes, true),
        Some(Charset::UTF16LE) => decode_utf16(bytes, false),
        Some(Charset::UTF8) | None => String::from_utf8(bytes.to_vec()).ok(),
    }
}

fn validate(charset: Option<&Charset>, bytes: &[u8]) -> Result<(), ()> {
    match charset {
        None | Some(Charset::Latin1) => Ok(()),
        Some(Charset::UTF8) => {
            if bytes.starts_with(UTF8_BOM) {
                Err(())
            } else {
                std::str::from_utf8(bytes).map(|_| ()).map_err(|_| ())
            }
        }
        Some(Charset::UTF8BOM) => bytes
            .strip_prefix(UTF8_BOM)
            .ok_or(())
            .and_then(|content| std::str::from_utf8(content).map(|_| ()).map_err(|_| ())),
        Some(Charset::UTF16BE) => validate_utf16(bytes, true),
        Some(Charset::UTF16LE) => validate_utf16(bytes, false),
    }
}

// EditorConfig does not specify whether UTF-16 must have a BOM.  We accept a
// missing BOM because the configured charset supplies byte order; a matching
// BOM is accepted and removed for line checks, while an opposite-endian BOM is
// rejected as an encoding mismatch.
fn validate_utf16(bytes: &[u8], big_endian: bool) -> Result<(), ()> {
    if bytes.len() % 2 != 0 {
        return Err(());
    }

    let opposite_bom = if big_endian { UTF16LE_BOM } else { UTF16BE_BOM };
    if bytes.starts_with(opposite_bom) {
        return Err(());
    }

    let content = if big_endian {
        bytes.strip_prefix(UTF16BE_BOM).unwrap_or(bytes)
    } else {
        bytes.strip_prefix(UTF16LE_BOM).unwrap_or(bytes)
    };
    decode_utf16_units(content, big_endian).map(|_| ())
}

fn decode_utf16(bytes: &[u8], big_endian: bool) -> Option<String> {
    let bom = if big_endian { UTF16BE_BOM } else { UTF16LE_BOM };
    let content = bytes.strip_prefix(bom).unwrap_or(bytes);
    decode_utf16_units(content, big_endian).ok()
}

fn decode_utf16_units(bytes: &[u8], big_endian: bool) -> Result<String, ()> {
    let units = bytes.chunks_exact(2).map(|pair| {
        if big_endian {
            u16::from_be_bytes([pair[0], pair[1]])
        } else {
            u16::from_le_bytes([pair[0], pair[1]])
        }
    });
    char::decode_utf16(units)
        .collect::<Result<String, _>>()
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use crate::{check_all, MockOutput};
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

    #[test]
    fn charset_fixtures_report_only_encoding_mismatches() {
        let valid = [
            "../../testdata/charset/utf8/no_error_non_ascii.target",
            "../../testdata/charset/utf8_bom/no_error_non_ascii.target",
            "../../testdata/charset/latin1/no_error_non_ascii.target",
            "../../testdata/charset/utf16be/no_error_non_ascii.target",
            "../../testdata/charset/utf16le/no_error_non_ascii.target",
            "../../testdata/charset/unset/nested/no_error_latin1.target",
        ];
        for target_path in valid {
            let config = Config::from_path(Path::new(target_path)).unwrap();
            let mut mock = MockOutput::new();
            mock.expect_output().never();
            check_all(&config, &mut mock).unwrap();
        }

        let invalid = [
            "../../testdata/charset/utf8/error_bom.target",
            "../../testdata/charset/utf8/error_invalid_sequence.target",
            "../../testdata/charset/utf8_bom/error_missing_bom.target",
            "../../testdata/charset/utf8_bom/error_empty.target",
            "../../testdata/charset/utf16be/error_odd_byte_length.target",
            "../../testdata/charset/utf16le/error_odd_byte_length.target",
        ];
        for target_path in invalid {
            let config = Config::from_path(Path::new(target_path)).unwrap();
            let mut mock = MockOutput::new();
            mock.expect_output()
                .once()
                .withf(move |line, start, length, path, _content, rule| {
                    *line == 1
                        && *start == 0
                        && path == target_path
                        && rule == "charset.invalid_value"
                        && (*length > 0 || (*length == 0 && path.ends_with("error_empty.target")))
                })
                .return_const(());
            check_all(&config, &mut mock).unwrap();
        }
    }

    #[test]
    fn utf16_bom_policy_accepts_matching_or_no_bom_and_rejects_opposite_bom() {
        assert!(super::validate_utf16(b"\x00a", true).is_ok());
        assert!(super::validate_utf16(b"\xFE\xFF\x00a", true).is_ok());
        assert!(super::validate_utf16(b"a\x00", false).is_ok());
        assert!(super::validate_utf16(b"\xFF\xFEa\x00", false).is_ok());
        assert!(super::validate_utf16(b"\xFF\xFEa\x00", true).is_err());
        assert!(super::validate_utf16(b"\xFE\xFF\x00a", false).is_err());
    }

    #[test]
    fn utf16_rejects_unpaired_surrogates() {
        assert!(super::validate_utf16(b"\xD8\x00", true).is_err());
        assert!(super::validate_utf16(b"\x00\xDC", false).is_err());
    }
}

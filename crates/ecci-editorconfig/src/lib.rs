use std::ffi::CString;
use std::path::{Path, PathBuf};

#[allow(dead_code, non_camel_case_types)]
mod bindings;

#[derive(Debug, PartialEq)]
pub enum IndentStyle {
    Tab,
    Space,
}

#[derive(Debug, PartialEq)]
pub enum EndOfLine {
    LF,
    CRLF,
    CR,
}

#[derive(Debug, PartialEq)]
pub enum Charset {
    Latin1,
    UTF8,
    UTF16BE,
    UTF16LE,
    UTF8BOM,
}

#[derive(Debug, PartialEq)]
pub struct Config {
    pub path: PathBuf,
    pub indent_style: Option<IndentStyle>,
    pub indent_size: Option<usize>,
    pub indent_size_is_tab: bool,
    pub tab_width: Option<usize>,
    pub end_of_line: Option<EndOfLine>,
    pub charset: Option<Charset>,
    pub trim_trailing_whitespace: Option<bool>,
    pub insert_final_newline: Option<bool>,
    pub max_line_length: Option<usize>,
}

impl Config {
    pub fn from_path(path: &Path) -> std::io::Result<Config> {
        parse_internal(path)
    }
}

fn invalid_value(property: &str, value: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("invalid {property} value: {value}"),
    )
}

fn parse_usize(property: &str, value: &str) -> std::io::Result<usize> {
    value.parse().map_err(|_| invalid_value(property, value))
}

fn parse_positive_usize(property: &str, value: &str) -> std::io::Result<usize> {
    let value = parse_usize(property, value)?;
    if value == 0 {
        return Err(invalid_value(property, "0"));
    }
    Ok(value)
}

fn parse_bool(property: &str, value: &str) -> std::io::Result<bool> {
    value.parse().map_err(|_| invalid_value(property, value))
}

fn parse_internal(path: &Path) -> std::io::Result<Config> {
    let canonical = path.canonicalize()?;
    let c_string = CString::new(canonical.to_str().unwrap()).unwrap();
    let ptr: *const i8 = c_string.as_ptr();
    let mut config = Config {
        path: path.to_path_buf(),
        indent_style: None,
        indent_size: None,
        indent_size_is_tab: false,
        tab_width: None,
        end_of_line: None,
        charset: None,
        trim_trailing_whitespace: None,
        insert_final_newline: None,
        max_line_length: None,
    };
    let mut indent_size_was_unset = false;
    let mut tab_width_was_unset = false;
    unsafe {
        let handle = bindings::editorconfig_handle_init();
        bindings::editorconfig_parse(ptr, handle);
        let count = bindings::editorconfig_handle_get_name_value_count(handle);
        for i in 0..count {
            let mut name: *const i8 = std::ptr::null();
            let mut value: *const i8 = std::ptr::null();
            bindings::editorconfig_handle_get_name_value(handle, i, &mut name, &mut value);
            let name = std::ffi::CStr::from_ptr(name).to_str().unwrap();
            let value = std::ffi::CStr::from_ptr(value).to_str().unwrap();
            match name.to_string().as_str() {
                "indent_style" => match value {
                    "tab" => config.indent_style = Some(IndentStyle::Tab),
                    "space" => config.indent_style = Some(IndentStyle::Space),
                    _ => {}
                },
                "indent_size" => {
                    if value.eq_ignore_ascii_case("unset") {
                        config.indent_size = None;
                        config.indent_size_is_tab = false;
                        indent_size_was_unset = true;
                    } else if value.eq_ignore_ascii_case("tab") {
                        config.indent_size = None;
                        config.indent_size_is_tab = true;
                    } else {
                        config.indent_size = Some(parse_usize("indent_size", value)?);
                        config.indent_size_is_tab = false;
                    }
                }
                "tab_width" => {
                    config.tab_width = if value.eq_ignore_ascii_case("unset") {
                        tab_width_was_unset = true;
                        None
                    } else {
                        Some(parse_positive_usize("tab_width", value)?)
                    };
                }
                "end_of_line" => match value {
                    "lf" => config.end_of_line = Some(EndOfLine::LF),
                    "crlf" => config.end_of_line = Some(EndOfLine::CRLF),
                    "cr" => config.end_of_line = Some(EndOfLine::CR),
                    _ => {}
                },
                "charset" => match value {
                    "latin1" => config.charset = Some(Charset::Latin1),
                    "utf-8" => config.charset = Some(Charset::UTF8),
                    "utf-16be" => config.charset = Some(Charset::UTF16BE),
                    "utf-16le" => config.charset = Some(Charset::UTF16LE),
                    "utf-8-bom" => config.charset = Some(Charset::UTF8BOM),
                    _ => {}
                },
                "trim_trailing_whitespace" => {
                    config.trim_trailing_whitespace = if value.eq_ignore_ascii_case("unset") {
                        None
                    } else {
                        Some(parse_bool("trim_trailing_whitespace", value)?)
                    };
                }
                "insert_final_newline" => {
                    config.insert_final_newline = if value.eq_ignore_ascii_case("unset") {
                        None
                    } else {
                        Some(parse_bool("insert_final_newline", value)?)
                    };
                }
                "max_line_length" => {
                    config.max_line_length = if value.eq_ignore_ascii_case("unset") {
                        None
                    } else {
                        Some(parse_usize("max_line_length", value)?)
                    };
                }
                _ => {}
            }
        }
        // libeditorconfig resolves `indent_size = tab` through `tab_width` and
        // reports the resolved value as `unset` when a nested section unsets
        // tab_width. Keep the original tab-based indent semantics in that case.
        if indent_size_was_unset
            && tab_width_was_unset
            && config.indent_style == Some(IndentStyle::Tab)
        {
            config.indent_size_is_tab = true;
        }
        let ret = bindings::editorconfig_handle_destroy(handle);
        if ret != 0 {
            panic!("Failed to destroy the editorconfig_handle object");
        }
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let config = Config::from_path(Path::new("../../testdata/simple/test.txt")).unwrap();
        assert_eq!(config.indent_style, Some(IndentStyle::Space));
        assert_eq!(config.indent_size, Some(4));
        assert!(!config.indent_size_is_tab);
        assert_eq!(config.tab_width, Some(8));
        assert_eq!(config.end_of_line, Some(EndOfLine::LF));
        assert_eq!(config.charset, Some(Charset::UTF8));
        assert_eq!(config.trim_trailing_whitespace, Some(true));
        assert_eq!(config.insert_final_newline, Some(true));
        assert_eq!(config.max_line_length, Some(100));
    }

    #[test]
    fn tab_width_parses_minimum_positive_value() {
        let config =
            Config::from_path(Path::new("../../testdata/tab_width/minimum/target.target")).unwrap();

        assert_eq!(config.tab_width, Some(1));
    }

    #[test]
    fn tab_width_is_independent_from_numeric_indent_size() {
        let config = Config::from_path(Path::new(
            "../../testdata/tab_width/numeric_indent_size/no_error.target",
        ))
        .unwrap();

        assert_eq!(config.tab_width, Some(2));
        assert_eq!(config.indent_size, Some(4));
        assert!(!config.indent_size_is_tab);
    }

    #[test]
    fn tab_width_interacts_with_indent_size_tab() {
        let config = Config::from_path(Path::new(
            "../../testdata/tab_width/indent_size_tab/no_error.target",
        ))
        .unwrap();

        assert_eq!(config.tab_width, Some(4));
        assert_eq!(config.indent_size, Some(4));
        assert!(!config.indent_size_is_tab);
    }

    #[test]
    fn tab_width_unset_is_case_insensitive() {
        let config = Config::from_path(Path::new(
            "../../testdata/tab_width/unset/nested/no_error.target",
        ))
        .unwrap();

        assert_eq!(config.tab_width, None);
        assert!(config.indent_size_is_tab);
    }

    #[test]
    fn tab_width_rejects_zero() {
        assert!(
            Config::from_path(Path::new("../../testdata/tab_width/zero/target.target")).is_err()
        );
    }

    #[test]
    fn tab_width_rejects_negative_value_without_panicking() {
        let result = std::panic::catch_unwind(|| {
            Config::from_path(Path::new("../../testdata/tab_width/negative/target.target"))
        });

        assert!(matches!(result, Ok(Err(_))));
    }

    #[test]
    fn tab_width_rejects_non_numeric_value_without_panicking() {
        let result = std::panic::catch_unwind(|| {
            Config::from_path(Path::new(
                "../../testdata/tab_width/non_numeric/target.target",
            ))
        });

        assert!(matches!(result, Ok(Err(_))));
    }
}

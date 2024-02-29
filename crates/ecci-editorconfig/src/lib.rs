use std::ffi::CString;
use std::path::{Path, PathBuf};

#[allow(non_camel_case_types)]
#[allow(dead_code)]
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
                    if value == "tab" {
                        config.indent_size_is_tab = true;
                    } else {
                        config.indent_size = Some(value.parse().unwrap());
                    }
                }
                "tab_width" => {
                    config.tab_width = Some(value.parse().unwrap());
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
                    config.trim_trailing_whitespace = Some(value.parse().unwrap());
                }
                "insert_final_newline" => {
                    config.insert_final_newline = Some(value.parse().unwrap());
                }
                "max_line_length" => {
                    config.max_line_length = Some(value.parse().unwrap());
                }
                _ => {}
            }
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
        let config = Config::from_path(Path::new("testdata/test.txt")).unwrap();
        assert_eq!(config.indent_style, Some(IndentStyle::Space));
        assert_eq!(config.indent_size, Some(4));
        assert_eq!(config.indent_size_is_tab, false);
        assert_eq!(config.tab_width, Some(8));
        assert_eq!(config.end_of_line, Some(EndOfLine::LF));
        assert_eq!(config.charset, Some(Charset::UTF8));
        assert_eq!(config.trim_trailing_whitespace, Some(true));
        assert_eq!(config.insert_final_newline, Some(true));
        assert_eq!(config.max_line_length, Some(100));
    }
}

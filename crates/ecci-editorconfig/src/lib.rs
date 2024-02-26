use std::collections::HashMap;
use std::ffi::CString;
use std::path::Path;

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

pub fn parse(path: &Path) -> Config {
    let name_value_map = parse_internal(path);
    let mut config = Config {
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
    if name_value_map.contains_key("indent_style") {
        let value = name_value_map.get("indent_style").unwrap();
        match value.as_str() {
            "tab" => config.indent_style = Some(IndentStyle::Tab),
            "space" => config.indent_style = Some(IndentStyle::Space),
            _ => {}
        }
    }
    if name_value_map.contains_key("indent_size") {
        let value = name_value_map.get("indent_size").unwrap();
        if value == "tab" {
            config.indent_size_is_tab = true;
        } else {
            config.indent_size = Some(value.parse().unwrap());
        }
    }
    if name_value_map.contains_key("tab_width") {
        let value = name_value_map.get("tab_width").unwrap();
        config.tab_width = Some(value.parse().unwrap());
    }
    if name_value_map.contains_key("end_of_line") {
        let value = name_value_map.get("end_of_line").unwrap();
        match value.as_str() {
            "lf" => config.end_of_line = Some(EndOfLine::LF),
            "crlf" => config.end_of_line = Some(EndOfLine::CRLF),
            "cr" => config.end_of_line = Some(EndOfLine::CR),
            _ => {}
        }
    }
    if name_value_map.contains_key("charset") {
        let value = name_value_map.get("charset").unwrap();
        match value.as_str() {
            "latin1" => config.charset = Some(Charset::Latin1),
            "utf-8" => config.charset = Some(Charset::UTF8),
            "utf-16be" => config.charset = Some(Charset::UTF16BE),
            "utf-16le" => config.charset = Some(Charset::UTF16LE),
            "utf-8-bom" => config.charset = Some(Charset::UTF8BOM),
            _ => {}
        }
    }
    if name_value_map.contains_key("trim_trailing_whitespace") {
        let value = name_value_map.get("trim_trailing_whitespace").unwrap();
        config.trim_trailing_whitespace = Some(value.parse().unwrap());
    }
    if name_value_map.contains_key("insert_final_newline") {
        let value = name_value_map.get("insert_final_newline").unwrap();
        config.insert_final_newline = Some(value.parse().unwrap());
    }
    if name_value_map.contains_key("max_line_length") {
        let value = name_value_map.get("max_line_length").unwrap();
        config.max_line_length = Some(value.parse().unwrap());
    }
    config
}

fn parse_internal(path: &Path) -> HashMap<String, String> {
    let canonical = path.canonicalize().unwrap();
    let c_string = CString::new(canonical.to_str().unwrap()).unwrap();
    let ptr: *const i8 = c_string.as_ptr();
    let mut result: HashMap<String, String> = HashMap::new();
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
            result.insert(name.to_string(), value.to_string());
        }
        let ret = bindings::editorconfig_handle_destroy(handle);
        if ret != 0 {
            panic!("Failed to destroy the editorconfig_handle object");
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = parse_internal(Path::new("testdata/test.txt"));
        assert_eq!(result.get("indent_style").unwrap(), "space");
        assert_eq!(result.get("indent_size").unwrap(), "4");
        assert_eq!(result.get("insert_final_newline").unwrap(), "true");

        let config = parse(Path::new("testdata/test.txt"));
        assert_eq!(config.indent_style.unwrap(), IndentStyle::Space);
        assert_eq!(config.indent_size.unwrap(), 4);
        assert_eq!(config.insert_final_newline.unwrap(), true);
    }
}

use std::collections::HashMap;
use std::ffi::CString;
use std::path::Path;

#[allow(non_camel_case_types)]
#[allow(dead_code)]
mod bindings;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn parse(path: &Path) -> HashMap<String, String> {
    let canonical = path.canonicalize().unwrap();
    let c_string = CString::new(canonical.to_str().unwrap()).unwrap();
    let ptr: *const i8 = c_string.as_ptr();
    let mut result: HashMap<String, String> = HashMap::new();
    unsafe {
        let handle = bindings::editorconfig_handle_init();
        bindings::editorconfig_parse(ptr, handle);
        let count = bindings::editorconfig_handle_get_name_value_count(handle);
        println!("count: {}", count);
        for i in 0..count {
            let mut name: *const i8 = std::ptr::null();
            let mut value: *const i8 = std::ptr::null();
            bindings::editorconfig_handle_get_name_value(handle, i, &mut name, &mut value);
            let name = std::ffi::CStr::from_ptr(name).to_str().unwrap();
            let value = std::ffi::CStr::from_ptr(value).to_str().unwrap();
            println!("{}: {}", name, value);
            result.insert(name.to_string(), value.to_string());
        }
        let ret = bindings::editorconfig_handle_destroy(handle);
        if ret != 0 {
            panic!("Failed to destroy the editorconfig_handle object");
        }
    }
    result
}

pub fn get_version() -> String {
    let mut major = 1;
    let mut minor = 1;
    let mut patch = 1;
    unsafe {
        bindings::editorconfig_handle_get_version(
            bindings::editorconfig_handle_init(),
            &mut major,
            &mut minor,
            &mut patch,
        );
    }
    format!("{}.{}.{}", major, minor, patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);

        let version = get_version();
        assert_eq!(version, "0.0.0"); // why?

        let result = parse(Path::new("testdata/test.txt"));
        assert_eq!(result.get("indent_style").unwrap(), "space");
        assert_eq!(result.get("indent_size").unwrap(), "4");
        assert_eq!(result.get("insert_final_newline").unwrap(), "true");
    }
}

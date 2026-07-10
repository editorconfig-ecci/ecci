use crate::{check_all, MockOutput};

#[test]
fn tab_width_with_numeric_indent_size_does_not_change_indent_size_validation() {
    let target_path = "../../testdata/tab_width/numeric_indent_size/no_error.target";
    let config = ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
    let mut mock = MockOutput::new();
    mock.expect_output().never();

    check_all(&config, &mut mock).unwrap();
}

#[test]
fn tab_width_with_indent_size_tab_accepts_tab_indentation() {
    let target_path = "../../testdata/tab_width/indent_size_tab/no_error.target";
    let config = ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
    let mut mock = MockOutput::new();
    mock.expect_output().never();

    check_all(&config, &mut mock).unwrap();
}

#[test]
#[ignore = "known issue: tab_width = unset panics in ecci-editorconfig before checking"]
fn unset_tab_width_is_not_enforced() {
    let target_path = "../../testdata/tab_width/unset/nested/no_error.target";
    let config = ecci_editorconfig::Config::from_path(std::path::Path::new(target_path)).unwrap();
    let mut mock = MockOutput::new();
    mock.expect_output().never();

    check_all(&config, &mut mock).unwrap();
}

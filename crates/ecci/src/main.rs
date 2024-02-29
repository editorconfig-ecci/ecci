fn main() {
    let path = std::path::Path::new("Cargo.toml");
    let parsed_config = ecci_editorconfig::Config::from_path(path).unwrap();
    println!(
        "{} indent_style:{:?}",
        path.display(),
        parsed_config.indent_style.unwrap()
    );
}

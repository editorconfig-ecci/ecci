fn main() {
    let add_result = ecci_editorconfig::add(2, 2);
    let parse_result = ecci_editorconfig::parse(std::path::Path::new("Cargo.toml"));
    println!("Hello, world! {}", add_result);
    println!("{}", parse_result.get("indent_style").unwrap());
}

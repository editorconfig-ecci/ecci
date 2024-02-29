// use cmake;

fn main() {
    // let dst = cmake::build("editorconfig-core-c");
    // println!(
    //     "cargo:rustc-link-search=native={}",
    //     dst.join("lib").display()
    // );
    // println!("cargo:rustc-link-lib=static=editorconfig_static");
    println!("cargo:rustc-link-lib=editorconfig");
}

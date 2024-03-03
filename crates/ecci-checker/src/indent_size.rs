use ecci_editorconfig::Config;

use crate::Output;

pub fn check_indent_size<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    // todo!();
}

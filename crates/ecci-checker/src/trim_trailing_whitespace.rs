use ecci_editorconfig::Config;

use crate::Output;

pub fn check_trim_trailing_whitespace<T: Output>(
    config: &Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    // todo!();
}

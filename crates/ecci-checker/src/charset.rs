use ecci_editorconfig::Config;

use crate::Output;

pub fn check_charset<T: Output>(
    _config: &Config,
    _output: &mut T,
    _line_number: usize,
    _content: &str,
) {
    // todo!();
}

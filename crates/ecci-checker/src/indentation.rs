pub(crate) fn without_block_comment_decoration_space(content: &str) -> Option<&str> {
    let indent_end = content
        .find(|character| !matches!(character, ' ' | '\t'))
        .unwrap_or(content.len());
    let (indent, remainder) = content.split_at(indent_end);
    let indent = indent.strip_suffix(' ')?;
    let after_asterisk = remainder.strip_prefix('*')?;

    match after_asterisk.as_bytes().first() {
        None | Some(b' ' | b'\t' | b'\r' | b'\n' | b'/') => Some(indent),
        Some(_) => None,
    }
}

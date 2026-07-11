use ecci_editorconfig::Charset;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub const SAMPLE_LIMIT: usize = 8 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Classification {
    Text,
    Binary,
}

pub fn classify(sample: &[u8], charset: Option<&Charset>) -> Classification {
    if sample.is_empty()
        || sample.starts_with(&[0xff, 0xfe])
        || sample.starts_with(&[0xfe, 0xff])
        || matches!(charset, Some(Charset::UTF16LE | Charset::UTF16BE))
    {
        return Classification::Text;
    }

    if sample
        .iter()
        .any(|byte| *byte == 0 || (*byte < 0x20 && !matches!(*byte, b'\t' | b'\n' | b'\r' | 0x0c)))
    {
        Classification::Binary
    } else {
        Classification::Text
    }
}

pub fn classify_path(path: &Path, charset: Option<&Charset>) -> io::Result<Classification> {
    let mut file = File::open(path)?;
    let mut sample = Vec::with_capacity(SAMPLE_LIMIT);
    file.by_ref()
        .take(SAMPLE_LIMIT as u64)
        .read_to_end(&mut sample)?;
    Ok(classify(&sample, charset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_classifier_covers_text_binary_and_utf16() {
        for text in [&b""[..], b"plain\ttext\n", &[0xff, 0xfe, b'a', 0]] {
            assert_eq!(classify(text, None), Classification::Text);
        }
        assert_eq!(classify(&[0xfe, 0xff, 0, b'a'], None), Classification::Text);
        assert_eq!(classify(&[b'a', 0, b'b'], None), Classification::Binary);
        assert_eq!(classify(&[b'a', 1, b'b'], None), Classification::Binary);
        assert_eq!(classify(&[0xff, b'a'], None), Classification::Text);
        assert_eq!(
            classify(&[b'a', 0, b'b'], Some(&Charset::UTF16LE)),
            Classification::Text
        );
    }

    #[test]
    fn marker_after_the_sample_is_not_observed() {
        let mut bytes = vec![b'a'; SAMPLE_LIMIT + 1];
        bytes[SAMPLE_LIMIT] = 0;
        assert_eq!(classify(&bytes[..SAMPLE_LIMIT], None), Classification::Text);
    }
}

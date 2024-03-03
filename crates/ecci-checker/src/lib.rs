use ecci_editorconfig::*;
use std::io::BufRead;
mod charset;
mod end_of_line;
mod indent_size;
mod indent_style;
mod insert_final_newline;
mod max_line_length;
mod trim_trailing_whitespace;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Output {
    fn output(
        &mut self,
        line_number: usize,
        start: usize,
        length: usize,
        path: &str,
        content: &str,
        rule: &str,
    );
}

fn check_line<T: Output>(
    config: &ecci_editorconfig::Config,
    output: &mut T,
    line_number: usize,
    content: &str,
) {
    indent_style::check_indent_style(config, output, line_number, content);
    indent_size::check_indent_size(config, output, line_number, content);
    end_of_line::check_end_of_line(config, output, line_number, content);
    charset::check_charset(config, output, line_number, content);
    trim_trailing_whitespace::check_trim_trailing_whitespace(config, output, line_number, content);
    insert_final_newline::check_insert_final_newline(config, output, line_number, content);
    max_line_length::check_max_line_length(config, output, line_number, content);
}

pub fn check_all<T: Output>(config: &Config, output: &mut T) -> std::io::Result<()> {
    // test code
    println!("Checking {}", config.path.display());
    // output.output(1, 0, 5, "test.txt", "WRONG test", "testrule");

    // real implementation
    let file = std::fs::File::open(&config.path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut line_number = 0usize;
    let mut buf = Vec::new();

    if let Some(EndOfLine::LF) = &config.end_of_line {
        while reader.read_until(b'\n', &mut buf).unwrap() > 0 {
            line_number += 1;
            check_line(
                config,
                output,
                line_number,
                std::str::from_utf8(&buf).unwrap(),
            );
            buf.clear();
        }
    } else if let Some(EndOfLine::CRLF) = &config.end_of_line {
        while reader.read_until(b'\n', &mut buf).unwrap() > 0 {
            line_number += 1;
            check_line(
                config,
                output,
                line_number,
                std::str::from_utf8(&buf).unwrap(),
            );
            buf.clear();
        }
    } else if let Some(EndOfLine::CR) = &config.end_of_line {
        while reader.read_until(b'\r', &mut buf).unwrap() > 0 {
            line_number += 1;
            check_line(
                config,
                output,
                line_number,
                std::str::from_utf8(&buf).unwrap(),
            );
            buf.clear();
        }
    } else {
        while reader.read_until(b'\n', &mut buf).unwrap() > 0 {
            line_number += 1;
            check_line(
                config,
                output,
                line_number,
                std::str::from_utf8(&buf).unwrap(),
            );
            buf.clear();
        }
    }
    Ok(())
}

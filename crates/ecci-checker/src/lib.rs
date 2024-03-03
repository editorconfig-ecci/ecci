use ecci_editorconfig::*;
use std::io::BufRead;
mod end_of_line;

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
    end_of_line::check_end_of_line(config, output, line_number, content)
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

use ecci::selection::{select_paths, ErrorReason, Outcome, SkipReason};
use ecci_report::{
    render_summary, Diagnostic, ExecutionError, ExecutionErrorKind, Finding, IntentionalSkip,
    Location, Report, SafeDebugDetail, TextRenderOptions,
};
use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct Options {
    paths: Vec<PathBuf>,
    show_skips: bool,
    debug: bool,
}

fn main() {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(run);
    std::panic::set_hook(previous_hook);
    let status = match result {
        Ok(status) => status,
        Err(_) => {
            let mut report = Report::default();
            report.push(Diagnostic::ExecutionError(ExecutionError::new(
                ExecutionErrorKind::Internal,
                "unexpected checker failure; rerun with --debug and report this error",
            )));
            write_report(&report, TextRenderOptions::default());
            report.exit_status() as i32
        }
    };
    std::process::exit(status);
}

fn run() -> i32 {
    let options = match parse_args(std::env::args_os().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            let mut report = Report::default();
            report.push(Diagnostic::ExecutionError(ExecutionError::new(
                ExecutionErrorKind::Configuration,
                message,
            )));
            write_report(&report, TextRenderOptions::default());
            return report.exit_status() as i32;
        }
    };
    let mut report = Report::default();
    let selection = select_paths(&options.paths);
    for outcome in selection.outcomes {
        match outcome {
            Outcome::Skip { path, reason } => {
                report.push(Diagnostic::IntentionalSkip(IntentionalSkip {
                    message: skip_message(reason).into(),
                    location: Some(Location::path(display_path(&path))),
                }))
            }
            Outcome::DirectFileIgnoreOverride { .. } => {}
            Outcome::Error {
                path,
                reason,
                operation,
                detail,
            } => {
                let kind = if reason == ErrorReason::Configuration {
                    ExecutionErrorKind::Configuration
                } else {
                    ExecutionErrorKind::Io
                };
                let error = ExecutionError::new(kind, format!("{operation} failed"))
                    .with_safe_debug_detail(SafeDebugDetail::from_sanitized(detail));
                let mut error = error;
                error.location = Some(Location::path(display_path(&path)));
                report.push(Diagnostic::ExecutionError(error));
            }
        }
    }
    for file in selection.files {
        let mut output = CheckerOutput {
            report: &mut report,
            config: &file.config,
        };
        match ecci_checker::check_all(&file.config, &mut output) {
            Ok(()) => report.record_checked_file(),
            Err(error) => {
                let diagnostic =
                    ExecutionError::new(ExecutionErrorKind::Io, "failed to read selected file")
                        .with_safe_debug_detail(SafeDebugDetail::from_sanitized(error.to_string()));
                let mut diagnostic = diagnostic;
                diagnostic.location = Some(Location::path(display_path(&file.path)));
                report.push(Diagnostic::ExecutionError(diagnostic));
            }
        }
    }
    write_report(
        &report,
        TextRenderOptions {
            show_skips: options.show_skips,
            debug: options.debug,
        },
    );
    report.exit_status() as i32
}

fn parse_args(args: impl Iterator<Item = OsString>) -> Result<Options, String> {
    let mut options = Options::default();
    let mut positional_only = false;
    for arg in args {
        if !positional_only && arg == "--" {
            positional_only = true;
        } else if !positional_only && arg == "--show-skips" {
            if std::mem::replace(&mut options.show_skips, true) {
                return Err("option --show-skips may be specified only once".into());
            }
        } else if !positional_only && arg == "--debug" {
            if std::mem::replace(&mut options.debug, true) {
                return Err("option --debug may be specified only once".into());
            }
        } else if !positional_only && arg.to_string_lossy().starts_with('-') {
            return Err(format!("unsupported option {:?}", arg.to_string_lossy()));
        } else {
            options.paths.push(PathBuf::from(arg));
        }
    }
    if options.paths.is_empty() {
        options.paths.push(PathBuf::from("."));
    }
    Ok(options)
}

struct CheckerOutput<'a> {
    report: &'a mut Report,
    config: &'a ecci_editorconfig::Config,
}

impl ecci_checker::Output for CheckerOutput<'_> {
    fn output(
        &mut self,
        line: usize,
        start: usize,
        _length: usize,
        path: &str,
        content: &str,
        rule: &str,
    ) {
        let (code, expected, observed) = finding_values(self.config, rule, content, start);
        let message = match (&expected, &observed) {
            (Some(expected), Some(observed)) => {
                format!("{rule} must be {expected}; found {observed}")
            }
            _ => format!("{rule} does not conform"),
        };
        let mut finding = Finding::new(code, rule, message);
        finding.expected = expected;
        finding.observed = observed;
        finding.location = Some(Location::at(
            display_path(Path::new(path)),
            NonZeroUsize::new(line.max(1)).unwrap(),
            NonZeroUsize::new(start + 1).unwrap(),
        ));
        self.report.push(Diagnostic::Finding(finding));
    }
}

fn finding_values(
    config: &ecci_editorconfig::Config,
    rule: &str,
    content: &str,
    start: usize,
) -> (&'static str, Option<String>, Option<String>) {
    let code = match rule {
        "indent_style" => "ECCI001",
        "indent_size" => "ECCI002",
        "end_of_line" => "ECCI003",
        "charset" => "ECCI004",
        "trim_trailing_whitespace" => "ECCI005",
        "insert_final_newline" => "ECCI006",
        "max_line_length" => "ECCI007",
        _ => "ECCI000",
    };
    let observed = match rule {
        "indent_style" => content
            .as_bytes()
            .get(start)
            .map(|b| if *b == b'\t' { "tab" } else { "space" }.into()),
        "max_line_length" => Some(
            content
                .trim_end_matches(['\r', '\n'])
                .chars()
                .count()
                .to_string(),
        ),
        _ => None,
    };
    let expected = match rule {
        "indent_style" => config.indent_style.as_ref().map(|value| match value {
            ecci_editorconfig::IndentStyle::Tab => "tab".into(),
            ecci_editorconfig::IndentStyle::Space => "space".into(),
        }),
        "indent_size" => config.indent_size.map(|value| value.to_string()),
        "end_of_line" => config.end_of_line.as_ref().map(|value| match value {
            ecci_editorconfig::EndOfLine::LF => "lf".into(),
            ecci_editorconfig::EndOfLine::CRLF => "crlf".into(),
            ecci_editorconfig::EndOfLine::CR => "cr".into(),
        }),
        "charset" => config
            .charset
            .as_ref()
            .map(|value| format!("{value:?}").to_ascii_lowercase()),
        "trim_trailing_whitespace" => Some("no trailing whitespace".into()),
        "insert_final_newline" => Some("a final newline".into()),
        "max_line_length" => config.max_line_length.map(|value| value.to_string()),
        _ => None,
    };
    (code, expected, observed)
}

fn skip_message(reason: SkipReason) -> &'static str {
    match reason {
        SkipReason::Gitignore => "excluded by .gitignore; skipped",
        SkipReason::Ecciignore => "excluded by .ecciignore; skipped",
        SkipReason::NoEditorConfig => "no .editorconfig applies; skipped",
        SkipReason::Binary => "binary file; skipped",
        SkipReason::Symlink => "symbolic link encountered during traversal; skipped",
        SkipReason::Duplicate => "duplicate file; skipped",
    }
}

fn display_path(path: &Path) -> PathBuf {
    let cwd = std::env::current_dir().ok();
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.as_deref().unwrap_or(Path::new(".")).join(path)
    };
    cwd.and_then(|base| pathdiff(&absolute, &base))
        .unwrap_or(absolute)
}

fn pathdiff(path: &Path, base: &Path) -> Option<PathBuf> {
    path.strip_prefix(base).ok().map(Path::to_path_buf)
}

fn write_report(report: &Report, options: TextRenderOptions) {
    let rendered = ecci_report::render_text(report, options);
    let summary = render_summary(report);
    let diagnostics = rendered
        .strip_suffix('\n')
        .and_then(|text| text.strip_suffix(&summary))
        .unwrap_or("")
        .trim_end_matches('\n');
    if !diagnostics.is_empty() {
        eprintln!("{diagnostics}");
    }
    println!("{summary}");
}

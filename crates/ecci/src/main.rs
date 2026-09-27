use clap::Parser;
use ecci::selection::{select_paths, ErrorReason, Outcome, SkipReason};
use ecci_report::{
    render_summary, Diagnostic, ExecutionError, ExecutionErrorKind, Finding, IntentionalSkip,
    Location, Report, SafeDebugDetail, TextRenderOptions,
};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(disable_help_flag = true, disable_version_flag = true)]
struct Cli {
    #[arg(long)]
    github_action: bool,

    #[arg(long)]
    show_skips: bool,

    #[arg(long)]
    debug: bool,

    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,
}

struct Options {
    paths: Vec<PathBuf>,
    show_skips: bool,
    debug: bool,
    github_action: Option<ecci::action::ActionOptions>,
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
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let action_requested = args.iter().any(|arg| arg == "--github-action");
    let options = match parse_args(args.into_iter()) {
        Ok(options) => options,
        Err(message) => {
            let mut report = Report::default();
            report.push(Diagnostic::ExecutionError(ExecutionError::new(
                ExecutionErrorKind::Configuration,
                message,
            )));
            if action_requested {
                ecci::action::render_configuration_error(&report);
            } else {
                write_report(&report, TextRenderOptions::default());
            }
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
    if let Some(action) = &options.github_action {
        ecci::action::render(&report, action)
    } else {
        write_report(
            &report,
            TextRenderOptions {
                show_skips: options.show_skips,
                debug: options.debug,
            },
        );
        report.exit_status() as i32
    }
}

fn parse_args(args: impl IntoIterator<Item = std::ffi::OsString>) -> Result<Options, String> {
    let args: Vec<_> = args.into_iter().collect();
    let cli = Cli::try_parse_from(std::iter::once("ecci".into()).chain(args.iter().cloned()))
        .map_err(|_| unsupported_option(&args))?;

    let mut options = Options {
        paths: cli.paths,
        show_skips: cli.show_skips,
        debug: cli.debug,
        github_action: None,
    };
    if cli.github_action {
        let action = ecci::action::ActionOptions::from_env()?;
        options.paths = action.paths.clone();
        options.paths.extend(paths_after_github_action(&args));
        options.github_action = Some(action);
    }
    if options.paths.is_empty() {
        options.paths.push(PathBuf::from("."));
    }
    Ok(options)
}

fn unsupported_option(args: &[std::ffi::OsString]) -> String {
    let option = args
        .iter()
        .take_while(|arg| *arg != "--")
        .find(|arg| {
            let arg = arg.to_string_lossy();
            arg.starts_with('-')
                && !matches!(arg.as_ref(), "--github-action" | "--show-skips" | "--debug")
        })
        .map(|arg| arg.to_string_lossy());
    match option {
        Some(option) => format!("unsupported option {option:?}"),
        None => "invalid command-line arguments".into(),
    }
}

fn paths_after_github_action(args: &[std::ffi::OsString]) -> Vec<PathBuf> {
    let mut action_seen = false;
    let mut positional_only = false;
    let mut paths = Vec::new();
    for arg in args {
        if !positional_only && arg == "--" {
            positional_only = true;
        } else if !positional_only && arg == "--github-action" {
            action_seen = true;
            paths.clear();
        } else if !positional_only && matches!(arg.to_str(), Some("--show-skips" | "--debug")) {
        } else if action_seen {
            paths.push(PathBuf::from(arg));
        }
    }
    paths
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
        length: usize,
        path: &str,
        content: &str,
        rule: &str,
    ) {
        let (property, _kind) = rule
            .split_once('.')
            .expect("checker diagnostic codes must use property.kind");
        let (expected, observed) = finding_values(self.config, property, content, start, length);
        let message = format_finding_message(property, expected.as_deref(), observed.as_deref());
        let mut finding = Finding::new(rule, property, message);
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
    length: usize,
) -> (Option<String>, Option<String>) {
    let observed = match rule {
        "indent_style" => content
            .as_bytes()
            .get(start)
            .map(|b| if *b == b'\t' { "tab" } else { "space" }.into()),
        "indent_size" => Some(length.to_string()),
        "end_of_line" => Some(detected_line_endings(config, content, start)),
        "charset" => Some(detected_charset_detail(config).into()),
        "trim_trailing_whitespace" => Some(format!("{length} trailing whitespace bytes")),
        "insert_final_newline" => Some("final newline absent".into()),
        "max_line_length" => Some(format!(
            "{} bytes",
            content.trim_end_matches(['\r', '\n']).len()
        )),
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
        "charset" => config.charset.as_ref().map(charset_name),
        "trim_trailing_whitespace" => config
            .trim_trailing_whitespace
            .map(|value| value.to_string()),
        "insert_final_newline" => config.insert_final_newline.map(|value| value.to_string()),
        "max_line_length" => config.max_line_length.map(|value| value.to_string()),
        _ => None,
    };
    (expected, observed)
}

fn format_finding_message(
    property: &str,
    expected: Option<&str>,
    observed: Option<&str>,
) -> String {
    match (expected, observed) {
        (Some(expected), Some(observed)) => {
            format!("expected {property}={expected}; detected {property}={observed}")
        }
        (Some(expected), None) => {
            format!("expected {property}={expected}; no detected value available")
        }
        (None, Some(observed)) => format!("detected {property}={observed}"),
        (None, None) => format!("{property} check failed without an observable scalar value"),
    }
}

fn detected_line_endings(
    config: &ecci_editorconfig::Config,
    content: &str,
    start: usize,
) -> String {
    if let Ok(bytes) = std::fs::read(&config.path) {
        let mut crlf = 0;
        let mut cr = 0;
        let mut lf = 0;
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index..].get(..2) {
                Some(b"\r\n") => {
                    crlf += 1;
                    index += 2;
                    continue;
                }
                _ if bytes[index] == b'\r' => cr += 1,
                _ if bytes[index] == b'\n' => lf += 1,
                _ => {}
            }
            index += 1;
        }
        let kinds = usize::from(crlf > 0) + usize::from(cr > 0) + usize::from(lf > 0);
        if kinds > 1 {
            return format!("mixed (crlf={crlf}, cr={cr}, lf={lf})");
        }
        if crlf > 0 {
            return "crlf".into();
        }
        if cr > 0 {
            return "cr".into();
        }
        if lf > 0 {
            return "lf".into();
        }
    }
    match content.as_bytes().get(start..) {
        Some(bytes) if bytes.starts_with(b"\r\n") => "crlf".into(),
        Some(bytes) if bytes.starts_with(b"\r") => "cr".into(),
        Some(bytes) if bytes.starts_with(b"\n") => "lf".into(),
        _ => "unrecognized line ending".into(),
    }
}

fn charset_name(value: &ecci_editorconfig::Charset) -> String {
    use ecci_editorconfig::Charset;
    match value {
        Charset::Latin1 => "latin1",
        Charset::UTF8 => "utf-8",
        Charset::UTF8BOM => "utf-8-bom",
        Charset::UTF16BE => "utf-16be",
        Charset::UTF16LE => "utf-16le",
    }
    .into()
}

fn detected_charset_detail(config: &ecci_editorconfig::Config) -> &'static str {
    use ecci_editorconfig::Charset;
    let Ok(bytes) = std::fs::read(&config.path) else {
        return "encoding mismatch (bytes unavailable for classification)";
    };
    match config.charset.as_ref() {
        Some(Charset::UTF8) if bytes.starts_with(b"\xef\xbb\xbf") => "utf-8 BOM present",
        Some(Charset::UTF8) => "invalid utf-8 byte sequence",
        Some(Charset::UTF8BOM) if !bytes.starts_with(b"\xef\xbb\xbf") => "utf-8 BOM absent",
        Some(Charset::UTF8BOM) => "invalid utf-8 byte sequence after BOM",
        Some(Charset::UTF16BE) if bytes.starts_with(b"\xff\xfe") => "utf-16le BOM present",
        Some(Charset::UTF16LE) if bytes.starts_with(b"\xfe\xff") => "utf-16be BOM present",
        Some(Charset::UTF16BE | Charset::UTF16LE) if bytes.len() % 2 != 0 => "odd byte count",
        Some(Charset::UTF16BE | Charset::UTF16LE) => "invalid utf-16 code unit sequence",
        _ => "encoding mismatch",
    }
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

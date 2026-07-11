use ecci_report::{render_summary, Diagnostic, ExitStatus, Report, TextRenderOptions};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const SUMMARY_DIAGNOSTIC_LIMIT: usize = 20;
const SUMMARY_BYTE_LIMIT: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogLevel {
    Quiet,
    Summary,
    Diagnostic,
    Debug,
}

pub struct ActionOptions {
    pub paths: Vec<PathBuf>,
    pub fail_on_violation: bool,
    pub annotations: bool,
    pub summary: bool,
    pub max_annotations: usize,
    pub log_level: LogLevel,
}

impl ActionOptions {
    pub fn from_env() -> Result<Self, String> {
        let workspace = canonical_dir(
            "GITHUB_WORKSPACE",
            env::var_os("GITHUB_WORKSPACE").ok_or("GITHUB_WORKSPACE is required")?,
        )?;
        let working_value =
            env::var_os("INPUT_WORKING_DIRECTORY").ok_or("working-directory is required")?;
        let working = resolve_confined_dir(&workspace, Path::new(&working_value))?;
        env::set_current_dir(&working)
            .map_err(|_| "failed to enter working-directory".to_owned())?;
        let raw_paths =
            env::var("INPUT_PATHS").map_err(|_| "paths must be valid UTF-8".to_owned())?;
        let mut paths = Vec::new();
        for line in raw_paths.lines() {
            let value = line.strip_suffix('\r').unwrap_or(line);
            if value.is_empty() {
                return Err("paths must not contain empty lines".into());
            }
            let path = Path::new(value);
            if path.is_absolute() {
                return Err(format!("path {value:?} must be repository-relative"));
            }
            ensure_lexically_confined(&working, path, &workspace)?;
            let joined = working.join(path);
            if !canonical_existing_ancestor(&joined).starts_with(&workspace) {
                return Err(format!("path {value:?} resolves outside GITHUB_WORKSPACE"));
            }
            paths.push(path.to_owned());
        }
        if paths.is_empty() {
            return Err("paths must contain at least one non-empty line".into());
        }
        Ok(Self {
            paths,
            fail_on_violation: boolean("INPUT_FAIL_ON_VIOLATION")?,
            annotations: boolean("INPUT_ANNOTATIONS")?,
            summary: boolean("INPUT_SUMMARY")?,
            max_annotations: env::var("INPUT_MAX_ANNOTATIONS")
                .map_err(|_| "max-annotations is required")?
                .parse()
                .map_err(|_| "max-annotations must be a non-negative decimal integer".to_owned())?,
            log_level: match env::var("INPUT_LOG_LEVEL").as_deref() {
                Ok("quiet") => LogLevel::Quiet,
                Ok("summary") => LogLevel::Summary,
                Ok("diagnostic") => LogLevel::Diagnostic,
                Ok("debug") => LogLevel::Debug,
                _ => return Err("log-level must be quiet, summary, diagnostic, or debug".into()),
            },
        })
    }
}

fn canonical_existing_ancestor(path: &Path) -> PathBuf {
    let mut candidate = path;
    loop {
        if let Ok(canonical) = candidate.canonicalize() {
            return canonical;
        }
        candidate = candidate
            .parent()
            .expect("an absolute path has an existing ancestor");
    }
}

fn boolean(name: &str) -> Result<bool, String> {
    match env::var(name).as_deref() {
        Ok("true") => Ok(true),
        Ok("false") => Ok(false),
        _ => Err(format!(
            "{} must be exactly true or false",
            name.trim_start_matches("INPUT_")
                .to_ascii_lowercase()
                .replace('_', "-")
        )),
    }
}

fn canonical_dir(name: &str, value: std::ffi::OsString) -> Result<PathBuf, String> {
    PathBuf::from(value)
        .canonicalize()
        .map_err(|_| format!("{name} must name an existing directory"))
}

fn resolve_confined_dir(workspace: &Path, value: &Path) -> Result<PathBuf, String> {
    let candidate = if value.is_absolute() {
        value.to_owned()
    } else {
        workspace.join(value)
    };
    let canonical = candidate
        .canonicalize()
        .map_err(|_| "working-directory must name an existing directory".to_owned())?;
    if !canonical.starts_with(workspace) {
        return Err("working-directory resolves outside GITHUB_WORKSPACE".into());
    }
    if !canonical.is_dir() {
        return Err("working-directory must name a directory".into());
    }
    Ok(canonical)
}

fn ensure_lexically_confined(base: &Path, path: &Path, workspace: &Path) -> Result<(), String> {
    let mut candidate = base.to_path_buf();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(v) => candidate.push(v),
            Component::ParentDir => {
                candidate.pop();
            }
            _ => return Err("paths must be repository-relative".into()),
        }
    }
    if !candidate.starts_with(workspace) {
        return Err(format!("path {:?} resolves outside GITHUB_WORKSPACE", path));
    }
    Ok(())
}

pub fn render(report: &Report, options: &ActionOptions) -> i32 {
    let counters = report.counters();
    let status = report.exit_status();
    write_outputs(
        status,
        counters.checked_files,
        counters.violations,
        counters.skipped_files,
    );
    if options.annotations {
        write_annotations(report, options.max_annotations);
    }
    match options.log_level {
        LogLevel::Quiet => {}
        LogLevel::Summary => println!("{}", render_summary(report)),
        LogLevel::Diagnostic | LogLevel::Debug => eprint!(
            "{}",
            ecci_report::render_text(
                report,
                TextRenderOptions {
                    show_skips: true,
                    debug: options.log_level == LogLevel::Debug
                }
            )
        ),
    }
    if options.summary {
        write_summary(report);
    }
    if status == ExitStatus::Violations && !options.fail_on_violation {
        0
    } else {
        status as i32
    }
}

pub fn render_configuration_error(report: &Report) {
    let counters = report.counters();
    write_outputs(
        report.exit_status(),
        counters.checked_files,
        counters.violations,
        counters.skipped_files,
    );
    if env::var("INPUT_ANNOTATIONS").as_deref() != Ok("false") {
        write_annotations(report, 1);
    }
    if env::var("INPUT_SUMMARY").as_deref() != Ok("false") {
        write_summary(report);
    }
}

fn outcome(status: ExitStatus) -> &'static str {
    match status {
        ExitStatus::Success => "success",
        ExitStatus::Violations => "violations",
        ExitStatus::ConfigurationError => "configuration-error",
        ExitStatus::IoError => "io-error",
        ExitStatus::InternalError => "internal-error",
    }
}

fn write_outputs(status: ExitStatus, checked: usize, violations: usize, skipped: usize) {
    let Some(file) = env::var_os("GITHUB_OUTPUT") else {
        return;
    };
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(file) {
        let _ = writeln!(file, "outcome={}", outcome(status));
        let _ = writeln!(file, "violations={violations}");
        let _ = writeln!(file, "checked-files={checked}");
        let _ = writeln!(file, "skipped-files={skipped}");
    }
}

fn escape_data(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}
fn escape_property(value: &str) -> String {
    escape_data(value).replace(':', "%3A").replace(',', "%2C")
}

fn write_annotations(report: &Report, limit: usize) {
    let reportable: Vec<_> = report
        .diagnostics()
        .iter()
        .filter(|d| !matches!(d, Diagnostic::IntentionalSkip(_)))
        .collect();
    for diagnostic in reportable.iter().take(limit) {
        let mut properties = format!("title={}", escape_property(diagnostic.code().as_str()));
        if let Some(location) = diagnostic.location() {
            if let Some(path) = &location.path {
                properties.push_str(&format!(
                    ",file={}",
                    escape_property(&path.to_string_lossy().replace('\\', "/"))
                ));
            }
            if let Some(line) = location.line {
                properties.push_str(&format!(",line={line}"));
            }
            if let Some(column) = location.column {
                properties.push_str(&format!(",col={column}"));
            }
        }
        println!(
            "::error {properties}::{}",
            escape_data(diagnostic.message())
        );
    }
    if reportable.len() > limit {
        println!(
            "::notice title=ecci::{} annotations suppressed by max-annotations",
            reportable.len() - limit
        );
    }
}

fn write_summary(report: &Report) {
    let Some(file) = env::var_os("GITHUB_STEP_SUMMARY") else {
        return;
    };
    let counters = report.counters();
    let status = report.exit_status();
    let mut text = format!("## ecci\n\n**Outcome:** {}\n\n| Checked | Violations | Skipped | Execution errors |\n| ---: | ---: | ---: | ---: |\n| {} | {} | {} | {} |\n", outcome(status), counters.checked_files, counters.violations, counters.skipped_files, counters.execution_errors);
    let findings: Vec<_> = report
        .diagnostics()
        .iter()
        .filter(|d| matches!(d, Diagnostic::Finding(_)))
        .collect();
    if !findings.is_empty() {
        text.push_str("\n### Violations\n\n");
        for diagnostic in findings.iter().take(SUMMARY_DIAGNOSTIC_LIMIT) {
            let path = diagnostic
                .location()
                .and_then(|l| l.path.as_ref())
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|| "(unknown location)".into());
            text.push_str(&format!(
                "- `{}`: **{}** {}\n",
                markdown(&path),
                diagnostic.code().as_str(),
                markdown(diagnostic.message())
            ));
        }
        if findings.len() > SUMMARY_DIAGNOSTIC_LIMIT {
            text.push_str(&format!(
                "\n{} violations omitted.\n",
                findings.len() - SUMMARY_DIAGNOSTIC_LIMIT
            ));
        }
    }
    if text.len() > SUMMARY_BYTE_LIMIT {
        let mut end = SUMMARY_BYTE_LIMIT;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(file) {
        let _ = file.write_all(text.as_bytes());
    }
}

fn markdown(value: &str) -> String {
    value
        .replace(['\r', '\n'], " ")
        .replace('`', "\\`")
        .replace('|', "\\|")
}

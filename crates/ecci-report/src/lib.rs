//! Typed results shared by ecci front ends.
//!
//! Renderers consume [`Report`] directly. Rendered diagnostic text is a human
//! interface and must not be parsed to recover report data.

use std::fmt::{self, Write};
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// A stable, renderer-independent diagnostic identifier.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticCode {
    Finding(String),
    Skip,
    Configuration,
    Io,
    Internal,
}

impl DiagnosticCode {
    pub fn finding(code: impl Into<String>) -> Self {
        Self::Finding(code.into())
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Finding(code) => code,
            Self::Skip => "selection.skipped",
            Self::Configuration => "config.invalid",
            Self::Io => "io.failed",
            Self::Internal => "internal.unexpected",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Category {
    Finding,
    IntentionalSkip,
    ConfigurationError,
    IoError,
    InternalError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Warning,
    Error,
}

/// An optional path and one-based source position.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Location {
    pub path: Option<PathBuf>,
    pub line: Option<NonZeroUsize>,
    pub column: Option<NonZeroUsize>,
}

impl Location {
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: Some(path.into()),
            ..Self::default()
        }
    }

    pub fn at(path: impl Into<PathBuf>, line: NonZeroUsize, column: NonZeroUsize) -> Self {
        Self {
            path: Some(path.into()),
            line: Some(line),
            column: Some(column),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub code: DiagnosticCode,
    pub message: String,
    pub location: Option<Location>,
    pub property: String,
    pub expected: Option<String>,
    pub observed: Option<String>,
}

impl Finding {
    pub fn new(
        code: impl Into<String>,
        property: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: DiagnosticCode::finding(code),
            message: message.into(),
            location: None,
            property: property.into(),
            expected: None,
            observed: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentionalSkip {
    pub message: String,
    pub location: Option<Location>,
}

/// Debug-only causal text which the producer has explicitly classified as safe.
///
/// Producers must remove secrets, environment values, target-file content,
/// backtraces, and host-specific absolute paths before constructing this type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeDebugDetail(String);

impl SafeDebugDetail {
    pub fn from_sanitized(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionErrorKind {
    Configuration,
    Io,
    Internal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionError {
    pub kind: ExecutionErrorKind,
    pub message: String,
    pub location: Option<Location>,
    debug_details: Vec<SafeDebugDetail>,
}

impl ExecutionError {
    pub fn new(kind: ExecutionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            location: None,
            debug_details: Vec::new(),
        }
    }

    pub fn with_safe_debug_detail(mut self, detail: SafeDebugDetail) -> Self {
        self.debug_details.push(detail);
        self
    }

    pub fn debug_details(&self) -> &[SafeDebugDetail] {
        &self.debug_details
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Diagnostic {
    Finding(Finding),
    IntentionalSkip(IntentionalSkip),
    ExecutionError(ExecutionError),
}

impl Diagnostic {
    pub fn code(&self) -> &DiagnosticCode {
        match self {
            Self::Finding(finding) => &finding.code,
            Self::IntentionalSkip(_) => &DiagnosticCode::Skip,
            Self::ExecutionError(error) => match error.kind {
                ExecutionErrorKind::Configuration => &DiagnosticCode::Configuration,
                ExecutionErrorKind::Io => &DiagnosticCode::Io,
                ExecutionErrorKind::Internal => &DiagnosticCode::Internal,
            },
        }
    }

    pub fn category(&self) -> Category {
        match self {
            Self::Finding(_) => Category::Finding,
            Self::IntentionalSkip(_) => Category::IntentionalSkip,
            Self::ExecutionError(error) => match error.kind {
                ExecutionErrorKind::Configuration => Category::ConfigurationError,
                ExecutionErrorKind::Io => Category::IoError,
                ExecutionErrorKind::Internal => Category::InternalError,
            },
        }
    }

    pub fn severity(&self) -> Severity {
        match self {
            Self::IntentionalSkip(_) => Severity::Warning,
            Self::Finding(_) | Self::ExecutionError(_) => Severity::Error,
        }
    }

    pub fn location(&self) -> Option<&Location> {
        match self {
            Self::Finding(value) => value.location.as_ref(),
            Self::IntentionalSkip(value) => value.location.as_ref(),
            Self::ExecutionError(value) => value.location.as_ref(),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Finding(value) => &value.message,
            Self::IntentionalSkip(value) => &value.message,
            Self::ExecutionError(value) => &value.message,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counters {
    pub checked_files: usize,
    pub violations: usize,
    pub skipped_files: usize,
    pub execution_errors: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ExitStatus {
    Success = 0,
    Violations = 1,
    ConfigurationError = 2,
    IoError = 3,
    InternalError = 4,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
    checked_files: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Report {
    pub fn record_checked_file(&mut self) {
        self.checked_files += 1;
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        if matches!(
            diagnostic,
            Diagnostic::ExecutionError(ExecutionError {
                kind: ExecutionErrorKind::Internal,
                ..
            })
        ) && self.diagnostics.iter().any(|existing| {
            matches!(
                existing,
                Diagnostic::ExecutionError(ExecutionError {
                    kind: ExecutionErrorKind::Internal,
                    ..
                })
            )
        }) {
            return;
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn counters(&self) -> Counters {
        let mut counters = Counters {
            checked_files: self.checked_files,
            ..Counters::default()
        };
        for diagnostic in &self.diagnostics {
            match diagnostic {
                Diagnostic::Finding(_) => counters.violations += 1,
                Diagnostic::IntentionalSkip(_) => counters.skipped_files += 1,
                Diagnostic::ExecutionError(_) => counters.execution_errors += 1,
            }
        }
        counters
    }

    /// Computes the invocation result using internal > configuration > I/O >
    /// findings precedence. This deliberately does not use numeric maximum.
    pub fn exit_status(&self) -> ExitStatus {
        let mut has_finding = false;
        let mut has_io_error = false;
        let mut has_configuration_error = false;
        for diagnostic in &self.diagnostics {
            match diagnostic {
                Diagnostic::Finding(_) => has_finding = true,
                Diagnostic::ExecutionError(error) => match error.kind {
                    ExecutionErrorKind::Internal => return ExitStatus::InternalError,
                    ExecutionErrorKind::Configuration => has_configuration_error = true,
                    ExecutionErrorKind::Io => has_io_error = true,
                },
                Diagnostic::IntentionalSkip(_) => {}
            }
        }
        if has_configuration_error {
            ExitStatus::ConfigurationError
        } else if has_io_error {
            ExitStatus::IoError
        } else if has_finding {
            ExitStatus::Violations
        } else {
            ExitStatus::Success
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextRenderOptions {
    pub show_skips: bool,
    pub debug: bool,
}

pub fn render_text(report: &Report, options: TextRenderOptions) -> String {
    let mut output = String::new();
    for diagnostic in report.diagnostics() {
        if matches!(diagnostic, Diagnostic::IntentionalSkip(_)) && !options.show_skips {
            continue;
        }
        render_diagnostic(&mut output, diagnostic);
        if options.debug {
            if let Diagnostic::ExecutionError(error) = diagnostic {
                for detail in error.debug_details() {
                    let _ = writeln!(
                        output,
                        "  debug: caused by: {}",
                        single_line(detail.as_str())
                    );
                }
            }
        }
    }
    let _ = writeln!(output, "{}", render_summary(report));
    output
}

pub fn render_summary(report: &Report) -> String {
    let counters = report.counters();
    if counters.checked_files == 0
        && counters.violations == 0
        && counters.skipped_files == 0
        && counters.execution_errors == 0
    {
        return "Checked 0 files: no targets selected.".to_owned();
    }
    format!(
        "Checked {} files: {} violations, {} skipped, {} execution errors.",
        counters.checked_files,
        counters.violations,
        counters.skipped_files,
        counters.execution_errors
    )
}

fn render_diagnostic(output: &mut String, diagnostic: &Diagnostic) {
    let severity = match diagnostic.severity() {
        Severity::Warning => "warning",
        Severity::Error => "error",
    };
    let _ = write!(output, "{severity}[{}]", diagnostic.code().as_str());
    if let Some(location) = diagnostic.location() {
        render_location(output, location);
    }
    let _ = writeln!(output, ": {}", single_line(diagnostic.message()));
}

fn render_location(output: &mut String, location: &Location) {
    if let Some(path) = &location.path {
        let rendered = path.to_string_lossy().replace('\\', "/");
        let _ = write!(output, " {}", single_line(&rendered));
    }
    if let Some(line) = location.line {
        let _ = write!(output, ":{line}");
        if let Some(column) = location.column {
            let _ = write!(output, ":{column}");
        }
    }
}

fn single_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", *self as i32)
    }
}

#[cfg(test)]
mod tests;

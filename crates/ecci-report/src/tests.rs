use super::*;

fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}

fn finding() -> Diagnostic {
    let mut finding = Finding::new(
        "ECCI001",
        "indent_style",
        "indent_style must be space; found tab",
    );
    finding.expected = Some("space".into());
    finding.observed = Some("tab".into());
    finding.location = Some(Location::at("src\\lib.rs", nonzero(14), nonzero(1)));
    Diagnostic::Finding(finding)
}

fn execution_error(kind: ExecutionErrorKind) -> Diagnostic {
    Diagnostic::ExecutionError(ExecutionError::new(kind, "could not complete check"))
}

#[test]
fn exposes_stable_codes_categories_and_finding_values() {
    let finding = finding();
    assert_eq!(finding.code().as_str(), "ECCI001");
    assert_eq!(finding.category(), Category::Finding);
    let skip = Diagnostic::IntentionalSkip(IntentionalSkip {
        message: "no .editorconfig applies; skipped".into(),
        location: None,
    });
    assert_eq!(skip.code().as_str(), "ECCI-SKIP");
    assert_eq!(skip.category(), Category::IntentionalSkip);

    for (kind, code, category) in [
        (
            ExecutionErrorKind::Configuration,
            "ECCI-CONFIG",
            Category::ConfigurationError,
        ),
        (ExecutionErrorKind::Io, "ECCI-IO", Category::IoError),
        (
            ExecutionErrorKind::Internal,
            "ECCI-INTERNAL",
            Category::InternalError,
        ),
    ] {
        let diagnostic = execution_error(kind);
        assert_eq!(diagnostic.code().as_str(), code);
        assert_eq!(diagnostic.category(), category);
    }
}

#[test]
fn renders_diagnostics_with_and_without_locations() {
    let mut report = Report::default();
    report.record_checked_file();
    report.push(finding());
    report.push(execution_error(ExecutionErrorKind::Internal));

    assert_eq!(
        render_text(&report, TextRenderOptions::default()),
        concat!(
            "error[ECCI001] src/lib.rs:14:1: indent_style must be space; found tab\n",
            "error[ECCI-INTERNAL]: could not complete check\n",
            "Checked 1 files: 1 violations, 0 skipped, 1 execution errors.\n"
        )
    );
}

#[test]
fn summary_distinguishes_no_targets_and_aggregates_mixed_results() {
    assert_eq!(
        render_summary(&Report::default()),
        "Checked 0 files: no targets selected."
    );

    let mut report = Report::default();
    report.record_checked_file();
    report.record_checked_file();
    report.push(finding());
    report.push(Diagnostic::IntentionalSkip(IntentionalSkip {
        message: "no .editorconfig applies; skipped".into(),
        location: Some(Location::path("README.md")),
    }));
    report.push(execution_error(ExecutionErrorKind::Io));
    assert_eq!(
        report.counters(),
        Counters {
            checked_files: 2,
            violations: 1,
            skipped_files: 1,
            execution_errors: 1,
        }
    );
    assert_eq!(
        render_summary(&report),
        "Checked 2 files: 1 violations, 1 skipped, 1 execution errors."
    );
}

#[test]
fn exit_status_covers_zero_through_four_and_required_precedence() {
    assert_eq!(Report::default().exit_status(), ExitStatus::Success);

    let mut report = Report::default();
    report.push(finding());
    assert_eq!(report.exit_status(), ExitStatus::Violations);
    report.push(execution_error(ExecutionErrorKind::Io));
    assert_eq!(report.exit_status(), ExitStatus::IoError);
    report.push(execution_error(ExecutionErrorKind::Configuration));
    assert_eq!(report.exit_status(), ExitStatus::ConfigurationError);
    report.push(execution_error(ExecutionErrorKind::Internal));
    assert_eq!(report.exit_status(), ExitStatus::InternalError);

    assert_eq!(ExitStatus::Success as i32, 0);
    assert_eq!(ExitStatus::Violations as i32, 1);
    assert_eq!(ExitStatus::ConfigurationError as i32, 2);
    assert_eq!(ExitStatus::IoError as i32, 3);
    assert_eq!(ExitStatus::InternalError as i32, 4);
}

#[test]
fn debug_details_require_opt_in_and_do_not_change_normal_diagnostic() {
    let error = ExecutionError::new(ExecutionErrorKind::Io, "failed to read file")
        .with_safe_debug_detail(SafeDebugDetail::from_sanitized(
            "read returned permission denied",
        ));
    let mut report = Report::default();
    report.push(Diagnostic::ExecutionError(error));

    let normal = render_text(&report, TextRenderOptions::default());
    assert!(!normal.contains("permission denied"));
    assert_eq!(report.exit_status(), ExitStatus::IoError);

    let debug = render_text(
        &report,
        TextRenderOptions {
            debug: true,
            ..TextRenderOptions::default()
        },
    );
    assert!(debug.contains("  debug: caused by: read returned permission denied\n"));
    assert_eq!(report.exit_status(), ExitStatus::IoError);
}

#[test]
fn renderer_keeps_each_diagnostic_and_debug_cause_on_one_line() {
    let error = ExecutionError::new(ExecutionErrorKind::Io, "failed\nto read")
        .with_safe_debug_detail(SafeDebugDetail::from_sanitized("first\r\nsecond"));
    let mut report = Report::default();
    report.push(Diagnostic::ExecutionError(error));

    let rendered = render_text(
        &report,
        TextRenderOptions {
            debug: true,
            ..TextRenderOptions::default()
        },
    );
    assert!(
        rendered.starts_with("error[ECCI-IO]: failed to read\n  debug: caused by: first  second\n")
    );
}

#[test]
fn report_keeps_exactly_one_internal_error() {
    let mut report = Report::default();
    report.push(execution_error(ExecutionErrorKind::Internal));
    report.push(execution_error(ExecutionErrorKind::Internal));

    assert_eq!(report.counters().execution_errors, 1);
    assert_eq!(report.diagnostics().len(), 1);
}

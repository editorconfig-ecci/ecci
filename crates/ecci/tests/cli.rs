use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

const RELEASE_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/release-semantics"
);

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn project(config: &str, file: &str) -> (TempDir, std::path::PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    write(&temp.path().join(".editorconfig"), config);
    let target = temp.path().join("target.txt");
    write(&target, file);
    (temp, target)
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn release_fixture() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    copy_tree(Path::new(RELEASE_FIXTURE), temp.path());
    fs::write(
        temp.path().join("bom.utf16le"),
        [0xff, 0xfe, b'o', 0, b'k', 0, b'\n', 0],
    )
    .unwrap();
    fs::write(
        temp.path().join("bom.utf16be"),
        [0xfe, 0xff, 0, b'o', 0, b'k', 0, b'\n'],
    )
    .unwrap();
    fs::write(
        temp.path().join("configured.utf16le"),
        [b'o', 0, b'k', 0, b'\n', 0],
    )
    .unwrap();
    fs::write(
        temp.path().join("configured.utf16be"),
        [0, b'o', 0, b'k', 0, b'\n'],
    )
    .unwrap();
    fs::write(temp.path().join("binary.dat"), b"binary\0data").unwrap();
    fs::write(temp.path().join("forced.dat"), b"\tforced\0data\n").unwrap();
    fs::write(temp.path().join("ignored.txt"), b"\tignored\n").unwrap();
    temp
}

#[test]
fn conforming_file_uses_stdout_for_summary_and_keeps_stderr_empty() {
    let (_temp, target) = project("root = true\n[*]\nindent_style = space\n", "ok\n");
    Command::cargo_bin("ecci")
        .unwrap()
        .arg(target)
        .assert()
        .success()
        .stdout("Checked 1 files: 0 violations, 0 skipped, 0 execution errors.\n")
        .stderr("");
}

#[test]
fn violation_has_stable_code_and_one_based_location() {
    let (temp, target) = project("root = true\n[*]\nindent_style = space\n", "ok\n\tbad\n");
    Command::cargo_bin("ecci")
        .unwrap()
        .current_dir(temp.path())
        .arg(&target)
        .assert()
        .code(1)
        .stdout("Checked 1 files: 1 violations, 0 skipped, 0 execution errors.\n")
        .stderr(predicate::str::contains(
            "error[indent_style.invalid_value] target.txt:2:1: expected indent_style=space; detected indent_style=tab\n",
        ));
}

#[test]
fn every_property_violation_reports_expected_and_detected_evidence() {
    let cases = [
        ("indent_style = space\n", "\tbad\n", "expected indent_style=space; detected indent_style=tab"),
        ("indent_style = space\nindent_size = 4\n", "  bad\n", "expected indent_size=4; detected indent_size=2"),
        ("end_of_line = lf\n", "bad\r\nalso\r", "expected end_of_line=lf; detected end_of_line=mixed (crlf=1, cr=1, lf=0)"),
        ("charset = utf-8-bom\n", "bad\n", "expected charset=utf-8-bom; detected charset=utf-8 BOM absent"),
        ("trim_trailing_whitespace = true\n", "bad  \n", "expected trim_trailing_whitespace=true; detected trim_trailing_whitespace=2 trailing whitespace bytes"),
        ("insert_final_newline = true\n", "bad", "expected insert_final_newline=true; detected insert_final_newline=final newline absent"),
        ("max_line_length = 3\n", "1234\n", "expected max_line_length=3; detected max_line_length=4 bytes"),
    ];
    for (setting, contents, message) in cases {
        let config = format!("root = true\n[*]\n{setting}");
        let (temp, target) = project(&config, contents);
        Command::cargo_bin("ecci")
            .unwrap()
            .current_dir(temp.path())
            .arg(target)
            .assert()
            .code(1)
            .stderr(
                predicate::str::contains(message)
                    .and(predicate::str::contains("does not conform").not()),
            );
    }
}

#[test]
fn malformed_configuration_is_status_two_without_platform_error_prose() {
    let (_temp, target) = project("root = true\n[*]\nindent_size = many\n", "ok\n");
    Command::cargo_bin("ecci")
        .unwrap()
        .arg(target)
        .assert()
        .code(2)
        .stdout("Checked 0 files: 0 violations, 0 skipped, 1 execution errors.\n")
        .stderr(
            predicate::str::contains("error[config.invalid]")
                .and(predicate::str::contains("resolve .editorconfig failed")),
        );
}

#[test]
fn missing_target_is_status_three_and_message_is_operation_based() {
    let temp = tempfile::tempdir().unwrap();
    Command::cargo_bin("ecci")
        .unwrap()
        .arg(temp.path().join("missing.txt"))
        .assert()
        .code(3)
        .stdout("Checked 0 files: 0 violations, 0 skipped, 1 execution errors.\n")
        .stderr(
            predicate::str::contains("error[io.failed]")
                .and(predicate::str::contains("inspect direct path failed")),
        );
}

#[test]
fn empty_directory_is_successful_no_selection() {
    let temp = tempfile::tempdir().unwrap();
    Command::cargo_bin("ecci")
        .unwrap()
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout("Checked 0 files: no targets selected.\n")
        .stderr("");
}

#[test]
fn skipped_target_is_counted_and_only_shown_on_request() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("plain.txt");
    write(&target, "plain\n");
    Command::cargo_bin("ecci")
        .unwrap()
        .current_dir(temp.path())
        .arg(&target)
        .assert()
        .success()
        .stdout("Checked 0 files: 0 violations, 1 skipped, 0 execution errors.\n")
        .stderr("");
    Command::cargo_bin("ecci")
        .unwrap()
        .current_dir(temp.path())
        .args(["--show-skips"])
        .arg(&target)
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning[selection.skipped] plain.txt: no .editorconfig applies; skipped",
        ));
}

#[test]
fn independent_mixed_errors_continue_and_execution_error_wins() {
    let (temp, target) = project("root = true\n[*]\nindent_style = space\n", "\tbad\n");
    Command::cargo_bin("ecci")
        .unwrap()
        .current_dir(temp.path())
        .args([target.as_os_str(), temp.path().join("missing").as_os_str()])
        .assert()
        .code(3)
        .stdout("Checked 1 files: 1 violations, 0 skipped, 1 execution errors.\n")
        .stderr(
            predicate::str::contains("error[indent_style.invalid_value]")
                .and(predicate::str::contains("error[io.failed]")),
        );
}

#[test]
fn invalid_and_duplicate_controls_are_configuration_errors() {
    for args in [["--unknown", ""], ["--debug", "--debug"]] {
        let args = args.into_iter().filter(|arg| !arg.is_empty());
        Command::cargo_bin("ecci")
            .unwrap()
            .args(args)
            .assert()
            .code(2)
            .stderr(predicate::str::contains("error[config.invalid]"));
    }
}

#[test]
fn release_fixture_covers_encoding_binary_ignore_and_nested_config_semantics() {
    let fixture = release_fixture();
    Command::cargo_bin("ecci")
        .unwrap()
        .current_dir(fixture.path())
        .args(["--show-skips", "."])
        .assert()
        .code(1)
        .stdout("Checked 9 files: 2 violations, 2 skipped, 0 execution errors.\n")
        .stderr(
            predicate::str::contains("error[indent_style.invalid_value] forced.dat:1:1")
                .and(predicate::str::contains(
                    "error[max_line_length.exceeded] nested/nested.txt:1:4",
                ))
                .and(predicate::str::contains(
                    "warning[selection.skipped] binary.dat: binary file; skipped",
                ))
                .and(predicate::str::contains(
                    "warning[selection.skipped] ignored.txt: excluded by .gitignore; skipped",
                ))
                .and(predicate::str::contains("bom.utf16le").not())
                .and(predicate::str::contains("bom.utf16be").not())
                .and(predicate::str::contains("configured.utf16le").not())
                .and(predicate::str::contains("configured.utf16be").not()),
        );
}

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

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
            "error[ECCI001] target.txt:2:1: indent_style must be space; found tab\n",
        ));
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
            predicate::str::contains("error[ECCI-CONFIG]")
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
            predicate::str::contains("error[ECCI-IO]")
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
            "warning[ECCI-SKIP] plain.txt: no .editorconfig applies; skipped",
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
            predicate::str::contains("error[ECCI001]")
                .and(predicate::str::contains("error[ECCI-IO]")),
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
            .stderr(predicate::str::contains("error[ECCI-CONFIG]"));
    }
}

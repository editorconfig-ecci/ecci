use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

struct ActionFixture {
    temp: TempDir,
    workspace: PathBuf,
    output: PathBuf,
    summary: PathBuf,
}

impl ActionFixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        fs::create_dir(&workspace).unwrap();
        Self {
            output: temp.path().join("output"),
            summary: temp.path().join("summary"),
            temp,
            workspace,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ecci"));
        command
            .arg("--github-action")
            .env("GITHUB_WORKSPACE", &self.workspace)
            .env("GITHUB_OUTPUT", &self.output)
            .env("GITHUB_STEP_SUMMARY", &self.summary)
            .env("INPUT_PATHS", ".")
            .env("INPUT_WORKING_DIRECTORY", ".")
            .env("INPUT_FAIL_ON_VIOLATION", "true")
            .env("INPUT_ANNOTATIONS", "true")
            .env("INPUT_SUMMARY", "true")
            .env("INPUT_MAX_ANNOTATIONS", "50")
            .env("INPUT_LOG_LEVEL", "quiet");
        command
    }
}

#[test]
fn action_emits_escaped_limited_annotations_outputs_and_one_summary() {
    let fixture = ActionFixture::new();
    write(
        &fixture.workspace.join(".editorconfig"),
        "root = true\n[*]\nindent_style = space\n",
    );
    write(&fixture.workspace.join("a,100%.txt"), "\tbad\n");
    write(&fixture.workspace.join("second.txt"), "\tbad\n");
    fixture
        .command()
        .env("INPUT_PATHS", "a,100%.txt\nsecond.txt")
        .env("INPUT_MAX_ANNOTATIONS", "1")
        .env("INPUT_FAIL_ON_VIOLATION", "false")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("file=a%2C100%25.txt,line=1,col=1")
                .and(predicate::str::contains("1 annotations suppressed")),
        );
    assert_eq!(
        fs::read_to_string(&fixture.output).unwrap(),
        "outcome=violations\nviolations=2\nchecked-files=2\nskipped-files=0\n"
    );
    let summary = fs::read_to_string(&fixture.summary).unwrap();
    assert_eq!(summary.matches("## ecci").count(), 1);
    assert!(summary.contains("| 2 | 2 | 0 | 0 |"));
}

#[test]
fn execution_errors_are_not_remapped() {
    let fixture = ActionFixture::new();
    fixture
        .command()
        .env("INPUT_PATHS", "missing")
        .env("INPUT_FAIL_ON_VIOLATION", "false")
        .assert()
        .code(3);
    assert!(fs::read_to_string(&fixture.output)
        .unwrap()
        .contains("outcome=io-error"));
}

#[test]
fn workspace_escape_and_invalid_inputs_are_configuration_errors() {
    let fixture = ActionFixture::new();
    fixture
        .command()
        .env("INPUT_PATHS", "../outside")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("ECCI-CONFIG"));
    fixture
        .command()
        .env("INPUT_FAIL_ON_VIOLATION", "TRUE")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("must be exactly true or false"));
    let outside = fixture.temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fixture
        .command()
        .env("INPUT_WORKING_DIRECTORY", outside)
        .assert()
        .code(2)
        .stdout(predicate::str::contains("outside GITHUB_WORKSPACE"));

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(fixture.temp.path(), fixture.workspace.join("escape")).unwrap();
        fixture
            .command()
            .env("INPUT_PATHS", "escape/not-created")
            .assert()
            .code(2)
            .stdout(predicate::str::contains("outside GITHUB_WORKSPACE"));
    }
}

#[test]
fn action_metadata_declares_the_complete_contract() {
    let metadata =
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../action.yml")).unwrap();
    for name in [
        "paths:",
        "working-directory:",
        "fail-on-violation:",
        "annotations:",
        "summary:",
        "max-annotations:",
        "log-level:",
        "outcome:",
        "violations:",
        "checked-files:",
        "skipped-files:",
    ] {
        assert!(metadata.contains(name), "missing metadata field {name}");
    }
    assert!(metadata.contains("using: 'docker'"));
    assert!(metadata.contains("image: 'Dockerfile'"));
    assert!(!metadata.contains("token"));

    let workflow = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/action-workflow.yml"
    ))
    .unwrap();
    for input in [
        "paths:",
        "working-directory:",
        "fail-on-violation:",
        "annotations:",
        "summary:",
        "max-annotations:",
        "log-level:",
    ] {
        assert!(workflow.contains(input), "smoke fixture omits {input}");
    }
    assert!(workflow.contains("permissions:\n  contents: read"));
}

#[test]
fn container_entrypoint_smoke_test() {
    Command::new("sh")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/container-entrypoint.sh"
        ))
        .assert()
        .success();
}

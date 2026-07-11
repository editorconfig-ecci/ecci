use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const RELEASE_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/release-semantics"
);

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
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

fn populate_release_fixture(workspace: &Path) {
    copy_tree(Path::new(RELEASE_FIXTURE), workspace);
    fs::write(
        workspace.join("bom.utf16le"),
        [0xff, 0xfe, b'o', 0, b'k', 0, b'\n', 0],
    )
    .unwrap();
    fs::write(
        workspace.join("bom.utf16be"),
        [0xfe, 0xff, 0, b'o', 0, b'k', 0, b'\n'],
    )
    .unwrap();
    fs::write(
        workspace.join("configured.utf16le"),
        [b'o', 0, b'k', 0, b'\n', 0],
    )
    .unwrap();
    fs::write(
        workspace.join("configured.utf16be"),
        [0, b'o', 0, b'k', 0, b'\n'],
    )
    .unwrap();
    fs::write(workspace.join("binary.dat"), b"binary\0data").unwrap();
    fs::write(workspace.join("forced.dat"), b"\tforced\0data\n").unwrap();
    fs::write(workspace.join("ignored.txt"), b"\tignored\n").unwrap();
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
                .and(predicate::str::contains(
                    "expected indent_style=space; detected indent_style=tab",
                ))
                .and(predicate::str::contains("1 annotations suppressed")),
        );
    assert_eq!(
        fs::read_to_string(&fixture.output).unwrap(),
        "outcome=violations\nviolations=2\nchecked-files=2\nskipped-files=0\n"
    );
    let summary = fs::read_to_string(&fixture.summary).unwrap();
    assert_eq!(summary.matches("## ecci").count(), 1);
    assert!(summary.contains("| 2 | 2 | 0 | 0 |"));
    assert!(summary.contains("expected indent_style=space; detected indent_style=tab"));
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
fn action_uses_the_cli_selection_report_and_failure_semantics() {
    let fixture = ActionFixture::new();
    populate_release_fixture(&fixture.workspace);
    fixture
        .command()
        .env("INPUT_LOG_LEVEL", "diagnostic")
        .env("INPUT_FAIL_ON_VIOLATION", "true")
        .assert()
        .code(1)
        .stdout(
            predicate::str::contains("file=forced.dat,line=1,col=1").and(predicate::str::contains(
                "file=nested/nested.txt,line=1,col=4",
            )),
        )
        .stderr(
            predicate::str::contains("error[indent_style.invalid_value] forced.dat:1:1").and(
                predicate::str::contains("error[max_line_length.exceeded] nested/nested.txt:1:4"),
            ),
        );
    assert_eq!(
        fs::read_to_string(&fixture.output).unwrap(),
        "outcome=violations\nviolations=2\nchecked-files=9\nskipped-files=2\n"
    );
}

#[test]
fn action_can_suppress_annotations_and_summary_without_changing_the_report() {
    let fixture = ActionFixture::new();
    populate_release_fixture(&fixture.workspace);
    fixture
        .command()
        .env("INPUT_ANNOTATIONS", "false")
        .env("INPUT_SUMMARY", "false")
        .env("INPUT_FAIL_ON_VIOLATION", "false")
        .assert()
        .success()
        .stdout("")
        .stderr("");
    assert_eq!(
        fs::read_to_string(&fixture.output).unwrap(),
        "outcome=violations\nviolations=2\nchecked-files=9\nskipped-files=2\n"
    );
    assert!(!fixture.summary.exists());
}

#[test]
fn container_entrypoint_preserves_release_fixture_semantics() {
    let fixture = ActionFixture::new();
    populate_release_fixture(&fixture.workspace);
    Command::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../entrypoint.sh"))
        .env("ECCI_BIN", env!("CARGO_BIN_EXE_ecci"))
        .env("GITHUB_WORKSPACE", &fixture.workspace)
        .env("GITHUB_OUTPUT", &fixture.output)
        .env("GITHUB_STEP_SUMMARY", &fixture.summary)
        .env("INPUT_PATHS", ".")
        .env("INPUT_WORKING_DIRECTORY", ".")
        .env("INPUT_FAIL_ON_VIOLATION", "false")
        .env("INPUT_ANNOTATIONS", "true")
        .env("INPUT_SUMMARY", "true")
        .env("INPUT_MAX_ANNOTATIONS", "50")
        .env("INPUT_LOG_LEVEL", "quiet")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("file=forced.dat,line=1,col=1").and(predicate::str::contains(
                "file=nested/nested.txt,line=1,col=4",
            )),
        );
    assert!(fs::read_to_string(&fixture.output)
        .unwrap()
        .contains("violations=2\nchecked-files=9\nskipped-files=2"));
}

#[test]
fn workspace_escape_and_invalid_inputs_are_configuration_errors() {
    let fixture = ActionFixture::new();
    fixture
        .command()
        .env("INPUT_PATHS", "../outside")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("config.invalid"));
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

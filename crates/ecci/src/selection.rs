use crate::binary::{classify_path, Classification};
use ecci_editorconfig::Config;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::WalkBuilder;
use same_file::Handle;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SkipReason {
    Gitignore,
    Ecciignore,
    NoEditorConfig,
    Binary,
    Symlink,
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorReason {
    Configuration,
    Nonexistent,
    Unsupported,
    BrokenSymlink,
    Filesystem,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    Skip {
        path: PathBuf,
        reason: SkipReason,
    },
    DirectFileIgnoreOverride {
        path: PathBuf,
    },
    Error {
        path: PathBuf,
        reason: ErrorReason,
        operation: &'static str,
        detail: String,
    },
}

#[derive(Debug)]
pub struct SelectedFile {
    pub path: PathBuf,
    pub config: Config,
    pub direct: bool,
    pub force_check: bool,
}

#[derive(Debug, Default)]
pub struct Selection {
    pub files: Vec<SelectedFile>,
    pub outcomes: Vec<Outcome>,
}

pub fn select_paths<I, P>(paths: I) -> Selection
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut selection = Selection::default();
    let mut identities = HashMap::<Handle, usize>::new();

    for path in paths {
        let path = path.as_ref().to_path_buf();
        match fs::symlink_metadata(&path) {
            Err(error) => selection.outcomes.push(error_outcome(
                path,
                if error.kind() == std::io::ErrorKind::NotFound {
                    ErrorReason::Nonexistent
                } else {
                    ErrorReason::Filesystem
                },
                "inspect direct path",
                error,
            )),
            Ok(metadata) if metadata.file_type().is_symlink() => match fs::metadata(&path) {
                Ok(target) if target.is_file() => {
                    process_file(&path, true, false, None, &mut selection, &mut identities)
                }
                Ok(_) => selection.outcomes.push(Outcome::Error {
                    path,
                    reason: ErrorReason::Unsupported,
                    operation: "resolve direct symbolic link",
                    detail: "symbolic link does not resolve to a regular file".into(),
                }),
                Err(error) => selection.outcomes.push(error_outcome(
                    path,
                    ErrorReason::BrokenSymlink,
                    "resolve direct symbolic link",
                    error,
                )),
            },
            Ok(metadata) if metadata.is_file() => {
                process_file(&path, true, false, None, &mut selection, &mut identities)
            }
            Ok(metadata) if metadata.is_dir() => {
                walk_directory(&path, &mut selection, &mut identities)
            }
            Ok(_) => selection.outcomes.push(Outcome::Error {
                path,
                reason: ErrorReason::Unsupported,
                operation: "inspect direct path",
                detail: "path is neither a regular file nor a directory".into(),
            }),
        }
    }
    selection
}

fn walk_directory(root: &Path, selection: &mut Selection, identities: &mut HashMap<Handle, usize>) {
    if has_git_directory_component(root) {
        return;
    }

    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .follow_links(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .ignore(false);
    let filter_root = root.to_path_buf();
    builder.filter_entry(move |entry| {
        entry.path() == filter_root
            || !entry
                .file_type()
                .is_some_and(|file_type| file_type.is_dir())
            || (entry.file_name() != ".git"
                && ignore_decision(&filter_root, entry.path(), true)
                    .map(|decision| decision.excluded.is_none())
                    .unwrap_or(true))
    });

    for entry in builder.build() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                selection.outcomes.push(Outcome::Error {
                    path: root.to_path_buf(),
                    reason: ErrorReason::Filesystem,
                    operation: "traverse directory",
                    detail: error.to_string(),
                });
                continue;
            }
        };
        if entry.path() == root {
            continue;
        }
        let Some(file_type) = entry.file_type() else {
            selection.outcomes.push(Outcome::Error {
                path: entry.path().to_path_buf(),
                reason: ErrorReason::Filesystem,
                operation: "inspect traversal entry",
                detail: "file type is unavailable".into(),
            });
            continue;
        };
        if file_type.is_symlink() {
            selection.outcomes.push(Outcome::Skip {
                path: entry.path().to_path_buf(),
                reason: SkipReason::Symlink,
            });
        } else if file_type.is_file() {
            if entry.file_name() == ".ecciignore" {
                continue;
            }
            match ignore_decision(root, entry.path(), false) {
                Ok(decision) => process_file(
                    entry.path(),
                    false,
                    decision.force_check,
                    decision.excluded,
                    selection,
                    identities,
                ),
                Err((path, operation, detail)) => selection.outcomes.push(Outcome::Error {
                    path,
                    reason: ErrorReason::Filesystem,
                    operation,
                    detail,
                }),
            }
        }
    }
}

fn has_git_directory_component(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == ".git")
}

#[derive(Default)]
struct IgnoreDecision {
    excluded: Option<SkipReason>,
    force_check: bool,
}

fn ignore_decision(
    root: &Path,
    path: &Path,
    is_dir: bool,
) -> Result<IgnoreDecision, (PathBuf, &'static str, String)> {
    let git = build_ignore(root, path, ".gitignore")?;
    let ecci = build_ignore(root, path, ".ecciignore")?;
    let ecci_match = ecci.matched(path, is_dir);
    if ecci_match.is_whitelist() {
        return Ok(IgnoreDecision {
            force_check: true,
            excluded: None,
        });
    }
    if ecci_match.is_ignore() {
        return Ok(IgnoreDecision {
            force_check: false,
            excluded: Some(SkipReason::Ecciignore),
        });
    }
    Ok(IgnoreDecision {
        force_check: false,
        excluded: git
            .matched(path, is_dir)
            .is_ignore()
            .then_some(SkipReason::Gitignore),
    })
}

fn build_ignore(
    root: &Path,
    path: &Path,
    filename: &str,
) -> Result<Gitignore, (PathBuf, &'static str, String)> {
    let mut builder = GitignoreBuilder::new(root);
    let parent = path.parent().unwrap_or(root);
    let relative = parent.strip_prefix(root).unwrap_or(Path::new(""));
    let mut directory = root.to_path_buf();
    let mut candidates = vec![directory.join(filename)];
    for component in relative.components() {
        directory.push(component);
        candidates.push(directory.join(filename));
    }
    for candidate in candidates {
        if candidate.is_file() {
            if let Some(error) = builder.add(&candidate) {
                return Err((candidate, "read ignore file", error.to_string()));
            }
        }
    }
    builder.build().map_err(|error| {
        (
            root.to_path_buf(),
            "compile ignore rules",
            error.to_string(),
        )
    })
}

fn process_file(
    path: &Path,
    direct: bool,
    force_check: bool,
    excluded: Option<SkipReason>,
    selection: &mut Selection,
    identities: &mut HashMap<Handle, usize>,
) {
    if direct {
        if direct_path_is_ignored(path) {
            selection.outcomes.push(Outcome::DirectFileIgnoreOverride {
                path: path.to_path_buf(),
            });
        }
    } else if let Some(reason) = excluded {
        selection.outcomes.push(Outcome::Skip {
            path: path.to_path_buf(),
            reason,
        });
        return;
    }

    let config = match Config::from_path(path) {
        Ok(config) => config,
        Err(error) => {
            selection.outcomes.push(error_outcome(
                path.to_path_buf(),
                if error.kind() == std::io::ErrorKind::InvalidData {
                    ErrorReason::Configuration
                } else {
                    ErrorReason::Filesystem
                },
                "resolve .editorconfig",
                error,
            ));
            return;
        }
    };
    if !has_applicable_config(&config) {
        selection.outcomes.push(Outcome::Skip {
            path: path.to_path_buf(),
            reason: SkipReason::NoEditorConfig,
        });
        return;
    }
    match classify_path(path, config.charset.as_ref()) {
        Ok(Classification::Binary) if !force_check => {
            selection.outcomes.push(Outcome::Skip {
                path: path.to_path_buf(),
                reason: SkipReason::Binary,
            });
            return;
        }
        Err(error) => {
            selection.outcomes.push(error_outcome(
                path.to_path_buf(),
                ErrorReason::Filesystem,
                "read candidate file",
                error,
            ));
            return;
        }
        _ => {}
    }
    let identity = match Handle::from_path(path) {
        Ok(identity) => identity,
        Err(error) => {
            selection.outcomes.push(error_outcome(
                path.to_path_buf(),
                ErrorReason::Filesystem,
                "identify candidate file",
                error,
            ));
            return;
        }
    };
    if let Some(index) = identities.get(&identity).copied() {
        selection.files[index].direct |= direct;
        selection.files[index].force_check |= force_check;
        selection.outcomes.push(Outcome::Skip {
            path: path.to_path_buf(),
            reason: SkipReason::Duplicate,
        });
        return;
    }
    identities.insert(identity, selection.files.len());
    selection.files.push(SelectedFile {
        path: path.to_path_buf(),
        config,
        direct,
        force_check,
    });
}

fn direct_path_is_ignored(path: &Path) -> bool {
    let Ok(cwd) = std::env::current_dir() else {
        return false;
    };
    let Ok(canonical_cwd) = cwd.canonicalize() else {
        return false;
    };
    let Ok(canonical_path) = path.canonicalize() else {
        return false;
    };
    canonical_path.starts_with(&canonical_cwd)
        && ignore_decision(&canonical_cwd, &canonical_path, false)
            .map(|decision| decision.excluded.is_some())
            .unwrap_or(false)
}

fn has_applicable_config(config: &Config) -> bool {
    config.indent_style.is_some()
        || config.indent_size.is_some()
        || config.indent_size_is_tab
        || config.tab_width.is_some()
        || config.end_of_line.is_some()
        || config.charset.is_some()
        || config.trim_trailing_whitespace.is_some()
        || config.insert_final_newline.is_some()
        || config.max_line_length.is_some()
}

fn error_outcome(
    path: PathBuf,
    reason: ErrorReason,
    operation: &'static str,
    error: std::io::Error,
) -> Outcome {
    Outcome::Error {
        path,
        reason,
        operation,
        detail: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(path: &Path, contents: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn configured_tree() -> TempDir {
        let temp = tempfile::tempdir().unwrap();
        write(
            &temp.path().join(".editorconfig"),
            b"root = true\n\n[*]\nindent_style = space\n",
        );
        temp
    }

    fn selected_names(selection: &Selection) -> Vec<String> {
        let mut names: Vec<_> = selection
            .files
            .iter()
            .map(|file| {
                file.path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }

    fn copy_tree(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let destination = destination.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &destination);
            } else {
                fs::copy(entry.path(), destination).unwrap();
            }
        }
    }

    fn assert_discovery_fixture(ignore_filename: &str, fixture_filename: &str) {
        let temp = configured_tree();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../testdata/file-discovery")
            .join(ignore_filename.trim_matches('.'));
        copy_tree(&fixture, temp.path());
        fs::rename(
            temp.path().join(fixture_filename),
            temp.path().join(ignore_filename),
        )
        .unwrap();

        let selection = select_paths([temp.path()]);
        let mut relative: Vec<_> = selection
            .files
            .iter()
            .filter_map(|file| file.path.strip_prefix(temp.path()).ok())
            .map(|path| path.to_string_lossy().into_owned())
            .collect();
        relative.sort();
        assert!(relative.contains(&"visible.txt".into()));
        assert!(relative.contains(&"ignored/reincluded.txt".into()));
        assert!(!relative.contains(&"ignored/not-visited.txt".into()));
        assert!(!relative.iter().any(|path| path.starts_with("target/")));
    }

    #[test]
    fn ignored_directories_are_pruned_for_both_ignore_file_types() {
        assert_discovery_fixture(".gitignore", "gitignore.fixture");
        assert_discovery_fixture(".ecciignore", "ecciignore.fixture");
    }

    fn assert_git_directory_fixture(fixture: &str) {
        let temp = configured_tree();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../testdata/file-discovery")
            .join(fixture);
        copy_tree(&fixture, temp.path());
        fs::rename(
            temp.path().join("git-dir.fixture"),
            temp.path().join(".git"),
        )
        .unwrap();
        fs::rename(
            temp.path().join("nested/git-dir.fixture"),
            temp.path().join("nested/.git"),
        )
        .unwrap();
        fs::rename(
            temp.path().join("ordinary/git-file.fixture"),
            temp.path().join("ordinary/.git"),
        )
        .unwrap();

        let selection = select_paths([temp.path()]);
        let relative: Vec<_> = selection
            .files
            .iter()
            .filter_map(|file| file.path.strip_prefix(temp.path()).ok())
            .map(|path| path.to_string_lossy().into_owned())
            .collect();

        assert!(relative.contains(&"visible.txt".into()));
        assert!(relative.contains(&"ordinary/.git".into()));
        assert!(relative.contains(&".github/workflow.txt".into()));
        assert!(relative.contains(&"nested/visible.txt".into()));
        assert!(!relative.iter().any(|path| path.starts_with(".git/")));
        assert!(!relative.iter().any(|path| path.starts_with("nested/.git/")));
    }

    #[test]
    fn git_directories_are_pruned_without_an_ignore_file() {
        assert_git_directory_fixture("git-directory-no-ignore");
    }

    #[test]
    fn git_directories_cannot_be_reincluded_by_ignore_patterns() {
        assert_git_directory_fixture("git-directory-reincluded");
    }

    #[test]
    fn explicitly_named_paths_do_not_bypass_git_directory_pruning() {
        let temp = configured_tree();
        let git_file = temp.path().join(".git");
        write(&git_file, b"ordinary file\n");
        let direct_file = select_paths([&git_file]);
        assert_eq!(direct_file.files.len(), 1);

        fs::remove_file(&git_file).unwrap();
        let nested = temp.path().join(".git/objects");
        write(&nested.join("object.txt"), b"not inspected\n");
        assert!(select_paths([temp.path().join(".git")]).files.is_empty());
        assert!(select_paths([nested]).files.is_empty());
    }

    fn has_skip(selection: &Selection, name: &str, reason: SkipReason) -> bool {
        selection.outcomes.iter().any(|outcome| {
            matches!(
                outcome,
                Outcome::Skip { path, reason: actual }
                    if path.file_name().is_some_and(|value| value == name) && *actual == reason
            )
        })
    }

    #[test]
    fn hierarchical_ignores_dotfiles_and_force_check_follow_the_contract() {
        let temp = configured_tree();
        write(
            &temp.path().join(".gitignore"),
            b"*.tmp\ngit-only.txt\nforced.bin\n",
        );
        write(
            &temp.path().join(".ecciignore"),
            b"ecci-only.txt\n*.special\n!git-only.txt\n!forced.bin\n",
        );
        write(&temp.path().join("nested/.gitignore"), b"!keep.tmp\n");
        write(&temp.path().join("nested/.ecciignore"), b"!keep.special\n");
        write(&temp.path().join(".git/info/exclude"), b"local.txt\n");
        for name in [
            "plain.txt",
            ".hidden",
            "drop.tmp",
            "git-only.txt",
            "ecci-only.txt",
            "drop.special",
            "local.txt",
            "nested/keep.tmp",
            "nested/keep.special",
        ] {
            write(&temp.path().join(name), b"text\n");
        }
        write(&temp.path().join("forced.bin"), b"binary\0data");

        let selection = select_paths([temp.path()]);
        assert_eq!(
            selected_names(&selection),
            vec![
                ".editorconfig",
                ".gitignore",
                ".gitignore",
                ".hidden",
                "forced.bin",
                "git-only.txt",
                "keep.special",
                "keep.tmp",
                "local.txt",
                "plain.txt"
            ]
        );
        assert!(
            selection
                .files
                .iter()
                .find(|file| file.path.ends_with("forced.bin"))
                .unwrap()
                .force_check
        );
        assert!(has_skip(&selection, "drop.tmp", SkipReason::Gitignore));
        assert!(has_skip(
            &selection,
            "ecci-only.txt",
            SkipReason::Ecciignore
        ));
        assert!(has_skip(&selection, "drop.special", SkipReason::Ecciignore));
    }

    #[test]
    fn direct_file_overrides_ignore_while_direct_directory_does_not() {
        let temp = configured_tree();
        write(&temp.path().join(".gitignore"), b"ignored.txt\n");
        let ignored = temp.path().join("ignored.txt");
        write(&ignored, b"text\n");

        let direct = select_paths([&ignored]);
        assert_eq!(direct.files.len(), 1);
        // The override notice depends on the file being below the invocation cwd;
        // selection itself is unconditional for every direct regular file.
        assert!(direct.files[0].direct);

        let traversal = select_paths([temp.path()]);
        assert!(has_skip(&traversal, "ignored.txt", SkipReason::Gitignore));
    }

    #[test]
    fn no_configuration_binary_and_same_file_are_structured_skips() {
        let temp = configured_tree();
        let text = temp.path().join("text.txt");
        let alias = temp.path().join("alias.txt");
        let binary = temp.path().join("image.txt");
        write(&text, b"text\n");
        fs::hard_link(&text, &alias).unwrap();
        write(&binary, b"a\0b");

        let selection = select_paths([&text, &alias, &binary]);
        assert_eq!(selection.files.len(), 1);
        assert!(has_skip(&selection, "alias.txt", SkipReason::Duplicate));
        assert!(has_skip(&selection, "image.txt", SkipReason::Binary));

        let unconfigured = tempfile::tempdir().unwrap();
        let file = unconfigured.path().join("file.txt");
        write(&file, b"text\n");
        let selection = select_paths([&file]);
        assert!(has_skip(&selection, "file.txt", SkipReason::NoEditorConfig));
    }

    #[test]
    fn direct_path_errors_continue_to_later_arguments() {
        let temp = configured_tree();
        let missing = temp.path().join("missing");
        let valid = temp.path().join("valid.txt");
        write(&valid, b"text\n");
        let selection = select_paths([&missing, &valid]);
        assert_eq!(selection.files.len(), 1);
        assert!(selection.outcomes.iter().any(|outcome| matches!(
            outcome,
            Outcome::Error {
                reason: ErrorReason::Nonexistent,
                operation: "inspect direct path",
                ..
            }
        )));
    }

    #[cfg(unix)]
    #[test]
    fn traversal_skips_symlinks_but_a_direct_file_symlink_is_selected() {
        use std::os::unix::fs::symlink;
        let temp = configured_tree();
        let target = temp.path().join("target.txt");
        let file_link = temp.path().join("file-link");
        let dir = temp.path().join("dir");
        let dir_link = temp.path().join("dir-link");
        let broken = temp.path().join("broken");
        write(&target, b"text\n");
        fs::create_dir(&dir).unwrap();
        symlink(&target, &file_link).unwrap();
        symlink(&dir, &dir_link).unwrap();
        symlink(temp.path().join("absent"), &broken).unwrap();

        let walked = select_paths([temp.path()]);
        assert!(has_skip(&walked, "file-link", SkipReason::Symlink));
        assert!(has_skip(&walked, "dir-link", SkipReason::Symlink));
        let direct = select_paths([&file_link, &broken]);
        assert_eq!(direct.files.len(), 1);
        assert!(direct.outcomes.iter().any(|outcome| matches!(
            outcome,
            Outcome::Error {
                reason: ErrorReason::BrokenSymlink,
                ..
            }
        )));
    }
}

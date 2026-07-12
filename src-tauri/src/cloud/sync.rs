// AI-generated (Claude)
use std::path::Path;
use git2::{IndexAddOption, Repository, Signature, Status, StatusOptions, Statuses};

const COMMIT_AUTHOR_NAME: &str = "Subsurface";
const COMMIT_AUTHOR_EMAIL: &str = "subsurface-app-account@subsurface-divelog.org";

fn sig() -> Result<Signature<'static>, String> {
    Signature::now(COMMIT_AUTHOR_NAME, COMMIT_AUTHOR_EMAIL).map_err(|e| e.to_string())
}

fn build_commit_message(statuses: &Statuses) -> String {
    let mut entries: Vec<(String, &'static str)> = statuses
        .iter()
        .map(|entry| {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();
            let letter = if status.intersects(Status::WT_DELETED | Status::INDEX_DELETED) {
                "D"
            } else if status.intersects(Status::WT_NEW | Status::INDEX_NEW) {
                "A"
            } else {
                "M"
            };
            (path, letter)
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut message = format!(
        "Save: {} file{} changed\n\n",
        entries.len(),
        if entries.len() == 1 { "" } else { "s" }
    );
    for (path, letter) in &entries {
        message.push_str(&format!("{letter} {path}\n"));
    }
    message.truncate(message.trim_end_matches('\n').len());
    message
}

/// Stages and commits any dirty working-tree state as a single commit. No-op if clean.
pub(super) fn commit_local_changes(repo: &Repository) -> Result<(), String> {
    // `StatusOptions::new()` zero-initializes flags (unlike passing `None`, which makes
    // libgit2 apply `GIT_STATUS_OPT_DEFAULTS`), so the non-ignored defaults must be set
    // explicitly here alongside disabling `include_ignored`.
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false);
    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.to_string())?;
    if statuses.is_empty() {
        return Ok(());
    }
    let message = build_commit_message(&statuses);

    let mut index = repo.index().map_err(|e| e.to_string())?;
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())?;
    let tree_oid = index.write_tree().map_err(|e| e.to_string())?;
    let tree = repo.find_tree(tree_oid).map_err(|e| e.to_string())?;

    let signature = sig()?;
    let parent = repo
        .head()
        .and_then(|h| h.peel_to_commit())
        .map_err(|e| e.to_string())?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &[&parent],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Ensures `<cache_dir>/.git/info/exclude` filters junk files (`.DS_Store`, swap files, …)
/// out of `git add -A`. Local to this clone only, never part of the tracked working tree —
/// regenerated on every clone/fetch, so it needs no propagation.
pub(super) fn ensure_git_exclude(cache_dir: &Path) -> Result<(), String> {
    let exclude_path = cache_dir.join(".git").join("info").join("exclude");
    if let Some(parent) = exclude_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&exclude_path, ".DS_Store\nThumbs.db\n*.swp\n*~\n").map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature};
    use git2::build::RepoBuilder;

    const BRANCH: &str = "test-branch";

    fn init_bare(bare_dir: &Path, file_name: &str, content: &str) -> Repository {
        std::fs::remove_dir_all(bare_dir).ok();
        let bare = Repository::init_bare(bare_dir).unwrap();
        let blob_oid = bare.blob(content.as_bytes()).unwrap();
        let tree_oid = {
            let mut tb = bare.treebuilder(None).unwrap();
            tb.insert(file_name, blob_oid, 0o100644).unwrap();
            tb.write().unwrap()
        };
        {
            let tree = bare.find_tree(tree_oid).unwrap();
            let signature = Signature::now("Remote", "remote@example.com").unwrap();
            bare.commit(
                Some(&format!("refs/heads/{BRANCH}")),
                &signature,
                &signature,
                "initial",
                &tree,
                &[],
            )
            .unwrap();
        }
        bare
    }

    fn clone_work(bare_dir: &Path, work_dir: &Path) -> Repository {
        std::fs::remove_dir_all(work_dir).ok();
        let mut builder = RepoBuilder::new();
        builder.branch(BRANCH);
        builder.clone(bare_dir.to_str().unwrap(), work_dir).unwrap()
    }

    fn fixture_dirs(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let base = std::env::temp_dir().join("cloud_sync_test").join(name);
        (base.join("bare"), base.join("work"))
    }

    fn head_oid(repo: &Repository) -> git2::Oid {
        repo.head().unwrap().target().unwrap()
    }

    #[test]
    fn ensure_git_exclude_writes_expected_content() {
        let (bare_dir, work_dir) = fixture_dirs("exclude_file");
        init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        clone_work(&bare_dir, &work_dir);

        ensure_git_exclude(&work_dir).unwrap();
        let contents = std::fs::read_to_string(work_dir.join(".git/info/exclude")).unwrap();
        assert_eq!(contents, ".DS_Store\nThumbs.db\n*.swp\n*~\n");
    }

    #[test]
    fn commit_local_changes_is_noop_when_clean() {
        let (bare_dir, work_dir) = fixture_dirs("commit_noop");
        init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        let repo = clone_work(&bare_dir, &work_dir);
        let before = head_oid(&repo);

        commit_local_changes(&repo).unwrap();
        assert_eq!(head_oid(&repo), before, "no commit created when working tree is clean");
    }

    #[test]
    fn commit_message_format_matches_exactly() {
        let (bare_dir, work_dir) = fixture_dirs("message_format");
        init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        let repo = clone_work(&bare_dir, &work_dir);

        std::fs::write(work_dir.join("00-Subsurface"), "version 4\n").unwrap();
        std::fs::write(work_dir.join("a-new-dive.txt"), "new\n").unwrap();
        commit_local_changes(&repo).unwrap();

        let message = repo.head().unwrap().peel_to_commit().unwrap().message().unwrap().to_string();
        assert_eq!(
            message,
            "Save: 2 files changed\n\nM 00-Subsurface\nA a-new-dive.txt"
        );
    }

    #[test]
    fn commit_local_changes_ignores_excluded_files() {
        let (bare_dir, work_dir) = fixture_dirs("commit_ignored");
        init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        let repo = clone_work(&bare_dir, &work_dir);
        ensure_git_exclude(&work_dir).unwrap();
        let before = head_oid(&repo);

        std::fs::write(work_dir.join(".DS_Store"), "junk").unwrap();
        commit_local_changes(&repo).unwrap();

        assert_eq!(
            head_oid(&repo),
            before,
            "no commit created when only ignored files are present"
        );
    }
}

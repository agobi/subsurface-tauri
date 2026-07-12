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

/// Drives a started rebase to completion. Callers must abort the rebase (via
/// `rebase.abort()`) whenever this returns `Err` — see the caller-side comment in
/// `rebase_onto_remote` for why that can't be done in here.
fn run_rebase_ops(
    repo: &Repository,
    rebase: &mut git2::Rebase<'_>,
    signature: &Signature<'_>,
) -> Result<(), String> {
    while let Some(op) = rebase.next() {
        op.map_err(|e| e.to_string())?;
        let has_conflicts = repo.index().map_err(|e| e.to_string())?.has_conflicts();
        if has_conflicts {
            return Err(
                "Sync failed: local changes conflict with the cloud copy. Your changes are \
                 saved locally — try Sync again after the conflict is resolved."
                    .to_string(),
            );
        }
        rebase.commit(None, signature, None).map_err(|e| e.to_string())?;
    }
    rebase.finish(Some(signature)).map_err(|e| e.to_string())?;
    Ok(())
}

/// Rebases local HEAD onto `refs/remotes/origin/<branch>`. No-op if already up to date.
/// On any failure once the rebase has started — a real conflict, a checkout error, a failed
/// commit, etc. — aborts the rebase (leaving the local commit from `commit_local_changes`
/// untouched and `.git/rebase-merge` cleaned up) and returns a user-facing error.
///
/// All rebase-stepping logic lives in `run_rebase_ops` specifically so every one of its early
/// returns funnels through this single `rebase.abort()` call. `Rebase`'s `Drop` impl only calls
/// `git_rebase_free` (frees memory) — it does NOT call `git_rebase_abort` — so an error that
/// skipped this would strand the clone mid-rebase on disk with no user-facing recovery path.
pub(super) fn rebase_onto_remote(repo: &Repository, branch: &str) -> Result<(), String> {
    let head_ref = repo.head().map_err(|e| e.to_string())?;
    let head_ac = repo
        .reference_to_annotated_commit(&head_ref)
        .map_err(|e| e.to_string())?;
    let upstream_ref = repo
        .find_reference(&format!("refs/remotes/origin/{branch}"))
        .map_err(|e| e.to_string())?;
    let upstream_ac = repo
        .reference_to_annotated_commit(&upstream_ref)
        .map_err(|e| e.to_string())?;
    // Resolved before the rebase is started so its (practically infallible) error path never
    // needs to worry about aborting a rebase that doesn't exist yet.
    let signature = sig()?;

    let mut rebase = repo
        .rebase(Some(&head_ac), Some(&upstream_ac), None, None)
        .map_err(|e| e.to_string())?;

    let result = run_rebase_ops(repo, &mut rebase, &signature);
    if result.is_err() {
        rebase.abort().ok();
    }
    result
}

/// Fast-forward pushes local `branch` to `remote_name`. Single credential attempt, matching
/// `make_fetch_opts` — a rejected credential or a remote that moved again since our fetch
/// (a push race) both surface as a plain error; the local commit is preserved for retry.
pub(super) fn push_to_remote(
    repo: &Repository,
    remote_name: &str,
    branch: &str,
    email: &str,
    password: &str,
) -> Result<(), String> {
    let mut remote = repo.find_remote(remote_name).map_err(|e| e.to_string())?;
    let mut opts = git2::PushOptions::new();
    opts.remote_callbacks(super::credential_callbacks(email, password));
    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
    remote
        .push(&[refspec.as_str()], Some(&mut opts))
        .map_err(super::map_git_error)?;
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

    /// Adds a commit directly to the bare repo's history (simulating another device's push),
    /// bypassing the local working clone entirely.
    fn advance_bare(bare: &Repository, file_name: &str, content: &str) {
        let head_oid = bare
            .find_reference(&format!("refs/heads/{BRANCH}"))
            .unwrap()
            .target()
            .unwrap();
        let parent = bare.find_commit(head_oid).unwrap();
        let parent_tree = parent.tree().unwrap();
        let blob_oid = bare.blob(content.as_bytes()).unwrap();
        let mut tb = bare.treebuilder(Some(&parent_tree)).unwrap();
        tb.insert(file_name, blob_oid, 0o100644).unwrap();
        let tree_oid = tb.write().unwrap();
        let tree = bare.find_tree(tree_oid).unwrap();
        let signature = Signature::now("Remote", "remote@example.com").unwrap();
        bare.commit(
            Some(&format!("refs/heads/{BRANCH}")),
            &signature,
            &signature,
            "remote advance",
            &tree,
            &[&parent],
        )
        .unwrap();
    }

    fn fetch(repo: &Repository) {
        let mut remote = repo.find_remote("origin").unwrap();
        remote.fetch(&[] as &[&str], None, None).unwrap();
    }

    fn remote_branch_oid(bare: &Repository) -> git2::Oid {
        bare.find_reference(&format!("refs/heads/{BRANCH}"))
            .unwrap()
            .target()
            .unwrap()
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

    #[test]
    fn remote_advanced_no_local_changes_fast_forwards() {
        let (bare_dir, work_dir) = fixture_dirs("remote_advanced");
        let bare = init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        let repo = clone_work(&bare_dir, &work_dir);

        advance_bare(&bare, "other-file.txt", "from another device\n");

        commit_local_changes(&repo).unwrap();
        fetch(&repo);
        rebase_onto_remote(&repo, BRANCH).unwrap();
        assert_eq!(
            head_oid(&repo),
            remote_branch_oid(&bare),
            "fast-forwards to remote tip"
        );
    }

    #[test]
    fn diverged_no_conflict_rebases_cleanly() {
        let (bare_dir, work_dir) = fixture_dirs("diverged_ok");
        let bare = init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        let repo = clone_work(&bare_dir, &work_dir);

        advance_bare(&bare, "remote-file.txt", "from another device\n");
        std::fs::write(work_dir.join("local-file.txt"), "from this device\n").unwrap();
        commit_local_changes(&repo).unwrap();

        fetch(&repo);
        rebase_onto_remote(&repo, BRANCH).unwrap();
        assert_eq!(
            repo.state(),
            git2::RepositoryState::Clean,
            "rebase leaves a clean repo state"
        );
        assert!(work_dir.join("remote-file.txt").exists());
        assert!(work_dir.join("local-file.txt").exists());
    }

    #[test]
    fn diverged_real_conflict_aborts_and_preserves_local_commit() {
        let (bare_dir, work_dir) = fixture_dirs("diverged_conflict");
        let bare = init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        let repo = clone_work(&bare_dir, &work_dir);

        advance_bare(&bare, "00-Subsurface", "version 4\nremote change\n");
        std::fs::write(work_dir.join("00-Subsurface"), "version 4\nlocal change\n").unwrap();
        commit_local_changes(&repo).unwrap();
        let local_commit = head_oid(&repo);

        fetch(&repo);
        let result = rebase_onto_remote(&repo, BRANCH);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("conflict with the cloud copy"));
        assert_eq!(
            repo.state(),
            git2::RepositoryState::Clean,
            "abort leaves no in-progress rebase state"
        );
        assert_eq!(head_oid(&repo), local_commit, "local commit is untouched");
    }

    #[test]
    fn clean_sync_no_local_changes_remote_unchanged_is_noop() {
        let (bare_dir, work_dir) = fixture_dirs("clean_sync");
        let bare = init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        let repo = clone_work(&bare_dir, &work_dir);
        let before = head_oid(&repo);

        commit_local_changes(&repo).unwrap();
        fetch(&repo);
        rebase_onto_remote(&repo, BRANCH).unwrap();
        push_to_remote(&repo, "origin", BRANCH, "", "").unwrap();

        assert_eq!(head_oid(&repo), before);
        assert_eq!(remote_branch_oid(&bare), before);
    }

    #[test]
    fn local_changes_only_are_committed_and_pushed() {
        let (bare_dir, work_dir) = fixture_dirs("local_only");
        let bare = init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        let repo = clone_work(&bare_dir, &work_dir);
        let before = head_oid(&repo);

        std::fs::write(work_dir.join("new-file.txt"), "hello\n").unwrap();
        commit_local_changes(&repo).unwrap();
        let after_commit = head_oid(&repo);
        assert_ne!(after_commit, before, "commit created for local change");

        fetch(&repo);
        rebase_onto_remote(&repo, BRANCH).unwrap();
        push_to_remote(&repo, "origin", BRANCH, "", "").unwrap();

        assert_eq!(remote_branch_oid(&bare), after_commit);
    }

    #[test]
    fn diverged_no_conflict_rebases_and_pushes() {
        let (bare_dir, work_dir) = fixture_dirs("diverged_push_ok");
        let bare = init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        let repo = clone_work(&bare_dir, &work_dir);

        advance_bare(&bare, "remote-file.txt", "from another device\n");
        std::fs::write(work_dir.join("local-file.txt"), "from this device\n").unwrap();
        commit_local_changes(&repo).unwrap();

        fetch(&repo);
        rebase_onto_remote(&repo, BRANCH).unwrap();
        push_to_remote(&repo, "origin", BRANCH, "", "").unwrap();

        assert_eq!(remote_branch_oid(&bare), head_oid(&repo));
    }
}

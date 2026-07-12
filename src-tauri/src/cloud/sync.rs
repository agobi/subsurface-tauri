// AI-generated (Claude)
use std::path::Path;

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

    #[test]
    fn ensure_git_exclude_writes_expected_content() {
        let (bare_dir, work_dir) = fixture_dirs("exclude_file");
        init_bare(&bare_dir, "00-Subsurface", "version 3\n");
        clone_work(&bare_dir, &work_dir);

        ensure_git_exclude(&work_dir).unwrap();
        let contents = std::fs::read_to_string(work_dir.join(".git/info/exclude")).unwrap();
        assert_eq!(contents, ".DS_Store\nThumbs.db\n*.swp\n*~\n");
    }
}

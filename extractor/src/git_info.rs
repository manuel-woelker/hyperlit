use chrono::DateTime;
use git2::Oid;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::last_modification_info::LastModificationInfo;
use std::path::{Path, absolute};

pub struct GitInfo {
    repository: git2::Repository,
}

impl GitInfo {
    pub fn new() -> HyperlitResult<GitInfo> {
        let repository = git2::Repository::discover(".")?;
        Ok(GitInfo { repository })
    }
    pub fn get_last_modification_info(&self, path: &Path) -> HyperlitResult<LastModificationInfo> {
        let repository = &self.repository;
        let absolute_path = absolute(path)?;
        let repository_path = absolute(repository.path())?;
        let base_path = repository_path.parent().unwrap();
        let actual_path = absolute_path.strip_prefix(base_path).expect("Not a prefix");
        let mut revwalk = repository.revwalk()?;
        revwalk.push_head()?;
        let reference = repository.head()?;
        let head_commit = repository.find_commit(reference.target().unwrap())?;
        let mut current_file_id = Oid::zero();
        if let Ok(entry) = head_commit.tree()?.get_path(actual_path) {
            current_file_id = entry.id();
        }
        let mut last_commit_oid = head_commit.id();
        for commit_oid in revwalk {
            let commit_oid = commit_oid?;
            let commit = repository.find_commit(commit_oid)?;
            let id = commit.tree()?.get_path(actual_path).map(|e| e.id());
            if Ok(current_file_id) != id {
                break;
            }
            last_commit_oid = commit_oid;
        }
        let commit = repository.find_commit(last_commit_oid)?;
        let author_date = commit.author().when();
        Ok(LastModificationInfo {
            date: DateTime::from_timestamp(author_date.seconds(), 0),
            author: commit.author().name().map(|s| s.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::git_info::GitInfo;
    use chrono::DateTime;
    use hyperlit_base::result::HyperlitResult;
    use hyperlit_model::last_modification_info::LastModificationInfo;
    use std::path::Path;

    #[test]
    fn test_get_last_modification_info() -> HyperlitResult<()> {
        let info = GitInfo::new()?.get_last_modification_info(Path::new("../LICENSE"))?;
        assert_eq!(
            info,
            LastModificationInfo {
                date: DateTime::from_timestamp(1749243071, 0),
                author: Some("Manuel Woelker".to_string())
            }
        );
        Ok(())
    }
}

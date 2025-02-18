use anyhow::Result;
use git2::{Repository, DiffOptions, BranchType};

pub struct GitDiff {
    repo: Repository,
}

impl GitDiff {
    pub fn commit(&self, message: &str) -> Result<()> {
        let signature = self.repo.signature()?;
        let tree_id = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let parent_commit = self.repo.head()?.peel_to_commit()?;
        
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit]
        )?;
        
        Ok(())
    }


    pub fn new(path: &str) -> Result<Self> {
        let repo = Repository::discover(path)?;
        Ok(Self { repo })
    }

    pub fn get_staged_diff(&self) -> Result<String> {
        let mut diff_opts = DiffOptions::new();
        let head_tree = self.repo.head()?.peel_to_tree()?;
        let diff = self.repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?;
        self.format_diff(&diff)
    }

    fn format_diff(&self, diff: &git2::Diff) -> Result<String> {
        let mut diff_content = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            use std::str;
            if let Ok(content) = str::from_utf8(line.content()) {
                let prefix = match line.origin() {
                    '+' => "+",
                    '-' => "-",
                    _ => "",
                };
                diff_content.push_str(prefix);
                diff_content.push_str(content);
            }
            true
        })?;
        Ok(diff_content)
    }

    pub fn get_summary_diff(&self, base: &str) -> Result<String> {
        let mut diff_opts = DiffOptions::new();
        
        let base_commit = if self.repo.revparse(base)?.mode().contains(git2::RevparseMode::SINGLE) {
            // Base is a commit or tag
            self.repo.revparse_single(base)?.peel_to_commit()?
        } else {
            // Try to find branch
            let base_branch = self.repo.find_branch(base, BranchType::Local)
                .or_else(|_| self.repo.find_branch(base, BranchType::Remote))?;
            base_branch.get().peel_to_commit()?
        };

        let head_commit = self.repo.head()?.peel_to_commit()?;
        let base_tree = base_commit.tree()?;
        let head_tree = head_commit.tree()?;

        let diff = self.repo.diff_tree_to_tree(
            Some(&base_tree),
            Some(&head_tree),
            Some(&mut diff_opts)
        )?;

        self.format_diff(&diff)
    }
}
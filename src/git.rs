use anyhow::Result;
use git2::{Repository, DiffOptions};

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
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    pub fn get_staged_diff(&self) -> Result<String> {
        let mut diff_opts = DiffOptions::new();
        let head_tree = self.repo.head()?.peel_to_tree()?;
        let diff = self.repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?;
        
        let mut diff_content = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            use std::str;
            if let Ok(content) = str::from_utf8(line.content()) {
                // 根据行的类型添加对应的前缀
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
}
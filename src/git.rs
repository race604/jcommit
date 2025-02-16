use anyhow::Result;
use git2::{Repository, DiffOptions};

pub struct GitDiff {
    repo: Repository,
}

impl GitDiff {
    pub fn new(path: &str) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    pub fn get_staged_diff(&self) -> Result<String> {
        let mut diff_opts = DiffOptions::new();
        let diff = self.repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;
        
        let mut diff_content = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            use std::str;
            if let Ok(content) = str::from_utf8(line.content()) {
                diff_content.push_str(content);
            }
            true
        })?;

        Ok(diff_content)
    }
}
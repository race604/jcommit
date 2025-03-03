use anyhow::{Result, anyhow};
use std::process::Command;

pub struct GitDiff {
    repo_path: String,
}

impl GitDiff {
    pub fn commit(&self, message: &str) -> Result<()> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .arg("commit")
            .arg("-m")
            .arg(message)
            .output()?;

        // 打印命令输出
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            return Err(anyhow!("Git commit failed"));
        }

        Ok(())
    }

    pub fn new(path: &str) -> Result<Self> {
        // 验证目录是否是一个有效的 git 仓库
        let output = Command::new("git")
            .current_dir(path)
            .arg("rev-parse")
            .arg("--git-dir")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Not a git repository"));
        }

        Ok(Self { 
            repo_path: path.to_string() 
        })
    }

    pub fn get_staged_diff(&self, all: bool) -> Result<String> {
        let mut command = Command::new("git");
        command.current_dir(&self.repo_path).arg("diff");
        
        if all {
            command.arg("HEAD");
        } else {
            command.arg("--cached");
        }

        let output = command.output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to get staged diff: {}", 
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn get_summary_diff(&self, base: &str) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .arg("diff")
            .arg(base)
            .arg("HEAD")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to get diff summary: {}", 
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn add_all(&self) -> Result<()> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .arg("add")
            .arg(".")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to add changes: {}", 
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}
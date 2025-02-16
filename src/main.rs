use anyhow::Result;
use clap::Parser;
mod git;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// 用户提供的额外提示信息
    #[arg(short, long)]
    message: Option<String>,

    /// 指定 Git 仓库路径，默认为当前目录
    #[arg(short, long, default_value = ".")]
    path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // 读取 Git 仓库的 diff 信息
    let git_diff = git::GitDiff::new(&cli.path)?;
    let diff_content = git_diff.get_staged_diff()?;
    println!("Git diff content: {}\n", diff_content);
    
    // TODO: 实现 AI 服务调用
    // TODO: 实现 commit message 生成
    
    Ok(())
}
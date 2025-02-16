use anyhow::Result;
use clap::Parser;
mod git;
mod ai;
mod config;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// 用户提供的额外提示信息
    #[arg(short, long)]
    message: Option<String>,

    /// 指定 Git 仓库路径，默认为当前目录
    #[arg(short, long, default_value = ".")]
    path: String,

    /// 是否输出 commit message 的 body 部分
    #[arg(short, long)]
    body: bool,

    /// 是否直接执行 git commit 命令
    #[arg(short = 'c', long)]
    commit: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // 读取 Git 仓库的 diff 信息
    let git_diff = git::GitDiff::new(&cli.path)?;
    let diff_content = git_diff.get_staged_diff()?;
    println!("Git diff content: {}\n", diff_content);
    
    // 读取配置
    let config = config::Config::new()?;
    
    // 调用 AI 服务生成 commit message
    let ai_service = ai::AiService::new(config.api_endpoint, config.model, config.api_key);
    let commit_message = ai_service.generate_commit_message(diff_content, cli.message, cli.body).await?;
    println!("Commit message:\n{}", commit_message);
    
    if cli.commit {
        git_diff.commit(&commit_message)?;
        println!("Successfully committed changes.");
    }
    
    Ok(())
}
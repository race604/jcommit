use anyhow::Result;
use clap::Parser;

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
    
    // TODO: 实现 Git diff 读取
    // TODO: 实现 AI 服务调用
    // TODO: 实现 commit message 生成
    
    Ok(())
}
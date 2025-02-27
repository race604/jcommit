use anyhow::Result;
use clap::Parser;
use std::io::Write;
use futures_util::StreamExt;
mod git;
mod ai;
mod config;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Additional hints provided by the user
    #[arg(short, long)]
    message: Option<String>,

    /// Specify Git repository path, defaults to current directory
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Whether to include commit message body
    #[arg(short, long)]
    body: bool,

    /// Whether to execute git commit command directly
    #[arg(short = 'c', long)]
    commit: bool,

    /// Whether to print detailed AI conversation logs
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    /// Generate a summary of changes between current HEAD and specified base commit/branch
    #[arg(short = 's', long)]
    summary: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Read Git repository diff information
    let git_diff = git::GitDiff::new(&cli.path)?;
    
    let diff_content = if let Some(base) = cli.summary {
        git_diff.get_summary_diff(&base)?
    } else {
        let diff = git_diff.get_staged_diff()?;
        if diff.trim().is_empty() {
            println!("Error: No staged changes found. Please use 'git add' command to stage your changes.");
            return Ok(());
        }
        diff
    };
    // println!("Git diff content: {}\n", diff_content);
    
    // Read configuration
    let config = config::Config::new()?;
    
    // Call AI service to generate commit message
    let ai_service = ai::AiService::new(
        config.api_endpoint,
        config.model,
        config.api_key,
        config.is_azure.unwrap_or(false),
        config.api_version,
        config.prompt
    );
    let mut commit_message = String::new();
    let mut stream = ai_service.generate_commit_message(diff_content, cli.message, cli.body, cli.debug).await?;
    println!("Commit message:\n");
    while let Some(content) = stream.next().await {
        let content = content?;
        if !content.is_empty() {
            print!("{}", content);
            std::io::stdout().flush()?;
            commit_message.push_str(&content);
        }
    }
    println!("\n");

    if cli.commit {
        git_diff.commit(&commit_message)?;
        println!("Successfully committed changes.");
    } else {
        print!("Do you want to commit these changes? [y/N] ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            git_diff.commit(&commit_message)?;
            println!("Successfully committed changes.");
        }
    }
    
    Ok(())
}
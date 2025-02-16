use anyhow::Result;
use clap::Parser;
use std::io::Write;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Read Git repository diff information
    let git_diff = git::GitDiff::new(&cli.path)?;
    let diff_content = git_diff.get_staged_diff()?;
    if diff_content.trim().is_empty() {
        println!("Error: No staged changes found. Please use 'git add' command to stage your changes.");
        return Ok(());
    }
    // println!("Git diff content: {}\n", diff_content);
    
    // Read configuration
    let config = config::Config::new()?;
    
    // Call AI service to generate commit message
    let ai_service = ai::AiService::new(config.api_endpoint, config.model, config.api_key);
    let commit_message = ai_service.generate_commit_message(diff_content, cli.message, cli.body).await?;
    println!("Commit message:\n\n{}\n", commit_message);
    
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
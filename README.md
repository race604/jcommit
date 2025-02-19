# jcommit

An AI-powered tool to generate git commit messages using OpenAI's GPT models.

## Features

- Automatically generates meaningful commit messages based on staged changes
- Supports custom hints to guide message generation
- Optional commit message body generation
- Direct commit option available
- Configurable OpenAI API endpoint and model
- Generate commit message based on changes between branches/commits

## Installation

```bash
cargo install jcommit
```

## Configuration

Before using jcommit, you need to set up your OpenAI API key. You can do this by either:

1. Setting the `OPENAI_API_KEY` environment variable
2. Creating a configuration file at `~/.jcommit.toml` with your API key

Example configuration file:

```toml
# OpenAI API key (or Azure OpenAI API key)
api_key = "your-api-key-here"

# Optional: Custom API endpoint (for Azure OpenAI, use your deployment endpoint)
# api_endpoint = "https://api.openai.com/v1"

# Optional: Custom model name
# model = "gpt-3.5-turbo"

# Optional: Enable Azure OpenAI API
# is_azure = false

# Optional: Azure OpenAI API version
# api_version = "2023-05-15"
```

For Azure OpenAI Service users:
1. Set `is_azure = true` in the configuration
2. Use your Azure OpenAI deployment endpoint as `api_endpoint`
3. Use your Azure OpenAI API key as `api_key`
4. Optionally specify the API version using `api_version`

## Usage

```bash
# Generate commit message for staged changes
jcommit

# Include additional hints
jcommit -m "Fix login bug"

# Include commit message body
jcommit -b

# Commit changes directly
jcommit -c

# Specify repository path
jcommit -p /path/to/repo

# Generate commit message based on changes between branches or commits
jcommit -s main     # Compare with main branch
jcommit -s HEAD~1   # Compare with previous commit
jcommit -s v1.0.0   # Compare with a tag
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
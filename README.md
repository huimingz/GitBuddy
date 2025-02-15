# GitBuddy

[![Rust CI](https://github.com/fujianbang/GitBuddy/actions/workflows/rust.yaml/badge.svg)](https://github.com/fujianbang/GitBuddy/actions/workflows/rust.yaml)

GitBuddy is an AI-driven tool designed to simplify your Git commit process. With GitBuddy, you can generate meaningful
commit messages, streamline your workflow, and enhance your productivity.

> [!WARNING]
> This project is currently in **development**.

## Features

- **AI-Powered Commit Messages**: Generate intelligent and context-aware commit messages based on your code changes.
- **Conventional Commits Support**: Automatically formats commit messages following the Conventional Commits specification.
- **Multiple Commit Options**: Provides 3-5 commit message suggestions for you to choose from.
- **Beautiful CLI Interface**: 
  - Multiple separator themes for a personalized experience
  - Colorful output with emojis
  - Enhanced statistics and configuration display
- **Customizable Models**: Support for using different AI models, not only GPT-3.5.
- **Multiple Vendor Flexibility**: Compatible with various AI service providers.
- **Proxy Support**: Easily configure proxy settings for network-restricted environments.
- **Customizable Prompts**: Tailor the AI's suggestions to fit your project's specific needs.
- **Seamless Integration**: Works seamlessly with your existing Git workflow.
- **Improved Productivity**: Spend less time thinking about commit messages and more time coding.

## Installation

To get started with GitBuddy, follow these simple steps:

```sh
cargo install gitbuddy
```

### Configuration

To use GitBuddy, simply run the following command in your terminal:

**Using default model**
```sh
gitbuddy config --api-key <your-api-key> deepseek
```

**Using custom model**
```sh
gitbuddy config --api-key <your-api-key> --model gpt-4 openai
```

## Usage

Using GitBuddy is straightforward. After making your changes, run the following command to generate a commit message:

```sh
gitbuddy ai
```

### Commit Message Format

GitBuddy follows the [Conventional Commits](https://www.conventionalcommits.org/) specification, generating commit messages in this format:

```
<type>(<optional scope>): <subject>

<optional body>

<optional footer>
```

Where `type` can be:
- feat: A new feature
- fix: A bug fix
- docs: Documentation changes
- style: Code style changes (formatting, etc)
- refactor: Code refactoring
- perf: Performance improvements
- test: Adding or updating tests
- chore: Maintenance tasks

## Support models

| Vendor      | Model(s)                        | Support |
|-------------|---------------------------------|:-------:|
| DeepSeek    | deepseek-chat                   |   yes   |
| OpenAI      | gpt-3.5-turbo                   |   no    |
| ByteDance   | Doubao-lite-4k<br>Doubao-pro-4k |   no    |
| Baidu       | ERNIE 4.0                       |   no    |
| Alibaba     | qwen-turbo                      |   no    |
| Moonshot AI | moonshot-v1-8k                  |   no    |

## Roadmap

- [x] Enhance the User Interface
  - [x] Add multiple separator themes
  - [x] Implement colorful output with emojis
  - [x] Improve statistics and configuration display
- [x] Using configuration file instead of environment variables
- [x] Implement Conventional Commits support
- [x] Add multiple commit message suggestions
- [ ] Support for more AI models
- [ ] Add statistics and analytics for GitBuddy usage of kinds of Models
- [ ] Support http proxy
- [ ] Custom prompts
- [ ] **Install** for using GitBuddy by **Git Hooks** (without `gitbuddy ai`)
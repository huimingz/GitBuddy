use std::process::Command;
use colored::Colorize;

/// Returns a list of staged file names in the git repository.
/// This function executes `git diff --cached --name-only` to get the list.
pub fn git_stage_filenames() -> Vec<String> {
    let output = Command::new("git")
        .args([
            "diff",
            "--cached",
            "--no-ext-diff",
            "--diff-algorithm=minimal",
            "--name-only",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        return vec![];
    }

    String::from_utf8(output.stdout)
        .unwrap()
        .split('\n')
        .map_while(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
        .collect::<Vec<_>>()
}

/// Returns the diff content of staged files in the git repository.
/// This function executes `git diff --cached` while excluding certain files
/// specified in the ignore list.
pub fn git_stage_diff() -> String {
    let exclude_path: Vec<String> = ignore_filenames()
        .iter()
        .map(|path| format!(":(exclude){}", path))
        .collect();

    let mut command = Command::new("git");
    command.args(&[
        "diff",
        "--cached",
        "--no-ext-diff",
        "--diff-algorithm=minimal",
    ]);

    for path in exclude_path {
        command.arg(path);
    }

    let output = command.output().unwrap();

    if !output.status.success() {
        return "".to_string();
    }

    String::from_utf8(output.stdout).unwrap()
}

/// Returns a list of file patterns to ignore when generating diffs.
/// These patterns typically include build artifacts and dependency lock files.
fn ignore_filenames() -> Vec<&'static str> {
    vec![
        /* Rust files */
        "Cargo.lock",
        /* Node.js files */
        "node_modules",
        "dist",
        "package-lock.json",
        "pnpm-lock.json",
        "*.lock",
    ]
}

/// Commits the changes to the repository.
/// 
/// # Arguments
/// * `message` - The commit message to use
/// * `dry_run` - If true, only simulate the commit without actually performing it
/// 
/// # Returns
/// * `Ok(())` if the commit was successful or it was a dry run
/// * `Err` if the commit failed
pub fn git_commit(message: &str, dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        return Ok(());
    }

    println!(
        "\n{} {} {}",
        "🚀".bright_yellow(),
        "Executing Git Commit".bright_cyan().bold(),
        "...".bright_yellow()
    );

    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()?;

    if output.status.success() {
        println!(
            "{} {} {}\n",
            "✨".bright_green(),
            "Commit Successfully".bright_green().bold(),
            "🎉".bright_green()
        );
        Ok(())
    } else {
        println!(
            "{} {} {}\n",
            "❌".bright_red(),
            "Commit Failed".bright_red().bold(),
            "💔".bright_red()
        );
        Err(anyhow::anyhow!("commit failed"))
    }
}

/// Pushes the changes to the remote repository.
/// 
/// # Arguments
/// * `dry_run` - If true, only simulate the push without actually performing it
/// 
/// # Returns
/// * `Ok(())` if the push was successful or it was a dry run
/// * `Err` if the push failed
pub fn git_push(dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        return Ok(());
    }

    let output = Command::new("git")
        .args(["push", "origin", "HEAD"])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("push failed"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_git_stage_filename() {
        let filenames = git_stage_filenames();

        println!("filenames: {:?}", filenames);
        assert!(!filenames.iter().any(|s| s.is_empty()));
    }

    #[test]
    fn test_git_stage_diff() {
        let diff = git_stage_diff();

        println!("diff: {:?}", diff);
        assert!(!diff.is_empty());
    }
}

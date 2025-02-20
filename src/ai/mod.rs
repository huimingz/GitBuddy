use colored::Colorize;
use std::time::Instant;

use crate::ai::git::{git_stage_diff, git_stage_filenames};
use crate::llm;
use crate::llm::Confirm;
use crate::prompt::Prompt;

mod git;
mod theme;

fn get_stats_separator() -> String {
    format!(
        "{}  {}  {}",
        "⚡".bright_yellow(),
        "Performance Stats".bright_cyan().bold(),
        "⚡".bright_yellow()
    )
}

fn get_command_message() -> String {
    format!(
        "{} {}  {}",
        "🎯".bright_yellow(),
        "Initializing AI Assistant".bright_cyan().bold(),
        "⚡".bright_yellow()
    )
}

fn format_stat(label: &str, value: i64, emoji: &str) -> Option<String> {
    if value <= 0 {
        return None;
    }
    Some(format!(
        "{}  {}  {}",
        emoji.bright_yellow(),
        format!("{}: ", label).bright_cyan(),
        format!("{}", value).bright_green().bold()
    ))
}

pub fn handler(
    push: bool,
    dry_run: bool,
    vendor: Option<String>,
    model: Option<String>,
    prompt: Prompt,
    hint: Option<String>,
    number: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    if !is_git_directory() {
        println!("Not git directory");
        return Ok(());
    }

    if !is_git_installed() {
        println!("Please install git");
        return Ok(());
    }

    let filenames = git_stage_filenames();
    if filenames.is_empty() {
        println!("No files added to staging! Did you forget to run `git add` ?");
        return Ok(());
    }

    let diff_content = git_stage_diff();
    // let diff_content = format!("Code changes: \n```\n{}\n```", git_stage_diff());

    println!("{}", get_command_message());

    let start = Instant::now();
    let llm_result = llm::llm_request(&diff_content, vendor, model, prompt, hint, number)?;

    theme::print_stats(&llm_result, start.elapsed());

    let confirm = llm::confirm_commit(&llm_result)?;
    let commit_message = match confirm {
        Confirm::Retry | Confirm::Exit => {
            println!("{}", "Cancel commit".red());
            None
        }
        Confirm::Ok(msg) => Some(msg),
    };

    let result = git::git_commit(commit_message.unwrap(), dry_run);
    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e)
        }
    }

    // push
    if push {
        match git::git_push(dry_run) {
            Ok(_) => {
                println!("{}", "Push success!!!".green())
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        }
    }

    Ok(())
}

fn is_git_directory() -> bool {
    return std::process::Command::new("git").arg("rev-parse").output().is_ok();
}

fn is_git_installed() -> bool {
    return std::process::Command::new("git").arg("--version").output().is_ok();
}

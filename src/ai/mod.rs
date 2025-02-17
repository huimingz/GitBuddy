use std::time::Instant;

use colored::Colorize;

use crate::ai::git::{git_stage_diff, git_stage_filenames};
use crate::llm;
use crate::llm::Confirm;
use crate::prompt::Prompt;

mod git;

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
    prefix: Option<String>,
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
    let llm_result = llm::llm_request(&diff_content, vendor, model, prompt, prefix).expect("request llm success");
    let duration = start.elapsed();

    let separator = get_stats_separator();
    let mut stats = Vec::new();
    if let Some(stat) = format_stat("Duration", duration.as_millis() as i64, "⏱️") {
        stats.push(stat);
    }
    if let Some(stat) = format_stat("Usage", llm_result.total_tokens, "💰") {
        stats.push(stat);
    }
    if let Some(stat) = format_stat("Completion", llm_result.completion_tokens, "🎯") {
        stats.push(stat);
    }
    if let Some(stat) = format_stat("Prompt Tokens", llm_result.prompt_tokens, "🔤") {
        stats.push(stat);
    }

    if !stats.is_empty() {
        println!("\n{}", separator);
        for stat in stats {
            println!("  {}", stat);
        }
        println!();
    }

    let confirm = llm::confirm_commit(&llm_result, llm_result.commit_message.as_str()).unwrap();

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

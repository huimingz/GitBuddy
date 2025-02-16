use crate::llm::LLMResult;
use colored::Colorize;

pub const DEFAULT_COMMIT_OPTION_STYLE: u8 = 5;

/// Get the commit message separator with specified style
pub fn get_commit_separator(style: u8) -> (String, String, String) {
    match style {
        1 => (
            format!(
                "{} {} {}",
                "╭─".bright_magenta(),
                "Available Commit Options".bright_cyan().bold(),
                "─".repeat(30).bright_magenta()
            ),
            format!("{}", "│ ".bright_magenta()),
            format!(
                "{} {} {}",
                "╰─".bright_magenta(),
                "End of Options".bright_cyan().bold(),
                "─".repeat(37).bright_magenta()
            ),
        ),
        2 => (
            format!(
                "{} {} {}",
                "⚡".bright_yellow(),
                "Smart Commit Suggestions".bright_cyan().bold(),
                "★".repeat(28).bright_yellow()
            ),
            format!("{}", "✧ ".bright_yellow()),
            format!(
                "{} {} {}",
                "⚡".bright_yellow(),
                "Choose Your Commit".bright_cyan().bold(),
                "★".repeat(32).bright_yellow()
            ),
        ),
        3 => (
            format!(
                "{} {} {}",
                "◆".bright_green(),
                "Git Commit Selection".bright_cyan().bold(),
                "◇".repeat(32).bright_green()
            ),
            format!("{}", "◈ ".bright_green()),
            format!(
                "{} {} {}",
                "◆".bright_green(),
                "Selection Complete".bright_cyan().bold(),
                "◇".repeat(32).bright_green()
            ),
        ),
        4 => (
            format!(
                "{} {} {}",
                "🌸".bright_magenta(),
                "Commit Garden".bright_cyan().bold(),
                "✿".repeat(35).bright_magenta()
            ),
            format!("{}", "❀ ".bright_magenta()),
            format!(
                "{} {} {}",
                "🌸".bright_magenta(),
                "Plant Your Changes".bright_cyan().bold(),
                "✿".repeat(32).bright_magenta()
            ),
        ),
        5 => (
            format!(
                "{} {} {}",
                "🚀".bright_blue(),
                "Launch Pad".bright_cyan().bold(),
                "•".repeat(37).bright_blue()
            ),
            format!("{}", "∴ ".bright_blue()),
            format!(
                "{} {} {}",
                "🛸".bright_blue(),
                "Ready for Takeoff".bright_cyan().bold(),
                "•".repeat(32).bright_blue()
            ),
        ),
        6 => (
            format!(
                "{} {} {}",
                "⚔️".bright_red(),
                "Commit Arena".bright_cyan().bold(),
                "†".repeat(36).bright_red()
            ),
            format!("{}", "» ".bright_red()),
            format!(
                "{} {} {}",
                "🛡️".bright_red(),
                "Victory Achieved".bright_cyan().bold(),
                "†".repeat(32).bright_red()
            ),
        ),
        7 => (
            format!(
                "{} {} {}",
                "🎵".bright_yellow(),
                "Commit Symphony".bright_cyan().bold(),
                "♪".repeat(33).bright_yellow()
            ),
            format!("{}", "♫ ".bright_yellow()),
            format!(
                "{} {} {}",
                "🎼".bright_yellow(),
                "Finale".bright_cyan().bold(),
                "♪".repeat(40).bright_yellow()
            ),
        ),
        _ => (
            "-----------------------Commit Message-----------------------".to_string(),
            "".to_string(),
            "--------------------------------------------------------------".to_string(),
        ),
    }
}

pub fn print_commit_options(result: &LLMResult, style: u8) {
    let (header, prefix, footer) = get_commit_separator(style);
    println!("{}", header);
    for (idx, message) in result.commit_messages.iter().enumerate() {
        if idx < result.commit_messages.len() - 1 {
            println!(
                "{}{}\n{}\n",
                prefix,
                format!("Option {}:", idx + 1).bold().bright_cyan(),
                message.cyan()
            );
        } else {
            println!(
                "{}{}\n{}",
                prefix,
                format!("Option {}:", idx + 1).bold().bright_cyan(),
                message.cyan()
            );
        }
    }
    println!("{}", footer);
}

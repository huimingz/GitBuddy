use colored::Colorize;

pub fn get_stream_separator(style: u8) -> (String, String) {
    match style {
        1 => (
            "🚀 Generating Commit Messages ".bright_cyan().to_string() + &"•".repeat(30).bright_magenta(),
            "✨ Generation Complete ".bright_cyan().to_string() + &"•".repeat(33).bright_magenta(),
        ),
        2 => (
            format!(
                "{} {} {}",
                "┌".bright_green(),
                "Initializing AI Assistant".bright_cyan(),
                "─".repeat(30).bright_green()
            ),
            format!(
                "{} {} {}",
                "└".bright_green(),
                "AI Assistant Completed".bright_cyan(),
                "─".repeat(32).bright_green()
            ),
        ),
        3 => (
            format!(
                "{} {} {}",
                "▶".bright_yellow(),
                "Starting Commit Analysis".bright_cyan(),
                "═".repeat(32).bright_yellow()
            ),
            format!(
                "{} {} {}",
                "■".bright_yellow(),
                "Analysis Complete".bright_cyan(),
                "═".repeat(36).bright_yellow()
            ),
        ),
        _ => (
            "------------------------- Stream Start -------------------------".to_string(),
            "------------------------- Stream End -------------------------".to_string(),
        ),
    }
}

pub fn wrap_text(text: &str, width: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + word.len() + 1 <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines.join("\n")
}

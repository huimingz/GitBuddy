use crate::ai;
use crate::llm::LLMResult;
use std::time::Duration;

pub fn print_stats(llm_result: &LLMResult, duration: Duration) {
    let separator = ai::get_stats_separator();
    let mut stats = Vec::new();
    if let Some(stat) = ai::format_stat("Duration", duration.as_millis() as i64, "⏱️") {
        stats.push(stat);
    }
    if let Some(stat) = ai::format_stat("Usage", llm_result.total_tokens, "💰") {
        stats.push(stat);
    }
    if let Some(stat) = ai::format_stat("Completion", llm_result.completion_tokens, "🎯") {
        stats.push(stat);
    }
    if let Some(stat) = ai::format_stat("Prompt Tokens", llm_result.prompt_tokens, "🔤") {
        stats.push(stat);
    }

    if !stats.is_empty() {
        println!("\n{}", separator);
        for stat in stats {
            println!("  {}", stat);
        }
        println!();
    }
}

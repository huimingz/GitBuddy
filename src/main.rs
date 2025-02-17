use crate::llm::PromptModelVendor;
use clap::{Parser, Subcommand};
use prompt::Prompt;

mod ai;
mod config;
mod llm;
mod prompt;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "An AI-driven tool designed to simplify your Git commit process."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short = 'v', long)]
    vendor: Option<PromptModelVendor>,

    #[arg(short = 'm', long)]
    model: Option<String>,

    #[arg(long, default_value_t=Prompt::P1)]
    prompt: Prompt,

    #[arg(short = 'p', long)]
    prefix: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a commit message based on the current state of the repository
    Ai {
        /// push the commit to the remote repository
        #[arg(short, long, default_value_t = false)]
        push: bool,
        /// test argument, generate commit message but not commit
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        // #[arg(long, default_value_t=String::from("deepseek"))]
        // vendor: String,
    },
    Config {
        #[arg(value_enum)]
        vendor: llm::PromptModelVendor,
        #[arg(long)]
        api_key: String,
        #[arg(long)]
        model: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Ai {
            push,
            dry_run,
            // vendor,
        }) => {
            ai::handler(*push, *dry_run, cli.vendor, cli.model, cli.prompt, cli.prefix).unwrap();
        }
        Some(Commands::Config { vendor, api_key, model }) => {
            let model = if let Some(model) = model {
                model.to_string()
            } else {
                vendor.default_model().to_string()
            };

            config::handler(vendor, api_key, model.as_str()).unwrap();
        }
        None => ai::handler(false, false, cli.vendor, cli.model, cli.prompt, cli.prefix).unwrap(),
    }
}

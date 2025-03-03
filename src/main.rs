use clap::{Parser, Subcommand};
use prompt::Prompt;

mod ai;
mod args;
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
    /// plan to deprecate.
    #[command(subcommand)]
    command: Option<Commands>,

    /// model vendor, default is the vendor specified in the configuration file
    #[arg(short = 'v', long)]
    vendor: Option<String>,

    /// model name, default is the vendor's default model, eg. mistral
    #[arg(short = 'm', long)]
    model: Option<String>,

    /// prompt template, default is p1.
    /// plan to deprecate this, do not use it.
    #[arg(long, default_value_t=Prompt::P1)]
    prompt: Prompt,

    /// provide a hint for the AI to generate a better commit message, eg. "type: fix, scope: core"
    #[arg(long = "hint")]
    hint: Option<String>,

    /// number of commit options, default is 3
    #[arg(short = 'n', long = "number", default_value_t = 3)]
    number_of_commit_options: u8,

    /// the issue number or PR number, eg. #123
    #[arg(short = 'r', long = "ref")]
    reference: Option<String>,
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
            let cmd_args = args::CommandArgs::new(
                *push,
                *dry_run,
                cli.vendor.clone(),
                cli.model.clone(),
                cli.prompt,
                cli.hint.clone(),
                cli.number_of_commit_options,
                cli.reference.clone(),
            );
            ai::handler(cli.prompt, cmd_args).unwrap();
        }
        Some(Commands::Config { vendor, api_key, model }) => {
            let model = if let Some(model) = model {
                model.to_string()
            } else {
                vendor.default_model().to_string()
            };

            config::handler(vendor, api_key, model).unwrap();
        }
        None => {
            let cmd_args = args::CommandArgs::new(
                false,
                false,
                cli.vendor.clone(),
                cli.model.clone(),
                cli.prompt,
                cli.hint.clone(),
                cli.number_of_commit_options,
                cli.reference.clone(),
            );
            ai::handler(cli.prompt, cmd_args).unwrap()
        }
    }
}

//! hcscoder — High-performance AI coding assistant by hcsmedia
//!
//! Privacy-first, OpenRouter-powered CLI. Zero telemetry.

#![deny(warnings)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{self, EnvFilter};

use hcscoder::hcscoder_buddy;
use hcscoder::hcscoder_engine;
use hcscoder::hcscoder_memory;
use hcscoder::hcscoder_openrouter;
use hcscoder::hcscoder_openrouter::models;
use hcscoder::hcscoder_openrouter::HcscoderOpenRouterConfig;
use hcscoder::hcscoder_tools;
use hcscoder::hcscoder_ui;

pub const CONTACT_INSTAGRAM: &str = "@timfromhcs";
pub const CONTACT_EMAIL: &str = "hcsmediagroup@gmail.com";

pub const LOGO_ASCII: &str = r#"
   ███╗   ███╗ ██████╗███████╗ ██████╗ ██████╗ ██████╗ ███████╗██████╗
   ████╗ ████║██╔════╝██╔════╝██╔═══██╗██╔══██╗██╔══██╗██╔════╝██╔══██╗
   ██╔████╔██║██║     █████╗  ██║   ██║██║  ██║██║  ██║█████╗  ██████╔╝
   ██║╚██╔╝██║██║     ██╔══╝  ██║   ██║██║  ██║██║  ██║██╔══╝  ██╔══██╗
   ██║ ╚═╝ ██║╚██████╗██║     ╚██████╔╝██████╔╝██████╔╝███████╗██║  ██║
   ╚═╝     ╚═╝ ╚═════╝╚═╝      ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝
"#;

pub const VERSION_LONG: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " — hcscoder by hcsmedia | Stable Release\n",
    "Instagram: @timfromhcs  |  Email: hcsmediagroup@gmail.com\n",
    "OpenRouter-powered · MIT License (c) 2026 hcsmedia — attribution required · Made with ❤️",
);

#[derive(Parser)]
#[command(name = "hcscoder")]
#[command(author = "hcsmedia (Tim) <hcsmediagroup@gmail.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(long_version = VERSION_LONG)]
#[command(about = "High-performance AI coding assistant by hcsmedia", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// OpenRouter API key (or set OPENROUTER_API_KEY env var)
    #[arg(long, env = "OPENROUTER_API_KEY")]
    api_key: Option<String>,

    /// Model to use (CLI > OPENROUTER_MODEL > ~/.hcscoder/openrouter_default_model > catalog default)
    #[arg(long, env = "OPENROUTER_MODEL")]
    model: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Disable all UI elements (plain text output)
    #[arg(long)]
    plain: bool,

    /// Show developer contact (Instagram, email) and attribution
    #[arg(long)]
    contact: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive coding session
    Chat {
        /// Initial prompt
        #[arg()]
        prompt: Option<String>,
    },

    /// Run a single query and exit
    Ask {
        /// The question or task
        #[arg(required = true)]
        query: String,
    },

    /// Execute a shell command with AI assistance
    Run {
        /// Command to execute
        #[arg(required = true)]
        command: String,
    },

    /// Analyze and improve code
    Review {
        /// File or directory to review
        #[arg(required = true)]
        path: String,
    },

    /// Initialize hcscoder configuration
    Init,

    /// Manage Buddy companions
    Buddy {
        #[command(subcommand)]
        action: BuddyCommands,
    },

    /// View and manage memory
    Memory {
        #[command(subcommand)]
        action: MemoryCommands,
    },

    /// Display system status and configuration
    Status,
}

#[derive(Subcommand)]
enum BuddyCommands {
    /// Summon a new Buddy companion
    Summon,
    /// List your current Buddies
    List,
    /// View Buddy details
    Show { name: String },
    /// Release a Buddy
    Release { name: String },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// View consolidated memory
    View,
    /// Clear all memory
    Clear,
    /// Export memory to file
    Export { path: String },
}

fn print_contact() {
    println!("{}", LOGO_ASCII);
    println!("hcscoder — developer contact & attribution");
    println!("  Author:    hcsmedia (Tim) <{}>", CONTACT_EMAIL);
    println!("  Instagram: {}", CONTACT_INSTAGRAM);
    println!("  Email:     {}", CONTACT_EMAIL);
    println!("  License:   MIT (c) 2026 hcsmedia — attribution mandatory when redistributing.");
    println!("  Made with ❤️  by hcsmedia | Stable Release\n");
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .init();

    let cli = Cli::parse();

    if cli.contact {
        print_contact();
        return Ok(());
    }

    let model = cli
        .model
        .or_else(HcscoderOpenRouterConfig::load_saved_model)
        .unwrap_or_else(|| models::get_default_model().to_string());

    match cli.command {
        Some(Commands::Chat { prompt }) => {
            hcscoder_ui::run_chat_interface(cli.api_key, model, prompt, cli.plain).await?;
        }
        Some(Commands::Ask { query }) => {
            hcscoder_engine::handle_single_query(cli.api_key, model, &query, cli.plain).await?;
        }
        Some(Commands::Run { command }) => {
            hcscoder_tools::execute_with_assistance(cli.api_key, model, &command, cli.plain)
                .await?;
        }
        Some(Commands::Review { path }) => {
            hcscoder_engine::review_code(cli.api_key, model, &path, cli.plain).await?;
        }
        Some(Commands::Init) => {
            println!("{}", LOGO_ASCII);
            println!(
                "hcscoder v{} — initializing configuration…",
                env!("CARGO_PKG_VERSION")
            );
            println!("Contact: {} | {}", CONTACT_INSTAGRAM, CONTACT_EMAIL);
            println!();
            hcscoder_openrouter::init_config().await?;
            println!("Configuration complete.");
            println!("Next: set OPENROUTER_API_KEY or run hcscoder-setup");
        }
        Some(Commands::Buddy { action }) => match action {
            BuddyCommands::Summon => {
                hcscoder_buddy::summon_buddy().await?;
            }
            BuddyCommands::List => hcscoder_buddy::list_buddies()?,
            BuddyCommands::Show { name } => hcscoder_buddy::show_buddy(&name)?,
            BuddyCommands::Release { name } => hcscoder_buddy::release_buddy(&name)?,
        },
        Some(Commands::Memory { action }) => match action {
            MemoryCommands::View => hcscoder_memory::view_memory()?,
            MemoryCommands::Clear => hcscoder_memory::clear_memory()?,
            MemoryCommands::Export { path } => hcscoder_memory::export_memory(&path)?,
        },
        Some(Commands::Status) => {
            hcscoder_openrouter::show_status(cli.api_key, model);
        }
        None => {
            hcscoder_ui::run_chat_interface(cli.api_key, model, None, cli.plain).await?;
        }
    }

    Ok(())
}

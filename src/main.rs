use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use log::info;
use remarkable_reader_buddy::{OpenAI, Orchestrator, TriggerCorner, Workflow};
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "ReMarkable Reader Buddy - AI-powered reading assistant for reMarkable tablets")]
#[command(
    long_about = "ReMarkable Reader Buddy watches for circled content and handwritten questions, \
                        then uses ChatGPT to provide answers directly on your reMarkable tablet."
)]
pub struct Args {
    /// OpenAI API key (can also be set via OPENAI_API_KEY env var)
    #[arg(long, env = "OPENAI_API_KEY")]
    api_key: Option<String>,

    /// OpenAI model to use
    #[arg(long, short, default_value = "gpt-4o")]
    model: String,

    /// OpenAI base URL (for custom endpoints)
    #[arg(long, env = "OPENAI_BASE_URL")]
    base_url: Option<String>,

    /// Disable drawing/output (testing mode)
    #[arg(long)]
    no_draw: bool,

    /// Disable trigger waiting (run immediately)
    #[arg(long)]
    no_trigger: bool,

    /// Run only once instead of looping
    #[arg(long)]
    once: bool,

    /// Input PNG file for testing (instead of taking screenshot)
    #[arg(long)]
    input_png: Option<String>,

    /// Save screenshot to file
    #[arg(long)]
    save_screenshot: Option<String>,

    /// Trigger corner (UR, UL, LR, LL)
    #[arg(long, default_value = "LR")]
    trigger_corner: String,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<()> {
    // Load .env file if it exists
    dotenv().ok();

    let args = Args::parse();

    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&args.log_level))
        .format_timestamp_millis()
        .init();

    info!("=== ReMarkable Reader Buddy Starting ===");
    info!("Model: {}", args.model);
    info!("Trigger Corner: {} (lower-right)", args.trigger_corner);

    // Parse trigger corner
    let trigger_corner = TriggerCorner::from_string(&args.trigger_corner)?;

    // Initialize workflow
    let workflow = Workflow::new(args.no_draw, trigger_corner)?;

    // Give time for the virtual devices to be initialized
    sleep(Duration::from_millis(1000));

    // Initialize LLM
    let llm = if let Some(api_key) = args.api_key {
        OpenAI::new(args.model, api_key, args.base_url)
    } else {
        OpenAI::from_env(Some(args.model))?
    };

    // Create orchestrator
    let mut orchestrator = Orchestrator::new(workflow, llm);

    info!("Initialization complete");

    // Run the workflow
    if args.once {
        info!("Running single iteration");
        orchestrator.run_iteration()?;
    } else {
        info!("Starting main loop");
        orchestrator.run_loop()?;
    }

    Ok(())
}

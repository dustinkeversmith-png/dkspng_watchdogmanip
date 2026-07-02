use anyhow::Result;
use clap::{Parser, Subcommand};
use macro_os_engines::{context, history, navigation, parse, watchdog};
use std::{fs, path::PathBuf};
use history::adapters::{HistoryAdapter, MockHistoryAdapter};

#[derive(Debug, Parser)]
#[command(name = "macro-os")]
#[command(about = "Unified modular CLI for parse, context, navigation, history, and watchdog engines")]
struct Cli {
    #[command(subcommand)]
    engine: EngineCommand,
}

#[derive(Debug, Subcommand)]
enum EngineCommand {
    Parse(ParseCommand),
    Context(ContextCommand),
    Nav(NavCommand),
    History(HistoryCommand),
    Watchdog(WatchdogCommand),
}

#[derive(Debug, Parser)]
struct ParseCommand {
    input: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long)]
    pretty: bool,
}

#[derive(Debug, Parser)]
struct ContextCommand {
    #[command(subcommand)]
    command: ContextSubcommand,
}

#[derive(Debug, Subcommand)]
enum ContextSubcommand {
    Index { file: PathBuf, #[arg(long, default_value = "project")] default_context: String },
    Tree { file: PathBuf, #[arg(long, default_value = "project")] root: String },
    Inspect { file: PathBuf, context: String },
}

#[derive(Debug, Parser)]
struct NavCommand {
    #[command(subcommand)]
    command: NavSubcommand,
}

#[derive(Debug, Subcommand)]
enum NavSubcommand {
    Mock,
    Resolve { query: String, #[arg(long, default_value = "project")] scope: String, #[arg(long)] index_file: Option<PathBuf> },
    Plan { query: String, #[arg(long, default_value = "project")] scope: String, #[arg(long, default_value = "resolve")] action: String, #[arg(long)] index_file: Option<PathBuf> },
}

#[derive(Debug, Parser)]
struct HistoryCommand {
    #[command(subcommand)]
    command: HistorySubcommand,
}

#[derive(Debug, Subcommand)]
enum HistorySubcommand {
    Mock { #[arg(long, default_value = ".macro/history.jsonl")] out: PathBuf },
    Stats { input: PathBuf, #[arg(long, default_value_t = 10)] limit: usize },
    Suggest { input: PathBuf, text: Option<String>, #[arg(long)] context: Option<String>, #[arg(long)] workspace: Option<String>, #[arg(long, default_value_t = 10)] limit: usize },
    Print { input: PathBuf },
}

#[derive(Debug, Parser)]
struct WatchdogCommand {
    #[command(subcommand)]
    command: WatchdogSubcommand,
}

#[derive(Debug, Subcommand)]
enum WatchdogSubcommand {
    Validate { spec: PathBuf },
    ListRules { spec: PathBuf },
    Simulate { spec: PathBuf, events: PathBuf, #[arg(long)] expand_routines: bool },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.engine {
        EngineCommand::Parse(args) => run_parse(args),
        EngineCommand::Context(args) => run_context(args),
        EngineCommand::Nav(args) => run_nav(args),
        EngineCommand::History(args) => run_history(args),
        EngineCommand::Watchdog(args) => run_watchdog(args),
    }
}

fn run_parse(args: ParseCommand) -> Result<()> {
    let text = fs::read_to_string(&args.input)?;
    let pipeline = parse::MacroPipeline::default();
    let parsed = pipeline.parse(args.input.display().to_string(), text);
    let json = if args.pretty { serde_json::to_string_pretty(&parsed)? } else { serde_json::to_string(&parsed)? };
    if let Some(path) = args.output { fs::write(path, json)?; } else { println!("{}", json); }
    Ok(())
}

fn run_context(args: ContextCommand) -> Result<()> {
    match args.command {
        ContextSubcommand::Index { file, default_context } => {
            let input = fs::read_to_string(file)?;
            let index = context::build_index_from_document(&input, context::ParseConfig { default_context_id: default_context })?;
            println!("{}", index.export_json_pretty()?);
        }
        ContextSubcommand::Tree { file, root } => {
            let input = fs::read_to_string(file)?;
            let index = context::build_index_from_document(&input, context::ParseConfig::default())?;
            println!("{}", serde_json::to_string_pretty(&index.tree_from(&root)?)?);
        }
        ContextSubcommand::Inspect { file, context: context_id } => {
            let input = fs::read_to_string(file)?;
            let index = context::build_index_from_document(&input, context::ParseConfig::default())?;
            let lookup_order = index.context_lookup_order(&context_id)?;
            let aliases = index.aliases_visible_from(&context_id)?.into_iter().map(|(ctx, alias)| format!("{ctx}:{}", alias.name)).collect::<Vec<_>>();
            let symbols = index.symbols_visible_from(&context_id)?.into_iter().map(|(ctx, symbol)| format!("{ctx}:{}:{}", symbol.kind, symbol.name)).collect::<Vec<_>>();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "context": context_id,
                "lookup_order": lookup_order,
                "visible_aliases": aliases,
                "visible_symbols": symbols
            }))?);
        }
    }
    Ok(())
}

fn run_nav(args: NavCommand) -> Result<()> {
    match args.command {
        NavSubcommand::Mock => println!("{}", navigation::mock_navigation_index().export_json_pretty()?),
        NavSubcommand::Resolve { query, scope, index_file } => {
            let index = load_nav_index(index_file)?;
            let resolver = navigation::NavigationResolver::new(&index);
            println!("{}", serde_json::to_string_pretty(&resolver.resolve(&query, &scope)?)?);
        }
        NavSubcommand::Plan { query, scope, action, index_file } => {
            let index = load_nav_index(index_file)?;
            let resolver = navigation::NavigationResolver::new(&index);
            let plan = resolver.plan(navigation::NavigationRequest { action: parse_nav_action(&action), query, scope_id: scope })?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
    }
    Ok(())
}

fn load_nav_index(path: Option<PathBuf>) -> navigation::Result<navigation::NavigationIndex> {
    match path {
        Some(path) => navigation::NavigationIndex::import_json(&fs::read_to_string(path)?) ,
        None => Ok(navigation::mock_navigation_index()),
    }
}

fn parse_nav_action(value: &str) -> navigation::NavigationAction {
    match value.to_ascii_lowercase().as_str() {
        "open" => navigation::NavigationAction::Open,
        "preview" => navigation::NavigationAction::Preview,
        "list" => navigation::NavigationAction::List,
        "jump" => navigation::NavigationAction::Jump,
        _ => navigation::NavigationAction::Resolve,
    }
}

fn run_history(args: HistoryCommand) -> Result<()> {
    match args.command {
        HistorySubcommand::Mock { out } => {
            let mut adapter = MockHistoryAdapter::default();
            let events = adapter.collect()?;
            history::JsonlEventStore::new(&out).append_many(&events)?;
            println!("wrote {} mock events to {}", events.len(), out.display());
        }
        HistorySubcommand::Stats { input, limit } => {
            let events = history::read_jsonl_events(&input)?;
            let index = history::FrequencyIndex::build(&events, 14);
            println!("{}", serde_json::to_string_pretty(&index.top(limit))?);
        }
        HistorySubcommand::Suggest { input, text, context, workspace, limit } => {
            let events = history::read_jsonl_events(&input)?;
            let index = history::FrequencyIndex::build(&events, 14);
            let results = history::suggest(&index, &history::SuggestionQuery { text, context_id: context, workspace_id: workspace, limit });
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        HistorySubcommand::Print { input } => {
            let events = history::read_jsonl_events(&input)?;
            println!("{}", serde_json::to_string_pretty(&events)?);
        }
    }
    Ok(())
}

fn run_watchdog(args: WatchdogCommand) -> Result<()> {
    match args.command {
        WatchdogSubcommand::Validate { spec } => {
            let spec = watchdog::read_watch_spec(&spec)?;
            println!("valid watch spec: {} with {} rules and {} routines", spec.name, spec.rules.len(), spec.routines.len());
        }
        WatchdogSubcommand::ListRules { spec } => {
            let spec = watchdog::read_watch_spec(&spec)?;
            println!("{}", serde_json::to_string_pretty(&spec.rules)?);
        }
        WatchdogSubcommand::Simulate { spec, events, expand_routines } => {
            let spec = watchdog::read_watch_spec(&spec)?;
            let events = watchdog::read_file_events_jsonl(&events)?;
            let planned = watchdog::WatchdogPlanner::plan(&spec, &events)?;
            let planned = if expand_routines { watchdog::WatchdogPlanner::expand_routine_actions(&spec, &planned) } else { planned };
            println!("{}", serde_json::to_string_pretty(&planned)?);
        }
    }
    Ok(())
}

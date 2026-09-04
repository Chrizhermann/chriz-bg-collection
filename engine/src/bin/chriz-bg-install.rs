use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Mutex;

use bg_engine::cli::{
    diagnostics_for_managed_install, discover_games, human_plan_lines, inspect_game,
    install_campaign_controlled, install_interrupt_handler, plan_recipe, report_managed_install,
    resume_campaign_controlled, validate_recipe, CampaignStatus, CliError, InstallCommandRequest,
    SelectionOverrides, ValidationProfile,
};
use bg_engine::events::{ConsoleSink, EngineEvent, EventSink};
use bg_engine::games::GameRole;
use bg_engine::weidu::runner::RunnerControlHandle;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Parser)]
#[command(
    name = "chriz-bg-install",
    version,
    about = "Safe Chriz BG Collection installer engine"
)]
struct Cli {
    /// Emit one deterministic JSON value instead of human-readable output.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Load and validate an executable recipe directory.
    Validate(ValidateArgs),
    /// Resolve a preset and semantic overrides into exact ordered runs.
    Plan(PlanArgs),
    /// Discover or inspect clean source games.
    Games(GamesArgs),
    /// Start one new isolated managed installation.
    Install(InstallArgs),
    /// Resume one exact frozen managed installation.
    Resume { managed_root: PathBuf },
    /// Read the newest immutable attempt receipt.
    Report { managed_root: PathBuf },
    /// Export a create-once sanitized diagnostic bundle.
    Diagnostics {
        managed_root: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Debug, Args)]
struct ValidateArgs {
    recipe: PathBuf,
    #[arg(long, value_enum, default_value_t = ProfileArg::Authoring)]
    profile: ProfileArg,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum ProfileArg {
    #[default]
    Authoring,
    PublicAlpha,
}

impl From<ProfileArg> for ValidationProfile {
    fn from(value: ProfileArg) -> Self {
        match value {
            ProfileArg::Authoring => Self::Authoring,
            ProfileArg::PublicAlpha => Self::PublicAlpha,
        }
    }
}

#[derive(Debug, Args)]
struct PlanArgs {
    recipe: PathBuf,
    #[arg(long)]
    preset: String,
    #[arg(long, default_value = "windows")]
    platform: String,
    /// Override one semantic feature as `id=on|off`.
    #[arg(long = "feature")]
    features: Vec<String>,
    /// Override one typed input as `feature/input=kind:value`.
    #[arg(long = "input")]
    inputs: Vec<String>,
}

impl PlanArgs {
    fn overrides(&self) -> SelectionOverrides {
        SelectionOverrides {
            features: self.features.clone(),
            inputs: self.inputs.clone(),
        }
    }
}

#[derive(Debug, Args)]
struct GamesArgs {
    #[command(subcommand)]
    command: GamesCommand,
}

#[derive(Debug, Subcommand)]
enum GamesCommand {
    /// Discover installed sources using recipe-owned storefront profiles.
    Discover { recipe: PathBuf },
    /// Inspect an explicit source path against recipe-owned clean profiles.
    Inspect {
        recipe: PathBuf,
        #[arg(value_enum)]
        role: RoleArg,
        path: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum RoleArg {
    #[value(name = "bgee-sod")]
    BgeeSod,
    Bg2ee,
}

impl From<RoleArg> for GameRole {
    fn from(value: RoleArg) -> Self {
        match value {
            RoleArg::BgeeSod => Self::BgeeSod,
            RoleArg::Bg2ee => Self::Bg2ee,
        }
    }
}

#[derive(Debug, Args)]
struct InstallArgs {
    recipe: PathBuf,
    #[arg(long, default_value = "Chriz Easy BG")]
    name: String,
    #[arg(long)]
    preset: String,
    #[arg(long)]
    bg1: PathBuf,
    #[arg(long)]
    bg2: PathBuf,
    #[arg(long)]
    managed_root: PathBuf,
    #[arg(long)]
    cache: PathBuf,
    #[arg(long, default_value = "windows")]
    platform: String,
    #[arg(long = "feature")]
    features: Vec<String>,
    #[arg(long = "input")]
    inputs: Vec<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match execute(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_error(cli.json, command_name(&cli.command), &error);
            ExitCode::from(1)
        }
    }
}

fn execute(cli: &Cli) -> Result<(), CliError> {
    match &cli.command {
        Command::Validate(args) => {
            let report = validate_recipe(&args.recipe, args.profile.into())?;
            if cli.json {
                print_json(&json!({
                    "command": "validate",
                    "ok": true,
                    "recipe": report.recipe,
                    "profile": report.profile,
                    "findings": report.findings,
                }));
            } else {
                println!(
                    "Recipe is valid: {} ({} finding(s), profile {})",
                    report.recipe.display(),
                    report.findings.len(),
                    report.profile.as_str()
                );
                for finding in report.findings {
                    println!(
                        "{}[{}]: {}",
                        finding.severity, finding.rule, finding.message
                    );
                }
            }
        }
        Command::Plan(args) => {
            let report = plan_recipe(
                &args.recipe,
                &args.preset,
                &args.platform,
                &args.overrides(),
            )?;
            if cli.json {
                print_json(&json!({
                    "command": "plan",
                    "ok": true,
                    "recipe": report.recipe,
                    "preset": report.preset,
                    "normalized_selection": report.evaluation.normalized_selection,
                    "findings": report.evaluation.findings,
                    "plan": report.evaluation.plan,
                }));
            } else {
                println!(
                    "Resolved preset {:?} from {}:",
                    report.preset,
                    report.recipe.display()
                );
                for line in human_plan_lines(&report) {
                    println!("{line}");
                }
            }
        }
        Command::Games(args) => match &args.command {
            GamesCommand::Discover { recipe } => {
                let candidates = discover_games(recipe)?;
                if cli.json {
                    print_json(&json!({
                        "command": "games discover",
                        "ok": true,
                        "candidates": candidates,
                    }));
                } else if candidates.is_empty() {
                    println!("No supported game installations were discovered.");
                } else {
                    for candidate in candidates {
                        println!(
                            "{:?} {:?} {:?} {}",
                            candidate.role,
                            candidate.storefront,
                            candidate.eligibility,
                            candidate.root.display()
                        );
                    }
                }
            }
            GamesCommand::Inspect { recipe, role, path } => {
                let report = inspect_game(recipe, (*role).into(), path)?;
                if cli.json {
                    print_json(&json!({
                        "command": "games inspect",
                        "ok": true,
                        "candidate": report.candidate,
                    }));
                } else {
                    let candidate = report.candidate;
                    println!(
                        "{:?} {:?} {:?} {}",
                        candidate.role,
                        candidate.storefront,
                        candidate.eligibility,
                        candidate.root.display()
                    );
                    for finding in candidate.findings {
                        println!("- {:?}: {}", finding.kind, finding.message);
                    }
                }
            }
        },
        Command::Install(args) => {
            let controls = RunnerControlHandle::new();
            install_interrupt_handler(&controls)?;
            let request = InstallCommandRequest {
                application_version: None,
                display_name: args.name.clone(),
                recipe: args.recipe.clone(),
                preset: args.preset.clone(),
                platform: args.platform.clone(),
                overrides: SelectionOverrides {
                    features: args.features.clone(),
                    inputs: args.inputs.clone(),
                },
                bg1: args.bg1.clone(),
                bg2: args.bg2.clone(),
                managed_root: args.managed_root.clone(),
                cache: args.cache.clone(),
            };
            if cli.json {
                let sink = BufferedSink::default();
                match install_campaign_controlled(&request, &sink, &controls) {
                    Ok(report) => finish_json_campaign("install", report, sink.into_events())?,
                    Err(error) => return Err(error.with_events(sink.into_events())),
                }
            } else {
                finish_human_campaign(install_campaign_controlled(
                    &request,
                    &ConsoleSink,
                    &controls,
                )?)?;
            }
        }
        Command::Resume { managed_root } => {
            let controls = RunnerControlHandle::new();
            install_interrupt_handler(&controls)?;
            if cli.json {
                let sink = BufferedSink::default();
                match resume_campaign_controlled(managed_root, &sink, &controls) {
                    Ok(report) => finish_json_campaign("resume", report, sink.into_events())?,
                    Err(error) => return Err(error.with_events(sink.into_events())),
                }
            } else {
                finish_human_campaign(resume_campaign_controlled(
                    managed_root,
                    &ConsoleSink,
                    &controls,
                )?)?;
            }
        }
        Command::Report { managed_root } => {
            let report = report_managed_install(managed_root)?;
            if cli.json {
                print_json(&json!({
                    "command": "report",
                    "ok": true,
                    "managed_root": report.managed_root,
                    "receipt": report.receipt,
                }));
            } else {
                println!(
                    "Attempt {} for {}: {:?}",
                    report.receipt.attempt_id,
                    report.managed_root.display(),
                    report.receipt.outcome
                );
                println!("Plan SHA-256: {}", report.receipt.plan_sha256);
            }
        }
        Command::Diagnostics {
            managed_root,
            output,
        } => {
            let bundle = diagnostics_for_managed_install(managed_root, output)?;
            if cli.json {
                print_json(&json!({
                    "command": "diagnostics",
                    "ok": true,
                    "path": bundle.path,
                    "entries": bundle.entries,
                }));
            } else {
                println!(
                    "Diagnostics written to {} ({} files)",
                    bundle.path.display(),
                    bundle.entries.len()
                );
            }
        }
    }
    Ok(())
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::Validate(_) => "validate",
        Command::Plan(_) => "plan",
        Command::Games(GamesArgs {
            command: GamesCommand::Discover { .. },
        }) => "games discover",
        Command::Games(GamesArgs {
            command: GamesCommand::Inspect { .. },
        }) => "games inspect",
        Command::Install(_) => "install",
        Command::Resume { .. } => "resume",
        Command::Report { .. } => "report",
        Command::Diagnostics { .. } => "diagnostics",
    }
}

fn print_error(json_output: bool, command: &str, error: &CliError) {
    if json_output {
        let mut value = json!({
            "command": command,
            "ok": false,
            "error": {
                "code": error.code(),
                "message": error.to_string(),
            },
            "events": error.events(),
        });
        if let Some(report) = error.campaign_report() {
            value["install_id"] = json!(report.install_id);
            value["managed_root"] = json!(report.managed_root);
            value["plan_sha256"] = json!(report.plan_sha256);
            value["status"] = json!(report.status);
        }
        print_json(&value);
    } else {
        eprintln!("error[{}]: {error}", error.code());
    }
}

fn finish_json_campaign(
    command: &str,
    report: bg_engine::cli::CampaignReport,
    events: Vec<EngineEvent>,
) -> Result<(), CliError> {
    if report.status == CampaignStatus::Complete {
        print_json(&json!({
            "command": command,
            "ok": true,
            "install_id": report.install_id,
            "managed_root": report.managed_root,
            "plan_sha256": report.plan_sha256,
            "status": report.status,
            "events": events,
        }));
        Ok(())
    } else {
        Err(CliError::from_campaign(report).with_events(events))
    }
}

fn finish_human_campaign(report: bg_engine::cli::CampaignReport) -> Result<(), CliError> {
    if report.status == CampaignStatus::Complete {
        println!(
            "Managed installation ready at {}",
            report.managed_root.display()
        );
        Ok(())
    } else {
        Err(CliError::from_campaign(report))
    }
}

#[derive(Default)]
struct BufferedSink(Mutex<Vec<EngineEvent>>);

impl BufferedSink {
    fn into_events(self) -> Vec<EngineEvent> {
        self.0
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl EventSink for BufferedSink {
    fn emit(&self, event: EngineEvent) {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(event);
    }
}

fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string(value) {
        Ok(text) => println!("{text}"),
        Err(error) => eprintln!("error[json_output_failed]: {error}"),
    }
}

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "kevin",
    version,
    about = "One mind. Many voices. Always you.",
    long_about = "Local-first adaptive communication. No model selected: emits a prompt, not a generated reply. Model commands receive supplied profile, evidence and selected memory. All outputs remain drafts."
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
    #[command(flatten)]
    pub options: Options,
}
#[derive(Debug, Default, clap::Args)]
pub struct Options {
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    #[arg(long, global = true)]
    pub language: Option<String>,
    #[arg(long, global = true)]
    pub style: Option<String>,
    #[arg(long, global = true)]
    pub destination: Option<PathBuf>,
    #[arg(long, global = true)]
    pub description: Option<String>,
    #[arg(long, global = true)]
    pub answers: Option<PathBuf>,
    #[arg(long, global = true)]
    pub save: bool,
    #[arg(long, global = true)]
    pub request: Option<PathBuf>,
    #[arg(long, global = true)]
    pub intent: Option<String>,
    #[arg(long, global = true)]
    pub platform: Option<String>,
    #[arg(long, global = true)]
    pub task: Option<String>,
    #[arg(long, global = true)]
    pub audience: Option<String>,
    #[arg(long, global = true)]
    pub seriousness: Option<String>,
    #[arg(long, global = true)]
    pub energy: Option<String>,
    #[arg(long, global = true)]
    pub voice: Option<String>,
    #[arg(long, global = true)]
    pub blend: Vec<String>,
    #[arg(long, global = true)]
    pub workspace: Option<String>,
    #[arg(long, global = true)]
    pub channel: Option<String>,
    #[arg(long, global = true)]
    pub recipient: Option<String>,
    #[arg(long, global = true)]
    pub priority: Option<i64>,
    #[arg(long, global = true)]
    pub memory: Option<PathBuf>,
    #[arg(long, global = true)]
    pub text: Option<String>,
    #[arg(long, global = true)]
    pub kind: Option<String>,
    #[arg(long, global = true)]
    pub all: bool,
    #[arg(long, global = true)]
    pub no_humor: bool,
    #[arg(long, global = true)]
    pub json: bool,
    #[arg(long, global = true)]
    pub prompt: bool,
    #[arg(long, global = true)]
    pub model_command: Option<String>,
    #[arg(long, global = true, allow_hyphen_values = true)]
    pub model_arg: Vec<String>,
    #[arg(long, global = true)]
    pub model_id: Option<String>,
    #[arg(long, global = true)]
    pub settings: Option<String>,
    #[arg(long, global = true)]
    pub output_dir: Option<PathBuf>,
}
#[derive(Debug, Subcommand)]
pub enum Command {
    Init,
    Doctor,
    Voices,
    Rules,
    Voice {
        #[command(subcommand)]
        command: VoiceCommand,
    },
    Rule {
        #[command(subcommand)]
        command: RuleCommand,
    },
    Memory {
        #[command(subcommand)]
        command: Option<MemoryCommand>,
    },
    Identity {
        #[command(subcommand)]
        command: IdentityCommand,
    },
    Skill {
        #[command(subcommand)]
        command: SkillCommand,
    },
    Reply,
    Rewrite,
    Review,
    Explain,
    Summarize,
    Draft,
    /// Run public routing scenarios; model output recording is explicitly opt-in.
    Eval,
    /// Local Ollama bridge: reads an envelope on stdin and writes output JSON.
    Ollama,
}
#[derive(Debug, Subcommand)]
pub enum VoiceCommand {
    Default { name: Option<String> },
    Create { name: String },
    Edit { name: String },
}
#[derive(Debug, Subcommand)]
pub enum RuleCommand {
    Set { name: String },
    Remove { name: String },
}
#[derive(Debug, Subcommand)]
pub enum MemoryCommand {
    Add,
    List,
    Remove { id: String },
}
#[derive(Debug, Subcommand)]
pub enum IdentityCommand {
    Import { path: PathBuf },
    Export,
}
#[derive(Debug, Subcommand)]
pub enum SkillCommand {
    Install { host: String },
}

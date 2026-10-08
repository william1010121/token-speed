use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Command {
    #[default]
    Tui,
    Summary,
    Recent,
    Watch,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Provider {
    #[default]
    All,
    Codex,
    Claude,
}
impl Provider {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Codex => "Codex",
            Self::Claude => "Claude",
        }
    }
    pub fn matches(self, p: &str) -> bool {
        self == Self::All || self.label().eq_ignore_ascii_case(p)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Group {
    Hour,
    #[default]
    Day,
    Model,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Format {
    #[default]
    Table,
    Json,
    Csv,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Screen {
    #[default]
    Overview,
    Timeline,
    Models,
    Requests,
}
impl Screen {
    pub const ALL: [Self; 4] = [Self::Overview, Self::Timeline, Self::Models, Self::Requests];
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap()
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Timeline => "Timeline",
            Self::Models => "Models",
            Self::Requests => "Requests",
        }
    }
}

#[derive(Parser, Clone, Debug)]
#[command(
    version,
    about = "Codex + Claude · local token throughput observatory",
    after_help = "Run token-speed with no arguments for the TUI. Use summary/recent for scriptable reports."
)]
pub struct Args {
    #[arg(value_enum, default_value = "tui")]
    pub command: Command,
    #[arg(
        long,
        value_enum,
        default_value = "overview",
        help = "Initial TUI view"
    )]
    pub view: Screen,
    #[arg(long, value_enum, default_value = "all")]
    pub provider: Provider,
    #[arg(long, value_enum, default_value = "day")]
    pub group: Group,
    #[arg(long, conflicts_with_all=["days", "all_time", "since"])]
    pub today: bool,
    #[arg(long, default_value_t=7, conflicts_with_all=["today", "all_time", "since"])]
    pub days: u32,
    #[arg(long, conflicts_with_all=["today", "days", "since"])]
    pub all_time: bool,
    #[arg(long, conflicts_with_all=["today", "days", "all_time"], help="Inclusive ISO date/time")]
    pub since: Option<String>,
    #[arg(long, help = "Exclusive ISO time, or inclusive calendar date")]
    pub until: Option<String>,
    #[arg(long, help = "IANA timezone, e.g. Asia/Taipei")]
    pub timezone: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long)]
    pub session: Option<String>,
    #[arg(long, value_enum, default_value = "table")]
    pub format: Format,
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
    #[arg(long, default_value_t = 300.0)]
    pub max_gap: f64,
    #[arg(long, default_value_t = 5.0)]
    pub interval: f64,
    #[arg(long, default_value_os_t=codex_dir())]
    pub codex_dir: PathBuf,
    #[arg(long, default_value_os_t=claude_dir())]
    pub claude_dir: PathBuf,
    #[arg(long, default_value_os_t=cache_path())]
    pub cache: PathBuf,
    #[arg(long)]
    pub refresh: bool,
}
pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
fn codex_dir() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".codex"))
}
fn claude_dir() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".claude"))
}
fn cache_path() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".cache"))
        .join("token-speed/index.sqlite3")
}

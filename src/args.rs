use chrono::{DateTime, Utc};
use clap::Parser;

use std::path::PathBuf;
use std::{fmt, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct AppDuration(Duration);

impl fmt::Display for AppDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let secs = self.0.as_secs();
        let minutes = secs / 60;
        if secs % 86400 == 0 {
            write!(f, "{}d", secs / 86400)
        } else if secs % 3600 == 0 {
            write!(f, "{}h", secs / 3600)
        } else if secs % 60 == 0 {
            write!(f, "{}m", minutes)
        } else {
            write!(f, "{}s", secs)
        }
    }
}

impl FromStr for AppDuration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (num_part, unit_part) = s.split_at(s.len() - 1);
        let num: u64 = num_part.parse().map_err(|_| "Invalid num")?;

        match unit_part {
            "s" => Ok(AppDuration(Duration::from_secs(num))),
            "m" => Ok(AppDuration(Duration::from_secs(num * 60))),
            "h" => Ok(AppDuration(Duration::from_secs(num * 3600))),
            _ => Err("Bad unit".to_string()),
        }
    }
}

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub start: Option<DateTime<Utc>>,

    #[arg(short, long)]
    pub end: Option<DateTime<Utc>>,

    #[arg(short, long)]
    pub lines: Option<usize>,

    #[clap(required = true)]
    pub files: Vec<PathBuf>,

    #[arg(long, short)]
    pub duration: Option<AppDuration>,
}

pub fn cli_args() -> Args {
    let args = Args::parse();
    return args;
}

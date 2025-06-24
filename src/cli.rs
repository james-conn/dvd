use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "vhs")]
#[command(about = "Manage your .dvd or .tape files")]
#[command(version)]
pub struct Cli {
    /// Publish your GIF to yeet and get a shareable URL
    #[arg(short, long)]
    pub publish: bool,

    /// Quiet - do not log messages. If publish flag is provided, it will log shareable URL
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands
}

enum Outputs {
    Movie,
    Gif,
    Svg,
    Csv,
}

impl Outputs {
    fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp4" | "mov" | "avi" | "mkv" | "webm" => Some(Self::Movie),
            "gif" => Some(Self::Gif),
            "svg" => Some(Self::Svg),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }

    fn allowed_extensions() -> &'static [&'static str] {
        &["mp4", "mov", "avi", "mkv", "webm", "gif", "svg", "csv"]
    }
}

fn default_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| "/bin/bash".to_string())
        .split('/')
        .next_back()
        .unwrap_or("bash")
        .to_string()
}

fn validate_output_path(path_str: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);

    // Get the extension of the provided path
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| {
            format!(
                "Output file '{}' must have a valid extension. Allowed extensions: {}",
                path_str,
                Outputs::allowed_extensions().join(", ")
            )
        })?;

    // Check if that provided path extension is valid agaisnt the allowed ones.
    Outputs::from_extension(extension).ok_or_else(|| {
        format!(
            "Unsupported output format '{}'. Allowed extensions: {}",
            extension,
            Outputs::allowed_extensions().join(", ")
        )
    })?;

    Ok(path)
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all the available themes, one per line
    Themes {
        /// Output as markdown
        #[arg(long, hide = true)]
        markdown: bool,
    },

    Burn(BurnArgs),

    /// Create a new tape file by recording your actions
    Record {
        /// Shell for recording
        #[arg(short, long, default_value_t = default_shell())]
        shell: String,
    },

    /// Play a tape file
    Play {
        /// Files to play (sequentially)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Create a new tape file with example tape file contents and documentation
    New {
        /// Name of the new tape file
        name: String,
    },

    /// Validate a glob file path and parses all the files to ensure they are valid without running them
    Check {
        /// Files to validate
        #[arg(required = true)]
        files: Vec<PathBuf>,
    }
}

#[derive(Args)]
pub struct BurnArgs {
	/// Input tape file (use "-" for stdin)
	pub input_file: PathBuf,

	/// File name(s) of video output
	#[arg(
		value_parser = validate_output_path,
		value_hint = clap::ValueHint::FilePath
	)]
	pub output_file: PathBuf
}

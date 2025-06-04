// src/main.rs (for testing)
use clap;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

enum Outputs {
    Movie,
    Gif,
    SVG,
    CSV,
}

impl Outputs {
    fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp4" | "mov" | "avi" | "mkv" | "webm" => Some(Self::Movie),
            "gif" => Some(Self::Gif),
            "svg" => Some(Self::SVG),
            "csv" => Some(Self::CSV),
            _ => None,
        }
    }

    fn allowed_extensions() -> &'static [&'static str] {
        &["mp4", "mov", "avi", "mkv", "webm", "gif", "svg", "csv"]
    }
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
    pub command: Commands,
}

fn default_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| "/bin/bash".to_string())
        .split('/')
        .last()
        .unwrap_or("bash")
        .to_string()
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all the available themes, one per line
    Themes {
        /// Output as markdown
        #[arg(long, hide = true)]
        markdown: bool,
    },

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
    },
    // Publish your GIF to vhs.charm.sh and get a shareable URL
    // Publish {
    //     /// GIF file to publish
    //     gif_file: PathBuf,
    // },
}

fn main() -> anyhow::Result<()> {
    let input = r#"
    Output examples/gum/pager.gif

Set Padding 32
Set FontSize 16
Set Height 600

Type "gum pager < ~/src/gum/README.md --border normal"
Sleep 0.5s
Enter
Sleep 1s
Down@15ms 40
Sleep 0.5s
Up@15ms 30
Sleep 0.5s
Down@15ms 20
Sleep 2s

"#;

    use dvd::*;
    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);

    let commands = parser.parse();

    for cmd in commands {
        println!("{:#?}", cmd);
    }

    for error in parser.errors() {
        eprintln!("Error: {}", error);
    }

    Ok(())
}

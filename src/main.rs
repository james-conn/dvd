// src/main.rs (for testing)
use clap;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "vhs")]
#[command(about = "Run a given tape file and generates its outputs.")]
#[command(version)]
pub struct Cli {
    /// Input tape file (use "-" for stdin)
    pub file: Option<PathBuf>,

    /// Publish your GIF to vhs.charm.sh and get a shareable URL
    #[arg(short, long)]
    pub publish: bool,

    /// Quiet - do not log messages. If publish flag is provided, it will log shareable URL
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// File name(s) of video output
    #[arg(short, long, value_name = "FILE")]
    pub output: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
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

use alacritty_terminal::event::Event;
use alacritty_terminal::event::{EventListener, WindowSize};
use alacritty_terminal::event_loop::EventLoop;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::tty::{self, EventedReadWrite, Options, Shell};
use alacritty_terminal::{
    Term,
    term::{Config, test::TermSize},
};
use clap::{Parser, Subcommand};
use dvd_render::image::Rgba;

// Standard library imports
use std::cell::RefCell;
use std::collections::HashMap;
use std::env::current_dir;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, channel};
use std::time::Duration;

const WIDTH: usize = 50;
const HEIGHT: usize = 50;

enum Outputs {
    Movie,
    Gif,
    Svg,
    Csv,
}

#[derive(Clone)]
struct Listener {
    mister: RefCell<Option<mpsc::Sender<()>>>,
    term: std::sync::OnceLock<Arc<FairMutex<Term<Listener>>>>,
}

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        match event {
            Event::Wakeup => {
                if let Some(ref sender) = *self.mister.borrow() {
                    println!("AAAA");
                    sender.send(()).unwrap();
                    println!("BBB");
                }
            }
            Event::Exit => {
                println!("{:?}", event);
                *self.mister.borrow_mut() = None; // This drops the sender
            }
            _ => println!("{:?}", event),
        }
    }
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
        .next_back()
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

    Burn {
        /// File name(s) of video output
        #[arg(short, long, value_name = "FILE", value_parser = validate_output_path)]
        output: Vec<PathBuf>,

        /// Input tape file (use "-" for stdin)
        file: Option<PathBuf>,
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

use dvd_render::{ab_glyph, prelude::*};
use pollster::FutureExt;

fn main() {
    let (sender, receiver) = channel();

    let sender = RefCell::new(Some(sender));
    let listener = Listener {
        mister: sender,
        term: std::sync::OnceLock::new(),
    };

    let term = Term::new(
        Config::default(),
        &TermSize::new(WIDTH, HEIGHT),
        listener.clone(),
    );

    let shell = Shell::new("/bin/sh".to_string(), vec![]);

    let pty_options = Options {
        shell: Some(shell),
        working_directory: Some(current_dir().unwrap()),
        drain_on_exit: true,
        env: HashMap::default(),
    };

    let mut pty = tty::new(
        &pty_options,
        WindowSize {
            num_lines: 50,
            num_cols: 50,
            cell_width: 1,
            cell_height: 1,
        },
        59,
    )
    .unwrap();

    let mut pty_writer = pty.writer().try_clone().unwrap(); // Clone the File handle

    let term = Arc::new(FairMutex::new(term));
    let _ = listener.term.set(term.clone());

    let loopp = EventLoop::new(term.clone(), listener, pty, true, false).unwrap();
    loopp.spawn();

    // Now you can use pty_writer in your thread
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(800));

        // Write to the actual shell process
        pty_writer.write_all(b"nvim\n").unwrap();
        pty_writer.flush().unwrap(); // Important: flush to ensure it's sent

        std::thread::sleep(Duration::from_millis(10));
        println!("Command sent to shell");
    });

    let mut grid = Grid::<WIDTH, HEIGHT>::default();

    let mut seq = GridSequence::new(Pt(40.0));
    seq.framerate = core::num::NonZeroU8::new(10).unwrap();

    let mut count = 0;
    while let Ok(()) = receiver.recv() {
        let term_term = term.lock();

        for cell in term_term.grid().display_iter() {
            // let fg_color = cell.cell.fg;
            // let bg_color = cell.cell.bg;
            let fg_color = Rgba([124, 40, 32, 128]);
            let bg_color = Rgba([20, 5, 28, 128]);

            println!("{:?}", fg_color);

            grid.set(
                cell.point.column.0,
                cell.point.line.0 as usize,
                GridCell::new_full_color(cell.cell.c, fg_color, bg_color),
            );
        }

        seq.append(Frame::variable(
            grid.clone(),
            core::num::NonZeroU8::new(10).unwrap(),
        ));

        count += 1;
        println!("{count}");

        if count == 10 {
            break;
        }
    }

    seq.append(Frame::variable(
        grid,
        core::num::NonZeroU8::new(50).unwrap(),
    ));

    let font = ab_glyph::FontRef::try_from_slice(include_bytes!(
        "/Users/philocalyst/Library/Fonts/HackNerdFont-BoldItalic.ttf"
    ))
    .unwrap();
    let renderer = WgpuRenderer::new(font, seq).block_on();

    let encoder = dvd_render::video::DvdEncoder::new(renderer);
    encoder.save_video_to("video.mkv");
}

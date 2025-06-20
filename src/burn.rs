use alacritty_terminal::event::Event;
use alacritty_terminal::event::{EventListener, WindowSize};
use alacritty_terminal::event_loop::EventLoop;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::tty::{self, EventedReadWrite, Options, Shell};
use alacritty_terminal::{
	Term,
	term::{Config, test::TermSize},
};
use dvd_render::image::Rgba;
use dvd_render::ab_glyph;
use dvd_render::prelude::*;
use pollster::FutureExt;

// Standard library imports
use std::cell::RefCell;
use std::collections::HashMap;
use std::env::current_dir;
use std::io::Write;
use std::sync::Arc;
use std::sync::mpsc::{self, channel};
use std::time::Duration;
use crate::cli::BurnArgs;
use crate::lexer::Lexer;
use crate::parser::{Parser, Commands};

const WIDTH: usize = 50;
const HEIGHT: usize = 50;

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

pub fn burn(args: &BurnArgs) -> Result<(), ()> {
	let in_str = std::fs::read_to_string(&args.input_file).unwrap();

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
		let mut lexer = Lexer::new(&in_str);
		let mut parser = Parser::new(&mut lexer);
		let mut utf8_buf = [0u8; 4];

		for command in parser.parse().into_iter() {
			match command {
				Commands::Type(type_cmd) => {
					let rate = type_cmd.rate.unwrap_or(Duration::from_millis(50));
					for c in type_cmd.text.chars() {
						let len = c.len_utf8();
						c.encode_utf8(&mut utf8_buf);
						pty_writer.write_all(&utf8_buf[..len]).unwrap();
						pty_writer.flush().unwrap();
						std::thread::sleep(rate);
					}
				},
				_ => todo!()
			}
		}
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
		"../fonts/liberation_mono/LiberationMono-Regular.ttf"
	))
	.unwrap();
	let renderer = WgpuRenderer::new(font, seq).block_on();

	let encoder = dvd_render::video::DvdEncoder::new(renderer);
	encoder.save_video_to(&args.output_file);

	Ok(())
}

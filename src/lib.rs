pub mod cli;

mod lexer;
mod parser;
mod token;

mod burn;

pub fn run(cli: cli::Cli) -> std::process::ExitCode {
	let output = match cli.command {
		cli::Commands::Burn(args) => burn::burn(&args),
		_ => todo!()
	};

	match output {
		Ok(()) => std::process::ExitCode::SUCCESS,
		Err(()) => std::process::ExitCode::FAILURE
	}
}

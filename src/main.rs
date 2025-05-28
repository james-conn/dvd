// src/main.rs (for testing)
use vhs_parser::*;

fn main() -> anyhow::Result<()> {
    let input = r#"
Output examples/out.gif
Set FontSize 42
Type "Hello, world!"
Enter
Sleep 1s
"#;

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);

    let commands = parser.parse();

    for cmd in commands {
        println!("{}", cmd);
    }

    for error in parser.errors() {
        eprintln!("Error: {}", error);
    }

    Ok(())
}

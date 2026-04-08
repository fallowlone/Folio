mod lexer;
mod parser;
mod renderer;

use lexer::Lexer;
use parser::Parser;

fn main() {
    let input = r#"
STYLES({
  #mainColor: #FF0000
})

PAGE(
  H1({color: #mainColor} Hello World)
  P(This is a paragraph.)
)
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let doc = parser.parse();
    let doc = parser::resolver::resolve(doc);

    println!("=== JSON ===");
    println!("{}", renderer::json::render(&doc));

    println!("\n=== Plain Text ===");
    println!("{}", renderer::text::render(&doc));
}

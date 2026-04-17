#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Block types
    Ident(String), // H1, P, PAGE, STYLES, GRID, IMAGE, ...

    // Delimiters
    LParen,  // (
    RParen,  // )
    LBrace,  // {
    RBrace,  // }
    LBracket, // [
    RBracket, // ]

    // Attributes
    Colon,   // :
    Comma,   // ,

    // Values
    Text(String),    // raw text content
    /// Verbatim body of a raw-body block (CODE). Preserves newlines and whitespace;
    /// no inline parsing, no block recognition, no `#` breaking. Balanced-paren only.
    RawText(String),
    String(String),  // "quoted string"
    Number(f64),     // 24, 1.5
    Unit(f64, String), // 25mm, 1fr
    Hash(String),    // #mainColor, #FF0000

    // Special
    Eof,
}

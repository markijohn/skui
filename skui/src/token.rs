use logos::Logos;

fn parse_rgb(s: &str) -> Option<(u8, u8, u8)> {
    let inner = s.trim_start_matches("rgb(").trim_end_matches(')');
    let mut it = inner.split(',').map(|v| v.trim().parse::<u8>().ok());
    Some((it.next()??, it.next()??, it.next()??))
}

fn parse_rgba(s: &str) -> Option<(u8, u8, u8, u8)> {
    let inner = s.trim_start_matches("rgba(").trim_end_matches(')');
    let mut it = inner.split(',').map(|v| v.trim().parse::<u8>().ok());
    Some((it.next()??, it.next()??, it.next()??, it.next()??))
}

#[derive(Logos, Debug, Clone, Copy, PartialEq)]
pub enum Token<'a> {
    #[regex(
        r"rgba\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)",
        |lex| parse_rgba(lex.slice())
    )]
    Rgba((u8, u8, u8, u8)),

    #[regex(
        r"rgb\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)",
        |lex| parse_rgb(lex.slice())
    )]
    Rgb((u8, u8, u8)),

    #[regex(r"[0-9]+(\.[0-9]+)?em", |lex| {
        let s = lex.slice();
        s[..s.len()-2].parse::<f64>().ok()
    })]
    Em(f64),

    #[regex(r"[0-9]+(\.[0-9]+)?pt", |lex| {
        let s = lex.slice();
        s[..s.len()-2].parse::<f64>().ok()
    })]
    Pt(f64),

    #[regex(r"[0-9]+(\.[0-9]+)?px", |lex| {
        let s = lex.slice();
        s[..s.len()-2].parse::<f64>().ok()
    })]
    Px(f64),

    #[regex(r"[0-9]+(\.[0-9]+)?%", |lex| {
        let s = lex.slice();
        s[..s.len()-1].parse::<f64>().ok()
    })]
    Percent(f64),

    #[regex(r"[A-Za-z_][A-Za-z0-9_-]*", |lex| lex.slice())]
    Ident(&'a str),

    #[regex(r"#[A-Za-z0-9_][A-Za-z0-9_-]*", |lex| &lex.slice()[1..])]
    Id(&'a str),

    #[regex(r"\.[A-Za-z_][A-Za-z0-9_-]*", |lex| &lex.slice()[1..])]
    Class(&'a str),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        &s[1..s.len()-1]
    })]
    Str(&'a str),

    #[regex(r"-?\d+\.\d+", |lex| lex.slice().parse().ok())]
    Float(f64),

    #[regex(r"-?\d+", |lex| lex.slice().parse().ok())]
    Integer(i64),

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[regex(r"\$\{([^}]+)\}", |lex| &lex.slice()[2..lex.slice().len()-1] )]
    Relative( &'a str ),

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("<")]
    Lt,

    #[token(">")]
    Gt,

    #[token("=")]
    Equal,

    #[token("|")]
    Pipe,

    // #[regex(r"[ \t\r\n]+", logos::skip)]
    #[regex(r"[ \t\r\n]+")]
    Whitespace,

    None,
}

impl <'a> Token<'a> {
    pub fn block_brace() -> (Self,Self) {
        (Token::LBrace, Token::RBrace)
    }

    pub fn block_bracket() -> (Self,Self) {
        (Token::LBracket, Token::RBracket)
    }

    pub fn block_paren() -> (Self,Self) {
        (Token::LParen, Token::RParen)
    }
}

impl <'a> Default for Token<'a> {
    fn default() -> Self {
        Self::None
    }
}
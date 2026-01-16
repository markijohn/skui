use logos::Logos;
use chumsky::prelude::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum CssKeyword {
    Auto,
    None,
    Inherit,
}

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

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum CssValueToken {
    #[token("auto", |_| CssKeyword::Auto)]
    #[token("none", |_| CssKeyword::None)]
    #[token("inherit", |_| CssKeyword::Inherit)]
    Keyword(CssKeyword),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse().ok() )]
    Number(f64),

    #[regex(r"[0-9]+(\.[0-9]+)?%", |lex| {
        let s = lex.slice();
        s[..s.len()-1].parse::<f64>().ok()
    })]
    Percent(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    Str(String),

    #[regex(r"[0-9]+(\.[0-9]+)?px", |lex| {
        let s = lex.slice();
        s[..s.len()-2].parse::<f64>().ok()
    })]
    Px(f64),

    #[regex(r"#([0-9a-fA-F]{6}|[0-9a-fA-F]{8})", |lex| {
        Some(lex.slice().to_string())
    })]
    HexColor(String),

    #[regex(
        r"rgba\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)",
        |lex| parse_rgba(lex.slice()) )
    ]
    Rgba( (u8,u8,u8,u8) ),

    #[regex(
        r"rgb\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)",
        |lex| parse_rgb(lex.slice())
    )]
    Rgb( (u8,u8,u8) ),

    List(Vec<CssValueToken>),

    #[token(",")]
    Comma,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    Keyword(CssKeyword),
    Ident(String),
    Number(f64),
    Percent(f64),
    Str(String),
    Px(f64),
    HexColor(String),
    Rgba(u8, u8, u8, u8),
    Rgb(u8, u8, u8),
    List(Vec<CssValue>),
}

// ==================== Token Definition ====================
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    #[regex(r"[A-Za-z_][A-Za-z0-9_-]*", |lex| lex.slice().to_string())]
    Ident(String),

    //#[regex(r"#[A-Za-z_][A-Za-z0-9_-]*", |lex| lex.slice()[1..].to_string())]
    #[regex(r"#[A-Za-z0-9_][A-Za-z0-9_-]*", |lex| lex.slice()[1..].to_string())] //for hex value
    Id(String),

    #[regex(r"\.[A-Za-z_][A-Za-z0-9_-]*", |lex| lex.slice()[1..].to_string())]
    Class(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    Text(String),

    #[regex(r"-?\d+\.\d+", |lex| lex.slice().parse().ok())]
    Float(f64),

    #[regex(r"-?\d+", |lex| lex.slice().parse().ok())]
    Integer(i64),

    #[token("true")]
    True,

    #[token("false")]
    False,

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

    #[token("=")]
    Equal,

    #[regex(r"[ \t\r\n]+", logos::skip)]
    Whitespace,
}

// ==================== AST Types ====================
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    Plain(String),
    Id(String),
    Class(String),
}

#[derive(Debug, Clone)]
pub struct Style {
    pub selectors: Vec<Selector>,
    pub properties: Vec<(String, CssValue)>,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub params: Option<Parameters>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub text: Option<String>,
    pub children: Vec<Component>,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Style(Style),
    Component(Component),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    I64(i64),
    F64(f64),
}

impl Number {
    pub fn as_i64(&self) -> i64 {
        match self {
            Number::I64(i) => *i,
            Number::F64(f) => *f as i64,
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Number::I64(i) => *i as f64,
            Number::F64(f) => *f,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Ident(String),
    Bool(bool),
    Number(Number),
    String(String),
    Color(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
}

#[derive(Debug, Clone)]
pub enum Parameters {
    Args(Vec<Value>),
    Map(HashMap<String, Value>),
}


fn parse_css_value_simple(s: &str) -> CssValue {
    let trimmed = s.trim();

    // CssValueToken으로 렉싱 시도
    let tokens: Vec<CssValueToken> = CssValueToken::lexer(trimmed)
        .filter_map(|t| t.ok())
        .collect();

    if tokens.len() == 1 {
        match &tokens[0] {
            CssValueToken::Keyword(k) => return CssValue::Keyword(k.clone()),
            CssValueToken::Number(n) => return CssValue::Number(*n),
            CssValueToken::Percent(p) => return CssValue::Percent(*p),
            CssValueToken::Px(p) => return CssValue::Px(*p),
            CssValueToken::HexColor(c) => return CssValue::HexColor(c.clone()),
            CssValueToken::Rgba( (r, g, b, a) ) => return CssValue::Rgba(*r, *g, *b, *a),
            CssValueToken::Rgb( (r, g, b) ) => return CssValue::Rgb(*r, *g, *b),
            CssValueToken::Ident(s) => return CssValue::Ident(s.clone()),
            _ => {}
        }
    }

    // 기본값: Ident로 처리
    CssValue::Ident(trimmed.to_string())
}

// Component 내부 아이템 (Property 또는 Component)
#[derive(Debug, Clone)]
enum ComponentItem {
    Text(String),
    Property(String, Value),
    Child(Component),
}

// ==================== Parser ====================
pub fn parser<'a>() -> impl Parser<'a, &'a [Token], Vec<AstNode>, extra::Err<Rich<'a, Token>>> {
    let value = recursive(|value| {
        let bool_val = select! {
            Token::True => Value::Bool(true),
            Token::False => Value::Bool(false),
        };

        let number = select! {
            Token::Integer(s) => Value::Number(Number::I64(s)),
            Token::Float(s) => Value::Number(Number::F64(s)),
        };

        let string = select! {
            Token::Text(s) => Value::String(s),
        };

        let ident = select! {
            Token::Ident(s) => Value::Ident(s),
        };

        let array = value
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(Value::Array);

        choice((bool_val, number, string, array, ident))
    });

    let parameters = {
        let key_value = select! { Token::Ident(s) => s }
            .then_ignore(just(Token::Equal))
            .then(value.clone())
            .map(|(k, v)| (k, v));

        let map_params = key_value
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .map(|pairs| {
                Parameters::Map(pairs.into_iter().collect())
            });

        let args_params = value
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .map(Parameters::Args);

        args_params.or(map_params)
    };

    let selector = choice((
        select! { Token::Id(s) => Selector::Id(s) },
        select! { Token::Class(s) => Selector::Class(s) },
    ));

    let style = selector
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .then(
            select! { Token::Ident(s) => s }
                .then_ignore(just(Token::Colon))
                .then(select! { Token::Ident(s) => s })
                .separated_by(choice((just(Token::Comma), just(Token::Semicolon))))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .map(|(selectors, props)| {
            let properties = props.into_iter()
                .map(|(k, v)| (k, parse_css_value_simple(&v)))
                .collect();
            AstNode::Style(Style { selectors, properties })
        });

    // let style = selector
    //     .repeated()
    //     .at_least(1)
    //     .collect::<Vec<_>>()
    //     .then(
    //         style_property
    //             .separated_by(choice((just(Token::Comma), just(Token::Semicolon))))
    //             .allow_trailing()
    //             .collect::<Vec<_>>()
    //             .delimited_by(just(Token::LBrace), just(Token::RBrace))
    //     )
    //     .map(|(selectors, properties)| {
    //         AstNode::Style(Style { selectors, properties })
    //     });

    let component = recursive(|component| {
        let name = select! { Token::Ident(s) => s };
        let params = parameters
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .or_not();
        let id = select! { Token::Id(s) => s }.or_not();
        let classes = select! { Token::Class(s) => s }.repeated().collect::<Vec<_>>();

        // 3가지 아이템 타입
        let text_item = select! { Token::Text(s) => ComponentItem::Text(s) };
        let property_item = select! { Token::Ident(s) => s }
            .then_ignore(just(Token::Colon))
            .then(value.clone())
            .map(|(k, v)| ComponentItem::Property(k, v));
        let child_item = component.clone().map(ComponentItem::Child);

        // 순서 무관하게 모두 파싱
        let items = choice((text_item, property_item, child_item))
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .or_not();

        name.then(params)
            .then(id)
            .then(classes)
            .then(items)
            .map(|((((name, params), id), classes), items)| {
                let mut text = None;
                let mut properties = HashMap::new();
                let mut children = vec![];

                if let Some(items) = items {
                    for item in items {
                        match item {
                            ComponentItem::Text(t) => text = Some(t),
                            ComponentItem::Property(k, v) => { properties.insert(k, v); },
                            ComponentItem::Child(c) => children.push(c),
                        }
                    }
                }

                Component { name, params, id, classes, text, children, properties }
            })
    });

    //let node = choice((style, component));
    let node = choice((style, component.map(AstNode::Component)));
    node.repeated().collect::<Vec<_>>().then_ignore(end())
}

// ==================== Main Parse Function ====================
pub fn parse(tokens:&[Token]) -> Result<Vec<AstNode>, Vec<Rich<Token>>> {
    // let tokens: Vec<Token> = Token::lexer(input)
    //     .filter_map(|t| t.ok())
    //     .collect();

    parser().parse(tokens).into_result()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = r#"
            #list { background:WHITE }
            .mytype { bold:true }
            .content_item { background:WHITE }

            Flex(Horizontal,Start,Start,MainAxisFill) #list .mytype {
                Button { "OK" }
                Button { "Cancel" }
                Label { "Hello" }
                FlexItem(1.0) { Label {"Hi"} }
                width : 100
            }

            Grid(4,4) #contentList {
            }
        "#;

        let tokens = Token::lexer(input).collect::<Result<Vec<_>, _>>().unwrap();

        let result = parse( tokens.as_slice() );

        match result {
            Ok(nodes) => {
                println!("Parsed successfully!");
                for node in &nodes {
                    println!("{:#?}", node);
                }
            }
            Err(errors) => {
                println!("Parse errors:");
                for error in errors {
                    println!("{:?}", error);
                }
            }
        }
    }
}
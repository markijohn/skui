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
pub enum CssValue {
    #[token("auto", |_| CssKeyword::Auto)]
    #[token("none", |_| CssKeyword::None)]
    #[token("inherit", |_| CssKeyword::Inherit)]
    Keyword(CssKeyword),

    #[regex(r"[0-9]+(\.[0-9]+)?px", |lex| {
        let s = lex.slice();
        s[..s.len()-2].parse::<f64>().ok()
    })]
    Px(f64),

    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse().ok() )]
    Number(f64),

    #[regex(r"[0-9]+(\.[0-9]+)?%", |lex| {
        let s = lex.slice();
        s[..s.len()-1].parse::<f64>().ok()
    })]
    Percent(f64),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    Str(String),

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
    Str(String),

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
    pub properties: Vec<(String, Vec<CssValue>)>,
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



// Component 내부 아이템 (Property 또는 Component)
#[derive(Debug, Clone)]
enum ComponentItem {
    Text(String),
    Property(String, Value),
    Child(Component),
}

// CSS Value 파싱 함수
fn parse_css_values(input: &str) -> Vec<CssValue> {
    CssValue::lexer(input)
        .filter_map(|t| t.ok())
        .collect()
}

// 기본적으로 생성함수는 성공/실패를 가지며, 개념적으로 Option-like 하다
// select! 는 파서가 성공할때의 리턴값 정의 함수 생성을 의미
// separated_by 는 repeat 를 내포한다.
// then 이 map 과 다른점은 then 은 처리가 성공이 되어야 다음 체인으로 처리될 수 있음을 내포
// then_ignore ex:) a.then_ignore(b) a가 성공한 뒤, b가 성공하는지를 검사하고 그 입력은 소비하되 결과는 버리며, 최종적으로 a의 결과만 다음 체인으로 전달한다
// recurrsive 는 현재 생성함수를 내부에서 다시 호출할 수 있도록 하기 위함
// allow_trailing 은 separated_by 를 마지막에 한번 더 허용할 수 있다는 의미 ex:) 1,2,3,
// delimited_by 는 최초 그리고 루프 매번에 점검하는것이 아님. 소진이 완료되고 나서야 점검되는것임. separated_by와 같은 경우 종료가 명확하지만 repeat 의 경우는 종료 시점이 모호하므로 어디까지인지 알려야(then_ignore 와 같은) 함
// or 는 실패 시 입력 커서를 되돌리고 다음 파서를 시도하는것
// or_not 은 성공하던 실패하던 Option 으로 래핑해준다. 성공이면 Some
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
            Token::Str(s) => Value::String(s),
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

        let map = select! { Token::Ident(s) => s }
            .then_ignore(just(Token::Colon))
            .then(value.clone())
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .map(|pairs: Vec<(String, Value)>| {
                Value::Map(pairs.into_iter().collect())
            });

        choice((bool_val, number, string, ident, array, map))
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

    // Style: selector { prop:value value value; }
    // CSS value는 세미콜론/컴마까지의 모든 토큰을 수집
    let style_value = none_of([Token::Colon, Token::Semicolon, Token::Comma, Token::LBrace, Token::RBrace])
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|tokens: Vec<Token>| {
            // 토큰을 문자열로 변환
            let value_str = tokens.iter()
                .filter_map(|t| match t {
                    Token::Ident(s) => Some(s.clone()),
                    Token::True => Some("true".to_string()),
                    Token::False => Some("false".to_string()),
                    Token::Integer(i) => Some(i.to_string()),
                    Token::Float(f) => Some(f.to_string()),
                    Token::Str(s) => Some(format!("\"{}\"", s)),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ");
            parse_css_values(&value_str)
        });

    let style = selector
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .then(
            select! { Token::Ident(s) => s }
                .then_ignore(just(Token::Colon))
                .then(style_value)
                .separated_by(choice((just(Token::Comma), just(Token::Semicolon))))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .map(|(selectors, properties)| {
            AstNode::Style(Style { selectors, properties })
        });

    let component = recursive(|component| {
        let name = select! { Token::Ident(s) => s };
        let params = parameters
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .or_not();
        let id = select! { Token::Id(s) => s }.or_not();
        let classes = select! { Token::Class(s) => s }.repeated().collect::<Vec<_>>();

        // 3가지 아이템 타입
        let text_item = select! { Token::Str(s) => ComponentItem::Text(s) };
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
                (name, params, id, classes, items)
            })
            .map(|(name, params, id, classes, items)| {
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
            .content_item { background:WHITE; border:1px solid BLACK }

            Flex (Horizontal,Start,Start,MainAxisFill) #list .mytype {
                Button { "OK" }
                Button { "Cancel" }
                Label { "Hello" }
                FlexItem(1.0) { Label {"Hi"} }
                width : 100
                etc : {key:value, key2:value2}
            }

            Grid(4,4) #contentList {
            }
        "#;

        let tokens = Token::lexer(input).collect::<Result<Vec<_>, _>>().expect("Failed to tokenize input");

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
mod token;
mod cmt;
mod cursor;

use token::Token;
use cursor::TokenCursor;

use std::collections::HashMap;
use chumsky::Parser;

pub type Cursor<'a> = TokenCursor<'a,Token<'a>>;

#[derive(Debug, PartialEq, Clone)]
pub enum CssKeyword {
    Auto,
    None,
    Inherit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    Keyword(CssKeyword),
    Px(f64),
    Number(f64),
    Percent(f64),
    Ident(String),
    Str(String),
    HexColor(String),
    Rgba( (u8,u8,u8,u8) ),
    Rgb( (u8,u8,u8) ),
}

impl <'a> TryFrom<Token<'a>> for CssValue {
    type Error = ();
    fn try_from(tok: Token) -> Result<Self, Self::Error> {
        match tok {
            Token::Ident("auto") => Ok(CssValue::Keyword(CssKeyword::Auto)),
            Token::Ident("none") => Ok(CssValue::Keyword(CssKeyword::None)),
            Token::Ident("inherit") => Ok(CssValue::Keyword(CssKeyword::Inherit)),
            Token::Px(v) => Ok(CssValue::Px(v)),
            Token::Percent(v) => Ok(CssValue::Percent(v)),
            Token::Float(v) => Ok(CssValue::Number(v)),
            Token::Integer(v) => Ok(CssValue::Number(v as f64)),
            Token::Rgb(rgb) => Ok(CssValue::Rgb(rgb)),
            Token::Rgba(rgba) => Ok(CssValue::Rgba(rgba)),
            Token::Id(s) => Ok(CssValue::HexColor(s.to_string())),
            Token::Str(s) => Ok(CssValue::Str(s.to_string())),
            Token::Ident(s) => Ok(CssValue::Ident(s.to_string())),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    Id(String),
    Class(String),
    Tag(String),
}

#[derive(Debug, Clone)]
pub struct StyleProperty {
    pub key: String,
    pub values: Vec<CssValue>,
}

#[derive(Debug, Clone)]
pub struct Style {
    pub selector: Selector,
    pub properties: Vec<StyleProperty>,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub params: Vec<Value>,
    pub ids: Vec<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Ident(String),
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Closure(String),
}


fn optional_parse_style_properties(mut cursor: Cursor) -> Vec<StyleProperty> {
    let mut style_props = Vec::new();

    while !cursor.is_eof() {
        if let Some( key ) = cursor.take_if_map( | ts | {
            if let [Token::Ident(key), Token::Colon] = ts {
                Some( key )
            } else {
                None
            }
        }) {
            let css_val = cursor.take_collect_if( | t | CssValue::try_from(*t).ok() );
            style_props.push( StyleProperty { key:key.to_string(), values:css_val } );
        }
    }
    style_props
}

fn must_parse_style_item(cursor: &mut Cursor) -> Result<Style, String> {
    let selector = match cursor.take::<1>()[0] {
        Token::Id(s) => Selector::Id(s.to_string()),
        Token::Class(s) => Selector::Class(s.to_string()),
        Token::Ident(s) => Selector::Tag(s.to_string()),
        _ => return Err("Expected selector".to_string()),
    };
    if let Some(block_cursor)  = cursor.take_delimited( Token::block_brace() ) {
        let properties = optional_parse_style_properties(block_cursor);
        Ok(Style { selector, properties })
    } else {
        Err("Expected block brace".to_string())
    }
}

fn parse_style_block(cursor: &mut TokenCursor) -> Result<Style, String> {
    let mut items = Vec::new();

    while !cursor.is_eof() {
        if cursor.is_eof() {
            break;
        }

        // Selector { ... }
        items.push(parse_style_item(cursor)?);
    }

    Ok(Style { items })
}

// ==================== Value Parsing ====================
fn parse_value(cursor: &mut Cursor) -> Option<Value> {
    if let Some(block) = cursor.take_delimited( Token::block_brace() ) {

    } else {

    }
    let v = match cursor.take_one() {
        Token::Str(s) => Value::String(s.to_string()),
        Token::Ident(s) => Value::Ident(s.to_string()),
        Token::Integer(v) => Value::Number(Number::from(*v)),
        Token::Float(v) => Value::Number(Number::from(*v)),
        Token::True => Value::Bool(true),
        Token::False => Value::Bool(false),
        Token::Ident(s) => Value::Ident(s.to_string()),
        _ => return None
    };
    Some(v)
    match cursor.peek::<1>()[0] {
        Token::Str(s) => {
            let val = Value::String(s.to_string());
            cursor.take::<1>();
            Ok(val)
        }
        Token::Integer(n) => {
            let val = Value::Number(Number::I64(*n));
            cursor.take::<1>();
            Ok(val)
        }
        Token::Float(f) => {
            let val = Value::Number(Number::F64(*f));
            cursor.take::<1>();
            Ok(val)
        }
        Token::True => {
            cursor.take::<1>();
            Ok(Value::Bool(true))
        }
        Token::False => {
            cursor.take::<1>();
            Ok(Value::Bool(false))
        }
        Token::Ident(s) => {
            let val = Value::Ident(s.clone());
            cursor.take::<1>();
            Ok(val)
        }
        Token::LBracket => {
            let mut block = cursor.extract_brackets()?;
            let mut arr = Vec::new();

            while !block.is_eof() {
                if block.is_eof() {
                    break;
                }
                arr.push(parse_value(&mut block)?);
                if matches!(block.peek::<1>()[0], Token::Comma) {
                    block.take::<1>();
                }
            }

            Ok(Value::Array(arr))
        }
        Token::LBrace => {
            let mut block = cursor.extract_block()?;
            let mut map = HashMap::new();

            while !block.is_eof() {
                if block.is_eof() {
                    break;
                }

                if let [Token::Ident(key), Token::Colon] = block.peek() {
                    let key = key.clone();
                    block.take::<2>();
                    let val = parse_value(&mut block)?;
                    map.insert(key, val);

                    if matches!(block.peek::<1>()[0], Token::Comma) {
                        block.take::<1>();
                    }
                } else {
                    break;
                }
            }

            Ok(Value::Map(map))
        }
        _ => Err(format!("Unexpected token in value: {:?}", cursor.peek::<1>()[0])),
    }
}


macro_rules! taker {
    ( [ $($args:ty),* ] , $wrap:stmt ) => {
        |ts| {
            if let [ $($args),* ] = ts {
                Some( $wrap )
            } else {
                None
            }
        }
    }
}

fn parse_parameters(cursor: &mut Cursor) -> Option<Parameters> {
    let block_cursor = cursor.take_delimited( Token::block_paren() )?;

    while !cursor.is_eof() {
        if let [Token::Ident(_), Token::Equal] = cursor.peek() {

        }
        let value = parse_value(cursor);
        cursor.take_if( [Token::Comma] );

    }

    if let Some( (key,value)) = cursor.take_if_map( taker!( [Token::Ident( key ), Token::Equal, value], (key,value) ) ) {

    }
    // Map 형태 확인: Ident = Value
    if let [Token::Ident(_), Token::Equal] = cursor.peek() {
        let mut map = HashMap::new();

        loop {
            if cursor.is_eof() {
                break;
            }

            if let [Token::Ident(key), Token::Equal] = cursor.peek() {
                let key = key.clone();
                cursor.take::<2>();
                let val = parse_value(cursor)?;
                map.insert(key, val);

                if matches!(cursor.peek::<1>()[0], Token::Comma) {
                    cursor.take::<1>();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(Parameters::Map(map))
    } else {
        // Args
        let mut args = Vec::new();

        while !cursor.is_eof() {
            if cursor.is_eof() {
                break;
            }
            args.push(parse_value(cursor)?);
            if matches!(cursor.peek::<1>()[0], Token::Comma) {
                cursor.take::<1>();
            } else {
                break;
            }
        }

        Ok(Parameters::Args(args))
    }
}

fn parse_component(cursor: &mut Cursor) -> Result<Component, String> {
    match cursor.peek_one() {
        Token::Ident("style") => {
            parse_style_block()
        }
        Token::Id(id) => {

        }
        Token::Class(class) => {

        }
        Token::Ident(widget) => {

        }
        @unk _  => {
            return Err(format!("Unexpected token in component: {:?}", unk);
        }
    }


    // Name
    let name = match cursor.take::<1>()[0] {
        Token::Ident(s) => s.clone(),
        _ => return Err("Expected component name".to_string()),
    };

    // Parameters (연속 가능)
    let mut params = Vec::new();
    while matches!(cursor.peek::<1>()[0], Token::LParen) {
        let mut param_tokens = cursor.extract_parens()?;
        params.push(parse_parameters(&mut param_tokens)?);
    }

    // IDs
    let mut ids = Vec::new();
    while matches!(cursor.peek::<1>()[0], Token::Id(_)) {
        if let Token::Id(id) = cursor.take::<1>()[0] {
            ids.push(id.clone());
        }
    }

    // Classes
    let mut classes = Vec::new();
    while matches!(cursor.peek::<1>()[0], Token::Class(_)) {
        if let Token::Class(cls) = cursor.take::<1>()[0] {
            classes.push(cls.clone());
        }
    }

    // Body
    let mut text = None;
    let mut properties = HashMap::new();
    let mut children = Vec::new();

    if matches!(cursor.peek::<1>()[0], Token::LBrace) {
        let mut body = cursor.extract_block()?;

        while !body.is_eof() {
            if body.is_eof() {
                break;
            }

            match body.peek::<3>() {
                // "text"
                [Token::Str(s), ..] => {
                    text = Some(s.clone());
                    body.take::<1>();
                }
                // property: value
                [Token::Ident(key), Token::Colon, ..] => {
                    let key = key.clone();
                    body.take::<2>(); // ident, colon
                    let val = parse_value(&mut body)?;
                    properties.insert(key, val);
                }
                // { children... }
                [Token::LBrace, ..] => {
                    let mut children_block = body.extract_block()?;

                    while !children_block.is_eof() {
                        if children_block.is_eof() {
                            break;
                        }
                        children.push(parse_component(&mut children_block)?);
                    }
                }
                // Component
                [Token::Ident(_), ..] => {
                    children.push(parse_component(&mut body)?);
                }
                _ => {
                    body.take::<1>();
                }
            }
        }
    }

    Ok(Component {
        name,
        params,
        ids,
        classes,
        text,
        children,
        properties,
    })
}

// ==================== Main Parse ====================
pub fn parse(input: &str) -> Result<Vec<AstNode>, String> {
    let tokens: Vec<Token> = Token::lexer(input)
        .filter_map(|t| t.ok())
        .collect();

    let mut cursor = TokenCursor::new(&tokens);
    let mut nodes = Vec::new();

    while !cursor.is_eof() {
        // style { ... } 또는 #id { ... } 또는 .class { ... }
        match cursor.peek::<3>() {
            [Token::Ident(name), ..] if name == "style" => {
                cursor.take::<1>();
                let mut block = cursor.extract_block()?;
                let style = parse_style_block(&mut block)?;
                nodes.push(AstNode::Style(style));
            }
            [Token::Id(_) | Token::Class(_), ..] => {
                let mut item_cursor = cursor.clone();
                let item = parse_style_item(&mut item_cursor)?;
                cursor.idx = item_cursor.idx;
                nodes.push(AstNode::Style(Style { items: vec![item] }));
            }
            [Token::Ident(_), ..] => {
                let component = parse_component(&mut cursor)?;
                nodes.push(AstNode::Component(component));
            }
            _ => {
                cursor.take::<1>();
            }
        }
    }

    Ok(nodes)
}

// ==================== Tests ====================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = r#"
            style {
                Flex { background-color: black; padding:1px }
                #list { border: 1px solid black }
                .myBtn { border: 2px }
            }

            #myFlex { border:2px }
            .background_white { background-color: WHITE }

            Flex(MainFill) #myFlex .background_white {
                "Text"
                myProperty1 : "data"
                propertyMap : {key:1, key2: true}
                {
                    FlexItem(1.0) {
                    }
                    Button {
                    }
                }
                propertyAnother : [ 1,2,3 ]
            }

            Grid(2,3) {
                Label {
                }
            }
        "#;

        match parse(input) {
            Ok(nodes) => {
                println!("Parsed successfully!");
                for node in &nodes {
                    println!("{:#?}", node);
                }
                assert!(nodes.len() > 0);
            }
            Err(e) => {
                println!("Parse error: {}", e);
                panic!("Parse failed");
            }
        }
    }
}
mod token;
mod cursor;

use token::Token;
use cursor::TokenCursor;

use std::collections::HashMap;
use logos::{Logos, Span};
use thiserror::Error;

pub type Cursor<'a> = TokenCursor<'a,Token<'a>>;

pub type Result<T,E=ParseError> = std::result::Result<T, E>;

pub type CursorResult<'a, T> = std::result::Result<(Cursor<'a>,T), ParseError>;

#[derive(Debug)]
pub struct ParseError {
    token_idx: usize,
    kind: ParseErrorKind,
}

impl ParseError {

    pub fn expect_ident(cursor: &Cursor) -> Self {
        Self { token_idx:cursor.idx(), kind:ParseErrorKind::ExpectIdent }
    }

    pub fn expect_value(cursor: &Cursor) -> Self {
        Self { token_idx:cursor.idx(), kind:ParseErrorKind::ExpectValue }
    }

    pub fn invalid_css_value(idx:usize) -> Self {
        Self { token_idx: idx, kind:ParseErrorKind::InvalidCssValue }
    }

    pub fn not_selector(cursor: &Cursor) -> Self {
        Self { token_idx:cursor.idx(), kind:ParseErrorKind::InvalidCssSelector }
    }

    pub fn expect_kv(cursor: &Cursor) -> Self {
        Self { token_idx:cursor.idx(), kind:ParseErrorKind::ExpectKeyValue }
    }

    pub fn not_parameter(cursor: &Cursor) -> Self {
        Self { token_idx:cursor.idx(), kind:ParseErrorKind::ExpectParameter }
    }

    pub fn expect_brace_block(cursor: &Cursor) -> Self {
        Self { token_idx:cursor.idx(), kind:ParseErrorKind::ExpectBraceBlock }
    }

    pub fn expect_parent_block(cursor: &Cursor) -> Self {
        Self { token_idx:cursor.idx(), kind:ParseErrorKind::ExpectParentBlock }
    }

    pub fn unknown_start(cursor:&Cursor) -> Self {
        Self { token_idx:cursor.idx(), kind:ParseErrorKind::UnknownStart }
    }
}


#[derive(Clone, Debug, Error)]
pub enum ParseErrorKind {
    #[error("expected an identifier. e.g. name, button, flex")]
    ExpectIdent,

    #[error("expected a value. e.g. myident, Component(), 123, 123.456, \"mytext..\", [4,5,], {{key=value}}, true, false, #ff0000")]
    ExpectValue,

    #[error("invalid CSS value. e.g. 123px, 1.0em, 123.456, \"mytext..\", true, false, #ff0000")]
    InvalidCssValue,

    #[error("invalid CSS selector. e.g. #myid, .myclass, TagName")]
    InvalidCssSelector,

    #[error("expected a key-value pair. e.g. key=value,key2=value2")]
    ExpectKeyValue,

    #[error("expected a parameter. e.g. (param1,2,\"text\"), (key=1,key2)")]
    ExpectParameter,

    #[error("expected a brace block. e.g. '{{ ... }}'")]
    ExpectBraceBlock,

    #[error("expected a parent block. e.g. '( .. )'")]
    ExpectParentBlock,

    #[error("unexpected start of statement. style {{ }}, #id {{ }}, .class {{ }}, Component()")]
    UnknownStart,
}

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

impl <'a> TryFrom< (usize, Token<'a>) > for CssValue {
    type Error = ParseError;
    fn try_from( (idx,tok):(usize, Token<'a>) ) -> Result<Self> {
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
            _ => Err( ParseError::invalid_css_value(idx) ),
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
    pub selector: Vec<Selector>,
    pub properties: Vec<StyleProperty>,
}

#[derive(Debug, Clone)]
pub enum Parameters {
    Map(HashMap<String,Value>),
    Args(Vec<Value>),
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub params: Parameters,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub children: Vec<Component>,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub styles: Vec<Style>,
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    I64(i64),
    F64(f64),
}

#[derive(Debug, Clone)]
pub enum Value {
    Ident(String),
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Closure(String),
    Component(Component),
}


fn parse_style_nested_properties(mut cursor: Cursor) -> Result< Vec<StyleProperty> > {
    let mut style_props = Vec::new();

    while !cursor.is_eof() {
        if let [Token::Ident(key), Token::Colon] = cursor.take() {
            let idx = cursor.idx();
            let css_val = cursor.take_map_until( | t | CssValue::try_from( (idx,t) ).ok() );
            style_props.push( StyleProperty { key:key.to_string(), values:css_val } );
        } else {
            return Err(ParseError::expect_ident(&cursor))
        }
        let _ = cursor.take_ignore( [Token::Semicolon] );
    }
    Ok( style_props )
}

fn parse_def_selector(mut cursor:Cursor) -> CursorResult<Selector> {
    let v = take_if!(cursor,
        [Token::Id(s)] => Selector::Id(s.to_string()),
        [Token::Class(s)] => Selector::Class(s.to_string()),
        [Token::Ident(s)] => Selector::Tag(s.to_string()),
        _ => return Err(ParseError::not_selector( &cursor ))
    );
    cursor.ok_with(v)
}

fn parse_def_selectors(mut cursor:Cursor) -> CursorResult<Vec<Selector>> {
    let mut selectors = Vec::new();
    while !cursor.is_eof() && cursor.peek_one() != Token::LBrace {
        let (next_cursor,selector) = parse_def_selector(cursor.fork())?;
        cursor = next_cursor;
        selectors.push( selector );
    }
    cursor.ok_with(selectors)
}

fn parse_style_item(mut cursor:Cursor) -> CursorResult<Style> {
    let (mut cursor,selector) = parse_def_selectors(cursor.fork())?;
    let block = cursor.take_if_delimited( Token::block_brace() ).ok_or_else(|| ParseError::expect_brace_block(&cursor))?;
    let properties = parse_style_nested_properties( block )?;
    cursor.ok_with( Style { selector, properties })
}

fn parse_style_block(mut cursor:Cursor) -> CursorResult<Vec<Style>> {
    let mut items = Vec::new();
    if let [Token::Ident("style")] = cursor.take() {
        let mut block = cursor.take_if_delimited( Token::block_brace() ).ok_or_else(|| ParseError::expect_brace_block(&cursor))?;
        while !block.is_eof() {
            let (next, style_item) = parse_style_item(block)?;
            block = next;
            items.push( style_item );
        }
        cursor.ok_with(items)
    } else {
        Err(ParseError::expect_ident(&cursor))
    }
}

fn parse_nested_map(mut cursor:Cursor) -> Result<HashMap<String, Value>> {
    let mut map = HashMap::new();
    while !cursor.is_eof() {
        if let [Token::Ident(key), Token::Equal] = cursor.take() {
            let (next_cursor,value) = parse_value(cursor.fork())?;
            cursor = next_cursor;
            map.insert(key.to_string(), value);
            let _ = cursor.take_ignore( [Token::Comma] );
        } else {
            return Err(ParseError::expect_kv(&cursor));
        }
    }
    Ok(map)
}

fn parse_nested_array(mut cursor:Cursor) -> Result<Vec<Value>> {
    let mut values = vec![];
    while !cursor.is_eof() {
        let (next_cursor, value) = parse_value(cursor)?;
        cursor = next_cursor;
        values.push( value );
        let _ = cursor.take_ignore( [Token::Comma] );
    }
    Ok(values)
}


fn parse_value(mut cursor:Cursor) -> CursorResult<Value> {

    let (cursor,value) = if let Ok( (cursor, comp) ) = parse_component(cursor.fork()) {
        (cursor, Value::Component(comp))
    } else if let Some(mut block) = cursor.take_if_delimited(Token::block_brace()) {
        let map = parse_nested_map(block)?;
        (cursor, Value::Map( map ))
    } else if let Some(mut block) = cursor.take_if_delimited( Token::block_bracket() ) {
        let arr = parse_nested_array(block)?;
        (cursor, Value::Array( arr ))
    }
    else {
        let v = match cursor.take_one() {
            Token::Str(s) => Value::String(s.to_string()),
            Token::Ident(s) => Value::Ident(s.to_string()),
            Token::Integer(v) => Value::Number(Number::I64(v)),
            Token::Float(v) => Value::Number(Number::F64(v)),
            Token::True => Value::Bool(true),
            Token::False => Value::Bool(false),
            Token::Ident(s) => Value::Ident(s.to_string()),
            _ => return Err(ParseError::expect_value(&cursor))
        };
        (cursor, v)
    };
    cursor.ok_with(value)
}


fn parse_nested_parameters(mut cursor:Cursor) -> Result<Parameters> {
    if cursor.is_eof() {
        Ok( Parameters::Args( Vec::new() ) )
    } else if let Ok( map ) = parse_nested_map(cursor.fork()) {
        Ok( Parameters::Map(map) )
    } else if let Ok( arr ) = parse_nested_array(cursor.fork()) {
        Ok( Parameters::Args( arr ) )
    }
    // else if let Ok( (_, comp) ) = parse_component(cursor.fork()) {
    //     Ok( Parameters::Args( vec![Value::Component(comp)] ) )
    // }
    else {
        Err( ParseError::not_parameter(&cursor) )
    }
}

fn parse_component(mut cursor:Cursor) -> CursorResult<Component> {
    let Token::Ident(name) = cursor.take_one() else { return Err(ParseError::expect_ident(&cursor)) };
    let name = name.to_string();

    let param_block = cursor.take_if_delimited( Token::block_paren() )
        .ok_or_else(|| ParseError::expect_parent_block(&cursor))?;
    let params = parse_nested_parameters(param_block)?;

    let selectors = cursor.take_map_until( |t| {
        match t {
            Token::Id(id) => Some( Selector::Id(id.to_string()) ),
            Token::Class(cls) => Some( Selector::Class(cls.to_string()) ),
            _ => None
        }
    });

    let mut id = None;
    let mut classes = vec![];
    for s in selectors.into_iter() {
        match s {
            Selector::Id(identify) => id = Some(identify),
            Selector::Class(cls) => classes.push(cls),
            _ => unreachable!()
        }
    }

    let mut properties = HashMap::new();
    let mut children = Vec::new();
    if let Some(mut block) = cursor.take_if_delimited(Token::block_brace()) {
        while !block.is_eof() {
            if let Some(mut child_block) = block.take_if_delimited(Token::block_brace()) {
                while !child_block.is_eof() {
                    let (next, child) = parse_component(child_block)?;
                    children.push( child );
                    child_block = next;
                }
            } else if let [Token::Ident(key), Token::Colon] = block.take() {
                let (next, value) = parse_value(block)?;
                block = next;
                properties.insert( key.to_string(), value );
            } else {
                return Err(ParseError::expect_brace_block(&block));
            }
        }
    }

    cursor.ok_with(Component {
        name,
        params,
        id,
        classes,
        children,
        properties,
    })
}

fn parse_tokens( tokens: &[Token], spans:&[Span] ) -> Result<ParsedDocument> {
    let mut cursor = Cursor::new(&tokens);
    let mut styles = vec![];
    let mut components = vec![];

    while !cursor.is_eof() {
        if let [Token::Ident("style")] = cursor.peek() {
            let (next, style_block) = parse_style_block(cursor.fork())?;
            cursor = next;
            styles.extend(style_block);
            continue;
        }

        if let [Token::Id(_id)] = cursor.peek() {
            let (next, style) = parse_style_item(cursor.fork())?;
            cursor = next;
            styles.push(style);
            continue;
        }

        if let [Token::Class(_cls)] = cursor.peek() {
            let (next, style) = parse_style_item(cursor.fork())?;
            cursor = next;
            styles.push(style);
            continue;
        }

        let (next, component) = parse_component(cursor)?;
        cursor = next;
        components.push(component);
    }

    Ok( ParsedDocument { styles, components } )
}

#[derive(Debug)]
pub struct ParseDetailError {
    pub kind: ParseError,
    pub span: Span,
}

pub fn parse(input: &str) -> Result<ParsedDocument,ParseDetailError> {
    let spanned:Vec<(Token,Span)> = Token::lexer(input)
        .spanned()
        .filter_map(| (t,s) | t.map( |v| (v,s) ).ok() )
        .collect::<Vec<_>>();

    let (tokens, spans):(Vec<Token>, Vec<Span>) = spanned.into_iter().unzip();

    match parse_tokens(&tokens, &spans) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            Err( ParseDetailError {
                span : spans[e.token_idx].clone(),
                kind : e,
            })
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = r#"
            style {
                Flex { background-color: black; padding:1px }
                #list { border: 1px solid yellow }
                .myBtn { border: 2px }
            }

            #myFlex { border:2px }

            .background_white { background-color: WHITE }

            Flex(MainFill) #myFlex .background_white {
                myProperty1 : "data"
                propertyMap : {key=1, key2=true}
                {
                    FlexItem(1.0) {  { Button("FlexItem") }  }
                    FlexItem(2.0, Button("FlexItem2"))
                    Button() { }
                }
                propertyAnother : [ 1,2,3 ]
            }

            Grid(2,3) {
                {
                    Label() {
                    }
                }
            }
        "#;

        match parse(input) {
            Ok(parsed) => {
                println!("Parsed successfully!");

                for style in parsed.styles.iter() {
                    println!("{:#?}", style);
                }
                for comp in parsed.components.iter() {
                    println!("{:#?}", comp);
                }
            }
            Err(e) => {
                println!("Parse error: {:?}", e);
                panic!("Cause : {}", &input[e.span.start .. e.span.end]);
            }
        }
    }

    #[test]
    fn narr() {
        let token = vec![ Token::Ident("MainFill") ];
        let cursor = Cursor::new(&token);
        println!("{:?}", parse_nested_array(cursor).unwrap());
    }
}
mod token;
mod cmt;
mod cursor;

use token::Token;
use cursor::TokenCursor;

use std::collections::HashMap;
use chumsky::Parser;
use logos::{Logos, Span};
use crate::cursor::CursorMark;

pub type Cursor<'a> = TokenCursor<'a,Token<'a>>;

pub type Result<'a,T> = std::result::Result<T, ParseError>;

pub struct ParseError {
    idx: usize,
    kind: ParseErrorKind,
}

impl ParseError {

    pub fn expect_ident(cursor: &Cursor) -> Self {
        Self { token_info:cursor.peek_one().to_info(), kind:ParseErrorKind::ExpectIdent }
    }

    pub fn expect_value(mark:Option<CursorMark>, cursor: &Cursor<'a>) -> Self {
        Self { mark, cursor, kind:ParseErrorKind::ExpectValue }
    }

    pub fn invalid_css_value(mark:Option<CursorMark>, cursor: &Cursor<'a>) -> Self {
        Self { mark, cursor, kind:ParseErrorKind::InvalidCssValue }
    }

    pub fn not_selector(mark:Option<CursorMark>, cursor: &Cursor<'a>) -> Self {
        Self { mark, cursor, kind:ParseErrorKind::InvalidCssSelector }
    }

    pub fn expect_kv(mark:Option<CursorMark>, cursor: &Cursor<'a>) -> Self {
        Self { mark, cursor, kind:ParseErrorKind::ExpectKeyValue }
    }

    pub fn not_parameter(mark:Option<CursorMark>, cursor: &Cursor<'a>) -> Self {
        Self { mark, cursor, kind:ParseErrorKind::ExpectParameter }
    }

    pub fn expect_brace_block(mark:Option<CursorMark>, cursor: &Cursor<'a>) -> Self {
        Self { mark, cursor, kind:ParseErrorKind::ExpectBraceBlock }
    }

    pub fn unknown_start(mark:Option<CursorMark>, cursor:&Cursor<'a>) -> Self {
        Self { mark, cursor, kind:ParseErrorKind::ExpectParameter }
    }
}

pub enum ParseErrorKind {
    ExpectIdent,
    ExpectValue,
    InvalidCssValue,
    InvalidCssSelector,
    ExpectKeyValue,
    ExpectParameter,
    ExpectBraceBlock,
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

impl <'a> TryFrom< (CursorMark, &'a Cursor, Token) > for CssValue {
    type Error = ParseError<'a>;
    fn try_from( (mark,cursor,tok):(CursorMark, &'a Cursor, Token) ) -> Result<Self> {
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
            _ => Err( ParseError::invalid_css_value(Some(mark), cursor) ),
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
    Args(Vec<Value>)
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub params: Parameters,
    pub ids: Vec<String>,
    pub classes: Vec<String>,
    pub text: Option<String>,
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


fn parse_style_properties(mut cursor: Cursor) -> Result<Vec<StyleProperty>> {
    let mark = Some(cursor.mark());
    let mut style_props = Vec::new();

    while !cursor.is_eof() {
        take_if!(cursor,
            [Token::Ident(key), Token::Colon] => {
                let css_val = cursor.take_map_until( | t | CssValue::try_from(*t).ok() );
                style_props.push( StyleProperty { key:key.to_string(), values:css_val } );
            }
            _ => return Err(ParseError::expect_ident(mark, &cursor))
        );

        let _ = cursor.take_ignore( [Token::Semicolon] );
    }
    Ok( style_props )
}

fn parse_def_selector(cursor:&mut Cursor) -> Result<Selector> {
    let mark = Some(cursor.mark());
    take_if!(cursor,
        [Token::Id(s)] => Selector::Id(s.to_string()),
        [Token::Class(s)] => Selector::Class(s.to_string()),
        [Token::Ident(s)] => Selector::Tag(s.to_string()),
        _ => return Err(ParseError::not_selector(mark, cursor))
    );
}

fn parse_def_selectors(cursor:&mut Cursor) -> Result<Vec<Selector>> {
    let mut selectors = Vec::new();
    while !cursor.is_eof() || cursor.peek_one() != Token::LBrace {
        selectors.push( parse_def_selector(cursor)? );
    }
    Ok(selectors)
}

fn parse_style_item(cursor: &mut Cursor) -> Result<Style> {
    let mark = Some(cursor.mark());
    let selector = parse_def_selectors(cursor)?;
    let block = cursor.take_delimited( Token::block_brace() ).map_err(|_| ParseError::expect_brace_block(mark,cursor))?;
    let properties = parse_style_properties( block )?;
    Ok(Style { selector, properties })
}

fn parse_style_block(cursor: &mut Cursor) -> Result<Vec<Style>> {
    let mut items = Vec::new();
    if let Token::Ident("style") = cursor.take_one() {
        let mark = Some(cursor.mark());
        let mut block = cursor.take_delimited( Token::block_brace() ).map_err(|_| ParseError::expect_brace_block(mark,cursor))?;
        while !block.is_eof() {
            items.push(parse_style_item(&mut block)?);
        }
    }
    Ok(items)
}

fn parse_nested_map(block: &mut Cursor) -> Result<HashMap<String, Value>> {
    let mut map = HashMap::new();
    while !block.is_eof() {
        let mark = Some(block.mark());
        if let [Token::Ident(key), Token::Equal] = block.take() {
            map.insert(key.to_string(), parse_value(block)?);
        } else {
            return Err(ParseError::expect_kv(mark,block));
        }
    }
    Ok(map)
}

fn parse_nested_array(block: &mut Cursor) -> Result<Vec<Value>> {
    let mut values = vec![];
    while !block.is_eof() {
        values.push( parse_value(block)? );
        let _ = block.take_ignore( [Token::Comma] );
    }
    Ok(values)
}


fn parse_value(cursor: &mut Cursor) -> Result<Value> {
    let v = if let Some(mut block) = cursor.take_delimited(Token::block_brace()) {
        Value::Map( parse_nested_map(&mut block)? )
    } else if let Some(mut block) = cursor.take_delimited( Token::block_paren() ) {
        Value::Array( parse_nested_array(&mut block)? )
    } else {
        let mark = Some(cursor.mark());
        match cursor.take_one() {
            Token::Str(s) => Value::String(s.to_string()),
            Token::Ident(s) => Value::Ident(s.to_string()),
            Token::Integer(v) => Value::Number(Number::from(*v)),
            Token::Float(v) => Value::Number(Number::from(*v)),
            Token::True => Value::Bool(true),
            Token::False => Value::Bool(false),
            Token::Ident(s) => Value::Ident(s.to_string()),
            _ => return Err(ParseError::expect_value(mark,cursor))
        }
    };
    Ok(v)
}


fn parse_parameters(cursor: &mut Cursor) -> Result<Parameters> {
    let mark = Some(cursor.mark());
    if let Ok(map) = parse_nested_map(cursor).err_rollback() {
        Ok( Parameters::Map( map ) )
    } else if let Ok(arr) = parse_nested_array(cursor).err_rollback() {
        Ok( Parameters::Args( arr ) )
    } else {
        Err( ParseError::not_parameter(mark,cursor) )
    }
}

fn parse_component(cursor: &mut Cursor) -> Result<Component> {
    let mark = Some(cursor.mark());
    let Token::Ident(name) = cursor.take_one() else { return Err(ParseError::expect_ident(mark,cursor)) };

    let parameter = parse_parameters(cursor).err_rollback().ok();

    let selectors = cursor.take_map_until( |t| {
        match t {
            Token::Id(id) => Some( Selector::Id(id.to_string()) ),
            Token::Class(cls) => Some( Selector::Class(cls.to_string()) ),
            _ => None
        }
    });

    let mut id = None;
    let mut classes = vec![];
    while !cursor.is_eof() {
        take_if!(cursor,
            [Token::Id(identify)] => id = Some(identify.to_string()),
            [Token::Class(class)] => classes.push( class.to_string() ),
            _ => break
        );
    }

    if let Some(body) = cursor.take_delimited(Token::block_brace()) {
        // Body
        let mut text = None;
        let mut properties = HashMap::new();
        let mut children = Vec::new();


        while !body.is_eof() {
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

pub fn parse(input: &str) -> Result<ParsedDocument> {
    let spanned:Vec<(Token,Span)> = Token::lexer(input).spanned().filter_map(| (t,s) | t.map( |v| (v,s) ).ok() ).collect::<Vec<_>>();

    let (tokens, spans):(Vec<Token>, Vec<Span>) = spanned.into_iter().unzip();
    // let tokens: Vec<Token> = Token::lexer(input)
    //     .filter_map(|t| t.ok())
    //     .collect();

    let mut cursor = Cursor::new(&tokens);
    let mut nodes = Vec::new();

    let mut styles = vec![];
    let mut components = vec![];

    let mark = cursor.mark();
    while !cursor.is_eof() {
        if let Ok(style_block) = parse_style_block(&mut cursor).err_rollback() {
            styles.extend(style_block);
            continue;
        }

        if let Ok(style_item) = parse_style_item(&mut cursor).err_rollback() {
            styles.push(style_item);
            continue;
        }

        if let Ok(component) = parse_component(&mut cursor).err_rollback() {
            components.push(component);
            continue;
        }

        return Err(ParseError::unknown_start(Some(mark), &cursor))
    }

    Ok( ParsedDocument { styles, components } )
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
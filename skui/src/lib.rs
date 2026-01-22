mod token;
mod value;
mod params;
mod cursor;

use token::Token;
use cursor::TokenCursor;

use std::collections::HashMap;
use logos::{Logos, Span};
use thiserror::Error;
use crate::cursor::{CursorSpan, SplitCursor};

pub use value::*;
pub use params::*;

pub type Cursor<'a> = TokenCursor<'a,Token<'a>>;

pub type Result<T,E=ParseError> = std::result::Result<T, E>;

pub type CursorResult<'a, T> = std::result::Result<(Cursor<'a>,T), ParseError>;

#[derive(Debug,Clone)]
pub struct ParseError {
    span: CursorSpan,
    kind: ParseErrorKind,
}

impl ParseError {

    pub fn expect_ident(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::ExpectIdent }
    }

    pub fn expect_value(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::ExpectValue }
    }

    pub fn invalid_css_value(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::InvalidCssValue }
    }

    pub fn not_selector(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::InvalidCssSelector }
    }

    pub fn expect_kv(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::ExpectKeyValue }
    }

    pub fn not_parameter(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::ExpectParameter }
    }

    pub fn expect_brace_block(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::ExpectBraceBlock }
    }

    pub fn expect_parent_block(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::ExpectParentBlock }
    }

    pub fn unknown_start(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::UnknownStart }
    }

    pub fn id_already_defined(span: CursorSpan) -> Self {
        Self { span, kind:ParseErrorKind::IdAlreadyDefined }
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

    #[error("id already defined")]
    IdAlreadyDefined
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

impl <'a> TryFrom< (CursorSpan, Token<'a>) > for CssValue {
    type Error = ParseError;
    fn try_from( (span,tok):(CursorSpan, Token<'a>) ) -> Result<Self> {
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
            _ => Err( ParseError::invalid_css_value(span) ),
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
pub struct Component {
    pub name: String,
    pub params: Parameters,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub children: Vec<Component>,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct SKUI {
    pub styles: Vec<Style>,
    pub components: Vec<Component>,
    spans : Vec<Span>,
}

impl SKUI {
    pub fn parse(input: &str) -> Result<Self, SKUIParseError> {
        parse(input).map_err(|e| SKUIParseError { span: e.span, kind: e.kind })
    }
}


fn parse_style_inner_properties(cursor: Cursor) -> Result< Vec<StyleProperty> > {
    let (_cursor, styles) = cursor.consume_collect_until( |c| {
        let c = c.ignore_until( |t| t != Token::Semicolon );
        let span = c.span();
        if c.is_eof() {
            Ok( (c,None) )
        } else if let (mut new_cursor,[Token::Ident(key), Token::Colon]) = c.fork().consume() {
            let css_val;
            (new_cursor,css_val) = new_cursor.consume_collect_until( |c| {
                let span = c.span();
                let (n,t) = c.consume_one();
                Ok( (n,CssValue::try_from( (span,t) ).ok()) )
            } )?;
            let style_property = StyleProperty { key: key.to_string(), values: css_val };
            Ok( (new_cursor,Some(style_property)) )
        } else {
            Err(ParseError::expect_ident(span))
        }
    } )?;
    Ok( styles )
}

fn parse_def_selectors(cursor:Cursor) -> CursorResult<Vec<Selector>> {
    let Some(SplitCursor{next, result}) = cursor.fork().split_until( |t| t == Token::LBrace )
    else { return Err(ParseError::expect_brace_block(cursor.span())) };
    let (_,selectors) = result.consume_collect_until( |mut c| {
        let v = take_match!(c,
            [Token::Id(s)] => Selector::Id(s.to_string()),
            [Token::Class(s)] => Selector::Class(s.to_string()),
            [Token::Ident(s)] => Selector::Tag(s.to_string()),
            _ => return Ok( (c,None) )
        );
        Ok( (c,Some(v)) )
    })?;
    Ok( (next,selectors) )
}

fn parse_style_item(cursor:Cursor) -> CursorResult<Style> {
    let (cursor,selector) = parse_def_selectors(cursor)?;
    let span = cursor.span();
    let SplitCursor{next:cursor, result:block} = cursor.consume_delimited_inner( Token::block_brace() ).ok_or_else(|| ParseError::expect_brace_block(span))?;
    let properties = parse_style_inner_properties( block )?;
    cursor.ok_with( Style { selector, properties })
}

fn parse_inner_map(mut cursor:Cursor) -> Result<HashMap<String, Value>> {
    let mut map = HashMap::new();
    while !cursor.is_eof() {
        let span = cursor.span();
        if let (next_cursor, [Token::Ident(key), Token::Equal]) = cursor.consume() {
            cursor = next_cursor;
            let value;
            (cursor,value) = parse_value(cursor.fork())?;
            map.insert(key.to_string(), value);
            //TODO : check flag?
            (cursor,_) = cursor.ignore( [Token::Comma] );
        } else {
            return Err(ParseError::expect_kv(span));
        }
    }
    Ok(map)
}

fn parse_inner_array(mut cursor:Cursor) -> Result<Vec<Value>> {
    let mut values = vec![];
    while !cursor.is_eof() {
        let (next_cursor, value) = parse_value(cursor)?;
        cursor = next_cursor;
        values.push( value );
        (cursor,_) = cursor.ignore( [Token::Comma] );
    }
    Ok(values)
}


fn parse_value(cursor:Cursor) -> CursorResult<Value> {
    let (cursor,value) = if let Ok( (cursor, comp) ) = parse_component(cursor.fork()) {
        (cursor, Value::Component(comp))
    } else if let Some( SplitCursor{next:cursor,result:block} ) = cursor.fork().consume_delimited_inner(Token::block_brace()) {
        let map = parse_inner_map(block)?;
        (cursor, Value::Map( map ))
    } else if let Some( SplitCursor{next:cursor,result:block} ) = cursor.fork().consume_delimited_inner( Token::block_bracket() ) {
        let arr = parse_inner_array(block)?;
        (cursor, Value::Array( arr ))
    }
    else {
        let span = cursor.span();
        let (cursor,value) = cursor.consume_one();
        let v = match value {
            Token::Str(s) => Value::String(s.to_string()),
            Token::Ident(s) => Value::Ident(s.to_string()),
            Token::Integer(v) => Value::Number(Number::I64(v)),
            Token::Float(v) => Value::Number(Number::F64(v)),
            Token::True => Value::Bool(true),
            Token::False => Value::Bool(false),
            _ => return Err(ParseError::expect_value(span))
        };
        (cursor, v)
    };
    cursor.ok_with(value)
}


fn parse_inner_parameters(cursor:Cursor) -> Result<Parameters> {
    if cursor.is_eof() {
        Ok( Parameters::Args( Vec::new() ) )
    } else if let Ok( map ) = parse_inner_map(cursor.fork()) {
        Ok( Parameters::Map(map) )
    } else if let Ok( arr ) = parse_inner_array(cursor.fork()) {
        Ok( Parameters::Args( arr ) )
    } else {
        Err( ParseError::not_parameter( cursor.span() ) )
    }
}

fn parse_component(cursor:Cursor) -> CursorResult<Component> {
    let span = cursor.span();
    let (cursor, Token::Ident(name)) = cursor.consume_one()
    else { return Err(ParseError::expect_ident(span)) };

    let name = name.to_string();

    let Some( SplitCursor{next:cursor,result:param_block} ) = cursor.fork().consume_delimited_inner( Token::block_paren() )
    else { return Err(ParseError::expect_parent_block(cursor.span())) };
    let params = parse_inner_parameters(param_block)?;

    let span = cursor.span();
    let (mut cursor,selectors) = cursor.consume_collect_until( |cursor| {
        let (c, token) = cursor.fork().consume_one();
        match token {
            Token::Id(id) => Ok( (c,Some( Selector::Id(id.to_string()) ) ) ),
            Token::Class(cls) => Ok( (c,Some( Selector::Class(cls.to_string()) ) ) ),
            _ => Ok( (cursor,None) )
        }
    })?;

    let mut id = None;
    let mut classes = vec![];
    for s in selectors.into_iter() {
        match s {
            Selector::Id(identify) => {
                if id.is_some() {
                    return Err(ParseError::id_already_defined(span))
                }
                id = Some(identify)
            },
            Selector::Class(cls) => {
                classes.push(cls)
            },
            _ => unreachable!()
        }
    }
    classes.dedup();

    let mut properties = HashMap::new();
    let mut children = Vec::new();
    if let Some( SplitCursor{next,result:mut comp_block} ) = cursor.fork().consume_delimited_inner(Token::block_brace()) {
        cursor = next;
        while !comp_block.is_eof() {
            let span = comp_block.span();
            //Try child component block
            if let (_,[Token::Ident(key), Token::LParen]) = comp_block.fork().consume() {
                let child;
                (comp_block, child) = parse_component(comp_block)?;
                children.push( child );
            }
            //Try property
            else if let (next,[Token::Ident(key), Token::Colon]) = comp_block.fork().consume() {
                comp_block = next;
                let value;
                (comp_block, value) = parse_value(comp_block)?;
                properties.insert( key.to_string(), value );
            } else {
                return Err(ParseError::expect_brace_block(span));
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

fn parse_tokens( tokens: &[Token], spans:&[Span] ) -> Result<(Vec<Style>,Vec<Component>)> {
    let mut cursor = Cursor::new(&tokens);
    let mut styles = vec![];
    let mut components = vec![];

    while !cursor.is_eof() {
        let mut check_fork = cursor.fork();
        let is_style_item = take_match!(check_fork,
            [Token::Id(_)] => true,
            [Token::Class(_)] => true,
            [Token::Ident(_), Token::LParen] => false,
            [Token::Ident(_)] => true,
            _ => false
        );
        if is_style_item {
            let style;
            (cursor, style) = parse_style_item(cursor)?;
            styles.push(style);
            continue;
        }

        if let (_, [Token::Ident(_), Token::LParen]) = cursor.fork().consume() {
            let component;
            (cursor, component) = parse_component(cursor)?;
            components.push(component);
            continue;
        }

        //Error
        return Err(ParseError::unknown_start(cursor.span()));
    }

    Ok( (styles, components) )
}

#[derive(Debug,Clone)]
pub struct SKUIParseError {
    pub kind: ParseError,
    pub span: Span,
}

fn tokenize_from_str<'a:'b,'b>(input: &'a str) -> (Vec<Token<'b>>, Vec<Span>) {
    let spanned:Vec<(Token,Span)> = Token::lexer(input)
        .spanned()
        .filter_map(| (t,s) | t.map( |v| (v,s) ).ok() )
        .collect::<Vec<_>>();

    let (tokens, spans):(Vec<Token>, Vec<Span>) = spanned.into_iter().unzip();
    (tokens, spans)
}

fn parse(input: &str) -> Result<SKUI, SKUIParseError> {
    let (tokens, spans) = tokenize_from_str(input);
    match parse_tokens(&tokens, &spans) {
        Ok( (styles, components) ) => Ok( SKUI{ styles, components, spans } ),
        Err(e) => {
            Err( SKUIParseError {
                span : spans[ e.span.idx() ].clone(),
                kind : e,
            })
        },
    }
}

fn render_error(
    input: &str,
    span: Span,
    context_lines: usize, // 이전 몇 줄 보여줄지
) -> String {
    #[derive(Debug)]
    struct LineInfo {
        line_no: usize,      // 1-based
        line_start: usize,   // byte index
        line_end: usize,     // byte index (newline 제외)
    }

    fn find_line(input: &str, pos: usize) -> LineInfo {
        let mut line_no = 1;
        let mut last_nl = 0;

        for (i, c) in input.char_indices() {
            if i >= pos {
                break;
            }
            if c == '\n' {
                line_no += 1;
                last_nl = i + 1;
            }
        }

        let line_end = input[last_nl..]
            .find('\n')
            .map(|i| last_nl + i)
            .unwrap_or(input.len());

        LineInfo {
            line_no,
            line_start: last_nl,
            line_end,
        }
    }

    fn byte_to_column(line: &str, byte_offset: usize) -> usize {
        line[..byte_offset].chars().count()
    }

    let line = find_line(input, span.start);

    let mut out = String::new();

    // 이전 라인 출력
    let mut current_line_start = line.line_start;
    let mut current_line_no = line.line_no;

    for _ in 0..context_lines {
        if current_line_start == 0 {
            break;
        }
        let prev = find_line(input, current_line_start - 1);
        out = format!(
            "{:>4} | {}\n{}",
            prev.line_no,
            &input[prev.line_start..prev.line_end],
            out
        );
        current_line_start = prev.line_start;
        current_line_no = prev.line_no;
    }

    // 현재 라인
    let line_text = &input[line.line_start..line.line_end];
    out.push_str(&format!(
        "{:>4} | {}\n",
        line.line_no,
        line_text
    ));

    let col_start =
        byte_to_column(line_text, span.start - line.line_start);
    let col_end =
        byte_to_column(line_text, span.end.min(line.line_end) - line.line_start)
            .max(col_start + 1);

    // caret 라인
    out.push_str("     | ");
    out.push_str(&" ".repeat(col_start));
    out.push_str(&"^".repeat(col_end - col_start));
    out.push('\n');

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = r#"
            Flex { background-color: black; padding:1px }
            #list { border: 1px solid yellow }
            .myBtn { border: 2px }
            #myFlex { border:2px }
            .background_white { background-color: WHITE }

            Flex(MainFill) #myFlex .background_white {
                myProperty1 : "data"
                propertyMap : {key=1, key2=true}
                propertyAnother : [ 1,2,3 ]
                FlexItem(1.0, Button("FlexItem1"))
                FlexItem(2.0, Button("FlexItem2"))
                Button()
                Flex() {
                    Label("1") Label("2")
                }
            }

            Grid(2,3) {
                Label()
            }

            CustomWidget() {
                Flex() {
                    FlexItem( 0.5, Button("OK") )
                    FlexItem( 0.5, Button("Cancel") )
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
                panic!("Cause : \n{}", render_error(input, e.span, 2));
            }
        }
    }

    #[test]
    fn narr() {
        let token = vec![ Token::Ident("MainFill") ];
        let cursor = Cursor::new(&token);
        println!("{:?}", parse_inner_array(cursor).unwrap());
    }

    #[test]
    fn style_block() {
        fn parse_style_block(cursor:Cursor) -> CursorResult<Vec<Style>> {
            let mut items = Vec::new();
            let span = cursor.span();
            if let (cursor,Token::Ident("style")) = cursor.consume_one() {
                let span = cursor.span();
                let SplitCursor{next:cursor,result:mut block} = cursor.consume_delimited_inner( Token::block_brace() ).ok_or_else(|| ParseError::expect_brace_block(span))?;
                while !block.is_eof() {
                    let (next, style_item) = parse_style_item(block)?;
                    block = next;
                    items.push( style_item );
                }
                cursor.ok_with(items)
            } else {
                Err(ParseError::expect_ident(span))
            }
        }
        let input = r#"
style {
    Flex { background-color: black; padding:1px }
    #list { border: 1px solid yellow }
    .myBtn { border: 2px }
}
        "#;
        let (tokens, spans) = tokenize_from_str(input);
        let cursor = Cursor::new(&tokens);

        if let (_,Token::Ident("style")) = cursor.fork().consume_one() {
            match parse_style_block( cursor ) {
                Ok( (cursor,style_block) ) => {
                    println!("Parsed successfully!");
                    println!("{:#?}", style_block);
                }
                Err(e) => {

                    println!("Parse error: {:?}", e);
                    let span = &spans[e.span.idx()];
                    panic!("Cause : {}", &input[span.start .. span.end]);
                }
            }
        }
    }

    #[test]
    fn style_item() {
        let input = r#".myclass { background-color: black; padding:1px }"#;
        let (tokens, spans) = tokenize_from_str(input);
        let cursor = Cursor::new(&tokens);

        match parse_style_item(cursor) {
            Ok( (cursor,parsed) ) => {
                println!("Parsed successfully!");
                println!("{:#?}", parsed);
            }
            Err(e) => {
                let span = &spans[e.span.idx()];
                println!("Parse error: {:?}", e);
                panic!("Cause : {}", &input[span.start .. span.end]);
            }
        }
    }
}
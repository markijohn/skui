use crate::Component;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum PseudoClass {
    Hover,
    Active,
    Focus,
    Disabled,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Selector<'a> {
    // 단일 선택자
    Simple(SimpleSelector<'a>),

    // 복합 선택자 (쉼표) - OR 조건
    // .button, .link
    Group(Vec<Selector<'a>>),

    // 자손 선택자 (공백)
    // .container .button
    Descendant(Box<Selector<'a>>, Box<Selector<'a>>),

    // 자식 선택자 (>)
    // .container > .button
    Child(Box<Selector<'a>>, Box<Selector<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleSelector<'a> {
    pub kinds: Vec<SelectorKind<'a>>,
    pub pseudo_class: Option<PseudoClass>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectorKind<'a> {
    Id(&'a str),
    Class(&'a str),
    Tag(&'a str),
}


#[derive(Default)]
pub struct PseudoState {
    pub hovered: bool,
    pub active: bool,
    pub focused: bool,
    pub disabled: bool,
}

impl<'a> SimpleSelector<'a> {
    pub fn new() -> Self {
        Self {
            kinds: Vec::new(),
            pseudo_class: None,
        }
    }

    pub fn id(mut self, id: &'a str) -> Self {
        self.kinds.push(SelectorKind::Id(id));
        self
    }

    pub fn class(mut self, class: &'a str) -> Self {
        self.kinds.push(SelectorKind::Class(class));
        self
    }

    pub fn tag(mut self, tag: &'a str) -> Self {
        self.kinds.push(SelectorKind::Tag(tag));
        self
    }

    pub fn hover(mut self) -> Self {
        self.pseudo_class = Some(PseudoClass::Hover);
        self
    }

    pub fn active(mut self) -> Self {
        self.pseudo_class = Some(PseudoClass::Active);
        self
    }

    pub fn focus(mut self) -> Self {
        self.pseudo_class = Some(PseudoClass::Focus);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.pseudo_class = Some(PseudoClass::Disabled);
        self
    }

    pub fn is_matches(&self, element: &Component<'a>, state:PseudoState) -> bool {
        // 모든 SelectorKind 매칭 (AND)
        for kind in &self.kinds {
            let matches = match kind {
                SelectorKind::Id(id) => element.id == Some(id),
                SelectorKind::Class(class) => element.classes.contains(class),
                SelectorKind::Tag(tag) => element.name == *tag,
            };

            if !matches {
                return false;
            }
        }

        // pseudo_class 체크
        if let Some(pseudo) = &self.pseudo_class {
            match pseudo {
                PseudoClass::Hover => state.hovered,
                PseudoClass::Active => state.active,
                PseudoClass::Focus => state.focused,
                PseudoClass::Disabled => state.disabled,
            }
        } else {
            true
        }
    }
}

impl<'a> Selector<'a> {
    pub fn is_matches(&self, element: &Component<'a>, state:PseudoState) -> bool {
        match self {
            Selector::Simple(simple) => simple.is_matches(element, state),

            // Group: 하나라도 매칭 (OR)
            Selector::Group(selectors) => {
                selectors.iter().any(|sel| sel.is_matches(element, state))
            }

            // Descendant: 조상 중에 매칭되는 것이 있는지
            Selector::Descendant(ancestor_sel, descendant_sel) => {
                if !descendant_sel.is_matches(element, state) {
                    return false;
                }

                let mut current = element.parent;
                while let Some(parent) = current {
                    if ancestor_sel.is_matches(parent) {
                        return true;
                    }
                    current = parent.parent;
                }
                false
            }

            // Child: 직계 부모 매칭
            Selector::Child(parent_sel, child_sel) => {
                if !child_sel.is_matches(element) {
                    return false;
                }

                element.parent.map_or(false, |p| parent_sel.is_matches(p))
            }
        }
    }
}

// 헬퍼 함수
impl<'a> Selector<'a> {
    pub fn group(selectors: Vec<Selector<'a>>) -> Self {
        Selector::Group(selectors)
    }

    pub fn descendant(ancestor: Selector<'a>, descendant: Selector<'a>) -> Self {
        Selector::Descendant(Box::new(ancestor), Box::new(descendant))
    }

    pub fn child(parent: Selector<'a>, child: Selector<'a>) -> Self {
        Selector::Child(Box::new(parent), Box::new(child))
    }

    pub fn parse_selector<'a>(tokens: Vec<Token<'a>>) -> Result<Selector<'a>, ParseError> {
        SelectorParser::new(tokens).parse()
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    UnexpectedEnd,
    EmptySelector,
}

pub struct SelectorParser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> SelectorParser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Result<Selector<'a>, ParseError> {
        self.parse_selector_group()
    }

    // Group 파싱: selector1, selector2, selector3
    fn parse_selector_group(&mut self) -> Result<Selector<'a>, ParseError> {
        let mut selectors = vec![self.parse_combinator_chain()?];

        while self.peek() == Some(&Token::Comma) {
            self.advance(); // consume comma
            selectors.push(self.parse_combinator_chain()?);
        }

        if !self.is_end() {
            return Err(ParseError::UnexpectedToken(format!("{:?}", self.peek())));
        }

        if selectors.len() == 1 {
            Ok(selectors.into_iter().next().unwrap())
        } else {
            Ok(Selector::Group(selectors))
        }
    }

    // Combinator 파싱: A > B, A B
    fn parse_combinator_chain(&mut self) -> Result<Selector<'a>, ParseError> {
        let mut left = self.parse_simple_selector()?;

        loop {
            // 공백 (descendant) 또는 > (child)
            match self.peek() {
                Some(Token::Gt) => {
                    self.advance(); // consume >
                    let right = self.parse_simple_selector()?;
                    left = Selector::Child(Box::new(left), Box::new(right));
                }
                Some(Token::Id(_)) | Some(Token::Class(_)) | Some(Token::Ident(_)) | Some(Token::Colon) => {
                    // 공백으로 구분된 descendant (implicit)
                    let right = self.parse_simple_selector()?;
                    left = Selector::Descendant(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // Simple selector 파싱: button#id.class:hover
    fn parse_simple_selector(&mut self) -> Result<Selector<'a>, ParseError> {
        let mut simple = SimpleSelector::new();
        let mut has_any = false;

        // Tag, Id, Class를 순서 상관없이 파싱
        loop {
            match self.peek() {
                Some(Token::Ident(tag)) => {
                    simple = simple.tag(tag);
                    self.advance();
                    has_any = true;
                }
                Some(Token::Id(id)) => {
                    simple = simple.id(id);
                    self.advance();
                    has_any = true;
                }
                Some(Token::Class(class)) => {
                    simple = simple.class(class);
                    self.advance();
                    has_any = true;
                }
                Some(Token::Colon) => {
                    self.advance(); // consume :
                    if let Some(Token::Ident(pseudo)) = self.peek() {
                        simple = match *pseudo {
                            "hover" => simple.hover(),
                            "active" => simple.active(),
                            "focus" => simple.focus(),
                            "disabled" => simple.disabled(),
                            _ => return Err(ParseError::UnexpectedToken(
                                format!("Unknown pseudo-class: {}", pseudo)
                            )),
                        };
                        self.advance();
                        has_any = true;
                    } else {
                        return Err(ParseError::UnexpectedEnd);
                    }
                }
                _ => break,
            }
        }

        if !has_any {
            return Err(ParseError::EmptySelector);
        }

        Ok(Selector::Simple(simple))
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}

// 편의 함수



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let element = Element {
            id: Some("submit"),
            classes: &["button", "primary"],
            tag: "button",
            state: PseudoState {
                hovered: true,
                ..Default::default()
            },
            parent: None,
        };

        // button#submit.primary:hover
        let selector = Selector::Simple(
            SimpleSelector::new()
                .tag("button")
                .id("submit")
                .class("primary")
                .hover()
        );

        assert!(selector.is_matches(&element));
    }

    #[test]
    fn test_group() {
        let element = Element {
            id: None,
            classes: &["button"],
            tag: "button",
            state: Default::default(),
            parent: None,
        };

        // .button, .link
        let selector = Selector::group(vec![
            Selector::Simple(SimpleSelector::new().class("button")),
            Selector::Simple(SimpleSelector::new().class("link")),
        ]);

        assert!(selector.is_matches(&element));
    }

    #[test]
    fn test_hierarchy() {
        let container = Element {
            id: None,
            classes: &["container"],
            tag: "div",
            state: Default::default(),
            parent: None,
        };

        let button = Element {
            id: None,
            classes: &["button"],
            tag: "button",
            state: Default::default(),
            parent: Some(&container),
        };

        // .container > .button
        let selector = Selector::child(
            Selector::Simple(SimpleSelector::new().class("container")),
            Selector::Simple(SimpleSelector::new().class("button")),
        );

        assert!(selector.is_matches(&button));
    }
}
use std::fmt::{Display, Formatter};
use crate::Component;
use crate::cursor::TokenCursor;
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


#[derive(Default,Clone,Copy)]
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

    pub fn get_pseudo_class(&self) -> Option<&PseudoClass> {
        self.pseudo_class.as_ref()
    }

    pub fn has_pseudo_class(&self) -> bool {
        self.pseudo_class.is_some()
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
        // if let Some(pseudo) = &self.pseudo_class {
        //     match pseudo {
        //         PseudoClass::Hover => state.hovered,
        //         PseudoClass::Active => state.active,
        //         PseudoClass::Focus => state.focused,
        //         PseudoClass::Disabled => state.disabled,
        //     }
        // } else {
        //     true
        // }
        true
    }


}

impl<'a> Selector<'a> {
    /// Selector에서 PseudoClass를 가져옵니다.
    /// 복합 선택자의 경우 가장 오른쪽(마지막) 선택자의 PseudoClass를 반환합니다.
    pub fn get_pseudo_class(&self) -> Option<&PseudoClass> {
        match self {
            // 단일 선택자: SimpleSelector의 pseudo_class 반환
            Selector::Simple(simple) => simple.pseudo_class.as_ref(),

            // 그룹 선택자: 첫 번째 선택자의 pseudo_class 반환
            // (OR 조건이므로 각각 개별적으로 처리해야 할 수도 있음)
            Selector::Group(selectors) => {
                selectors.first().and_then(|s| s.get_pseudo_class())
            }

            // 자손/자식 선택자: 오른쪽(마지막) 선택자의 pseudo_class 반환
            // 예: .container .button:hover -> :hover 반환
            Selector::Descendant(_, right) | Selector::Child(_, right) => {
                right.get_pseudo_class()
            }
        }
    }

    /// Selector에서 모든 PseudoClass를 수집합니다.
    pub fn collect_pseudo_classes(&self) -> Vec<&PseudoClass> {
        match self {
            Selector::Simple(simple) => {
                simple.pseudo_class.as_ref().into_iter().collect()
            }

            Selector::Group(selectors) => {
                selectors
                    .iter()
                    .flat_map(|s| s.collect_pseudo_classes())
                    .collect()
            }

            Selector::Descendant(left, right) | Selector::Child(left, right) => {
                let mut result = left.collect_pseudo_classes();
                result.extend(right.collect_pseudo_classes());
                result
            }
        }
    }

    /// 특정 PseudoClass를 포함하는지 확인합니다.
    pub fn has_pseudo_class(&self, target: &PseudoClass) -> bool {
        match self {
            Selector::Simple(simple) => {
                simple.pseudo_class.as_ref() == Some(target)
            }

            Selector::Group(selectors) => {
                selectors.iter().any(|s| s.has_pseudo_class(target))
            }

            Selector::Descendant(left, right) | Selector::Child(left, right) => {
                left.has_pseudo_class(target) || right.has_pseudo_class(target)
            }
        }
    }



    pub fn is_matches(&self, parents:&[&Component<'a>], element: &Component<'a>, state:PseudoState) -> bool {
        match self {
            Selector::Simple(simple) => simple.is_matches(element, state),

            // Group: 하나라도 매칭 (OR)
            Selector::Group(selectors) => {
                selectors.iter().any(|sel| sel.is_matches(parents, element, state))
            }

            // Descendant: 조상 중에 매칭되는 것이 있는지
            Selector::Descendant(ancestor_sel, descendant_sel) => {
                if !descendant_sel.is_matches(parents, element, state) {
                    return false;
                }

                // 부모 체인을 역순으로 탐색
                for i in (1..parents.len()).rev() {
                    // parents[i]의 조상은 parents[..i]
                    if ancestor_sel.is_matches(&parents[..i], &parents[i], state) {
                        return true;
                    }
                }
                false
            }

            // Child: 직계 부모 매칭
            Selector::Child(parent_sel, child_sel) => {
                if !child_sel.is_matches(parents, element, state) {
                    return false;
                }

                parents.iter().rev().next().map_or(false, |p| parent_sel.is_matches(parents, p, state))
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

    pub fn parse_from_token(tks:&'a crate::TokenAndSpan) -> Result<Selector<'a> , SelectorParseError> {
        //let tks = crate::TokenAndSpan::new(selector_str).tokens;
        let cursor = TokenCursor::new( &tks.tokens );
        //Self::parse_from_token( cursor ).map(|(_,sel)| sel)
        SelectorParser::parse( cursor ).map( move |(_,sel)| sel)
    }
}

#[derive(Debug,Clone)]
pub enum SelectorParseError {
    UnexpectedToken(String),
    UnexpectedEnd,
    EmptySelector,
}

impl Display for SelectorParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct SelectorParser;

impl SelectorParser {
    pub fn parse<'a>(cursor: TokenCursor<'a, Token<'a>>) -> Result<(TokenCursor<'a, Token<'a>>, Selector<'a>), SelectorParseError> {
        // 앞의 WHITESPACE 건너뛰기
        let cursor = Self::skip_whitespace(cursor);

        let (cursor, selector) = Self::parse_selector_group(cursor)?;

        // LBrace로 끝나는지 확인
        let (_, token) = cursor.fork().consume_one();
        if token != Token::LBrace {
            return Err(SelectorParseError::UnexpectedToken(
                format!("Expected LBrace, found {:?}", token)
            ));
        }

        Ok((cursor, selector))
    }

    // Group 파싱: selector1, selector2, selector3
    fn parse_selector_group<'a>(cursor: TokenCursor<'a, Token<'a>>) -> Result<(TokenCursor<'a, Token<'a>>, Selector<'a>), SelectorParseError> {
        let (mut cursor, first) = Self::parse_combinator_chain(cursor)?;
        let mut selectors = vec![first];

        loop {
            let (next_cursor, token) = cursor.fork().consume_one();
            if token == Token::Comma {
                cursor = next_cursor;
                cursor = Self::skip_whitespace(cursor); // 쉼표 뒤 공백 무시
                let (next_cursor, selector) = Self::parse_combinator_chain(cursor)?;
                cursor = next_cursor;
                selectors.push(selector);
            } else {
                break;
            }
        }

        let selector = if selectors.len() == 1 {
            selectors.into_iter().next().unwrap()
        } else {
            Selector::Group(selectors)
        };

        Ok((cursor, selector))
    }

    // Combinator 파싱: A > B, A B
    fn parse_combinator_chain<'a>(cursor: TokenCursor<'a, Token<'a>>) -> Result<(TokenCursor<'a, Token<'a>>, Selector<'a>), SelectorParseError> {
        let (mut cursor, mut left) = Self::parse_simple_selector(cursor)?;

        loop {
            cursor = Self::skip_whitespace(cursor);

            let (next_cursor, token) = cursor.fork().consume_one();

            match token {
                Token::Gt => {
                    cursor = next_cursor;
                    cursor = Self::skip_whitespace(cursor); // > 뒤 공백 무시
                    let (next_cursor, right) = Self::parse_simple_selector(cursor)?;
                    cursor = next_cursor;
                    left = Selector::Child(Box::new(left), Box::new(right));
                }
                Token::Id(_) | Token::Class(_) | Token::Ident(_) | Token::Colon => {
                    // 공백으로 구분된 descendant (implicit)
                    let (next_cursor, right) = Self::parse_simple_selector(cursor)?;
                    cursor = next_cursor;
                    left = Selector::Descendant(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }

        Ok((cursor, left))
    }

    // Simple selector 파싱: button#id.class:hover
    fn parse_simple_selector<'a>(cursor: TokenCursor<'a, Token<'a>>) -> Result<(TokenCursor<'a, Token<'a>>, Selector<'a>), SelectorParseError> {
        let mut simple = SimpleSelector::new();
        let mut has_any = false;
        let mut cursor = cursor;

        // Tag, Id, Class를 순서 상관없이 파싱
        loop {
            let (next_cursor, token) = cursor.fork().consume_one();

            match token {
                Token::Ident(tag) => {
                    simple = simple.tag(tag);
                    cursor = next_cursor;
                    has_any = true;
                }
                Token::Id(id) => {
                    simple = simple.id(id);
                    cursor = next_cursor;
                    has_any = true;
                }
                Token::Class(class) => {
                    simple = simple.class(class);
                    cursor = next_cursor;
                    has_any = true;
                }
                Token::Colon => {
                    cursor = next_cursor;
                    let (next_cursor, pseudo_token) = cursor.consume_one();
                    if let Token::Ident(pseudo) = pseudo_token {
                        simple = match pseudo {
                            "hover" => simple.hover(),
                            "active" => simple.active(),
                            "focus" => simple.focus(),
                            "disabled" => simple.disabled(),
                            _ => return Err(SelectorParseError::UnexpectedToken(
                                format!("Unknown pseudo-class: {}", pseudo)
                            )),
                        };
                        cursor = next_cursor;
                        has_any = true;
                    } else {
                        return Err(SelectorParseError::UnexpectedEnd);
                    }
                }
                _ => break,
            }
        }

        if !has_any {
            return Err(SelectorParseError::EmptySelector);
        }

        Ok((cursor, Selector::Simple(simple)))
    }

    fn skip_whitespace<'a>(cursor: TokenCursor<'a, Token<'a>>) -> TokenCursor<'a, Token<'a>> {
        let mut cursor = cursor;
        loop {
            let (next_cursor, token) = cursor.fork().consume_one();
            if token == Token::Whitespace {
                cursor = next_cursor;
            } else {
                break;
            }
        }
        cursor
    }
}
// 편의 함수



#[cfg(test)]
mod tests {
    use tinyvec::ArrayVec;
    use crate::{Parameters, TokenAndSpan};
    use super::*;
    
    fn test_case(test:&str, expected:Selector) {
        let tks = TokenAndSpan::new(test);
        let selector = Selector::parse_from_token(&tks).unwrap();
        println!("Parsed ({test}) : {:?}", selector);
        assert_eq!(selector, expected);
    }
    
    #[test]
    fn test_selectors() {
        fn simple(kinds: Vec<SelectorKind>, pseudo: Option<PseudoClass>) -> Selector {
            Selector::Simple(SimpleSelector {
                kinds,
                pseudo_class: pseudo,
            })
        }

        fn tag(name: &str) -> SelectorKind {
            SelectorKind::Tag(name)
        }

        fn id(name: &str) -> SelectorKind {
            SelectorKind::Id(name)
        }

        fn class(name: &str) -> SelectorKind {
            SelectorKind::Class(name)
        }

        // ============ Simple Selectors ============

        // 1. Simple Tag Selector
        test_case(
            "button {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Tag("button")],
                pseudo_class: None
            })
        );

        // 2. Simple ID Selector
        test_case(
            "#submit {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Id("submit")],
                pseudo_class: None
            })
        );

        // 3. Simple Class Selector
        test_case(
            ".primary {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Class("primary")],
                pseudo_class: None
            })
        );

        // 4. Combined Tag + ID + Class
        test_case(
        "button#submit.primary {",
            Selector::Simple(SimpleSelector {
                kinds: vec![
                    SelectorKind::Tag("button"),
                    SelectorKind::Id("submit"),
                    SelectorKind::Class("primary")
                ],
                pseudo_class: None
            })
        );

        // 5. Multiple Classes
        test_case(
        ".btn.primary.large {",
            Selector::Simple(SimpleSelector {
                kinds: vec![
                    SelectorKind::Class("btn"),
                    SelectorKind::Class("primary"),
                    SelectorKind::Class("large")
                ],
                pseudo_class: None
            })
        );

        // 6. Tag + Multiple Classes
        test_case(
        "div.container.flex {",
            Selector::Simple(SimpleSelector {
                kinds: vec![
                    SelectorKind::Tag("div"),
                    SelectorKind::Class("container"),
                    SelectorKind::Class("flex")
                ],
                pseudo_class: None
            })
        );

        // 7. Single Pseudo-class
        test_case(
        "button:hover {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Tag("button")],
                pseudo_class: Some(PseudoClass::Hover)
            })
        );

        // 8. Pseudo-class with ID and Class
        test_case(
        "input#email.form-control:focus {",
            Selector::Simple(SimpleSelector {
                kinds: vec![
                    SelectorKind::Tag("input"),
                    SelectorKind::Id("email"),
                    SelectorKind::Class("form-control")
                ],
                pseudo_class: Some(PseudoClass::Focus)
            })
        );

        // 9. All Pseudo-classes
        test_case(
        "button:hover {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Tag("button")],
                pseudo_class: Some(PseudoClass::Hover)
            })
        );

        test_case(
        "button:active {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Tag("button")],
                pseudo_class: Some(PseudoClass::Active)
            })
        );

        test_case(
        "button:focus {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Tag("button")],
                pseudo_class: Some(PseudoClass::Focus)
            })
        );

        test_case(
        "button:disabled {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Tag("button")],
                pseudo_class: Some(PseudoClass::Disabled)
            })
        );

        // ============ Descendant Combinators ============

        // 10. Descendant Combinator (2 levels)
        test_case(
        "div button {",
            Selector::Descendant(
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("div")],
                    pseudo_class: None
                })),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }))
            )
        );

        // 11. Descendant Combinator (3 levels)
        test_case(
        "div section button {",
            Selector::Descendant(
                Box::new(Selector::Descendant(
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("div")],
                        pseudo_class: None
                    })),
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("section")],
                        pseudo_class: None
                    }))
                )),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }))
            )
        );

        // 12. Descendant with Classes
        test_case(
        ".container .btn {",
            Selector::Descendant(
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Class("container")],
                    pseudo_class: None
                })),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Class("btn")],
                    pseudo_class: None
                }))
            )
        );

        // ============ Child Combinators ============

        // 13. Child Combinator
        test_case(
        "div > button {",
            Selector::Child(
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("div")],
                    pseudo_class: None
                })),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }))
            )
        );

        // 14. Multiple Child Combinators
        test_case(
        "div > section > button {",
            Selector::Child(
                Box::new(Selector::Child(
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("div")],
                        pseudo_class: None
                    })),
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("section")],
                        pseudo_class: None
                    }))
                )),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }))
            )
        );

        // ============ Mixed Combinators ============

        // 15. Child then Descendant
        test_case(
        "div > section button {",
            Selector::Descendant(
                Box::new(Selector::Child(
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("div")],
                        pseudo_class: None
                    })),
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("section")],
                        pseudo_class: None
                    }))
                )),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }))
            )
        );

        // 16. Descendant then Child
        test_case(
        "div section > button {",
            Selector::Child(
                Box::new(Selector::Descendant(
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("div")],
                        pseudo_class: None
                    })),
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("section")],
                        pseudo_class: None
                    }))
                )),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }))
            )
        );

        // 17. Complex with Classes
        test_case(
        "div.container > button#submit.primary {",
            Selector::Child(
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![
                        SelectorKind::Tag("div"),
                        SelectorKind::Class("container")
                    ],
                    pseudo_class: None
                })),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![
                        SelectorKind::Tag("button"),
                        SelectorKind::Id("submit"),
                        SelectorKind::Class("primary")
                    ],
                    pseudo_class: None
                }))
            )
        );

        // 18. With Pseudo-classes
        test_case(
        "div:hover > button:active {",
            Selector::Child(
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("div")],
                    pseudo_class: Some(PseudoClass::Hover)
                })),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: Some(PseudoClass::Active)
                }))
            )
        );

        // ============ Selector Groups ============

        // 19. Simple Group (2 selectors)
        test_case(
        "button, input {",
            Selector::Group(vec![
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }),
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("input")],
                    pseudo_class: None
                })
            ])
        );

        // 20. Simple Group (3 selectors)
        test_case(
        "h1, h2, h3 {",
            Selector::Group(vec![
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("h1")],
                    pseudo_class: None
                }),
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("h2")],
                    pseudo_class: None
                }),
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("h3")],
                    pseudo_class: None
                })
            ])
        );

        // 21. Group with Classes
        test_case(
        ".primary, .secondary, .tertiary {",
            Selector::Group(vec![
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Class("primary")],
                    pseudo_class: None
                }),
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Class("secondary")],
                    pseudo_class: None
                }),
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Class("tertiary")],
                    pseudo_class: None
                })
            ])
        );

        // 22. Group with Complex Selectors
        test_case(
        "div > button, .container input {",
            Selector::Group(vec![
                Selector::Child(
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("div")],
                        pseudo_class: None
                    })),
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("button")],
                        pseudo_class: None
                    }))
                ),
                Selector::Descendant(
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Class("container")],
                        pseudo_class: None
                    })),
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("input")],
                        pseudo_class: None
                    }))
                )
            ])
        );

        // 23. Group with All Types
        test_case(
        "button, div > span, .class#id:hover {",
            Selector::Group(vec![
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }),
                Selector::Child(
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("div")],
                        pseudo_class: None
                    })),
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("span")],
                        pseudo_class: None
                    }))
                ),
                Selector::Simple(SimpleSelector {
                    kinds: vec![
                        SelectorKind::Class("class"),
                        SelectorKind::Id("id")
                    ],
                    pseudo_class: Some(PseudoClass::Hover)
                })
            ])
        );

        // ============ Whitespace Handling ============

        // 24. Leading Whitespace
        test_case(
        "   button {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Tag("button")],
                pseudo_class: None
            })
        );

        // 25. Whitespace Around Child Combinator
        test_case(
        "div  >  button {",
            Selector::Child(
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("div")],
                    pseudo_class: None
                })),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }))
            )
        );

        // 26. Whitespace Around Comma
        test_case(
        "button  ,  input {",
            Selector::Group(vec![
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("button")],
                    pseudo_class: None
                }),
                Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("input")],
                    pseudo_class: None
                })
            ])
        );

        // ============ Complex Real-world Examples ============

        // 27. Complex Example 1
        test_case(
        "div.container > button#submit.btn.primary:hover {",
            Selector::Child(
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![
                        SelectorKind::Tag("div"),
                        SelectorKind::Class("container")
                    ],
                    pseudo_class: None
                })),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![
                        SelectorKind::Tag("button"),
                        SelectorKind::Id("submit"),
                        SelectorKind::Class("btn"),
                        SelectorKind::Class("primary")
                    ],
                    pseudo_class: Some(PseudoClass::Hover)
                }))
            )
        );

        // 28. Complex Example 2 - Deep Nesting
        test_case(
        ".sidebar nav > ul li a:active {",
            Selector::Descendant(
                Box::new(Selector::Descendant(
                    Box::new(Selector::Child(
                        Box::new(Selector::Descendant(
                            Box::new(Selector::Simple(SimpleSelector {
                                kinds: vec![SelectorKind::Class("sidebar")],
                                pseudo_class: None
                            })),
                            Box::new(Selector::Simple(SimpleSelector {
                                kinds: vec![SelectorKind::Tag("nav")],
                                pseudo_class: None
                            }))
                        )),
                        Box::new(Selector::Simple(SimpleSelector {
                            kinds: vec![SelectorKind::Tag("ul")],
                            pseudo_class: None
                        }))
                    )),
                    Box::new(Selector::Simple(SimpleSelector {
                        kinds: vec![SelectorKind::Tag("li")],
                        pseudo_class: None
                    }))
                )),
                Box::new(Selector::Simple(SimpleSelector {
                    kinds: vec![SelectorKind::Tag("a")],
                    pseudo_class: Some(PseudoClass::Active)
                }))
            )
        );

        // 29. Only ID
        test_case(
        "#main {",
            Selector::Simple(SimpleSelector {
                kinds: vec![SelectorKind::Id("main")],
                pseudo_class: None
            })
        );

        // 30. ID + Multiple Classes
        test_case(
        "#header.sticky.top {",
            Selector::Simple(SimpleSelector {
                kinds: vec![
                    SelectorKind::Id("header"),
                    SelectorKind::Class("sticky"),
                    SelectorKind::Class("top")
                ],
                pseudo_class: None
            })
        );
    }
    

    #[test]
    fn test_match() {
        let sel_str = "button#submit.primary:hover {";
        let tks = TokenAndSpan::new(sel_str);
        let selector = Selector::parse_from_token(&tks).unwrap();
        let mut classes = ArrayVec::<[&'static str;5]>::new();
        classes.push("primary");
        let comp = Component {
            name: "button",
            params: Parameters::empty(),
            id: Some("submit"),
            classes: classes,
            children: vec![],
            properties: Default::default(),
        };
        
        println!("is_match? : {}", selector.is_matches(&[], &comp, PseudoState::default() ) );
        
        
    }


}
use crate::Component;

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

// Element 구조 (간소화)
pub struct Element<'a> {
    pub id: Option<&'a str>,
    pub classes: &'a [&'a str],
    pub tag: &'a str,
    pub state: ElementState,
    pub parent: Option<&'a Element<'a>>,
}

#[derive(Default)]
pub struct ElementState {
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

    pub fn is_matches(&self, element: &Element<'a>) -> bool {
        // 모든 SelectorKind 매칭 (AND)
        for kind in &self.kinds {
            let matches = match kind {
                SelectorKind::Id(id) => element.id == Some(id),
                SelectorKind::Class(class) => element.classes.contains(class),
                SelectorKind::Tag(tag) => element.tag == *tag,
            };

            if !matches {
                return false;
            }
        }

        // pseudo_class 체크
        if let Some(pseudo) = &self.pseudo_class {
            match pseudo {
                PseudoClass::Hover => element.state.hovered,
                PseudoClass::Active => element.state.active,
                PseudoClass::Focus => element.state.focused,
                PseudoClass::Disabled => element.state.disabled,
            }
        } else {
            true
        }
    }
}

impl<'a> Selector<'a> {
    pub fn is_matches(&self, element: &Element<'a>) -> bool {
        match self {
            Selector::Simple(simple) => simple.is_matches(element),

            // Group: 하나라도 매칭 (OR)
            Selector::Group(selectors) => {
                selectors.iter().any(|sel| sel.is_matches(element))
            }

            // Descendant: 조상 중에 매칭되는 것이 있는지
            Selector::Descendant(ancestor_sel, descendant_sel) => {
                if !descendant_sel.is_matches(element) {
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
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let element = Element {
            id: Some("submit"),
            classes: &["button", "primary"],
            tag: "button",
            state: ElementState {
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
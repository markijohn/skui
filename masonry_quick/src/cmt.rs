#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Normal,
    LineComment,
    BlockComment,
    String(char), // ' or "
}

pub fn strip_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut state = State::Normal;

    while let Some(c) = chars.next() {
        match state {
            State::Normal => match c {
                '/' if chars.peek() == Some(&'/') => {
                    chars.next();
                    state = State::LineComment;
                }
                '/' if chars.peek() == Some(&'*') => {
                    chars.next();
                    state = State::BlockComment;
                }
                '"' | '\'' => {
                    out.push(c);
                    state = State::String(c);
                }
                _ => out.push(c),
            },

            State::LineComment => {
                if c == '\n' {
                    out.push('\n');
                    state = State::Normal;
                }
            }

            State::BlockComment => {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    state = State::Normal;
                }
            }

            State::String(quote) => {
                out.push(c);
                if c == '\\' {
                    // escape 처리
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == quote {
                    state = State::Normal;
                }
            }
        }
    }

    out
}
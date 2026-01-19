
struct CursorMark(usize);

#[derive(Clone)]
pub struct TokenCursor<'a,T> {
    idx: usize,
    tokens: &'a [T],
    default: T,
}

impl <'a,T> TokenCursor<'a,T> where T: Clone + Copy + PartialEq + Default {
    pub fn new(tokens: &'a [T]) -> TokenCursor<'a,T> {
        Self { idx: 0, tokens, default: T::default() }
    }

    pub fn mark(&self) -> CursorMark {
        CursorMark( self.idx )
    }

    pub fn rollback(&mut self, mark: CursorMark) {
        self.idx = mark.0;
    }

    pub fn reset(&mut self) {
        self.idx = 0;
    }

    pub fn consume(&mut self) {
        self.tokens = &self.tokens[self.idx..];
        self.idx = 0;
    }

    pub fn is_eof(&self) -> bool {
        self.idx == self.tokens.len()
    }

    pub fn peek_slice(&self, size: usize) -> &[T] {
        let to = (self.idx + size).min(self.tokens.len());
        &self.tokens[self.idx..to]
    }

    pub fn peek_one(&self) -> T {
        self.peek::<1>()[0]
    }

    pub fn peek<const SIZED: usize>(&self) -> [T; SIZED] {
        let mut r = [T::default(); SIZED];
        let n = self.peek_slice(SIZED);
        for i in 0..n.len() {
            r[i] = n[i];
        }
        r
    }

    pub fn take_one(&mut self) -> T {
        self.take::<1>()[0]
    }

    pub fn take_if<const SIZED: usize>(&mut self, v:[T;SIZED]) -> bool {
        let org = self.idx;
        if v == self.take() {
            self.idx = org;
            true
        } else {
            false
        }
    }

    pub fn take_if_try<const SIZED: usize,R>(
        &mut self,
        pred: impl Fn(&[T; SIZED]) -> bool,
    ) -> bool {
        let org = self.idx;
        let arr = self.take::<SIZED>();
        if pred(&arr) {
            true
        } else {
            self.idx = org;
            false
        }
    }

    pub fn take_if_map<const SIZED: usize,R>(
        &mut self,
        pred: impl Fn([T; SIZED]) -> Option<R>,
    ) -> Option<R> {
        let org = self.idx;
        let arr = self.take::<SIZED>();
        if let Some(r) = pred(arr) {
            Some(r)
        } else {
            self.idx = org;
            None
        }
    }

    pub fn take<const SIZED: usize>(&mut self) -> [T; SIZED] {
        let n = self.peek_slice(SIZED);
        let mut r = [T::default(); SIZED];
        for i in 0..n.len() {
            r[i] = n[i];
        }
        self.idx = (self.idx+SIZED).min( self.tokens.len() );
        r
    }



    pub fn take_collect_if<R>(
        &mut self,
        //mut stop: impl FnMut(&T) -> bool,
        check: impl Fn(&T) -> Option<R>,
    ) -> Vec<R> {
        let mut rv = Vec::new();
        for t in self.tokens[self.idx..].iter() {
            if let Some(r) = check(t) {
                rv.push(r);
                self.idx += 1;
            } else {
                break;
            }
        }
        rv
    }

    pub fn peek_delimited(&mut self, (start,end):(T,T) ) -> Option<TokenCursor<'a,T>> {
        let org_idx = self.idx;
        let r = self.take_delimited( (start, end) );
        self.idx = org_idx;
        r
    }

    pub fn take_delimited(&mut self, (start,end):(T,T) ) -> Option<TokenCursor<'a,T>> {
        if start != self.take_one() {
            return None;
        }
        let block_start = self.idx;
        let mut depth = 1;
        while !self.is_eof() {
            if start == self.take_one() {
                depth += 1;
            } else if end == self.take_one() {
                depth -= 1;
                if depth == 0 {
                    let block = &self.tokens[block_start..self.idx];
                    return Some(TokenCursor::new(block));
                }
            }
        }

        None
    }
}

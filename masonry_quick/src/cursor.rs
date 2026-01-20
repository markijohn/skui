#[macro_export]
macro_rules! take_if {
    (
        $cursor:ident,
        [ $($p1:pat),+ ] => $b1:expr ,
        $(
            [ $($p:pat),+ ] => $b:expr ,
        )*
        $( _ => $be:expr )?
    ) => {{
        if let Some([ $($p1),+ ]) = $cursor.take_if_map(|ts| {
            #[allow(unused_variables)]
            matches!(ts, [ $($p1),+ ]).then(|| ts)
        }) {
            $b1
        }
        $(
            else if let Some([ $($p),+ ]) = $cursor.take_if_map(|ts| {
                #[allow(unused_variables)]
                matches!(ts, [ $($p),+ ]).then(|| ts)
            }) {
                $b
            }
        )*
        $(
            else {
                $be
            }
        )?
    }};
}

#[macro_export]
macro_rules! take_if_map {
    ( $cursor:ident,
        $(
            [ $($args:ty),+ ] => $body:expr
        )+
    ) => {
        || {
            $(
                if let Some(v) = $cursor.take_if_map( |ts| {
                    if let [ $($args),+ ] = ts {
                        Some( $expr )
                    } else {
                        None
                    }
                } ) {
                    return Some(v)
                }
            )+
            None
        } ()
    }
}

pub enum CursorBreak {
    BreakThis,
    BreakNext,
    NoBreak
}


#[derive(Debug)]
pub struct TokenCursor<'a,T> {
    base_idx: usize,
    idx: usize,
    tokens: &'a [T],
}


impl <'a,T> TokenCursor<'a,T> where T: Clone + Copy + PartialEq + Default {
    pub fn new(tokens: &'a [T]) -> TokenCursor<'a,T> {
        Self { base_idx:0, idx:0, tokens }
    }

    pub fn idx(&self) -> usize {
        self.base_idx + self.idx
    }

    pub fn fork(&self) -> TokenCursor<'a,T> {
        Self { base_idx:self.base_idx, idx:self.idx, tokens:self.tokens }
    }
    
    pub fn is_eof(&self) -> bool {
        self.idx >= self.tokens.len()
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

    pub fn take_ignore<const SIZED: usize>(&mut self, v:[T;SIZED]) -> bool {
        let org = self.idx;
        if v == self.take() {
            true
        } else {
            self.idx = org;
            false
        }
    }

    pub fn take_ignore_until<const SIZED: usize>(
        &mut self,
        allow_eof: bool,
        pred: impl Fn([T; SIZED]) -> bool,
    ) -> bool {
        let org = self.idx;
        loop {
            if self.is_eof() {
                if !allow_eof {
                    self.idx = org;
                }
                return allow_eof
            }
            if pred( self.take::<SIZED>() ) {
                return true
            }
        }
    }


    // 고정 크기의 배열을 take 하여 검사하고 원하는대로 맵핑되었으면 변환하며
    // 아닐 경우는 커서를 원복시킴
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

    // 고정 크기의 배열을 take
    pub fn take<const SIZED: usize>(&mut self) -> [T; SIZED] {
        let n = self.peek_slice(SIZED);
        let mut r = [T::default(); SIZED];
        for i in 0..n.len() {
            r[i] = n[i];
        }
        self.idx = (self.idx+SIZED).min( self.tokens.len() );
        r
    }

    pub fn take2<const SIZED: usize>(mut self) -> (TokenCursor<'a,T>,[T; SIZED]) {
        let r = self.take::<SIZED>();
        (self,r)
    }

    pub fn take_map_until<R>(
        &mut self,
        check: impl Fn(T) -> Option<R>,
    ) -> Vec<R> {
        let mut rv = Vec::new();
        for t in self.tokens[self.idx..].iter() {
            if let Some(r) = check(*t) {
                rv.push(r);
                self.idx += 1;
            } else {
                break;
            }
        }
        rv
    }

    //
    pub fn take_filtermap_until<R>(
        &mut self,
        check: impl Fn(T) -> (CursorBreak, Option<R>),
    ) -> Vec<R> {
        let mut rv = Vec::new();
        for t in self.tokens[self.idx..].iter() {
            let (brk,r) = check(*t);
            if let Some(r) = r {
                rv.push(r);
            }
            match brk {
                CursorBreak::BreakThis => { break }
                CursorBreak::BreakNext => { self.idx += 1; break }
                CursorBreak::NoBreak => { self.idx += 1; }
            }
        }
        rv
    }

    pub fn peek_delimited(&mut self, (start,end):(T,T) ) -> Option<TokenCursor<'a,T>> {
        let org_idx = self.idx;
        let r = self.take_if_delimited( (start, end) );
        self.idx = org_idx;
        r
    }

    pub fn take_if_delimited(&mut self, (start,end):(T, T) ) -> Option<TokenCursor<'a,T>> {
        let org_idx = self.idx;
        if start == self.take_one() {
            let block_start = self.idx;
            let mut depth = 1;
            while !self.is_eof() {
                let next = self.take_one();
                if start == next {
                    depth += 1;
                } else if end == next {
                    depth -= 1;
                    if depth == 0 {
                        let block = &self.tokens[block_start..self.idx-1];
                        let mut cursor = TokenCursor::new(block);
                        cursor.base_idx = self.base_idx + block_start;
                        return Some( cursor );
                    }
                }
            }
        }

        self.idx = org_idx;
        None
    }

    pub fn ok_with<RT,E>(self, t:RT) -> Result<(Self,RT),E> {
        Ok( (self,t) )
    }

    pub fn peek_all(&self) -> &[T] {
        self.tokens
    }
}




#[derive(Debug)]
pub struct TokenCursor2<'a,T> {
    base_idx: usize,
    tokens: &'a [T],
}


impl <'a,T> TokenCursor2<'a,T> where T: Clone + Copy + PartialEq + Default {
    pub fn new(tokens: &'a [T]) -> TokenCursor2<'a,T> {
        Self { base_idx:0, tokens }
    }

    pub fn idx(&self) -> usize {
        self.base_idx
    }

    pub fn fork(&self) -> TokenCursor2<'a,T> {
        Self { base_idx:self.base_idx, tokens:self.tokens }
    }

    pub fn is_eof(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn take_one(mut self) -> (Self,T) {
        let (c,t) = self.take::<1>();
        (c, t[0])
    }

    pub fn take_ignore_if<const SIZED: usize>(self, v:[T;SIZED]) -> (Self,bool) {
        let ct = self.fork();
        let (next,r) = ct.take::<SIZED>();
        if v == r {
            (next,true)
        } else {
            (self,false)
        }
    }

    pub fn take_ignore_until<const SIZED: usize>(
        mut self,
        allow_eof: bool,
        pred: impl Fn([T; SIZED]) -> bool,
    ) -> (Self,bool) {
        let mut ct = self.fork();
        loop {
            if ct.is_eof() {
                if !allow_eof {
                    ct.tokens = self.tokens;
                }
                return (ct,allow_eof)
            }
            let (next, r) = ct.take::<SIZED>();
            ct = next;
            if pred( r ) {
                return (ct,true)
            }
        }
    }

    // 고정 크기의 배열을 take
    pub fn take<const SIZED: usize>(mut self) -> (Self,[T; SIZED]) {
        let mut r = [T::default(); SIZED];
        let (slice,next) = self.tokens.split_at( SIZED.min(self.tokens.len()) );
        r[..slice.len()].copy_from_slice(slice);
        self.tokens = next;
        (self, r)
    }

    pub fn take_map_until<R>(
        &mut self,
        check: impl Fn(T) -> Option<R>,
    ) -> Vec<R> {
        let mut rv = Vec::new();
        for t in self.tokens[self.idx..].iter() {
            if let Some(r) = check(*t) {
                rv.push(r);
                self.idx += 1;
            } else {
                break;
            }
        }
        rv
    }

    //
    pub fn take_filtermap_until<R,E>(
        &mut self,
        check: impl Fn(T) -> Result<Option<R>,E>,
    ) -> Result<Vec<R>,E> {


        let mut rv = Vec::new();
        for t in self.tokens[self.idx..].iter() {
            let (brk,r) = check(*t);
            if let Some(r) = r {
                rv.push(r);
            }
        }
        rv
    }

    pub fn take_delimited_cursor_if(mut self, (start,end):(T, T) ) -> (Self,Option<TokenCursor<'a,T>>) {
        let mut ct = self.fork();
        let first;
        (ct,first) = ct.take_one();
        if start == first {
            let block_start = ct.tokens;
            let mut cnt = 0;
            let mut depth = 1;
            while !ct.is_eof() {
                let next;
                (ct,next) = ct.take_one();
                cnt += 1;
                if start == next {
                    depth += 1;
                } else if end == next {
                    depth -= 1;
                    if depth == 0 {
                        let mut cursor = TokenCursor::new(&block_start[ .. cnt]);
                        cursor.base_idx = self.base_idx + cnt;
                        return (ct, Some( cursor ) );
                    }
                }
            }
        }

        (self,None)
    }

    pub fn ok_with<RT,E>(self, t:RT) -> Result<(Self,RT),E> {
        Ok( (self,t) )
    }

    pub fn peek_all(&self) -> &[T] {
        self.tokens
    }
}

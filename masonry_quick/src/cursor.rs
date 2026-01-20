use std::fmt::Debug;

// 패턴에 매치되면 인자의 take 되고 인자의 cursor 가 새로운 커서로 대치됨
// 매치되지 않으면 cursor 에 변화 없음
#[macro_export]
macro_rules! take_match {
    (
        $cursor:ident,
        [ $($p1:pat),+ ] => $b1:expr ,
        $(
            [ $($p:pat),+ ] => $b:expr ,
        )*
        $( _ => $be:expr )?
    ) => {{
        if let (new_cursor, [ $($p1),+ ]) = $cursor.fork().take() {
            $cursor = new_cursor;
            $b1
        }
        $(
            else if let (new_cursor, [ $($p),+ ]) = $cursor.fork().take() {
                $cursor = new_cursor;
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

#[derive(Debug)]
pub struct CursorSpan {
    idx: usize
}

impl CursorSpan {
    pub fn idx(&self) -> usize {
        self.idx
    }
}

#[derive(Debug)]
pub struct TokenCursor<'a,T> {
    base_idx: usize,
    tokens: &'a [T],
}

impl <'a,T> TokenCursor<'a,T> where T: Debug + Clone + Copy + PartialEq + Default {
    // Create root cursor
    pub fn new(tokens: &'a [T]) -> Self {
        Self { base_idx:0, tokens }
    }

    // Index of root slice
    pub fn idx(&self) -> usize {
        self.base_idx
    }

    pub fn span(&self) -> CursorSpan {
        CursorSpan { idx: self.base_idx }
    }

    // Fork another cursor
    pub fn fork(&self) -> Self {
        Self { base_idx:self.base_idx, tokens:self.tokens }
    }

    // Is end of file
    pub fn is_eof(&self) -> bool {
        self.tokens.is_empty()
    }

    // consume cursor
    pub fn skip(self, size: usize) -> Self {
        let take_num = size.min( self.tokens.len() );
        Self {
            base_idx : self.base_idx + take_num,
            tokens : &self.tokens[ take_num .. ],
        }
    }

    // take one Token. if cursor is eof then return the Default
    pub fn take_one(self) -> (Self,T) {
        let (c,t) = self.take::<1>();
        (c, t[0])
    }


    pub fn ignore_if<const SIZED: usize>(self, v:[T;SIZED]) -> (Self,bool) {
        let ct = self.fork();
        let (next,r) = ct.take::<SIZED>();
        if v == r {
            (next,true)
        } else {
            (self,false)
        }
    }

    //
    pub fn ignore_oneof(self, v:&[T]) -> (Self,bool) {
        let ct = self.fork();
        let (next,r) = ct.take_one();
        if v.iter().find(|&&e| e == r).is_some() {
            (next,true)
        } else {
            (self,false)
        }
    }

    pub fn ignore_until(self, pred:impl Fn(T) -> bool) -> Self {
        let mut ct = self.fork();
        while !ct.is_eof() {
            let (next,t) = ct.fork().take_one();
            if pred( t ) {
                break;
            } else {
                ct = next;
            }
        }
        ct
    }

    // 고정 크기의 배열을 take
    pub fn take<const SIZED: usize>(self) -> (Self,[T; SIZED]) {
        let mut r = [T::default(); SIZED];
        let actual_num = SIZED.min(self.tokens.len());
        let slice = &self.tokens[ .. actual_num];
        r[..slice.len()].copy_from_slice(slice);
        (self.skip(actual_num), r)
    }

    // pub fn take_slice(self, size:usize) -> (Self, &'a[T]) {
    //     let actual_num = size.min(self.tokens.len());
    //     let slice = &self.tokens[ .. actual_num];
    //     (self.skip(actual_num), slice)
    // }

    pub fn collect_until<R,E>(
        self,
        check: impl Fn(Self) -> Result<(Self,Option<R>),E>,
    ) -> Result<(Self,Vec<R>),E> {
        let mut rv = Vec::new();
        let mut ct = self.fork();
        while !ct.is_eof() {
            match check(ct) {
                Ok( (c,Some(t)) ) => { ct = c;rv.push(t) },
                Ok( (c, None) ) => { ct = c; break },
                Err(e) => return Err(e),
            }
        }
        Ok( (ct,rv) )
    }

    pub fn take_until(self, pred:impl Fn(T) -> bool) -> (Self,Self) {
        if let Some( (idx,_) ) = self.tokens.iter().enumerate().find( |&(_,&t) | pred(t) ) {
            let mut cursor = TokenCursor::new( &self.tokens[ .. idx] );
            cursor.base_idx = self.base_idx;
            (self.skip(idx), cursor)
        } else {
            let len = self.tokens.len();
            (self.fork(), self.skip( len ))
        }
    }

    //0 is next cursor, 1 is block cursor
    pub fn take_delimited(self, (start,end):(T, T) ) -> Option<(Self,Self)> {
        assert_ne!( start, T::default() );
        assert_ne!( end, T::default() );
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
                        return Some( (ct,cursor) );
                    }
                }
            }
        }
        None
    }

    pub fn take_iter(self) -> impl Iterator {
        self.tokens.iter()
    }

    pub fn ok_with<RT,E>(self, t:RT) -> Result<(Self,RT),E> {
        Ok( self.with(t) )
    }

    pub fn with<R>(self, r:R) -> (Self,R) { (self,r) }

    fn peek_all(&self) -> &[T] {
        self.tokens
    }
}

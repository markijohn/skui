use std::fmt::Debug;
use tinyvec::ArrayVec;

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
        if let (new_cursor, [ $($p1),+ ]) = $cursor.fork().consume() {
            $cursor = new_cursor;
            $b1
        }
        $(
            else if let (new_cursor, [ $($p),+ ]) = $cursor.fork().consume() {
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

#[derive(Debug,Clone)]
pub struct CursorSpan {
    idx: usize
}

impl CursorSpan {
    pub fn idx(&self) -> usize {
        self.idx
    }
}

#[derive(Debug)]
pub struct SplitCursor<'a,T> {
    pub next : TokenCursor<'a,T>,
    pub result : TokenCursor<'a,T>,
}

impl <'a,T> SplitCursor<'a,T> {
    pub fn new(next:TokenCursor<'a,T>, result:TokenCursor<'a,T>) -> Self {
        Self { next, result }
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

    pub fn new_offset(tokens: &'a [T], offset: usize) -> Self {
        Self { base_idx:offset, tokens }
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
    pub fn consume_one(self) -> (Self, T) {
        let (c,t) = self.consume::<1>();
        (c, t[0])
    }


    pub fn ignore<const SIZED: usize>(self, v:[T;SIZED]) -> (Self, bool) {
        let ct = self.fork();
        let (next,r) = ct.consume::<SIZED>();
        if v == r {
            (next,true)
        } else {
            (self,false)
        }
    }

    //
    pub fn ignore_oneof(self, v:&[T]) -> (Self,bool) {
        let ct = self.fork();
        let (next,r) = ct.consume_one();
        if v.iter().find(|&&e| e == r).is_some() {
            (next,true)
        } else {
            (self,false)
        }
    }

    pub fn ignore_until(self, pred:impl Fn(T) -> bool) -> Self {
        let mut ct = self;
        while !ct.is_eof() {
            let (next,t) = ct.fork().consume_one();
            if pred( t ) {
                break;
            } else {
                ct = next;
            }
        }
        ct
    }

    // 고정 크기의 배열을 take
    pub fn consume<const SIZED: usize>(self) -> (Self, [T; SIZED]) {
        let mut r = [T::default(); SIZED];
        let actual_num = SIZED.min(self.tokens.len());
        let slice = &self.tokens[ .. actual_num];
        r[..slice.len()].copy_from_slice(slice);
        (self.skip(actual_num), r)
    }

    pub fn consume_collect_until<R,E>(
        self,
        check: impl Fn(Self) -> Result<(Self,Option<R>),E>,
    ) -> Result<(Self,Vec<R>),E> {
        let mut rv = Vec::new();
        let mut ct = self;
        while !ct.is_eof() {
            match check(ct) {
                Ok( (c,Some(t)) ) => { ct = c;rv.push(t) },
                Ok( (c, None) ) => { ct = c; break },
                Err(e) => return Err(e),
            }
        }
        Ok( (ct,rv) )
    }

    pub fn consume_collect_until_arrayvec<const SIZE:usize,R:Default,E>(
        self,
        check: impl Fn(Self) -> Result<(Self,Option<R>),E>,
    ) -> Result<(Self,ArrayVec<[R;SIZE]>),E> {
        let mut rv = ArrayVec::<[R;SIZE]>::default();
        let mut ct = self;
        while !ct.is_eof() {
            match check(ct) {
                Ok( (c,Some(t)) ) => { ct = c;rv.push(t) },
                Ok( (c, None) ) => { ct = c; break },
                Err(e) => return Err(e),
            }
        }
        Ok( (ct,rv) )
    }

    pub fn split_until(self, pred:impl Fn(T) -> bool) -> Option<SplitCursor<'a,T>> {
        if let Some( (idx,_) ) = self.tokens.iter().enumerate().find( |&(_,&t) | pred(t) ) {
            let mut cursor = TokenCursor::new( &self.tokens[ .. idx] );
            cursor.base_idx = self.base_idx;
            Some( SplitCursor::new( self.skip(idx), cursor ) )
        } else {
            None
        }
    }


    pub fn consume_delimited_inner(self, (start,end):(T, T) ) -> Option<SplitCursor<'a,T>> {
        assert_ne!( start, T::default() );
        assert_ne!( end, T::default() );

        let (mut ct,first) = self.consume_one();
        if start == first {
            let mut block_cursor = ct.fork();
            let mut cnt = 0;
            let mut depth = 1;
            while !ct.is_eof() {
                let next;
                (ct,next) = ct.consume_one();
                if start == next {
                    depth += 1;
                } else if end == next {
                    depth -= 1;
                    if depth == 0 {
                        block_cursor.tokens = &block_cursor.tokens[ .. cnt];
                        return Some( SplitCursor::new(ct, block_cursor) );
                    }
                }
                cnt += 1;
            }
        }
        None
    }

    pub fn ok_with<RT,E>(self, t:RT) -> Result<(Self,RT),E> {
        Ok( (self,t) )
    }
}



// #[derive(Debug)]
// pub struct TokenCursor<'a,T> {
//     base_idx: usize,
//     tokens: &'a [T],
// }
//
// impl <'a,T> TokenCursor<'a,T> where T: Debug + Clone + Copy + PartialEq + Default {
//     // Create root cursor
//     pub fn new(tokens: &'a [T]) -> Self {
//         Self { base_idx:0, tokens }
//     }
//
//     // Index of root slice
//     pub fn idx(&self) -> usize {
//         self.base_idx
//     }
//
//     pub fn span(&self) -> CursorSpan {
//         CursorSpan { idx: self.base_idx }
//     }
//
//     // Fork another cursor
//     pub fn fork(&self) -> Self {
//         Self { base_idx:self.base_idx, tokens:self.tokens }
//     }
//
//     // Is end of file
//     pub fn is_eof(&self) -> bool {
//         self.tokens.is_empty()
//     }
//
//     // consume cursor
//     pub fn skip(self, size: usize) -> Self {
//         let take_num = size.min( self.tokens.len() );
//         Self {
//             base_idx : self.base_idx + take_num,
//             tokens : &self.tokens[ take_num .. ],
//         }
//     }
//
//     // take one Token. if cursor is eof then return the Default
//     pub fn consume_one(self) -> (Self, T) {
//         let (c,t) = self.consume::<1>();
//         (c, t[0])
//     }
//
//
//     pub fn ignore<const SIZED: usize>(self, v:[T;SIZED]) -> (Self, bool) {
//         let ct = self.fork();
//         let (next,r) = ct.consume::<SIZED>();
//         if v == r {
//             (next,true)
//         } else {
//             (self,false)
//         }
//     }
//
//     //
//     pub fn ignore_oneof(self, v:&[T]) -> (Self,bool) {
//         let ct = self.fork();
//         let (next,r) = ct.consume_one();
//         if v.iter().find(|&&e| e == r).is_some() {
//             (next,true)
//         } else {
//             (self,false)
//         }
//     }
//
//     pub fn ignore_until(self, pred:impl Fn(T) -> bool) -> Self {
//         let mut ct = self;
//         while !ct.is_eof() {
//             let (next,t) = ct.fork().consume_one();
//             if pred( t ) {
//                 break;
//             } else {
//                 ct = next;
//             }
//         }
//         ct
//     }
//
//     // 고정 크기의 배열을 take
//     pub fn consume<const SIZED: usize>(self) -> (Self, [T; SIZED]) {
//         let mut r = [T::default(); SIZED];
//         let actual_num = SIZED.min(self.tokens.len());
//         let slice = &self.tokens[ .. actual_num];
//         r[..slice.len()].copy_from_slice(slice);
//         (self.skip(actual_num), r)
//     }
//
//     pub fn consume_collect_until<R,E>(
//         self,
//         check: impl Fn(Self) -> Result<(Self,Option<R>),E>,
//     ) -> Result<(Self,Vec<R>),E> {
//         let mut rv = Vec::new();
//         let mut ct = self;
//         while !ct.is_eof() {
//             match check(ct) {
//                 Ok( (c,Some(t)) ) => { ct = c;rv.push(t) },
//                 Ok( (c, None) ) => { ct = c; break },
//                 Err(e) => return Err(e),
//             }
//         }
//         Ok( (ct,rv) )
//     }
//
//     pub fn consume_collect_until_arrayvec<const SIZE:usize,R:Default,E>(
//         self,
//         check: impl Fn(Self) -> Result<(Self,Option<R>),E>,
//     ) -> Result<(Self,ArrayVec<[R;SIZE]>),E> {
//         let mut rv = ArrayVec::<[R;SIZE]>::default();
//         let mut ct = self;
//         while !ct.is_eof() {
//             match check(ct) {
//                 Ok( (c,Some(t)) ) => { ct = c;rv.push(t) },
//                 Ok( (c, None) ) => { ct = c; break },
//                 Err(e) => return Err(e),
//             }
//         }
//         Ok( (ct,rv) )
//     }
//
//     pub fn split_until(self, pred:impl Fn(T) -> bool) -> Option<SplitCursor<'a,T>> {
//         if let Some( (idx,_) ) = self.tokens.iter().enumerate().find( |&(_,&t) | pred(t) ) {
//             let mut cursor = TokenCursor::new( &self.tokens[ .. idx] );
//             cursor.base_idx = self.base_idx;
//             Some( SplitCursor::new( self.skip(idx), cursor ) )
//         } else {
//             None
//         }
//     }
//
//
//     pub fn consume_delimited_inner(self, (start,end):(T, T) ) -> Option<SplitCursor<'a,T>> {
//         assert_ne!( start, T::default() );
//         assert_ne!( end, T::default() );
//
//         let (mut ct,first) = self.consume_one();
//         if start == first {
//             let mut block_cursor = ct.fork();
//             let mut cnt = 0;
//             let mut depth = 1;
//             while !ct.is_eof() {
//                 let next;
//                 (ct,next) = ct.consume_one();
//                 if start == next {
//                     depth += 1;
//                 } else if end == next {
//                     depth -= 1;
//                     if depth == 0 {
//                         block_cursor.tokens = &block_cursor.tokens[ .. cnt];
//                         return Some( SplitCursor::new(ct, block_cursor) );
//                     }
//                 }
//                 cnt += 1;
//             }
//         }
//         None
//     }
//
//     pub fn ok_with<RT,E>(self, t:RT) -> Result<(Self,RT),E> {
//         Ok( (self,t) )
//     }
// }
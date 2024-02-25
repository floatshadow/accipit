use nom::{Compare, CompareResult, InputIter, InputLength, InputTake, Needed, Slice};
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};
use std::slice::Iter;
use std::iter::Enumerate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'a> {
    // Identifier
    TkIdent(&'a str),
    // Literals
    LtInt64(i64),
    LtInt1(i8),
    LtNone,
    LtNull,
    // primitive type keyword 
    TyInt64,
    TyInt1,
    // no `TyUnit` due to the conflict (LParen, RParen) v.s. TyUnit
    TyPtr,
    // Binary operator
    TkAdd,
    TkSub,
    TkMul,
    TkDiv,
    TkRem,
    TkAnd,
    TkOr,
    TkXor,
    TkLt,
    TkGt,
    TkLe,
    TkGe,
    TkEq,
    TkNe,
    // Offset operator
    TkOffset,
    // Memoty operator
    TkAlloca,
    TkLoad,
    TkStore,
    // Function Call
    TkFnCall,
    // Terminators
    TkJmp,
    TKBranch,
    TKRet,
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Equal,
    Arrow,
    Comma,
    Colon,
    SemiColon,
    Less,
    Asterisk,
    // Reserved keywords
    KwFn,
    KwLet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokens<'a> {
    tokens: &'a [Token<'a>],
    start: usize,
    end: usize,
}

impl<'a> Tokens<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            tokens,
            start: 0,
            end: tokens.len(),
        }
    }
}


impl<'a, 'b> Compare<Token<'b>> for Tokens<'a> {
    #[inline(always)]
    fn compare(&self, t: Token<'b>) -> nom::CompareResult {
        match self.iter_elements()
            .next()
            .map(| elem | elem.eq(&t)) 
        {
            Some(true) => CompareResult::Ok,
            Some(false) => CompareResult::Error,
            None => CompareResult::Incomplete
        }
    }

    #[inline(always)]
    fn compare_no_case(&self, t: Token<'b>) -> nom::CompareResult {
        panic!("token could not `compare_no_case`")
    }
}

impl<'a> InputLength for Token<'a> {
    fn input_len(&self) -> usize {
        1
    }
}

impl<'a> InputLength for Tokens<'a> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl<'a> InputTake for Tokens<'a> {
    fn take(&self, count: usize) -> Self {
        Tokens {
            tokens: &self.tokens[0..count],
            start: 0,
            end: count,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.tokens.split_at(count);
        let prefix = Tokens {
            tokens: prefix,
            start: 0,
            end: prefix.len(),
        };
        let suffix = Tokens {
            tokens: suffix,
            start: 0,
            end: suffix.len(),
        };
        (suffix, prefix)
    }
}

impl<'a> InputIter for Tokens<'a> {
    type Item = &'a Token<'a>;
    type Iter = Enumerate<Iter<'a, Token<'a>>>;
    type IterElem = Iter<'a, Token<'a>>;

    fn iter_indices(&self) -> Self::Iter {
        self.tokens.iter().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.tokens.iter()
    }

    fn position<P>(&self, pred: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.tokens.iter().position(pred)
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.tokens.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.tokens.len()))
        }
    }
}

impl<'a> Slice<Range<usize>> for Tokens<'a> {
    fn slice(&self, range: Range<usize>) -> Self {
        let start = self.start + range.start;
        let end = self.start + range.end;
        let slice = &self.tokens[range];
        Tokens {
            tokens: slice,
            start,
            end,
        }
    }
}

impl<'a> Slice<RangeTo<usize>> for Tokens<'a> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        self.slice(0..range.end)
    }
}

impl<'a> Slice<RangeFrom<usize>> for Tokens<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.slice(range.start..self.end - self.start)
    }
}

impl<'a> Slice<RangeFull> for Tokens<'a> {
    fn slice(&self, _: RangeFull) -> Self {
        Tokens {
            tokens: &self.tokens,
            start: self.start,
            end: self.end,
        }
    }
}
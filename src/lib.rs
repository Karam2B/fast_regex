#![allow(unused)]
#![warn(unused_must_use)]
use std::{
    iter::Peekable,
    mem,
    net::ToSocketAddrs,
    process::Child,
    range::Range,
    str::{CharIndices, Chars},
};

mod ideal_matching;

pub use ideal_matching::{double_u_one_i, repeat_alphanumeric as r#match};

pub struct CharsIndex<'a> {
    pub chars: Chars<'a>,
    pub index: usize,
}

pub enum Control {
    Shift(usize),
    Continue,
    Break,
}

use crate::Control::Shift;

pub trait WithCharsIndex {
    /// Takes a mutable reference for index and char iterator
    /// If no match, return None and increment the index
    /// If matches, return Some(shift) of the number of bytes to shift the index
    fn with_chars_index_shift(&self, chars_index: &mut CharsIndex) -> Control;
}

pub trait SingleCharMatch {
    fn single_char_match(&self, c: &char) -> bool;
}

pub struct WhiteSpace;

impl SingleCharMatch for WhiteSpace {
    fn single_char_match(&self, c: &char) -> bool {
        c.is_whitespace()
    }
}

pub struct Alphanumeric;

impl SingleCharMatch for Alphanumeric {
    fn single_char_match(&self, c: &char) -> bool {
        c.is_alphanumeric()
    }
}

pub struct Character<const CHAR: char>;

impl<const CHAR: char> SingleCharMatch for Character<CHAR> {
    fn single_char_match(&self, c: &char) -> bool {
        *c == CHAR
    }
}

pub struct AnyCharacter;

impl SingleCharMatch for AnyCharacter {
    fn single_char_match(&self, c: &char) -> bool {
        true
    }
}

pub struct Nest<T>(pub T);

pub struct Sequence<T>(pub T);

impl<R0> WithCharsIndex for Sequence<(R0,)>
where
    R0: SingleCharMatch,
{
    fn with_chars_index_shift(&self, chars_index: &mut CharsIndex) -> Control {
        let next = if let Some(s) = chars_index.chars.next() {
            s
        } else {
            return Control::Break;
        };
        let len = next.len_utf8();
        if self.0.0.single_char_match(&next) {
            // wrapped with ManyMatches, it will
            //  1. recieve the len
            //  2. return range from index to index + len
            //  3. incr index
            Control::Shift(len)
        } else {
            // non matching, increment index and return None
            // ManyMatches will keep this in a loop.
            chars_index.index += len;
            Control::Continue
        }
    }
}

impl<R0, R1> WithCharsIndex for Sequence<(R0, R1)>
where
    R0: SingleCharMatch,
    R1: SingleCharMatch,
{
    fn with_chars_index_shift(&self, chars_index: &mut CharsIndex) -> Control {
        let mut peek = if let Some(c) = chars_index.chars.next() {
            c
        } else {
            return Control::Break;
        };

        while let Some(s) = chars_index.chars.next() {
            if self.0.0.single_char_match(&peek) && self.0.1.single_char_match(&s) {
                let shift = peek.len_utf8() + s.len_utf8();
                peek = if let Some(c) = chars_index.chars.next() {
                    c
                } else {
                    return Control::Shift(shift);
                };
                return Control::Shift(shift);
            } else {
                chars_index.index += peek.len_utf8();
                peek = s;
            }
        }

        Control::Continue
    }
}

impl<N0, N1> WithCharsIndex for Sequence<(Nest<N0>, Nest<N1>)>
where
    (Nest<N0>, Nest<N1>): SequentialPeek,
{
    fn with_chars_index_shift(&self, chars_index: &mut CharsIndex) -> Control {
        let mut peek = self.0.default();

        if let Some(_) = self.0.reinit(&mut *chars_index, &mut peek) {
        } else {
            return Control::Break;
        }

        loop {
            if self.0.matches(&peek) {
                let shift: usize = self.0.shift(&peek);
                return Control::Shift(shift);
            } else {
                let success = self.0.rotate_move_index(&mut *chars_index, &mut peek);

                if success.is_none() {
                    break;
                }
            }
        }

        Control::Break
    }
}

pub trait SequentialPeek {
    type Peek;

    fn default(&self) -> Self::Peek;
    fn shift(&self, peek: &Self::Peek) -> usize;
    fn matches(&self, peek: &Self::Peek) -> bool;
    fn reinit(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()>;
    fn rotate_move_index(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()>;
    fn rotate(&self, next: char, peek: &mut Self::Peek) -> char;
}

impl SequentialPeek for () {
    type Peek = ();

    fn default(&self) -> Self::Peek {}

    fn shift(&self, peek: &Self::Peek) -> usize {
        0
    }

    fn matches(&self, peek: &Self::Peek) -> bool {
        true
    }
    fn reinit(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        Some(())
    }
    fn rotate_move_index(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        Some(())
    }
    fn rotate(&self, next: char, peek: &mut Self::Peek) -> char {
        next
    }
}

impl<R0> SequentialPeek for (R0,)
where
    R0: SingleCharMatch,
{
    type Peek = [char; 1];
    fn default(&self) -> Self::Peek {
        let mut peek1: [char; 1] = Default::default();

        peek1
    }
    fn shift(&self, peek: &Self::Peek) -> usize {
        peek[0].len_utf8()
    }
    fn matches(&self, peek: &Self::Peek) -> bool {
        self.0.single_char_match(&peek[0])
    }
    fn reinit(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        if let Some(s) = ctx.chars.next() {
            peeks[0] = s;
            Some(())
        } else {
            return None;
        }
    }
    fn rotate_move_index(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        if let Some(s) = ctx.chars.next() {
            peeks[0] = s;
            Some(())
        } else {
            return None;
        }
    }
    fn rotate(&self, next: char, peek: &mut Self::Peek) -> char {
        std::mem::replace(&mut peek[0], next)
    }
}

impl<R0, R1> SequentialPeek for (R0, R1)
where
    R0: SingleCharMatch,
    R1: SingleCharMatch,
{
    type Peek = [char; 2];
    fn default(&self) -> Self::Peek {
        [Default::default(), Default::default()]
    }

    fn shift(&self, peek: &Self::Peek) -> usize {
        peek[0].len_utf8() + peek[1].len_utf8()
    }

    fn matches(&self, peek: &Self::Peek) -> bool {
        self.0.single_char_match(&peek[0]) && self.1.single_char_match(&peek[1])
    }
    fn reinit(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        for i in 0..2usize {
            if let Some(s) = ctx.chars.next() {
                peeks[i] = s;
            } else {
                return None;
            }
        }
        Some(())
    }
    fn rotate_move_index(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        let dropping = peeks[0];
        if let Some(s) = ctx.chars.next() {
            *peeks = [peeks[1], s];
            ctx.index += dropping.len_utf8();
            Some(())
        } else {
            return None;
        }
    }
    fn rotate(&self, next: char, peek: &mut Self::Peek) -> char {
        let next = mem::replace(&mut peek[1], next);
        let next = mem::replace(&mut peek[0], next);
        next
    }
}

impl<R0> SequentialPeek for (Nest<R0>,)
where
    R0: SequentialPeek,
{
    type Peek = R0::Peek;
    fn default(&self) -> Self::Peek {
        self.0.0.default()
    }
    fn shift(&self, peek: &Self::Peek) -> usize {
        self.0.0.shift(peek)
    }
    fn matches(&self, peek: &Self::Peek) -> bool {
        self.0.0.matches(peek)
    }
    fn reinit(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        self.0.0.reinit(ctx, peeks)
    }
    fn rotate_move_index(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        self.0.0.rotate_move_index(ctx, peeks)
    }
    fn rotate(&self, next: char, peek: &mut Self::Peek) -> char {
        self.0.0.rotate(next, peek)
    }
}

impl<R0, R1> SequentialPeek for (Nest<R0>, Nest<R1>)
where
    R0: SequentialPeek,
    R1: SequentialPeek,
{
    type Peek = (R0::Peek, R1::Peek);
    fn default(&self) -> Self::Peek {
        (self.0.0.default(), self.1.0.default())
    }

    fn shift(&self, peek: &Self::Peek) -> usize {
        self.0.0.shift(&peek.0) + self.1.0.shift(&peek.1)
    }
    fn matches(&self, peek: &Self::Peek) -> bool {
        self.0.0.matches(&peek.0) && self.1.0.matches(&peek.1)
    }
    fn reinit(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        self.0.0.reinit(&mut *ctx, &mut peeks.0)?;
        self.1.0.reinit(&mut *ctx, &mut peeks.1)?;
        Some(())
    }

    fn rotate_move_index(&self, ctx: &mut CharsIndex, peeks: &mut Self::Peek) -> Option<()> {
        let nex = if let Some(s) = ctx.chars.next() {
            s
        } else {
            return None;
        };

        let first_from_second: char = self.1.0.rotate(nex, &mut peeks.1);
        let first_from_first: char = self.0.0.rotate(first_from_second, &mut peeks.0);

        ctx.index += first_from_first.len_utf8();

        Some(())
    }
    fn rotate(&self, next: char, peek: &mut Self::Peek) -> char {
        let next = self.1.0.rotate(next, &mut peek.1);
        self.0.0.rotate(next, &mut peek.0)
    }
}

pub struct OneMatch<R>(pub R);

impl<R> OneMatch<R> {
    pub fn find<'a>(&self, input: &'a str) -> Option<&'a str> {
        todo!("OneMatch::find")
    }
}

pub struct ManyMatches<R>(pub R);

impl<R> ManyMatches<R> {
    pub fn find<'a>(&self, input: &'a str) -> Vec<&'a str>
    where
        R: WithCharsIndex,
    {
        let mut result = Vec::new();
        let chars = input.chars();
        let index = 0usize;

        let mut chars_index = CharsIndex { chars, index };

        loop {
            match self.0.with_chars_index_shift(&mut chars_index) {
                Control::Shift(shift) => {
                    result.push(&input[chars_index.index..(chars_index.index + shift)]);
                    chars_index.index += shift;
                }
                Control::Continue => {
                    continue;
                }
                Control::Break => {
                    break;
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod main_tests {
    use super::*;

    #[test]
    fn one_match_one_char() {
        let regex = ManyMatches(Sequence((Character::<'u'>,)));
        let input = "the quick brown fox jumps over the lazy dog";
        let result = regex.find(input);

        assert_eq!(result, vec!("u", "u"));

        let regex = ManyMatches(Sequence((Character::<'u'>, Character::<'i'>)));
        let input = "the quick brown fox jumps over the lazy dog";
        let result = regex.find(input);

        assert_eq!(result, vec!("ui"));

        let regex = ManyMatches(Sequence((
            Nest((Character::<'u'>, Character::<'i'>)),
            Nest((Character::<'c'>,)),
        )));
        let input = "the quick brown fox jumps over the lazy dog";
        let result = regex.find(input);

        assert_eq!(result, vec!("uic"));

        let regex = ManyMatches(Sequence((
            Nest((
                Nest((Character::<'i'>, Character::<'c'>)),
                Nest(()), // empty is ok
            )),
            Nest((
                Nest((Character::<'k'>, Character::<' '>)),
                Nest((Character::<'b'>,)), // one is ok
            )),
        )));
        let input = "the quick brown fox jumps over the lazy dog";
        let result = regex.find(input);

        assert_eq!(result, vec!("ick b"));
    }

    fn main() {
        // \w\w\w\s
        let regex = ManyMatches(Sequence((Alphanumeric, WhiteSpace)));
        let input = "the quick brown fox jumps over the lazy dog";
        let result = regex.find(input);

        assert_eq!(result, vec!["e ", "k "]);
    }

    fn uft8_support() {
        let str = "bár";
        assert_eq!("b", &str[..1]);
        assert_eq!("á", &str[1..3]); // indexing is byte-wise!
        assert_eq!("r", &str[3..]);

        let s = str.chars().enumerate().map(|e| e.0).collect::<Vec<_>>();
        assert_eq!(s, vec![1, 2, 3]) // enumerate's usize is how many times `next` was called
    }
}

mod pointer_approch;

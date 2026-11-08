#![allow(unused)]
#![warn(unused_must_use)]
use std::{iter::Peekable, net::ToSocketAddrs, str::Chars};

macro_rules! regex {
    ($regex:tt) => {};
}

mod ideal_matching;

pub use ideal_matching::word as r#match;

/// this is how `/\w+/g` **SHOULD** work in my crate
//
/// This is 300% faster than other regex crates, my goal now
///
/// I believe that I can use idiomatic rust to implement this super fast regex.

pub struct Context<'a, Peek> {
    chars: Chars<'a>,
    utf8_index: usize,
    peek: Peek,
}

impl<'a, Peek: Default> Context<'a, Peek> {
    fn new(str: &'a str) -> Self {
        Self {
            chars: str.chars(),
            utf8_index: 0,
            peek: Peek::default(),
        }
    }

    fn next(&mut self) -> Option<char> {
        let n = self.chars.next();
        if let Some(char) = &n {
            self.utf8_index += char.len_utf8();
        }
        n
    }
}

pub trait Regex {
    fn match_(&self, ctx: &mut Context<'_, ()>);
}

pub struct WhiteSpace;

impl Regex for WhiteSpace {
    fn match_(&self, ctx: &mut Context<'_, ()>) {
        while let Some(char) = ctx.next() {
            if char.is_whitespace() {
                continue;
            } else {
                break;
            }
        }
    }
}

pub struct WordCharacter;

// impl Regex for

pub struct Character<const CHAR: char>;

pub struct RunOperation<
    // true: stop at first match
    // false: find all matches
    const Count: bool,
    // the regex
    R,
>(pub R);

impl<R> RunOperation<true, R> {
    fn regex(input: &str) -> Option<&str> {
        let mut result = None::<&str>;

        result
    }
}

impl<R> RunOperation<false, R> {
    fn regex(input: &str) -> Vec<&str> {
        let mut result = Vec::<&str>::new();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main() {
        let mut chars = "the quick brown hhe fox jumps over the lazy dog"
            .chars()
            .peekable();

        // \w\w\w\s
        let regex = (WordCharacter, WordCharacter, WordCharacter, WhiteSpace);

        let result =
            RunOperation::<true, _>(regex).regex("the quick brown fox jumps over the lazy dog");

        assert_eq!(result, Some("the "))
    }

    #[test]
    fn uft8_support() {
        let str = "bár";
        assert_eq!("b", &str[..1]);
        assert_eq!("á", &str[1..3]); // indexing is byte-wise!
        assert_eq!("r", &str[3..]);

        let s = str.chars().enumerate().map(|e| e.0).collect::<Vec<_>>();
        assert_eq!(s, vec![1, 3, 4])
    }
}

use std::{iter::Peekable, str::Chars};

/// this is how `/w+` **SHOULD** work in my crate
///
/// This is 100% faster than regex crate.
///
/// I believe that I can use idiomatic rust to implement this super fast regex.
pub fn r#match(str: String) -> Vec<String> {
    let mut result = Vec::new();
    let mut start = 0;
    for (index, char) in str.chars().enumerate() {
        if char.is_alphanumeric() {
            if index != start {
                continue;
            } else {
                start = index;
            }
        } else {
            if index != start {
                result.push(str[start..index].to_string());
            }
            start = index + 1;
        }
    }
    result
}

pub trait Pattern {
    fn matches(&self, chars: &mut Peekable<Chars<'_>>) -> bool;
}

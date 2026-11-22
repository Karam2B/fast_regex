//! This is a different approach to the problem that uses
//! another pointer to represent a portion of the string
//! that has been "peeked"

use core::fmt;
use std::{cmp, marker::PhantomData, ptr::NonNull};

/// Iterator by bytes over string with the ability to peek ahead and match start
///
/// This should conceptually work similar to `(std::slice::Iter<u8>, Vec<char>)`
/// but designed to be more efficient.
struct Chars<'a> {
    /// The pointer to the next element, or the past-the-end location
    /// if the iterator is empty
    next: NonNull<u8>,

    /// How much next has been shifted forward
    ///
    /// optimization opportunity: always 0 when looking for one match
    /// would the compiler dismiss this field?
    next_shift: usize,

    /// The non-null pointer to the past-the-end element.
    past_the_end: *const u8,

    /// The pointer to the peeked element
    // peek_shift: usize,
    peek_len: usize,

    /// shift to the peeked element
    ///
    /// optimization opportunity: always 0 when matching one pattern
    /// would the compiler dismiss this field?
    peek_shift: usize,

    /// Act as a reference `&'a [u8]`
    _marker: PhantomData<&'a u8>,
}

#[must_use]
enum OpResult {
    Success,
    Failed,
}

#[derive(Eq)]
pub struct PinnedRange<'a> {
    pub range: std::ops::Range<usize>,
    // keep string from being mutated or moved while holding the range
    pub pinned: PhantomData<&'a u8>,
}

impl cmp::PartialEq for PinnedRange<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range
    }
}

impl fmt::Debug for PinnedRange<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PinnedRange").field(&self.range).finish()
    }
}

impl<'a> Chars<'a> {
    pub fn new(str: &'a str) -> Self {
        let search_byte = str.as_bytes();
        let len = search_byte.len();
        let next = NonNull::from_ref(search_byte).cast::<u8>();
        let past_the_end = unsafe { next.as_ptr().add(len) };
        Self {
            next,
            past_the_end,
            peek_len: 0,
            peek_shift: 0,
            next_shift: 0,
            _marker: PhantomData,
        }
    }

    pub fn get_str_from_point_to_peek(&self, point: usize) -> &'a str {
        let slice: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self.next.as_ptr().add(point),
                self.peek_len + self.peek_shift - point,
            )
        };

        let str = unsafe { std::str::from_utf8_unchecked(slice) };

        str
    }

    pub fn view_first_peeked_byte(&self) -> &u8 {
        if self.peek_len != 1 {
            panic!("peek length is not 1: {:?}", self.peek_len);
        }
        unsafe { &*self.next.as_ptr().add(self.peek_shift) }
    }

    pub fn peek(&mut self, len: usize) -> OpResult {
        if self.next.as_ptr() as usize + len > self.past_the_end as usize {
            return OpResult::Failed;
        }

        self.peek_len += len;
        OpResult::Success
    }

    // reset past peek operation, this keeps the shift
    pub fn reset_peek(&mut self) {
        self.peek_len = 0;
    }

    pub fn reset_shift_peek(&mut self) {
        self.peek_shift = 0;
    }

    pub fn shift_peek(&mut self, shift: usize) -> OpResult {
        if self.next.as_ptr() as usize + self.peek_shift + self.peek_len + shift
            > self.past_the_end as usize
        {
            return OpResult::Failed;
        }

        self.peek_shift += self.peek_len;
        self.peek_len = shift;
        OpResult::Success
    }

    /// reset old `peek` operation and start new
    pub fn reset_and_start_new_peek(&mut self, len: usize) -> OpResult {
        self.peek_shift = 0;
        self.peek_len = 0;
        self.peek(len)
    }

    pub fn shift_iterator_forward(&mut self) -> OpResult {
        if self.next.as_ptr() as usize + 1 + self.peek_shift + self.peek_len
            > self.past_the_end as usize
        {
            return OpResult::Failed;
        }

        self.next = unsafe { self.next.add(1) };
        self.next_shift += 1;
        OpResult::Success
    }

    pub fn get_range_from_next_to_peek(&self) -> PinnedRange<'a> {
        let start = self.next_shift;
        PinnedRange {
            range: start..start + self.peek_len + self.peek_shift,
            pinned: PhantomData,
        }
    }

    pub fn get_str_from_next_to_peek(&self) -> &'a str {
        let range = 0usize..self.peek_len + self.peek_shift;

        let slice: &[u8] = unsafe {
            std::slice::from_raw_parts(self.next.as_ptr(), self.peek_len + self.peek_shift)
        };

        let str = unsafe { std::str::from_utf8_unchecked(slice) };

        str
    }

    pub fn set_next_to_peek(&mut self) {
        self.next = unsafe { self.next.add(self.peek_len + self.peek_shift) };
        self.peek_len = 0;
        self.peek_shift = 0;
    }
    pub fn compare_peeked(&self, pattern_bytes: &[u8]) -> bool {
        if pattern_bytes.len() != self.peek_len {
            panic!("pattern bytes length does not match peek length");
        }
        let mut count = 1usize;
        for byte in pattern_bytes.into_iter().rev() {
            // compare last first
            if unsafe {
                // attempt to subtract with overflow: patter len is always greater that peek_len, so this will never happen
                *self
                    .next
                    .as_ptr()
                    .add(self.peek_shift + self.peek_len - count)
            } != *byte
            {
                return false;
            }
            count += 1;
        }

        true
    }
}

#[test]
fn match_el_test() {
    let search = "hello woeld";
    let result = many_matches_el(search);
    assert_eq!(result, vec!["el", "el"]);
}

pub fn many_matches_el<'a>(search: &'a str) -> Vec<&'a str> {
    let pattern = "el";
    let pattern_bytes = pattern.as_bytes();
    let mut result: Vec<&str> = Vec::new();

    let mut chars = Chars::new(search);

    // to search for `el` in `hello woeld`:

    // advance the peek 2 letters `peek`
    match chars.peek(pattern.len()) {
        OpResult::Success => {}
        OpResult::Failed => {
            return result;
        }
    }

    loop {
        if chars.compare_peeked(pattern_bytes) {
            // if one match is needed, return the result
            result.push(chars.get_str_from_next_to_peek());

            // if more that one match is needed:
            chars.set_next_to_peek();
            match chars.peek(pattern.len()) {
                OpResult::Success => {}
                OpResult::Failed => {
                    return result;
                }
            }
        } else {
            match chars.shift_iterator_forward() {
                OpResult::Success => {}
                OpResult::Failed => {
                    return result;
                }
            }
        }
    }
}

#[test]
fn match_on_repeat_test() {
    let search = "1___2  1__2   1____2";
    let result = many_on_repeat(search);
    assert_eq!(result, vec!["1__2", "1____2"]);
}

#[derive(Debug, PartialEq)]
pub struct MatchResult<'a, C> {
    pub entire_match: &'a str,
    pub captures: C,
}

/// capture year `date(\d)*-`
/// for a pattern like: `date2026-11-22`
/// will capture only `2026`
pub fn basic_capture_group<'a>(search: &'a str) -> Vec<MatchResult<'a, &'a str>> {
    let first_pattern = "date";
    let pattern_bytes = first_pattern.as_bytes();

    let mut result = Vec::<MatchResult<'a, &'a str>>::new();

    let mut chars = Chars::new(search);

    match chars.peek(pattern_bytes.len()) {
        OpResult::Success => {}
        OpResult::Failed => {
            return result;
        }
    }

    'main: loop {
        if chars.compare_peeked(pattern_bytes) {
            // second pattern: `(\d)*`
            let capture_2 = 'all_captures: {
                Some((
                    'second_capture: {
                        let point = chars.peek_shift + chars.peek_len;
                        match chars.shift_peek(1) {
                            OpResult::Success => {}
                            OpResult::Failed => {
                                return result;
                            }
                        }
                        'sup: loop {
                            if !(b'0'..=b'9').contains(chars.view_first_peeked_byte()) {
                                chars.reset_peek();
                                break 'sup;
                            }
                            if let OpResult::Failed = chars.shift_peek(1) {
                                break 'sup;
                            }
                        }
                        chars.get_str_from_point_to_peek(point)
                    },
                    'third_capture: {
                        let pattern = "-";
                        let pattern_bytes = pattern.as_bytes();
                        match chars.shift_peek(pattern_bytes.len()) {
                            OpResult::Success => {}
                            OpResult::Failed => {
                                return result;
                            }
                        }

                        if !chars.compare_peeked(pattern_bytes) {
                            break 'all_captures None;
                        }
                    },
                ))
            };

            if let Some((year, _)) = capture_2 {
                result.push(MatchResult {
                    entire_match: chars.get_str_from_next_to_peek(),
                    captures: year,
                });
                chars.set_next_to_peek();
                match chars.peek(pattern_bytes.len()) {
                    OpResult::Success => {}
                    OpResult::Failed => return result,
                }
            } else {
                match chars.reset_and_start_new_peek(pattern_bytes.len()) {
                    OpResult::Success => {}
                    OpResult::Failed => return result,
                }
                match chars.shift_iterator_forward() {
                    OpResult::Success => continue 'main,
                    OpResult::Failed => return result,
                }
            }
        } else {
            match chars.shift_iterator_forward() {
                OpResult::Success => {}
                OpResult::Failed => {
                    return result;
                }
            }
        }
    }

    panic!("should be unreachable");
}

#[test]
fn basic_capture_group_test() {
    let search = "date2026-11-22";
    let result = basic_capture_group(search);
    assert_eq!(
        result,
        vec![MatchResult {
            entire_match: "date2026-",
            captures: "2026",
        }]
    );
}

/// `1(__)*2`
pub fn many_on_repeat(search: &str) -> Vec<&str> {
    let first_pattern = "1";
    let pattern_bytes = first_pattern.as_bytes();

    let mut result = Vec::<&str>::new();

    let mut chars = Chars::new(search);

    match chars.peek(pattern_bytes.len()) {
        OpResult::Success => {}
        OpResult::Failed => {
            return result;
        }
    }

    'main: loop {
        if chars.compare_peeked(pattern_bytes) {
            // second pattern: `(__)*`
            let inner_result = 'inner: {
                {
                    let pattern = "__";
                    let pattern_bytes = pattern.as_bytes();

                    match chars.shift_peek(pattern_bytes.len()) {
                        OpResult::Success => {}
                        OpResult::Failed => {
                            return result;
                        }
                    }

                    'sub: loop {
                        if chars.compare_peeked(pattern_bytes) {
                            match chars.shift_peek(pattern_bytes.len()) {
                                OpResult::Success => {}
                                OpResult::Failed => break 'sub,
                            }
                        } else {
                            // Drop the failed `__` and keep the text matched so far.
                            chars.reset_peek();
                            break;
                        }
                    }
                };
                // third pattern
                {
                    let pattern = "2";
                    let pattern_bytes = pattern.as_bytes();
                    match chars.shift_peek(pattern_bytes.len()) {
                        OpResult::Success => {}
                        OpResult::Failed => {
                            return result;
                        }
                    }

                    if !chars.compare_peeked(pattern_bytes) {
                        break 'inner false;
                    }
                };
                true
            };

            if inner_result {
                // if one match is needed, return the result
                // if more that one match is needed:
                result.push(chars.get_str_from_next_to_peek());
                chars.set_next_to_peek();
                match chars.peek(pattern_bytes.len()) {
                    OpResult::Success => {}
                    OpResult::Failed => {
                        return result;
                    }
                }
            } else {
                chars.reset_peek();
                chars.reset_shift_peek();
                match chars.peek(pattern_bytes.len()) {
                    OpResult::Success => {}
                    OpResult::Failed => {
                        return result;
                    }
                }
                match chars.shift_iterator_forward() {
                    OpResult::Success => continue 'main,
                    OpResult::Failed => return result,
                }
            }
        } else {
            match chars.shift_iterator_forward() {
                OpResult::Success => {}
                OpResult::Failed => {
                    return result;
                }
            }
        }
    }

    panic!("should be unreachable");
}

/// pattern `(__)*2`
pub fn many_on_repeat_2(search: &str) -> Vec<&str> {
    let mut result = Vec::<&str>::new();
    let mut chars = Chars::new(search);

    loop {
        // `(__)*` — a failed peek means there is not enough text for another pair.
        if let OpResult::Success = chars.peek(2) {
            loop {
                if chars.compare_peeked(b"__") {
                    match chars.shift_peek(2) {
                        OpResult::Success => {}
                        OpResult::Failed => break,
                    }
                } else {
                    chars.reset_peek();
                    break;
                }
            }
        }

        match chars.shift_peek(1) {
            OpResult::Success => {}
            OpResult::Failed => return result,
        }

        if chars.compare_peeked(b"2") {
            result.push(chars.get_str_from_next_to_peek());
            chars.set_next_to_peek();
            continue;
        }

        match chars.shift_iterator_forward() {
            OpResult::Success => {}
            OpResult::Failed => return result,
        }
        chars.reset_peek();
    }
}

#[test]
fn many_on_repeat_2_test() {
    let search = "1___2  1__2   1____2";
    let result = many_on_repeat_2(search);
    assert_eq!(result, vec!["__2", "__2", "____2"]);
}

/// pattern `(__)?2`
pub fn many_on_repeat_3(search: &str) -> Vec<&str> {
    let mut result = Vec::<&str>::new();
    let mut chars = Chars::new(search);

    loop {
        // `(__)?` — keep one pair when it matches, otherwise test `2` at `next`.
        if let OpResult::Success = chars.peek(2) {
            if !chars.compare_peeked(b"__") {
                chars.reset_peek();
            }
        }

        match chars.shift_peek(1) {
            OpResult::Success => {}
            OpResult::Failed => return result,
        }

        if chars.compare_peeked(b"2") {
            result.push(chars.get_str_from_next_to_peek());
            chars.set_next_to_peek();
            continue;
        }

        match chars.shift_iterator_forward() {
            OpResult::Success => {}
            OpResult::Failed => return result,
        }
        chars.reset_peek();
    }
}

#[test]
fn many_on_repeat_3_test() {
    let search = "2  1__2   1____2";
    let result = many_on_repeat_3(search);
    assert_eq!(result, vec!["2", "__2", "__2"]);
}

/// pattern `(__){2,3}2`
pub fn many_on_repeat_4(search: &str) -> Vec<&str> {
    let mut result = Vec::<&str>::new();
    let mut chars = Chars::new(search);

    loop {
        // Take two `__` pairs, and a third when it is present.
        let mut pairs = 0usize;
        if let OpResult::Success = chars.peek(2) {
            loop {
                if !chars.compare_peeked(b"__") {
                    chars.reset_peek();
                    break;
                }
                pairs += 1;
                if pairs == 3 {
                    break;
                }
                match chars.shift_peek(2) {
                    OpResult::Success => {}
                    OpResult::Failed => break,
                }
            }
        }

        if pairs >= 2 {
            let took_third = pairs == 3;
            match chars.shift_peek(1) {
                OpResult::Success => {}
                OpResult::Failed => return result,
            }
            if chars.compare_peeked(b"2") {
                result.push(chars.get_str_from_next_to_peek());
                chars.set_next_to_peek();
                continue;
            }
            if took_third {
                // The third pair ate the byte where `2` has to sit. Try exactly two pairs.
                match chars.reset_and_start_new_peek(2) {
                    OpResult::Success => {}
                    OpResult::Failed => return result,
                }
                if chars.compare_peeked(b"__") {
                    match chars.shift_peek(2) {
                        OpResult::Success if chars.compare_peeked(b"__") => {
                            match chars.shift_peek(1) {
                                OpResult::Success => {}
                                OpResult::Failed => return result,
                            }
                            if chars.compare_peeked(b"2") {
                                result.push(chars.get_str_from_next_to_peek());
                                chars.set_next_to_peek();
                                continue;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        match chars.reset_and_start_new_peek(1) {
            OpResult::Success => {}
            OpResult::Failed => return result,
        }
        match chars.shift_iterator_forward() {
            OpResult::Success => {}
            OpResult::Failed => return result,
        }
        chars.reset_peek();
    }
}

#[test]
fn many_on_repeat_4_test() {
    let search = "1__2  1____2   1______2  1________2";
    let result = many_on_repeat_4(search);
    assert_eq!(result, vec!["____2", "______2", "______2"]);
}

pub struct MatchMany<Pattern>(pub Pattern);

pub struct MatchOne<Pattern>(pub Pattern);

pub struct LiteralPattern<'a>(pub &'a [u8], pub usize);

impl<T> MatchMany<T> {
    fn search<'a>(self, search: &'a str) -> Vec<Match<PinnedRange<'a>, ()>> {
        let mut chars = Chars::new(search);
        let mut matches = Vec::<&str>::new();

        // prepare for first match

        loop {
            // check if match is there
            if true {
                matches.push("");

                // prepare for next match
            } else {
                // shift next forward
            }
        }

        panic!("should be unreachable, shifting should fail at some point")
    }
}

#[derive(Debug, PartialEq)]
pub struct Match<M, C> {
    pub entire_match: M,
    pub captures: C,
}

#[test]
fn literal_pattern_test() {
    let op = MatchMany(LiteralPattern(b"el", 1));
    let result = op.search("hello woeld");
    assert_eq!(
        result,
        vec![
            Match {
                entire_match: PinnedRange {
                    range: 1..2,
                    pinned: PhantomData,
                },
                captures: ()
            },
            Match {
                entire_match: PinnedRange {
                    range: 8..9,
                    pinned: PhantomData,
                },
                captures: ()
            },
        ]
    )
}

// impl<T> MatchOne<T> {
//     fn search(self, search: &str) -> Option<&str> {
//         let mut chars = Chars::new(search);

//         // prepare for first match

//         loop {
//             if true {
//                 return Some("");
//             } else {
//                 // shift next forward
//             }
//         }

//         return None;
//     }
// }

pub trait Pattern {
    fn pattern_peek(&self, chars: &mut Chars) -> OpResult;
}

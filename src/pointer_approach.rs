//! This is a different approach to the problem that uses
//! another pointer to represent a portion of the string
//! that has been "peeked"

use core::fmt;
use std::{cmp, marker::PhantomData, ptr::NonNull};

/// Iterator by bytes over string with the ability to peek ahead and match start
///
/// This should conceptually work similar to `(std::slice::Iter<u8>, Vec<char>)`
/// but designed to be more efficient.
pub struct Chars<'haystack> {
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
    _marker: PhantomData<&'haystack u8>,
}

#[must_use]
pub enum OpResult {
    Success,
    Failed,
}

#[derive(Eq)]
pub struct PinnedRange<'haystack> {
    pub range: std::ops::Range<usize>,
    // keep string from being mutated or moved while holding the range
    pub pinned: PhantomData<&'haystack u8>,
}

impl From<std::ops::Range<usize>> for PinnedRange<'_> {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self {
            range,
            pinned: PhantomData,
        }
    }
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

impl<'haystack> Chars<'haystack> {
    pub fn new(str: &'haystack str) -> Self {
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

    pub fn get_str_from_point_to_peek(&self, point: usize) -> &'haystack str {
        let slice: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self.next.as_ptr().add(point),
                self.peek_len + self.peek_shift - point,
            )
        };

        let str = unsafe { std::str::from_utf8_unchecked(slice) };

        str
    }

    pub fn view_first_peeked_byte(&self) -> &'haystack u8 {
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

        self.next_shift += self.peek_shift;
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

    pub fn get_range_from_next_to_peek(&self) -> PinnedRange<'haystack> {
        let start = self.next_shift;
        PinnedRange {
            range: start..start + self.peek_len + self.peek_shift,
            pinned: PhantomData,
        }
    }

    pub fn get_str_from_next_to_peek(&self) -> &'haystack str {
        let range = 0usize..self.peek_len + self.peek_shift;

        let slice: &[u8] = unsafe {
            std::slice::from_raw_parts(self.next.as_ptr(), self.peek_len + self.peek_shift)
        };

        let str = unsafe { std::str::from_utf8_unchecked(slice) };

        str
    }

    pub fn set_next_to_peek(&mut self) {
        let next_shift = self.peek_shift + self.peek_len;
        self.next_shift += next_shift;
        self.next = unsafe { self.next.add(next_shift) };
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

pub fn many_matches_el<'haystack>(search: &'haystack str) -> Vec<&'haystack str> {
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
pub struct MatchResult<'haystack, C> {
    pub entire_match: &'haystack str,
    pub captures: C,
}

/// capture year `date(\d)*-`
/// for a pattern like: `date2026-11-22`
/// will capture only `2026`
pub fn basic_capture_group<'haystack>(
    search: &'haystack str,
) -> Vec<MatchResult<'haystack, &'haystack str>> {
    let first_pattern = "date";
    let pattern_bytes = first_pattern.as_bytes();

    let mut result = Vec::<MatchResult<'haystack, &'haystack str>>::new();

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
                    OpResult::Success => {}
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

/// capture year `(date)(\d)*-`
/// for a pattern like: `date2026-11-22`
/// will capture only ("date", "2026")`
pub fn first_capture_group<'haystack>(
    search: &'haystack str,
) -> Vec<MatchResult<'haystack, (&'haystack str, &'haystack str)>> {
    let first_pattern = "date";
    let pattern_bytes = first_pattern.as_bytes();

    let mut result = Vec::<MatchResult<'haystack, (&'haystack str, &'haystack str)>>::new();

    let mut chars = Chars::new(search);

    let mut point = chars.peek_shift + chars.peek_len;
    match chars.peek(pattern_bytes.len()) {
        OpResult::Success => {}
        OpResult::Failed => {
            return result;
        }
    }

    'main: loop {
        if chars.compare_peeked(pattern_bytes) {
            let capture_1 = chars.get_str_from_point_to_peek(point);
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
                    captures: (capture_1, year),
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
                    OpResult::Success => {}
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
            // I don't think this does anything
            point = chars.peek_shift + chars.peek_len;
        }
    }

    panic!("should be unreachable");
}

#[test]
fn first_capture_group_test() {
    let search = "date2026-11-22";
    let result = first_capture_group(search);
    assert_eq!(
        result,
        vec![MatchResult {
            entire_match: "date2026-",
            captures: ("date", "2026"),
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
        } else {
            chars.reset_peek();
        }

        match chars.shift_iterator_forward() {
            OpResult::Success => {}
            OpResult::Failed => return result,
        }
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

pub struct LiteralPattern<'pattern>(pub &'pattern [u8], pub usize);

/// It has two use cases:
///     1. As a pattern to represent `pattern` or `(pattern){1}`
///     2. Control identifier used by `ManyMatch` and `MatchOne` to decide how the first pattern should be matched
///
pub struct OneRepitition<T>(pub T);

pub struct NoCapture<T>(pub T);

pub struct Capture<T>(pub T);

pub struct ZeroOrMoreRepitition<T>(pub T);

pub struct ConstBoundRepitition<T, const MIN: usize, const MAX: usize>(pub T);

impl<'haystack, T, R> MatchMany<(OneRepitition<T>, R)>
where
    T: Pattern<'haystack>,
    R: Pattern<'haystack>,
{
    fn search(
        self,
        search: &'haystack str,
    ) -> Vec<Match<PinnedRange<'haystack>, (T::Capture, R::Capture)>> {
        let mut chars = Chars::new(search);
        let mut matches = Vec::<Match<PinnedRange<'haystack>, (T::Capture, R::Capture)>>::new();

        // `init_capture` is the frontier before this peek. After `pattern_peek`
        // that same call points at the end, so the captured slice is empty.
        let mut capture_point = self.0.0.0.init_capture(&chars);
        let mut r_capture_point = self.0.1.init_capture(&chars);
        match self.0.0.0.pattern_peek(&mut chars) {
            OpResult::Success => {}
            OpResult::Failed => {
                return matches;
            }
        }

        loop {
            if self.0.0.0.pattern_compare(&mut chars) {
                let inner_matches = Some(());

                if let Some(_) = inner_matches {
                    let capture = self.0.0.0.pattern_capture(&mut chars, capture_point);
                    matches.push(Match {
                        entire_match: chars.get_range_from_next_to_peek(),
                        captures: (
                            capture,
                            self.0.1.pattern_capture(&mut chars, r_capture_point),
                        ),
                    });

                    chars.set_next_to_peek();
                    capture_point = self.0.0.0.init_capture(&chars);
                    r_capture_point = self.0.1.init_capture(&chars);

                    match self.0.0.0.pattern_peek(&mut chars) {
                        OpResult::Success => {}
                        OpResult::Failed => {
                            return matches;
                        }
                    }
                } else {
                    match self.0.0.0.pattern_reset_and_start_new_peek(&mut chars) {
                        OpResult::Success => {}
                        OpResult::Failed => {
                            return matches;
                        }
                    }
                    match chars.shift_iterator_forward() {
                        OpResult::Success => {}
                        OpResult::Failed => {
                            return matches;
                        }
                    }
                }
            } else {
                match chars.shift_iterator_forward() {
                    OpResult::Success => {}
                    OpResult::Failed => {
                        return matches;
                    }
                }
            }
        }
    }
}

impl<'haystack, T> MatchMany<(ZeroOrMoreRepitition<T>, ())>
where
    T: Pattern<'haystack>,
{
    fn search(self, search: &'haystack str) -> Vec<Match<PinnedRange<'haystack>, (T::Capture,)>> {
        let mut chars = Chars::new(search);
        let mut matches = Vec::<Match<PinnedRange<'haystack>, (T::Capture,)>>::new();

        loop {
            let capture_point = self.0.0.0.init_capture(&chars);
            if let OpResult::Success = self.0.0.0.pattern_peek(&mut chars) {
                loop {
                    if self.0.0.0.pattern_compare(&mut chars) {
                        match self.0.0.0.pattern_shift_peek(&mut chars) {
                            OpResult::Success => {}
                            OpResult::Failed => break,
                        }
                    } else {
                        chars.reset_peek();
                        break;
                    }
                }
            }

            let inner_matches = Some(());

            if let Some(_) = inner_matches {
                matches.push(Match {
                    entire_match: chars.get_range_from_next_to_peek(),
                    captures: (self.0.0.0.pattern_capture(&mut chars, capture_point),),
                });
                chars.set_next_to_peek();
                match chars.shift_iterator_forward() {
                    OpResult::Success => {}
                    OpResult::Failed => return matches,
                }
            } else {
                match chars.shift_iterator_forward() {
                    OpResult::Success => chars.reset_peek(),
                    OpResult::Failed => return matches,
                }
            };
        }
    }
}

pub trait Pattern<'haystack> {
    const WILL_CAPTURE: bool;
    type Capture;
    type CapturePoint;
    fn init_capture(&self, chars: &Chars<'haystack>) -> Self::CapturePoint;
    fn pattern_peek(&self, chars: &mut Chars) -> OpResult;
    fn pattern_match(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Option<Match<PinnedRange<'haystack>, Self::Capture>>;

    fn pattern_capture(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Self::Capture;

    fn pattern_match_no_capture(
        &self,
        chars: &mut Chars<'haystack>,
    ) -> Option<Match<PinnedRange<'haystack>, ()>>;
    fn pattern_compare(&self, chars: &mut Chars) -> bool;
    fn pattern_shift_peek(&self, chars: &mut Chars) -> OpResult;
    fn pattern_reset_and_start_new_peek(&self, chars: &mut Chars) -> OpResult {
        chars.reset_peek();
        chars.reset_shift_peek();
        self.pattern_peek(chars)
    }
}

impl<'haystack, T> Pattern<'haystack> for ZeroOrMoreRepitition<T>
where
    T: Pattern<'haystack>,
{
    const WILL_CAPTURE: bool = T::WILL_CAPTURE;

    type Capture = T::Capture;

    type CapturePoint = T::CapturePoint;

    fn init_capture(&self, chars: &Chars<'haystack>) -> Self::CapturePoint {
        todo!()
    }

    fn pattern_peek(&self, chars: &mut Chars) -> OpResult {
        todo!()
    }

    fn pattern_match(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Option<Match<PinnedRange<'haystack>, Self::Capture>> {
        todo!()
    }

    fn pattern_capture(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Self::Capture {
        todo!()
    }

    fn pattern_match_no_capture(
        &self,
        chars: &mut Chars<'haystack>,
    ) -> Option<Match<PinnedRange<'haystack>, ()>> {
        todo!()
    }

    fn pattern_compare(&self, chars: &mut Chars) -> bool {
        todo!()
    }

    fn pattern_shift_peek(&self, chars: &mut Chars) -> OpResult {
        todo!()
    }
}

impl<'haystack, T> Pattern<'haystack> for OneRepitition<T>
where
    T: Pattern<'haystack>,
{
    const WILL_CAPTURE: bool = T::WILL_CAPTURE;

    type Capture = T::Capture;

    type CapturePoint = T::CapturePoint;

    fn init_capture(&self, chars: &Chars<'haystack>) -> Self::CapturePoint {
        todo!()
    }

    fn pattern_peek(&self, chars: &mut Chars) -> OpResult {
        todo!()
    }

    fn pattern_match(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Option<Match<PinnedRange<'haystack>, Self::Capture>> {
        todo!()
    }

    fn pattern_capture(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Self::Capture {
        todo!()
    }

    fn pattern_match_no_capture(
        &self,
        chars: &mut Chars<'haystack>,
    ) -> Option<Match<PinnedRange<'haystack>, ()>> {
        todo!()
    }

    fn pattern_compare(&self, chars: &mut Chars) -> bool {
        todo!()
    }

    fn pattern_shift_peek(&self, chars: &mut Chars) -> OpResult {
        todo!()
    }
}

impl<'haystack, 'pattern> Pattern<'haystack> for LiteralPattern<'pattern> {
    const WILL_CAPTURE: bool = true;
    type Capture = &'haystack str;
    type CapturePoint = usize;
    fn init_capture(&self, chars: &Chars<'_>) -> Self::CapturePoint {
        chars.peek_shift + chars.peek_len
    }

    fn pattern_peek(&self, chars: &mut Chars) -> OpResult {
        chars.peek(self.1)
    }

    fn pattern_capture(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Self::Capture {
        chars.get_str_from_point_to_peek(init_capture)
    }

    fn pattern_reset_and_start_new_peek(&self, chars: &mut Chars) -> OpResult {
        chars.reset_and_start_new_peek(self.1)
    }

    fn pattern_match(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Option<Match<PinnedRange<'haystack>, Self::Capture>> {
        if chars.compare_peeked(self.0) {
            Some(Match {
                entire_match: chars.get_range_from_next_to_peek(),
                captures: chars.get_str_from_point_to_peek(init_capture),
            })
        } else {
            None
        }
    }

    fn pattern_match_no_capture(
        &self,
        chars: &mut Chars<'haystack>,
    ) -> Option<Match<PinnedRange<'haystack>, ()>> {
        if chars.compare_peeked(self.0) {
            Some(Match {
                entire_match: chars.get_range_from_next_to_peek(),
                captures: (),
            })
        } else {
            None
        }
    }

    fn pattern_compare(&self, chars: &mut Chars) -> bool {
        chars.compare_peeked(self.0)
    }

    fn pattern_shift_peek(&self, chars: &mut Chars) -> OpResult {
        chars.shift_peek(self.0.len())
    }
}

impl<'a, T> Pattern<'a> for NoCapture<T>
where
    T: Pattern<'a>,
{
    const WILL_CAPTURE: bool = false;
    type Capture = ();
    type CapturePoint = ();

    fn init_capture(&self, chars: &Chars<'a>) -> Self::CapturePoint {}

    fn pattern_peek(&self, chars: &mut Chars) -> OpResult {
        self.0.pattern_peek(chars)
    }

    fn pattern_capture(&self, _: &mut Chars<'a>, _: Self::CapturePoint) -> Self::Capture {}

    fn pattern_match(
        &self,
        chars: &mut Chars<'a>,
        _: Self::CapturePoint,
    ) -> Option<Match<PinnedRange<'a>, Self::Capture>> {
        self.0.pattern_match_no_capture(chars)
    }

    fn pattern_compare(&self, chars: &mut Chars) -> bool {
        self.0.pattern_compare(chars)
    }

    fn pattern_match_no_capture(
        &self,
        chars: &mut Chars<'a>,
    ) -> Option<Match<PinnedRange<'a>, ()>> {
        self.0.pattern_match_no_capture(chars)
    }

    fn pattern_shift_peek(&self, chars: &mut Chars) -> OpResult {
        self.0.pattern_shift_peek(chars)
    }

    fn pattern_reset_and_start_new_peek(&self, chars: &mut Chars) -> OpResult {
        self.0.pattern_reset_and_start_new_peek(chars)
    }
}

impl<'haystack> Pattern<'haystack> for () {
    const WILL_CAPTURE: bool = false;

    type Capture = ();

    type CapturePoint = ();

    fn init_capture(&self, _: &Chars<'haystack>) -> Self::CapturePoint {}

    fn pattern_peek(&self, _: &mut Chars) -> OpResult {
        OpResult::Success
    }

    fn pattern_match(
        &self,
        _: &mut Chars<'haystack>,
        _: Self::CapturePoint,
    ) -> Option<Match<PinnedRange<'haystack>, Self::Capture>> {
        None
    }

    fn pattern_capture(&self, _: &mut Chars<'haystack>, _: Self::CapturePoint) -> Self::Capture {}

    fn pattern_match_no_capture(
        &self,
        _: &mut Chars<'haystack>,
    ) -> Option<Match<PinnedRange<'haystack>, ()>> {
        None
    }

    fn pattern_compare(&self, _: &mut Chars) -> bool {
        true
    }

    fn pattern_shift_peek(&self, _: &mut Chars) -> OpResult {
        OpResult::Success
    }

    fn pattern_reset_and_start_new_peek(&self, _: &mut Chars) -> OpResult {
        OpResult::Success
    }
}

impl<'haystack, P0, P1> Pattern<'haystack> for (P0, P1)
where
    P0: Pattern<'haystack>,
    P1: Pattern<'haystack>,
{
    const WILL_CAPTURE: bool = P0::WILL_CAPTURE || P1::WILL_CAPTURE;

    type Capture = (P0::Capture, P1::Capture);

    type CapturePoint = (P0::CapturePoint, P1::CapturePoint);

    fn init_capture(&self, chars: &Chars<'haystack>) -> Self::CapturePoint {
        // (self.0.init_capture(chars), self.1.init_capture(chars))
        todo!()
    }

    fn pattern_peek(&self, chars: &mut Chars) -> OpResult {
        todo!()
        // if let OpResult::Failed = self.0.pattern_peek(chars) {
        //     return OpResult::Failed;
        // };
        // self.1.pattern_peek(chars)
    }

    fn pattern_match(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Option<Match<PinnedRange<'haystack>, Self::Capture>> {
        todo!()
    }

    fn pattern_capture(
        &self,
        chars: &mut Chars<'haystack>,
        init_capture: Self::CapturePoint,
    ) -> Self::Capture {
        todo!()
    }

    fn pattern_match_no_capture(
        &self,
        chars: &mut Chars<'haystack>,
    ) -> Option<Match<PinnedRange<'haystack>, ()>> {
        todo!()
    }

    fn pattern_compare(&self, chars: &mut Chars) -> bool {
        todo!()
    }

    fn pattern_shift_peek(&self, chars: &mut Chars) -> OpResult {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct Match<M, C> {
    pub entire_match: M,
    pub captures: C,
}

#[test]
fn literal_pattern_test() {
    let op = MatchMany(
        //
        (OneRepitition(LiteralPattern(b"el", 2)), ()),
    );
    let result = op.search("hello woeld");
    assert_eq!(
        result,
        vec![
            Match {
                entire_match: (1..3).into(),
                captures: ("el", ())
            },
            Match {
                entire_match: (8..10).into(),
                captures: ("el", ())
            },
        ]
    );

    let op = MatchMany(
        //
        (ZeroOrMoreRepitition(LiteralPattern(b"__", 2)), ()),
    );
    let result = op.search("1__2  1___2");
    assert_eq!(
        result,
        vec![
            Match {
                entire_match: (0..0).into(),
                captures: ("",)
            },
            Match {
                entire_match: (1..3).into(),
                captures: ("__",)
            },
            Match {
                entire_match: (4..4).into(),
                captures: ("",)
            },
            Match {
                entire_match: (5..5).into(),
                captures: ("",)
            },
            Match {
                entire_match: (6..6).into(),
                captures: ("",)
            },
            Match {
                entire_match: (7..9).into(),
                captures: ("__",)
            },
            Match {
                entire_match: (10..10).into(),
                captures: ("",)
            },
            Match {
                entire_match: (11..11).into(),
                captures: ("",)
            },
        ]
    );

    /// `(1)(__){0,}(2)`
    /// Input: `1___2 1__2`
    /// Match {
    ///     entire_match: 6..10,
    ///     captures: ("1", vec!("__"), "2")
    /// }
    ///
    ///
    let op = MatchMany(
        //
        (
            OneRepitition(LiteralPattern(b"1", 1)),
            (
                ZeroOrMoreRepitition(LiteralPattern(b"__", 2)),
                OneRepitition(LiteralPattern(b"2", 3)),
            ),
        ),
    );
    let result = op.search("1__2  1___2");
    assert_eq!(
        result,
        vec![Match {
            entire_match: (0..3).into(),
            captures: ("1", ("__", "2"))
        },]
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

//! This is a different approach to the problem that uses
//! another pointer to represent a portion of the string
//! that has been "peeked"

use std::{marker::PhantomData, ptr::NonNull};

pub struct Chars<'a> {
    // this contains one pointer that point to the start of the slice
    // and a metadata that represent the remaining length of the slice
    iter: std::slice::Iter<'a, u8>,
    // information about the portion that has been peeked
    peek: usize,
}

impl<'a> Chars<'a> {
    /// for now this can only be used for utf-8 strings,
    /// in the future I will define a trait that does this for
    /// any encoding
    pub fn new(str: &'a str) -> Self {
        todo!()
        // let peek = std::ptr::from_ref(str) as *const u8;
        // let iter = str.as_bytes().iter();
        // Self { iter, peek: 0 }
    }

    /// reset the peek pointer to the start of the string
    pub fn reset(&mut self) {
        todo!()
    }

    /// advance the iter pointer to the peek pointer
    pub fn set(&mut self) {
        todo!()
    }

    pub fn view_peeked(&self) -> &'a [u8] {
        todo!()
    }

    pub fn peeked_as_str(&self) -> &'a str {
        todo!()
    }

    pub fn next(&mut self) {
        todo!()
    }

    pub fn next_peek(&mut self) {}
}

// search for "uui"
// This represent a pseudo code of how Chars should work
pub fn double_u_one_i(str: &str) -> Vec<&str> {
    // let mut chars = Chars {
    //     iter: str.as_bytes().iter(),
    //     peek: 0,
    // };
    let mut result = Vec::new();
    // let pattern = b"uui";

    // chars.peek += pattern.len();

    // // chars.iter.as_slice();
    // // chars.iter.(init, f);

    // loop {
    //     break;
    // }

    result
}

fn double_u_one_i_test() {
    let str = "hello";

    let ptr = std::ptr::from_ref(str) as *const u8;

    println!("str: {:p}", str); // return Pointer { addr: 0x2f2f.., metadata: 5 }
    println!("ptr: {:?}", ptr);

    let mut iter = str.as_bytes().iter();

    println!("iter: {:?}", iter);

    iter.next();

    println!("iter: {:?}", iter);

    iter.next_back();

    println!("iter: {:?}", iter);

    let i = iter.as_slice();

    println!("i: {:?}", i);

    panic!();
}

mod attempt_2 {
    use std::{marker::PhantomData, ptr::NonNull};

    /// Iterator by bytes over string with the ability to peek ahead and match start
    ///
    /// This should conceptually work similar to `(std::slice::Iter<u8>, Vec<char>)`
    /// but designed to be more efficient.
    struct Chars<'a> {
        /// The pointer to the next element, or the past-the-end location
        /// if the iterator is empty
        next: NonNull<u8>,

        /// The non-null pointer to the past-the-end element.
        past_the_end: *const u8,

        /// The pointer to the peeked element
        peek_shift: usize,
        peek_len: usize,

        /// next is pointing to the start of the match
        match_found: bool,

        /// Act as a reference `&'a [u8]`
        _marker: PhantomData<&'a u8>,
    }

    #[must_use]
    enum OpResult {
        Success,
        Failed,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct PinnedRange<'a> {
        pub range: std::ops::Range<usize>,
        // keep string from being mutated or moved while holding the range
        pub pinned: PhantomData<&'a u8>,
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
                peek_shift: 0,
                peek_len: 0,
                match_found: false,
                _marker: PhantomData,
            }
        }

        pub fn peek(&mut self, shift: usize) -> OpResult {
            if self.next.as_ptr() as usize + shift > self.past_the_end as usize {
                return OpResult::Failed;
            }

            unsafe {
                self.peek_len += shift;
                OpResult::Success
            }
        }

        pub fn compare_peeked(&self, pattern_bytes: &[u8]) -> bool {
            if pattern_bytes.len() != self.peek_len {
                panic!("pattern bytes length does not match peek length");
            }
            let mut count = 0usize;
            for byte in pattern_bytes.into_iter().rev() {
                // compare last first
                if unsafe {
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

        pub fn get_str_from_next_to_peek(&self) -> &'a str {
            let range = self.get_range_from_next_to_peek();

            let slice: &[u8] =
                unsafe { std::slice::from_raw_parts(self.next.as_ptr(), range.range.end) };

            let str = unsafe { std::str::from_utf8_unchecked(slice) };

            str
        }

        pub fn get_range_from_next_to_peek(&self) -> PinnedRange<'a> {
            PinnedRange {
                range: 0usize..self.peek_len + self.peek_shift,
                pinned: PhantomData,
            }
        }

        pub fn set_next_to_peek(&mut self) {
            self.next = unsafe { self.next.add(self.peek_len + self.peek_shift) };
            self.peek_len = 0;
            self.peek_shift = 0;
        }

        pub fn shift_next_forward(&mut self) -> OpResult {
            if self.next.as_ptr() as usize + 1 > self.past_the_end as usize {
                return OpResult::Failed;
            }

            self.next = unsafe { self.next.add(1) };
            OpResult::Success
        }
    }

    #[test]
    fn match_el_test() {
        let search = "hello woeld";
        let result = match_el(search);
        assert_eq!(
            result,
            vec![
                PinnedRange {
                    range: 0..2,
                    pinned: PhantomData
                },
                PinnedRange {
                    range: 2..4,
                    pinned: PhantomData
                },
            ]
        );
    }

    fn many_matches_el(search: &str) -> Vec<PinnedRange> {
        let pattern = "el";
        let pattern_bytes = pattern.as_bytes();
        let mut result: Vec<PinnedRange> = Vec::new();

        let mut chars = Chars::new(search);

        // to search for `el` in `hello woeld`:

        // advance the peek 2 letters `peek`
        match chars.peek(pattern.len()) {
            OpResult::Success => {}
            OpResult::Failed => {
                return result;
            }
        }

        // compare the peeked letters with the pattern (will not match) `view_peeked` or `compare_peeked`
        loop {
            let res = chars.compare_peeked(pattern_bytes);
            if res {
                // chars.start_match_and_colapse_peek() // before pushing
                result.push(chars.get_range_from_next_to_peek());
                chars.set_next_to_peek();

                let s = 'a: {
                    break 'a;
                };

                match chars.peek(pattern.len()) {
                    OpResult::Success => {}
                    OpResult::Failed => {
                        return result;
                    }
                }
            } else {
                match chars.shift_next_forward() {
                    OpResult::Success => {}
                    OpResult::Failed => {
                        return result;
                    }
                }
            }
        }

        //         );
    }

    /// `el(o)*`
    fn many_matches_el_once_o_on_repeat(search: &str) -> Vec<PinnedRange> {
        todo!()
    }
}

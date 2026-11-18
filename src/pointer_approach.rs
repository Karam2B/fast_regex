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
        ptr: NonNull<u8>,

        /// The non-null pointer to the past-the-end element.
        end_or_len: *const u8,

        /// The pointer to the peeked element
        peek: *const u8,

        /// The pointer to the start of the match, if any
        ///
        /// this is different from peek because peek can be
        /// used many times to match a single pattern, ex:
        /// `h(el)*` this will
        ///     1. peek `h`
        ///     2. then `el`, set match_start to Some
        ///     3. then `el` on repeat
        match_start: Option<*const u8>,

        /// Act as a reference `&'a [u8]`
        _marker: PhantomData<&'a u8>,
    }

    #[must_use]
    enum OpResult {
        Success,
        Failed,
    }

    fn debug_0<'a>(a: &'a str, b: &'a str) {
        println!("a = {a:?} b = {b:?}");
    }

    fn debug_one_lt<'a>(a: &mut &'a str, b: &mut &'a str) {
        println!("a = {a:?} b = {b:?}");
    }
    fn debug_two_lt<'a, 'b>(a: &mut &'a str, b: &mut &'b str)
    where
        'a: 'b,
    {
        // *a = *b; // requires 'b: 'a
        *b = *a; // requires 'a: 'b
        println!("a = {a:?} b = {b:?}");
    }

    fn main() {
        let mut hello: &'static str = "hello";

        {
            let world = String::from("world");
            let mut world = world.as_str(); // 'world has a shorter lifetime than 'static
            debug_0(hello, world);
        }
        {
            let world = String::from("world");
            let mut world = world.as_str(); // 'world has a shorter lifetime than 'static
            debug_one_lt(&mut hello, &mut world);
        }
        {
            let world = String::from("world");
            let mut world = world.as_str(); // 'world has a shorter lifetime than 'static
            debug_two_lt(&mut hello, &mut world);
        }
    }

    impl<'a> Chars<'a> {
        pub fn new(str: &'a str) -> Self {
            let search_byte = str.as_bytes();
            let len = search_byte.len();
            let ptr = NonNull::from_ref(search_byte).cast::<u8>();
            let peek = ptr.as_ptr();
            let end_or_len = unsafe { ptr.as_ptr().add(len) };
            Self {
                ptr,
                end_or_len,
                peek,
                match_start: None,
                _marker: PhantomData,
            }
        }

        pub fn peek(&mut self, shift: usize) -> OpResult {
            if shift > unsafe { self.end_or_len.sub(self.peek as usize) as usize } {
                return OpResult::Failed;
            }

            unsafe {
                self.peek = self.peek.add(shift);
                OpResult::Success
            }
        }
    }

    #[test]
    fn match_el() {
        let search = "hello woeld";
        let pattern = "el";
        let mut result: Vec<&str> = Vec::new();

        let mut chars = Chars::new(search);

        // to search for `el` in `hello woeld`:

        // advance the peek 2 letters `peek`
        // match chars.peek(pattern.len());

        // compare the peeked letters with the pattern (will not match) `view_peeked` or `compare_peeked`

        // advance both the peek and iter 1 letter `shift`,
        // and check if they match, we know they won't, so will do the same thing one more time

        // if match, will add a match, then advance the peek 2 letters again

        // shift 4 times, compare again, add another match.

        // --- done

        // what about this pattern: h(el)*
        // 1. match h first, if no match shift.
        // 2. match el one time, if match set global flag to true, if no match colapse peeked, go to step 1.
        // 3. match el again,
        //      - if match repeat,
        //      - if no match and global flag is true, add a match,
        //      - if no match and global flag is false, this is a bug, you should not progress to this step if flag is false.

        panic!(
            "what to expect {:?}, {:?}, {:?}, 
end_ is shift by {:?}
peek is shift by {:?}",
            chars.ptr,
            chars.end_or_len,
            chars.peek,
            unsafe { chars.end_or_len.sub(chars.ptr.as_ptr() as usize) as usize },
            unsafe { chars.peek.sub(chars.ptr.as_ptr() as usize) as usize },
        );
    }
}

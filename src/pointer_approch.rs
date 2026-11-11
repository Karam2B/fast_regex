//! This is a different approach to the problem that uses
//! another pointer to represent a portion of the string
//! that has been "peeked"

pub struct Chars<'a> {
    iter: std::slice::Iter<'a, u8>,
    peek: *const u8,
}

impl<'a> Chars<'a> {
    /// for now this can only be used for utf-8 strings,
    /// in the future I will define a trait that does this for
    /// any encoding
    pub fn new(str: &'a str) -> Self {
        let peek = std::ptr::from_ref(str) as *const u8;
        let iter = str.as_bytes().iter();
        Self { iter, peek }
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
    let mut chars = Chars::new(str);
    let mut result = Vec::new();
    let pattern = b"uui";

    // three times
    chars.next_peek();
    chars.next_peek();
    chars.next_peek();

    loop {
        let peeked = chars.view_peeked();
        if peeked == pattern {
            result.push(chars.peeked_as_str());
            chars.reset();

            chars.next_peek();
            chars.next_peek();
            chars.next_peek();
        } else {
            chars.next();
            chars.next_peek();
        }
    }

    result
}

#[test]
fn understanding_pointers() {
    let str = "hello";

    let ptr = std::ptr::from_ref(str) as *const u8;

    println!("str: {:p}", str); // return Pointer { addr: 0x2f2f.., metadata: 5 }
    println!("ptr: {:?}", ptr);

    let iter = str.as_bytes().iter();

    println!("iter: {:?}", iter);
    panic!();
}

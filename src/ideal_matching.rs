use std::str::pattern::CharSearcher;

/// \w+
pub fn word(str: String) -> Vec<String> {
    // keep the String for backwards compatibility
    let mut result = Vec::<String>::new();
    let mut match_start = 0usize; // 0 means matching is not active
    let mut utf8_index = 0usize;

    let mut chars = str.chars();

    // first case
    if let Some(first) = chars.next() {
        utf8_index += first.len_utf8();

        if first.is_alphabetic() {
            match_start = first.len_utf16();
        }
    }

    for char in chars {
        if char.is_alphanumeric() {
            if match_start == 0 {
                // activate match
                match_start = utf8_index
            }
        } else {
            // reset an active matching
            if match_start != 0 {
                result.push(str[match_start..utf8_index].to_string());
                match_start = 0;
            }
        }
        utf8_index += char.len_utf8();
    }

    result
}

#[cfg(test)]
#[test]
fn word_matcher_test() {
    let search_str = "the quick brown fox jumps over the lazy dog";
    let result = word_matcher(search_str.to_string());

    #[rustfmt::skip]
    assert_eq!(result, vec![
        "the".to_string(), "quick".to_string(), "brown".to_string(), 
        "fox".to_string(), "jumps".to_string(), "over".to_string(), 
        "the".to_string(), "lazy".to_string(), "dog".to_string()
    ]);
}

/// \w\w\w\s
pub fn three_w_one_s(str: &str) -> Vec<&str> {
    let mut result = Vec::<&str>::new();

    let mut chars = str.chars();

    let mut peek: [char; 4] = [{
        if let Some(s) = chars.next() {
            s
        } else {
            return result;
        }
    }; 4];

    let mut start_from = 0usize;

    loop {
        let matches = peek[0].is_alphanumeric()
            && peek[1].is_alphanumeric()
            && peek[2].is_whitespace()
            && peek[3].is_alphanumeric();
        // if match push and reinit the peek otherwise rotate peeks
        if matches {
            //
            let shift = start_from
                + peek[0].len_utf8()
                + peek[1].len_utf8()
                + peek[2].len_utf8()
                + peek[3].len_utf8();
            result.push(&str[start_from..shift]);
            peek = [{
                if let Some(s) = chars.next() {
                    s
                } else {
                    return result;
                }
            }; 4];
        } else {
            // rotate peeks
            if let Some(s) = chars.next() {
                start_from += peek[0].len_utf8();
                peek = [peek[1], peek[2], peek[3], s]
            } else {
                return result;
            }
        }
    }
}

#[cfg(test)]
#[test]
fn three_w_one_s_test() {
    let search_str = "the quick brown fox jumps over the lazy dog";
    let result = three_w_one_s(search_str);

    #[rustfmt::skip]
    assert_eq!(result, vec![
        "the ", "ick ", "own ", 
        "fox ", "mps ", "ver ", 
        "the ", "azy "
    ]);
}

// s.s
pub fn dot(str: &str) -> Vec<&str> {
    let mut result = Vec::<&str>::new();

    let mut chars = str.chars();

    let mut peek: [char; 3] = [{
        if let Some(s) = chars.next() {
            s
        } else {
            return result;
        }
    }; 3];

    let mut start_from = 0usize;

    loop {
        let matches = peek[0] == 's' && (true) && peek[2] == 's';
        // if match push and reinit the peek otherwise rotate peeks
        if matches {
            //
            let shift = start_from + peek[0].len_utf8() + peek[1].len_utf8() + peek[2].len_utf8();
            result.push(&str[start_from..shift]);
            peek = [{
                if let Some(s) = chars.next() {
                    s
                } else {
                    return result;
                }
            }; 3];
        } else {
            // rotate peeks
            if let Some(s) = chars.next() {
                start_from += peek[0].len_utf8();
                peek = [peek[1], peek[2], s]
            } else {
                return result;
            }
        }
    }
}

// a|b
pub fn single_charechter_alternate(str: &str) -> Vec<&str> {
    let mut result = Vec::<&str>::new();
    let mut chars = str.chars();

    let mut index = 0;

    while let Some(c) = chars.next() {
        let len = c.len_utf8();
        if c == 'a' || c == 'b' {
            result.push(&str[index..index + len]);
        }

        index += len;
    }

    result
}

// (?:xy|z)
pub fn non_capturing_alternate(str: &str) -> Vec<&str> {
    let mut result = Vec::<&str>::new();
    let mut chars = str.chars();

    let mut index = 0usize;

    let mut peek_min = [{
        if let Some(s) = chars.next() {
            s
        } else {
            return result;
        }
    }; 1];

    while let Some(c) = chars.next() {
        let len = c.len_utf8();
        if peek_min[0] == 'x' && c == 'y' {
            let shift = index + peek_min[0].len_utf8() + len;
            result.push(&str[index..shift]);
            index = shift;
            peek_min = [{
                if let Some(s) = chars.next() {
                    s
                } else {
                    return result;
                }
            }; 1];
        } else if peek_min[0] == 'z' {
            let shift = index + peek_min[0].len_utf8();
            result.push(&str[index..shift]);
            index = shift;
            peek_min[0] = c;
        } else {
            index += peek_min[0].len_utf8();
            peek_min[0] = c;
        }
    }

    result
}

/// (/d+)-(\d+)-(\d+)
/// for 123-456-789 it will return ("123-456-789", ["123", "456", "789"])
fn capturing_group(str: &str) -> Vec<(&str, [&str; 3])> {
    panic!("not implemented");
}

/// (c)de
/// for "abcdefg" it will return vec![("cde", "c")]
fn capturing_group(str: &str) -> Vec<(&str, [&str; 1])> {
    let mut result = Vec::<(&str, [&str; 1])>::new();
    let mut chars = str.chars();
    let mut index = 0usize;

    let mut peek_min = [{
        if let Some(s) = chars.next() {
            s
        } else {
            return result;
        }
    }; 3];

    loop {
        let matches = peek_min[0] == 'c' && peek_min[1] == 'd' && peek_min[2] == 'e';
        if matches {
            let shift =
                index + peek_min[0].len_utf8() + peek_min[1].len_utf8() + peek_min[2].len_utf8();
            let capture_range = index..index + peek_min[0].len_utf8();
            result.push((&str[start..shift], [&str[capture_range]]));
            peek_min = [{
                if let Some(s) = chars.next() {
                    s
                } else {
                    return result;
                }
            }; 3];
            index = shift;
        } else {
            index += peek_min[0].len_utf8();
            let c = if let Some(s) = chars.next() {
                s
            } else {
                return result;
            };
            peek_min = [peek_min[1], peek_min[2], c];
        }
    }

    result
}

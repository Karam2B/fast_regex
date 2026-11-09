// u
pub fn single_u(str: &str) -> Vec<&str> {
    let mut chars = str.chars();
    let mut result = Vec::new();

    let mut index = 0usize;

    while let Some(c) = chars.next() {
        if c == 'u' {
            result.push(&str[index..index + c.len_utf8()]);
        }

        index += c.len_utf8();
    }

    result
}

#[test]
fn single_u_test() {
    let input = "the quick brown fox jumps over the lazy dog";
    let result = single_u(input);
    assert_eq!(result, vec!["u", "u"]);
}

pub fn double_u(str: &str) -> Vec<&str> {
    let mut chars = str.chars();
    let mut result = Vec::new();

    let mut peek = if let Some(c) = chars.next() {
        c
    } else {
        return result;
    };

    let mut index = 0usize;

    while let Some(c) = chars.next() {
        if peek == 'u' && c == 'u' {
            let shift = peek.len_utf8() + c.len_utf8();
            result.push(&str[index..index + shift]);
            index += shift;
            peek = if let Some(c) = chars.next() {
                c
            } else {
                return result;
            };
        } else {
            index += peek.len_utf8();
            peek = c;
        }
    }

    result
}

#[test]
fn double_u_test() {
    let input = "the quuick brown fox jumps over the lazy dog";
    let result = double_u(input);
    assert_eq!(result, vec!["uu"]);
}

// uui+
pub fn double_u_one_i(str: &str) -> Vec<&str> {
    let mut chars = str.chars();
    let mut result = Vec::new();

    let mut peek = if let Some(c) = chars.next() {
        c
    } else {
        return result;
    };

    let mut index = 0usize;

    while let Some(c) = chars.next() {
        if peek == 'u' && c == 'u' {
            let shift = peek.len_utf8() + c.len_utf8();
            result.push(&str[index..index + shift]);
            index += shift;
            peek = if let Some(c) = chars.next() {
                c
            } else {
                return result;
            };
        } else {
            index += peek.len_utf8();
            peek = c;
        }
    }

    result
}

/// \w+
pub fn repeat_alphanumeric(str: String) -> Vec<String> {
    // keep the String for backwards compatibility
    let mut chars = str.chars();
    let mut result = Vec::<String>::new();
    let mut index = 0usize;

    let mut chars = str.chars();

    let mut first_alphanumeric = loop {
        if let Some(c) = chars.next() {
            let breaking = (c, index);
            index += c.len_utf8();
            if c.is_alphanumeric() {
                break breaking;
            }
        } else {
            return result;
        }
    };
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() {
            index += c.len_utf8();
            continue;
        } else {
            result.push(str[first_alphanumeric.1..index].to_string());
            index += c.len_utf8();
            first_alphanumeric = loop {
                if let Some(c) = chars.next() {
                    let breaking = (c, index);
                    index += c.len_utf8();
                    if c.is_alphanumeric() {
                        break breaking;
                    }
                } else {
                    return result;
                }
            };
        }
    }

    result.push(str[first_alphanumeric.1..index].to_string());

    result
}

#[test]
fn repeat_alphanumeric_test() {
    let search_str = "the quick brown fox jumps over the lazy dog";
    let result = repeat_alphanumeric(search_str.to_string());

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

    let mut peek: [char; 4] = [' '; 4];

    for i in 0usize..4 {
        if let Some(s) = chars.next() {
            peek[i] = s;
        } else {
            return result;
        }
    }

    let mut index = 0usize;

    loop {
        let matches = peek[0].is_alphanumeric()
            && peek[1].is_alphanumeric()
            && peek[2].is_alphanumeric()
            && peek[3].is_whitespace();
        // if match push and reinit the peek otherwise rotate peeks
        if matches {
            //
            let shift =
                peek[0].len_utf8() + peek[1].len_utf8() + peek[2].len_utf8() + peek[3].len_utf8();
            result.push(&str[index..(index + shift)]);
            for i in 0usize..4 {
                if let Some(s) = chars.next() {
                    peek[i] = s;
                } else {
                    return result;
                }
            }
            index += shift;
        } else {
            // rotate peeks
            if let Some(s) = chars.next() {
                index += peek[0].len_utf8();
                peek = [peek[1], peek[2], peek[3], s]
            } else {
                return result;
            }
        }
    }
}

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

// \w.\w
pub fn dot(str: &str) -> Vec<&str> {
    let mut result = Vec::<&str>::new();

    let mut chars = str.chars();

    let mut peek: [char; 3] = [' '; 3];

    for i in 0usize..3 {
        if let Some(s) = chars.next() {
            peek[i] = s;
        } else {
            return result;
        }
    }

    let mut index = 0usize;

    loop {
        let matches = peek[0].is_alphanumeric() && (true) && peek[2].is_alphanumeric();
        // if match push and reinit the peek otherwise rotate peeks
        if matches {
            let shift = peek[0].len_utf8() + peek[1].len_utf8() + peek[2].len_utf8();
            result.push(&str[index..(index + shift)]);
            for i in 0usize..3 {
                if let Some(s) = chars.next() {
                    peek[i] = s;
                } else {
                    return result;
                }
            }
            index += shift;
        } else {
            // rotate peeks
            if let Some(s) = chars.next() {
                index += peek[0].len_utf8();
                peek = [peek[1], peek[2], s]
            } else {
                return result;
            }
        }
    }
}

#[test]
fn dot_test() {
    let search_str = "the quick brown fox jumps over the lazy dog";
    let result = dot(search_str);
    assert_eq!(
        result,
        vec![
            "the", "qui", "k b", "row", "n f", "x j", "ump", "s o", "ver", "the", "laz", "y d"
        ]
    );
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

#[test]
fn single_charechter_alternate_test() {
    let search_str = "the quick brown fox jumps over the lazy dog";
    let result = single_charechter_alternate(search_str);
    assert_eq!(result, vec!["b", "a"])
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

#[test]
fn non_capturing_alternate_test() {
    let search_str = "xyzzy";
    let result = non_capturing_alternate(search_str);
    assert_eq!(result, vec!["xy", "z", "z"]);
}

/// (/d+)-(\d+)-(\d+)
/// for 123-456-789 it will return ("123-456-789", ["123", "456", "789"])
fn capturing_group_complix(str: &str) -> Vec<(&str, [&str; 3])> {
    panic!("not implemented");
}

/// (c)de
/// for "abcdefg" it will return vec![("cde", "c")]
fn capturing_group(str: &str) -> Vec<(&str, [&str; 1])> {
    let mut result = Vec::<(&str, [&str; 1])>::new();
    let mut chars = str.chars();
    let mut index = 0usize;

    let mut peek_min: [char; 3] = Default::default();

    for i in 0usize..3 {
        if let Some(s) = chars.next() {
            peek_min[i] = s;
        } else {
            return result;
        }
    }

    loop {
        let matches = peek_min[0] == 'c' && peek_min[1] == 'd' && peek_min[2] == 'e';
        if matches {
            let shift =
                index + peek_min[0].len_utf8() + peek_min[1].len_utf8() + peek_min[2].len_utf8();
            let capture_range = index..index + peek_min[0].len_utf8();
            result.push((&str[index..shift], [&str[capture_range]]));
            for i in 0usize..3 {
                if let Some(s) = chars.next() {
                    peek_min[i] = s;
                } else {
                    return result;
                }
            }
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

#[test]
fn capturing_group_test() {
    let search_str = "abcdefg";
    let result = capturing_group(search_str);
    assert_eq!(result, vec!(("cde", ["c"])));
}

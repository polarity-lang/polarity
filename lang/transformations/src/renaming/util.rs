pub fn increment_name(mut name: String) -> String {
    if name.ends_with('\'') {
        name.push('\'');
        return name;
    }
    let (s, digits) = split_trailing_digits(&name);
    match digits {
        None => format!("{s}0"),
        Some(n) => format!("{s}{}", n + 1),
    }
}

pub fn split_trailing_digits(s: &str) -> (&str, Option<usize>) {
    let n_digits = s.chars().rev().take_while(char::is_ascii_digit).count();
    let (s, digits) = s.split_at(s.len() - n_digits);

    (s, digits.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_empty() {
        assert_eq!(split_trailing_digits(""), ("", None))
    }

    #[test]
    fn test_split_no_digits() {
        assert_eq!(split_trailing_digits("foo"), ("foo", None))
    }

    #[test]
    fn test_split_digits() {
        assert_eq!(split_trailing_digits("foo42"), ("foo", Some(42)))
    }
}

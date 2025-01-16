use ast::VarBind;

pub fn increment_name(name: VarBind) -> VarBind {
    match name {
        VarBind::Var { span, mut id } => {
            if id.ends_with('\'') {
                id.push('\'');
                return VarBind::Var { id, span };
            }
            let (s, digits) = split_trailing_digits(&id);
            match digits {
                None => VarBind::from_string(&format!("{s}0")),
                Some(n) => VarBind::from_string(&format!("{s}{}", n + 1)),
            }
        }
        wc @ VarBind::Wildcard { .. } => wc,
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

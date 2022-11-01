pub fn comma_separated<I: IntoIterator<Item = String>>(iter: I) -> String {
    separated(", ", iter)
}

pub fn separated<I: IntoIterator<Item = String>>(s: &str, iter: I) -> String {
    let vec: Vec<_> = iter.into_iter().collect();
    vec.join(s)
}

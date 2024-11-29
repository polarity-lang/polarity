use crate::*;

pub trait MapIdxExt<T> {
    fn map_idx<U, F: Fn(Idx, &T) -> U>(&self, f: F) -> MapIdx<'_, T, U, F>;
}

impl<T> MapIdxExt<T> for Vec<Vec<T>> {
    fn map_idx<U, F: Fn(Idx, &T) -> U>(&self, f: F) -> MapIdx<'_, T, U, F> {
        MapIdx { inner: self, f }
    }
}

pub struct MapIdx<'a, T, U, F: Fn(Idx, &T) -> U> {
    inner: &'a Vec<Vec<T>>,
    f: F,
}

impl<T, U, F: Fn(Idx, &T) -> U> MapIdx<'_, T, U, F> {
    pub fn collect(self) -> Vec<Vec<U>> {
        self.inner
            .iter()
            .enumerate()
            .map(|(fst, stack)| {
                stack
                    .iter()
                    .enumerate()
                    .map(|(snd, x)| {
                        (self.f)(
                            Idx { fst: self.inner.len() - 1 - fst, snd: stack.len() - 1 - snd },
                            x,
                        )
                    })
                    .collect()
            })
            .collect()
    }
}

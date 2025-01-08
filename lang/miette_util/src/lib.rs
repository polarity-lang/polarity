pub mod codespan;

pub trait ToMiette {
    type Target;

    fn to_miette(self) -> Self::Target;
}

pub trait FromMiette {
    type Target;

    #[allow(clippy::wrong_self_convention)]
    fn from_miette(self) -> Self::Target;
}

impl ToMiette for codespan::ByteOffset {
    type Target = miette::SourceOffset;

    fn to_miette(self) -> Self::Target {
        self.to_usize().into()
    }
}

impl ToMiette for codespan::ByteIndex {
    type Target = miette::SourceOffset;

    fn to_miette(self) -> Self::Target {
        self.to_usize().into()
    }
}

impl ToMiette for codespan::Span {
    type Target = miette::SourceSpan;

    fn to_miette(self) -> Self::Target {
        let length = self.end - self.start;
        miette::SourceSpan::new(self.start.to_miette(), length.to_usize())
    }
}

impl<T: ToMiette> ToMiette for Option<T> {
    type Target = Option<T::Target>;

    fn to_miette(self) -> Self::Target {
        self.map(ToMiette::to_miette)
    }
}

impl FromMiette for miette::SourceOffset {
    type Target = codespan::ByteIndex;

    fn from_miette(self) -> Self::Target {
        codespan::ByteIndex(self.offset() as u32)
    }
}

impl FromMiette for miette::SourceSpan {
    type Target = codespan::Span;

    fn from_miette(self) -> Self::Target {
        let start = codespan::ByteIndex(self.offset() as u32);
        let end = codespan::ByteIndex((self.offset() + self.len()) as u32);
        codespan::Span { start, end }
    }
}

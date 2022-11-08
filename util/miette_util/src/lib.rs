pub trait ToMiette {
    type Target;

    fn to_miette(self) -> Self::Target;
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
        let length = self.end() - self.start();
        miette::SourceSpan::new(self.start().to_miette(), length.to_miette())
    }
}

impl<T: ToMiette> ToMiette for Option<T> {
    type Target = Option<T::Target>;

    fn to_miette(self) -> Self::Target {
        self.map(ToMiette::to_miette)
    }
}

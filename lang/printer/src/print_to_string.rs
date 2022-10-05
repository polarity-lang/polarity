use super::Print;

pub trait PrintToString {
    fn print_to_string(&self) -> String;
}

impl<T: for<'a> Print<'a>> PrintToString for T {
    fn print_to_string(&self) -> String {
        let alloc = super::Alloc::new();
        let mut buf = Vec::new();
        {
            let doc_builder = self.print(&alloc);
            doc_builder
                .1
                .render(super::DEFAULT_WIDTH, &mut buf)
                .expect("Failed to print to string");
            drop(doc_builder)
        }
        unsafe { String::from_utf8_unchecked(buf) }
    }
}

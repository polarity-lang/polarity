pub mod ctx;
pub mod typecheck;

pub use crate::result::TypeError;
pub use tracer;
pub use typecheck::check;

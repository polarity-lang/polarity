pub mod ctx;
pub mod result;
pub mod typecheck;

pub use result::TypeError;
pub use tracer;
pub use typecheck::check;

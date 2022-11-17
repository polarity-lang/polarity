pub mod ctx;
pub mod prg;
pub mod result;
pub mod typecheck;
pub mod unify;

pub use result::TypeError;
pub use tracer;
pub use typecheck::check;

use super::ctx::*;
use super::result::*;
use super::types::*;

/// Extension trait for lowering in the empty context
///
/// The `lower` method is not included in the `Lower` trait, but
/// is instead provided in this separated module, to prevent accidental
/// context-ignoring invocations of `lower` (instead of `lower_in_ctx`)
/// in the implementations of the `Lower` trait.
pub trait LowerExt: Lower {
    fn lower(self) -> Result<<Self as Lower>::Target, LoweringError>;
}

impl<T: Lower> LowerExt for T {
    /// Lower in the empty context
    fn lower(self) -> Result<<Self as Lower>::Target, LoweringError> {
        let mut ctx = Ctx::empty();
        self.lower_in_ctx(&mut ctx)
    }
}

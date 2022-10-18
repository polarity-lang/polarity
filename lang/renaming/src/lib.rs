// mod ast;
mod ctx;
mod generic;

pub use ctx::Ctx;

pub trait Rename: Sized {
    fn rename(self) -> Self {
        let mut ctx = Ctx::empty();
        self.rename_in_ctx(&mut ctx)
    }
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self;
}

pub trait RenameTelescope {
    fn rename_telescope<T, F: FnOnce(&mut Ctx, Self) -> T>(&self, ctx: &mut Ctx, f: F) -> T;
}

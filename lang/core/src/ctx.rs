//! Variable context
//!
//! Tracks locally bound variables

use syntax::ctx::TypeCtx;
use syntax::ust::UST;

pub type Ctx = TypeCtx<UST>;

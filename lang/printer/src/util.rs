use std::path::{Path, PathBuf};

use pretty::DocAllocator;

use super::types::*;

pub trait BracesExt<'a, D>
where
    D: ?Sized + DocAllocator<'a, Anno>,
{
    fn braces_anno(self) -> pretty::DocBuilder<'a, D, Anno>;
}

pub trait BackslashExt<'a>: DocAllocator<'a, Anno> {
    fn backslash_anno(&'a self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, Self, Anno>;
}

pub trait IsNilExt<'a, D, A: 'a>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn is_nil(&self) -> bool;
}

impl<'a, D> BracesExt<'a, D> for pretty::DocBuilder<'a, D, Anno>
where
    D: ?Sized + DocAllocator<'a, Anno>,
{
    fn braces_anno(self) -> pretty::DocBuilder<'a, D, Anno> {
        let l = self.0.text("{".to_owned()).annotate(Anno::BraceOpen);
        let r = self.0.text("}".to_owned()).annotate(Anno::BraceClose);
        self.enclose(l, r)
    }
}

impl<'a, T> BackslashExt<'a> for T
where
    T: DocAllocator<'a, Anno>,
{
    fn backslash_anno(&'a self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, Self, Anno> {
        let backlash = if cfg.latex { " " } else { "\\" };
        self.text(backlash).annotate(Anno::Backslash)
    }
}

impl<'a, D, A> IsNilExt<'a, D, A> for pretty::DocBuilder<'a, D, A>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn is_nil(&self) -> bool {
        matches!(self.1, pretty::BuildDoc::Doc(pretty::Doc::Nil))
    }
}

pub fn get_target_path(path: &Path) -> PathBuf {
    let cwd = std::env::current_dir().expect("Failed to get current working directory");
    let cwd_name = cwd.file_name().expect("Failed to get current working directory name");

    let mut components = path.components().peekable();
    let mut new_path = PathBuf::new();

    for component in components.by_ref() {
        new_path.push(component);
        if component.as_os_str() == cwd_name {
            new_path.push("target_pol/docs");
            break;
        }
    }

    for component in components {
        new_path.push(component);
    }

    let stem = new_path.file_stem().map(|s| s.to_os_string());
    if let Some(stem) = stem {
        new_path.set_file_name(stem);
        new_path.set_extension("html");
    }

    new_path
}

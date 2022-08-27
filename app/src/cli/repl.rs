use reedline::{Reedline, Signal};

use crate::result::HandleErrorExt;

use super::prompt::CustomPrompt;
use super::terminal;

#[derive(Default, clap::Args)]
pub struct Args {}

pub fn exec(_cmd: Args) {
    let mut line_editor = Reedline::create();
    let prompt = CustomPrompt::default();

    loop {
        let sig = line_editor.read_line(&prompt).expect("Failed to read from console");
        match sig {
            Signal::Success(s) => {
                if !s.trim().is_empty() {
                    crate::rt::run_string(&s).handle(terminal::print_prg)
                }
            }
            Signal::CtrlD | Signal::CtrlC => {
                return;
            }
        }
    }
}

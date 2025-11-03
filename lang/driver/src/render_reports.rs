/// Terminal width for pretty-printing error messages.
const TERMINAL_WIDTH: usize = 200;

struct WriteAdapter<'a, O: std::io::Write>(pub &'a mut O);

impl<'a, O: std::io::Write> std::fmt::Write for WriteAdapter<'a, O> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        Ok(write!(self.0, "{s}").expect("Failed to write in adapter"))
    }
}

pub fn render_reports_to_string(reports: &[miette::Report], colorize: bool) -> String {
    let mut output = String::new();
    render_reports(&mut output, reports, colorize);
    output
}

pub fn render_reports_io<O>(output: &mut O, reports: &[miette::Report], colorize: bool)
where
    O: std::io::Write,
{
    let mut adapter = WriteAdapter(output);
    render_reports(&mut adapter, reports, colorize);
}

pub fn render_reports<O>(output: &mut O, reports: &[miette::Report], colorize: bool)
where
    O: std::fmt::Write,
{
    let theme = if colorize {
        miette::GraphicalTheme::unicode()
    } else {
        miette::GraphicalTheme::unicode_nocolor()
    };
    let handler = miette::GraphicalReportHandler::new_themed(theme).with_width(TERMINAL_WIDTH);
    for report in reports {
        handler.render_report(output, report.as_ref()).expect("Failed to render report");
    }
}

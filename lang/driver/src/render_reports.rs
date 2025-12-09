use miette::Report;

use polarity_lang_printer::ColorChoice;

/// Terminal width for pretty-printing error messages.
const TERMINAL_WIDTH: usize = 200;

struct WriteAdapter<'a, O: std::io::Write>(pub &'a mut O);

impl<O: std::io::Write> std::fmt::Write for WriteAdapter<'_, O> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        write!(self.0, "{s}").expect("Failed to write in adapter");
        Ok(())
    }
}

pub fn render_reports_to_string(reports: &[Report], colorize: ColorChoice) -> String {
    let mut output = String::new();
    render_reports(&mut output, reports, colorize);
    output
}

pub fn render_reports_io<O>(output: &mut O, reports: &[Report], colorize: ColorChoice)
where
    O: std::io::Write,
{
    let mut adapter = WriteAdapter(output);
    render_reports(&mut adapter, reports, colorize);
}

pub fn render_reports<O>(output: &mut O, reports: &[Report], colorize: ColorChoice)
where
    O: std::fmt::Write,
{
    let theme = match colorize {
        ColorChoice::Always | ColorChoice::AlwaysAnsi => miette::GraphicalTheme::unicode(),
        ColorChoice::Auto => miette::GraphicalTheme::default(),
        ColorChoice::Never => miette::GraphicalTheme::unicode_nocolor(),
    };
    let handler = miette::GraphicalReportHandler::new_themed(theme).with_width(TERMINAL_WIDTH);

    let mut reports = reports.iter().peekable();
    while let Some(report) = reports.next() {
        handler.render_report(output, report.as_ref()).expect("Failed to render report");
        if reports.peek().is_some() {
            writeln!(output).expect("Failed to render newline in report");
        }
    }
}

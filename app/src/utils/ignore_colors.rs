use std::io;

use polarity_lang_printer::termcolor::WriteColor;

pub struct IgnoreColors<W: io::Write> {
    inner: W,
}

impl<W: io::Write> IgnoreColors<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: io::Write> io::Write for IgnoreColors<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<W: io::Write> WriteColor for IgnoreColors<W> {
    fn supports_color(&self) -> bool {
        false
    }

    fn set_color(&mut self, _spec: &polarity_lang_printer::ColorSpec) -> io::Result<()> {
        Ok(())
    }

    fn reset(&mut self) -> io::Result<()> {
        Ok(())
    }
}

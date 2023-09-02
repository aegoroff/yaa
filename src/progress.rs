use color_eyre::eyre::Result;
use indicatif::{ProgressBar, ProgressStyle};

extern crate indicatif;

pub trait Size {
    fn size(&self) -> u64;
}

pub trait Progress {
    fn progress(&mut self, current: u64);
    fn finish(&self, message: &'static str);
    fn message(&self, message: String);
}

pub struct Progresser {
    bar: ProgressBar,
    items: u64,
}

impl Progresser {
    /// # Errors
    ///
    /// Will return Err in case of bad template
    pub fn new(total: u64) -> Result<Self> {
        let bar = ProgressBar::new(total);
        bar.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})\n{wide_msg}")?
            .progress_chars("#>-"));

        Ok(Self { bar, items: 0 })
    }
}

impl Progress for Progresser {
    fn progress(&mut self, current: u64) {
        self.bar.set_position(current);
    }

    fn finish(&self, message: &'static str) {
        self.bar.finish_with_message(message);
    }

    fn message(&self, message: String) {
        self.bar.set_message(message);
    }
}

impl Size for Progresser {
    fn size(&self) -> u64 {
        self.items
    }
}

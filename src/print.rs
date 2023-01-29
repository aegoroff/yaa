use std::path::PathBuf;

use indicatif::HumanBytes;
use num_format::{Locale, ToFormattedString};
use prettytable::{format, row, Table};

pub struct Resulter {
    table: Table,
}

pub struct Statistic {
    pub title: String,
    pub files: Vec<PathBuf>,
    pub size: u64,
}

impl Resulter {
    pub fn new() -> Self {
        let mut table = Table::new();

        let format = format::FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .separators(
                &[format::LinePosition::Title],
                format::LineSeparator::new('-', ' ', ' ', ' '),
            )
            .indent(0)
            .padding(0, 0)
            .build();
        table.set_format(format);
        table.set_titles(row![bF=> "", "Files", "Size"]);

        Self { table }
    }

    pub fn append(&mut self, res: Statistic) {
        self.append_row(&res.title, res.size, res.files.len() as u64);
    }

    pub fn append_row(&mut self, name: &str, size: u64, count: u64) {
        let files = count.to_formatted_string(&Locale::ru);
        let size = HumanBytes(size).to_string();
        self.table.add_row(row![bF->name, files, size]);
    }

    pub fn append_empty_row(&mut self) {
        self.table.add_empty_row();
    }

    pub fn print(&self) {
        println!();
        self.table.printstd();
    }
}

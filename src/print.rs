use indicatif::HumanBytes;
use num_format::{Locale, ToFormattedString};
use prettytable::{format, row, Row, Table};

use crate::FileStat;

pub struct Resulter {
    table: Table,
    output_as_html: bool,
}

pub struct Statistic {
    pub title: String,
    pub files: Vec<FileStat>,
    pub size: u64,
}

impl Resulter {
    #[must_use] pub fn new(output_as_html: bool) -> Self {
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

        Self {
            table,
            output_as_html,
        }
    }

    pub fn titles(&mut self, titles: Row) {
        self.table.set_titles(titles)
    }

    pub fn append(&mut self, res: &Statistic) {
        self.append_row(&res.title, res.size, res.files.len() as u64);
    }

    pub fn append_row(&mut self, name: &str, size: u64, count: u64) {
        let files = count.to_formatted_string(&Locale::ru);
        let size = HumanBytes(size).to_string();
        self.table.add_row(row![bF->name, files, size]);
    }

    pub fn append_count_row(&mut self, name: &str, num: usize, count: u64) {
        let ext_count = count.to_formatted_string(&Locale::ru);
        let num = if num > 0 {
            num.to_string()
        } else {
            String::default()
        };
        self.table.add_row(row![num, bF->name, ext_count]);
    }

    pub fn append_empty_row(&mut self) {
        self.table.add_empty_row();
    }

    pub fn print(&self) {
        println!();
        println!();
        if self.output_as_html {
            let mut w = std::io::stdout();
            self.table.print_html(&mut w).unwrap_or_default();
        } else {
            self.table.printstd();
        }
    }
}

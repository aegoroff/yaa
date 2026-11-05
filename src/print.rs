use comfy_table::{
    Attribute, Cell, ContentArrangement, ContentLineStyle, LineStyle, Table, TableStyle,
};
use indicatif::HumanBytes;
use itertools::Itertools;
use num_format::{Locale, ToFormattedString};

use crate::FileStat;

pub struct Resulter {
    table: Table,
}

pub struct Statistic {
    pub title: String,
    pub files: Vec<FileStat>,
    pub size: u64,
}

impl Default for Resulter {
    fn default() -> Self {
        Self::new()
    }
}

impl Resulter {
    #[must_use]
    pub fn new() -> Self {
        let mut table = Table::new();
        table
            .load_style(
                TableStyle::new()
                    .top_border(LineStyle::new(' ', ' ', ' ', ' '))
                    .header_lines(ContentLineStyle::new(' ', ' ', ' '))
                    .header_separator(LineStyle::new(' ', '-', ' ', ' '))
                    .content_lines(ContentLineStyle::new(' ', ' ', ' '))
                    .bottom_border(LineStyle::new(' ', ' ', ' ', ' ')),
            )
            .set_content_arrangement(ContentArrangement::Dynamic);
        Self { table }
    }

    /// Sets table's headline
    pub fn titles(&mut self, titles: &[&str]) {
        let heads = titles
            .iter()
            .map(|t| Cell::new(t).add_attribute(Attribute::Bold))
            .collect_vec();
        self.table.set_header(heads);
    }

    /// Appends Statistic as table row
    pub fn append_stat_as_row(&mut self, res: &Statistic) {
        self.append_row(&res.title, res.size, res.files.len() as u64);
    }

    pub fn append_row(&mut self, name: &str, size: u64, count: u64) {
        let files = count.to_formatted_string(&Locale::ru);
        let size = HumanBytes(size).to_string();
        self.table.add_row(vec![
            Cell::new(name).add_attribute(Attribute::Bold),
            Cell::new(files),
            Cell::new(size),
        ]);
    }

    pub fn append_count_row(&mut self, name: &str, num: usize, count: u64) {
        let ext_count = count.to_formatted_string(&Locale::ru);
        let num = if num > 0 {
            num.to_string()
        } else {
            String::default()
        };
        self.table.add_row(vec![
            Cell::new(num),
            Cell::new(name).add_attribute(Attribute::Bold),
            Cell::new(ext_count),
        ]);
    }

    pub fn append_empty_row(&mut self) {
        self.table.add_row(vec!["", "", ""]);
    }

    pub fn print(&self) {
        println!();
        println!();
        println!("{}", self.table);
    }
}

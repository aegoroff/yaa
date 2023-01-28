use indicatif::HumanBytes;
use num_format::{ToFormattedString, Locale};
use prettytable::{format, row, Table};

pub struct Resulter {
    table: Table,
}

pub struct Statistic {
    pub path: String,
    pub count: u64,
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
        let files = res.count.to_formatted_string(&Locale::ru);
        let size = HumanBytes(res.size).to_string();
        let name = res.path.as_str();
        self.table.add_row(row![bF->name, files, size]);
    }

    pub fn print(&self) {
        println!();
        self.table.printstd();
    }
}

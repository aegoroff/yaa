use num_format::{ToFormattedString, Locale};
use prettytable::{format, row, Table};

pub struct Resulter {
    table: Table,
}

pub struct Statistic {
    pub path: String,
    pub count: u64,
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
        table.set_titles(row![bF=> "", "Files"]);

        Self { table }
    }

    pub fn append(&mut self, res: Statistic) {
        let files = res.count.to_formatted_string(&Locale::ru);
        let name = res.path.as_str();
        self.table.add_row(row![bF->name, files]);
    }

    pub fn print(&self) {
        println!();
        self.table.printstd();
    }
}

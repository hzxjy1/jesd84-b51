use clap::{arg, Parser};
use prettytable::{format, Cell, Row, Table};

use crate::out_dto::OutDto;

#[derive(Parser, Debug)]
pub struct Cli {
    /// Specifies the JSON file
    #[arg(short = 'j', long = "json")]
    pub json_file: String,

    /// Specifies the text file
    #[arg(short = 'b', long = "binary")]
    pub binary_file: String,
}

pub fn make_table(out_dto: OutDto) -> Table {
    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new("Name"),
        Cell::new("Data"),
    ]));

    for obj in out_dto.into_iter() {
        table.add_row(Row::new(vec![
            Cell::new(&obj.id.to_string()),
            Cell::new(&obj.name),
            Cell::new(&format!("{:?}", obj.data)),
        ]));
    }

    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table
}

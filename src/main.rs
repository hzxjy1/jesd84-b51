use std::path::Path;

mod cli;
mod jesd84_b51;
mod out_dto;
mod sheet_orm;
use clap::Parser;
use cli::{make_table, Cli};
use jesd84_b51::Jesd84B51;
use out_dto::OutDto;
use sheet_orm::SheetOrm;

fn main() {
    let args = Cli::parse();
    let jesd84_b51 = Jesd84B51::new(Path::new(&args.binary_file)).unwrap();
    let sheet_orm = SheetOrm::new(Path::new(&args.json_file)).unwrap();
    let out_dto = OutDto::new(&sheet_orm, &jesd84_b51).unwrap();
    let _ = match args.path {
        Some(path) => sheet_orm.gen_simplify_conf(path, ","),
        None => Ok(()),
    };
    let table = make_table(out_dto);
    table.printstd()
}

use std::path::Path;

mod jesd84_b51;
pub mod out_dto;
mod sheet_orm;
use jesd84_b51::Jesd84B51;
use out_dto::OutDto;
use sheet_orm::SheetOrm;

fn main() {
    let jesd84_b51 = Jesd84B51::new(Path::new("/home/hzxjy/jesd84-b51/data/binary.txt")).unwrap();
    let sheet_orm =
        SheetOrm::new(Path::new("/home/hzxjy/jesd84-b51/data/JESD84-B51.json")).unwrap();
    let out_dto = OutDto::new(&sheet_orm, &jesd84_b51).unwrap();
    let json_str=out_dto.to_json().unwrap();
    println!("{}", json_str);
}

use std::path::Path;

mod jesd84_b51;
use jesd84_b51::Jesd84B51;
mod json_sheet;
use json_sheet::JsonSheet;

fn main() {
    let obj = Jesd84B51::new(Path::new("/home/hzxjy/jesd84-b51/data/binary.txt")).unwrap();
    println!("{:?}", obj.get_bytes());
    let json_obj=JsonSheet::new(Path::new("/home/hzxjy/jesd84-b51/data/JESD84-B51.json")).unwrap();
}

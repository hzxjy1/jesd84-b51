use std::{error::Error, io, vec};

use serde::{Deserialize, Serialize};

use crate::{
    jesd84_b51::Jesd84B51,
    sheet_orm::{SheetOrm, SheetOrmObj},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct OutDto {
    array: Vec<OutDtoObj>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutDtoObj {
    pub id: u16,
    pub name: String,
    pub data: Vec<u8>,
}

impl OutDto {
    pub fn new(sheet_orm: &SheetOrm, jesd84_b51: &Jesd84B51) -> Result<Self, Box<dyn Error>> {
        let mut vec = Vec::new();
        for i in sheet_orm {
            let data = search_binary(i, jesd84_b51)?;
            let obj = OutDtoObj {
                id: i.id,
                name: i.data.name.clone(),
                data,
            };
            vec.push(obj);
        }
        Ok(OutDto { array: vec })
    }

    pub fn to_json(&self) -> Result<String, Box<dyn Error>> {
        let json = serde_json::to_string(self)?;
        Ok(json)
    }
}

impl<'a> IntoIterator for &'a OutDto {
    type Item = &'a OutDtoObj;
    type IntoIter = std::slice::Iter<'a, OutDtoObj>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.iter()
    }
}

fn search_binary(obj: &SheetOrmObj, jesd84_b51: &Jesd84B51) -> Result<Vec<u8>, Box<dyn Error>> {
    let csd_slice = obj.data.csd_slice.clone();
    let (prev, next) = match csd_slice.len() {
        2 => {
            let prev = csd_slice[1] as usize;
            let next = csd_slice[0] as usize;
            (prev, next)
        }
        1 => {
            let prev = csd_slice[0] as usize;
            let next = usize::MAX;
            (prev, next)
        }
        _ => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "Content length is not 1 or 2 bytes",
            )));
        }
    };

    if next > jesd84_b51.bytes.len() && next != usize::MAX {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Index out of bounds",
        )));
    }

    if next != usize::MAX {
        return Ok(jesd84_b51.bytes[prev..next].to_vec());
    } else {
        return Ok(vec![jesd84_b51.bytes[prev]]);
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, path::Path};
    use tempfile::NamedTempFile;

    use crate::{jesd84_b51::Jesd84B51, sheet_orm::SheetOrm};
    use std::error::Error;

    use super::{search_binary, OutDto};

    fn mock_data() -> Result<SheetOrm, Box<dyn Error>> {
        let json_data = r#"
        {
            "array":[
            {"id":1,"data":{"Name":"Reserved","Field":null,"Size":6,"type":"TBD","CSD-slice":"[511:506]"}},
            {"id":2,"data":{"Name":"Extended Security Commands Error","Field":"EXT_SECURITY_ERR","Size":1,"type":"R","CSD-slice":"[505]"}},
            {"id":3,"data":{"Name":"Supported Command Sets","Field":"S_CMD_SET","Size":1,"type":"R","CSD-slice":"[504]"}},
            {"id":4,"data":{"Name":"HPI features","Field":"HPI_FEATURES","Size":1,"type":"R","CSD-slice":"[503]"}},
            {"id":5,"data":{"Name":"Background operations support","Field":"BKOPS_SUPPORT","Size":1,"type":"R","CSD-slice":"[502]"}},
            {"id":6,"data":{"Name":"Max packed read commands","Field":"MAX_PACKED_READS","Size":1,"type":"R","CSD-slice":"[501]"}},
            {"id":7,"data":{"Name":"Max packed write commands","Field":"MAX_PACKED_WRITES","Size":1,"type":"R","CSD-slice":"[500]"}}
            ]
        }
        "#;

        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "{}", json_data)?;

        let file_path = temp_file.path();
        Ok(SheetOrm::new(file_path)?)
    }

    fn prepare_mock_data() -> Result<(SheetOrm, Jesd84B51), Box<dyn Error>> {
        let sheet_orm = mock_data()?;
        let jesd84_b51 = Jesd84B51::new(Path::new("/home/hzxjy/jesd84-b51/data/binary.txt"))?;
        Ok((sheet_orm, jesd84_b51))
    }

    #[test]
    fn search_binary_test() -> Result<(), Box<dyn Error>> {
        let ret = prepare_mock_data()?;
        let i = search_binary(&ret.0[0], &ret.1)?;
        println!("{:?}", i);
        assert_eq!(i[0], 0);
        Ok(())
    }

    #[test]
    fn to_json_test() -> Result<(), Box<dyn Error>> {
        let mock_json = r#"{"array":[{"id":1,"name":"Reserved","data":[0,0,0,0,0]},{"id":2,"name":"Extended Security Commands Error","data":[0]},{"id":3,"name":"Supported Command Sets","data":[1]},{"id":4,"name":"HPI features","data":[1]},{"id":5,"name":"Background operations support","data":[1]},{"id":6,"name":"Max packed read commands","data":[63]},{"id":7,"name":"Max packed write commands","data":[63]}]}"#;
        let ret = prepare_mock_data()?;
        let out_dto = OutDto::new(&ret.0, &ret.1)?;
        let json_str = out_dto.to_json()?;
        assert_eq!(json_str, mock_json);
        Ok(())
    }
}

use std::io::Write;
use std::{
    error::Error,
    fs::{self, File},
    io,
    ops::Index,
    path::Path,
};

use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SheetOrm {
    array: Vec<SheetOrmObj>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SheetOrmObj {
    pub id: u16,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Field")]
    field: Option<String>,
    #[serde(rename = "Size")]
    size: u8,
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "CSD-slice", deserialize_with = "parse_csd_slice")]
    pub csd_slice: Vec<u16>,
}

impl SheetOrm {
    pub fn new(file_path: &Path) -> Result<Self, Box<dyn Error>> {
        let json_str = fs::read_to_string(file_path)?;
        let sheet_orm: SheetOrm = serde_json::from_str(&json_str)?;
        Ok(sheet_orm)
    }

    pub fn gen_simplify_conf(&self, path: String, token: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(&path)?;
        let mut writer = io::BufWriter::new(file);
        for item in self.array.iter() {
            let format = format!(
                "{}{}{}{}{}",
                item.id,
                token,
                item.data.name,
                token,
                if item.data.csd_slice.len() > 1 {
                    format!(
                        "{}{}{}",
                        item.data.csd_slice[0], token, item.data.csd_slice[1]
                    )
                } else if item.data.csd_slice.len() == 1 {
                    format!("{}{}{}", item.data.csd_slice[0], token, u16::MAX)
                } else {
                    "No CSD Slice".to_string()
                }
            );

            // 将格式化字符串写入文件，并换行
            writeln!(writer, "{}", format)?;
        }

        // 确保所有内容都被写入文件
        writer.flush()?;

        Ok(())
    }
}

impl Index<usize> for SheetOrm {
    type Output = SheetOrmObj;

    fn index(&self, index: usize) -> &Self::Output {
        &self.array[index]
    }
}

impl<'a> IntoIterator for &'a SheetOrm {
    type Item = &'a SheetOrmObj;
    type IntoIter = std::slice::Iter<'a, SheetOrmObj>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.iter()
    }
}

fn parse_csd_slice<'de, D>(deserializer: D) -> Result<Vec<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let trimmed = s.trim_matches(&['[', ']'][..]);
    let parts: Vec<&str> = trimmed.split(':').collect();

    match parts.len() {
        2 => {
            let start = parts[0].trim().parse().map_err(|_| {
                de::Error::invalid_type(Unexpected::Str(parts[0]), &"expected an integer")
            })?;
            let end = parts[1].trim().parse().map_err(|_| {
                de::Error::invalid_type(Unexpected::Str(parts[1]), &"expected an integer")
            })?;
            let vec: Vec<u16> = vec![start, end];
            Ok(vec)
        }
        1 => {
            let num = parts[0].trim().parse().map_err(|_| {
                de::Error::invalid_type(Unexpected::Str(parts[0]), &"expected an integer")
            })?;
            let vec: Vec<u16> = vec![num];
            Ok(vec)
        }
        _ => Err(de::Error::custom(
            "Invalid format: expected format [start:end] or [num]",
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::NamedTempFile;

    use super::*;

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

    #[test]
    fn new() -> Result<(), Box<dyn Error>> {
        let sheet_orm = mock_data()?;

        assert_eq!(sheet_orm.array.len(), 7);
        assert_eq!(sheet_orm.array[0].id, 1);
        assert_eq!(sheet_orm[0].data.csd_slice[0], 511);
        assert_eq!(sheet_orm[1].data.csd_slice[0], 505);
        assert_eq!(
            sheet_orm.array[1].data.name,
            "Extended Security Commands Error"
        );

        Ok(())
    }

    #[test]
    fn new_in_file() -> Result<(), Box<dyn Error>> {
        let _ = SheetOrm::new(&Path::new("/home/hzxjy/jesd84-b51/data/JESD84-B51.json"))?;
        Ok(())
    }

    #[test]
    fn bracket() -> Result<(), Box<dyn Error>> {
        let sheet_orm = mock_data()?;

        assert_eq!(sheet_orm[0].id, 1);
        assert_eq!(sheet_orm[1].data.name, "Extended Security Commands Error");
        Ok(())
    }

    #[test]
    fn gen_simplify_conf_test() -> Result<(), Box<dyn Error>> {
        let obj = mock_data()?;
        obj.gen_simplify_conf("/home/hzxjy/aaaa.txt".to_string(), ",s")?;
        assert_eq!(2, 1);
        Ok(())
    }
}

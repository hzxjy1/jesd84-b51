use std::{error::Error, fs, io, path::Path};

#[derive(Debug)]
pub struct Jesd84B51 {
    pub bytes: [u8; 512],
}

impl Jesd84B51 {
    pub fn new(file_path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = Self::read_from_file(file_path)?.to_uppercase();
        let content = Self::get_num_array(content)?;
        let content = Self::to_byte_array(content)?;

        let bytes_array: [u8; 512] = content.try_into().expect("Vec has wrong length");
        Ok(Jesd84B51 { bytes: bytes_array })
    }

    fn read_from_file(file_path: &Path) -> Result<String, Box<dyn Error>> {
        let content = fs::read_to_string(file_path)?;
        Ok(content)
    }

    fn get_num_array(str_line: String) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut content = str_line.as_bytes();
        if content.last() == Some(&10) {
            // Exclude "\n"
            content = &content[..content.len() - 1];
        }
        if content.len() != 1024 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "Content length is not 512 bytes",
            )));
        }
        let vec: Vec<u8> = content
            .into_iter()
            .map(|i| *i - if *i <= 57 { 48 } else { 55 })
            .collect();
        Ok(vec)
    }

    fn to_byte_array(num_array: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let vec1: Vec<u8> = num_array
            .iter()
            .enumerate()
            .filter_map(|(i, &v)| if i % 2 == 0 { Some(v) } else { None })
            .map(|i| return i << 4)
            .collect();
        let vec2: Vec<u8> = num_array
            .iter()
            .enumerate()
            .filter_map(|(i, &v)| if i % 2 != 0 { Some(v) } else { None })
            .collect();
        let product_vec: Vec<u8> = vec1.iter().zip(vec2.iter()).map(|(a, b)| a + b).collect();
        Ok(product_vec)
    }
}

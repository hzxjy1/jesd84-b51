use std::{error::Error, fs, path::Path};

use json::JsonValue;

pub struct JsonSheet {
    json_obj: JsonValue,
}

impl JsonSheet {
    pub fn new(file_path: &Path) -> Result<Self, Box<dyn Error>> {
        let binding = fs::read_to_string(file_path)?;
        let json_str = binding.as_str();
        let json_obj = json::parse(json_str)?;
        Ok(JsonSheet { json_obj })
    }
}
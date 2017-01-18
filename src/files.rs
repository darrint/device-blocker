use serde::de::Deserialize;
use serde::ser::Serialize;
use serde_json;
use std::fs::File;

use errors::{Result, ResultExt};

pub fn write_json_file<T: Serialize>(file_name: &str, obj: &T) -> Result<()> {
    let mut writer =
        File::create(file_name).chain_err(|| format!("Failed to open {} for writing", file_name))?;
    serde_json::to_writer_pretty(&mut writer, obj)
        .chain_err(|| format!("Failed to write json file {}", file_name))
}

pub fn read_json_file<T: Deserialize>(file_name: &str) -> Result<T> {
    serde_json::from_reader(
        File::open(file_name)
            .chain_err(|| format!("Failed to open {}", file_name))?)
        .chain_err(|| format!("Failed to read json file {}", file_name))
}

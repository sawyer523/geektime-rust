use std::fs;

use csv::Reader;
use serde::{Deserialize, Serialize};

use crate::cli::CsvOutputFormat;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Player {
    name: String,
    position: String,
    #[serde(rename = "DOB")]
    dob: String,
    nationality: String,
    #[serde(rename = "Kit Number")]
    kit: u8,
}

pub fn process_csv(input: &str, output: String, format: CsvOutputFormat) -> anyhow::Result<()> {
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::with_capacity(128);
    let headers = reader.headers()?.clone();
    for result in reader.records() {
        let record = result?;
        let iter = headers.iter().zip(record.iter());
        let value = match format {
            CsvOutputFormat::Json => iter.collect::<serde_json::Value>(),
            CsvOutputFormat::Yaml => iter.collect::<serde_json::Value>(),
        };
        ret.push(value);
    }

    let content = match format {
        CsvOutputFormat::Json => serde_json::to_string_pretty(&ret)?,
        CsvOutputFormat::Yaml => serde_yaml::to_string(&ret)?,
    };
    fs::write(output, content)?;
    Ok(())
}

use std::{fmt, str::FromStr};

use clap::Parser;

use crate::CmdExecutor;

use super::verify_file;

#[derive(Debug, Clone, Copy)]
pub enum CsvOutputFormat {
    Json,
    Yaml,
}

#[derive(Debug, Parser)]
pub struct CsvOpts {
    #[arg(short, long, value_parser = verify_file)]
    pub input: String,

    #[arg(short, long)] // "output.json".into()
    pub output: Option<String>,

    #[arg(long, value_parser = parse_format, default_value = "json")]
    pub format: CsvOutputFormat,

    #[arg(short, long, default_value_t = ',')]
    pub delimiter: char,

    #[arg(long, default_value_t = true)]
    pub header: bool,
}

impl CmdExecutor for CsvOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let output = if let Some(output) = self.output {
            output
        } else {
            format!("output.{}", self.format)
        };
        crate::process_csv(&self.input, output, self.format)
    }
}

fn parse_format(format: &str) -> Result<CsvOutputFormat, anyhow::Error> {
    format.parse()
}

impl From<CsvOutputFormat> for &'static str {
    fn from(format: CsvOutputFormat) -> Self {
        match format {
            CsvOutputFormat::Json => "json",
            CsvOutputFormat::Yaml => "yaml",
        }
    }
}

impl FromStr for CsvOutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(CsvOutputFormat::Json),
            "yaml" => Ok(CsvOutputFormat::Yaml),
            _ => Err(anyhow::anyhow!("Invalid format")),
        }
    }
}

impl fmt::Display for CsvOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

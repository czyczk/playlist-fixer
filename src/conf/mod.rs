use std::fs::File;
use std::io::{BufReader, Read};

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub input_file: String,
    pub output_file: String,
    pub new_ext: String,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn load_conf(path: &str) -> Result<Config, Error> {
    let file = File::open(path).map_err(|err| Error {
        message: format!("failed to load the config file: {}", err),
    })?;

    let r = BufReader::new(file);
    Ok(parse_conf(Box::new(r))?)
}

fn parse_conf<R>(yaml_stream: R) -> Result<Config, Error>
where
    R: Read,
{
    Ok(serde_yaml::from_reader(yaml_stream).map_err(|err| Error {
        message: format!("failed to parse YAML content: {}", err),
    })?)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    #[test]
    fn test_parse_conf() {
        let input_file = "/path/to/input.m3u8";
        let output_file = "/path/to/output.m3u8";
        let new_ext = ".m4a";

        let example_yaml = format!(
            "\
        inputFile: \"{input_file}\"\n\
        outputFile: \"{output_file}\"\n\
        newExt: \"{new_ext}\"
        "
        );
        let stream = BufReader::new(example_yaml.as_bytes());

        let result = super::parse_conf(stream);
        assert!(result.is_ok());
        {
            let config = result.unwrap();
            assert_eq!(input_file, config.input_file);
            assert_eq!(output_file, config.output_file);
            assert_eq!(new_ext, config.new_ext);
        }
    }

    #[test]
    fn test_parse_invalid_conf() {
        let example_yaml = "missingRequiredKey: \"whatever value\"
        ";
        let stream = BufReader::new(example_yaml.as_bytes());

        let result = super::parse_conf(Box::new(stream));
        assert!(result.is_err());
    }
}

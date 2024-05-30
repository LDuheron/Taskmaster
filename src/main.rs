mod error;
mod parser;

use error::{Error, Result};
use parser::config::Config;

fn main() -> Result<()> {
    let mut config: Config = Config::new();
    config.parse_config_file(String::from("config.ini"))?;
    println!("{:#?}", config);
    Ok(())
}

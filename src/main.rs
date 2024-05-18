mod error;
mod parser;

pub use self::error::{Error, Result};
use parser::Config;

fn main() -> Result<()> {
    let mut config: Config = Config::new();
    config.parse_config_file(String::from("onfig.ini"))?;
    println!("{:#?}", config);
    Ok(())
}

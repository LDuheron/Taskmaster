mod error;
mod parser;

pub use self::error::{Error, Result};
use parser::Config;

fn main() -> Result<()> {
    let mut config: Config = Config::new();
    config.parse_config_file("config.ini".into())?;
    //println!("Config: {:?}", config.map);
    Ok(())
}

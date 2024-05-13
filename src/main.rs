use parsing::Config;
use std::error::Error;
mod parsing;

fn main() -> Result<(), Box<dyn Error>> {
    let mut config: Config = Config::new();
    config.parse_config_file(String::from("config.ini"))?;
    //println!("Config: {:?}", config.map);
    Ok(())
}

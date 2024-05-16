mod error;
mod parser;

// https://doc.rust-lang.org/std/net/struct.TcpListener.html
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html

use std::net::{TcpListener, TcpStream};

pub use self::error::{Error, Result};
use parser::Config;

// fn handle_connection() {

// }

// create socket, biend, listen, accept
// proteger le bind qui peut fail
fn init_server() {
    let listener = TcpListener::bind("127.0.0.1:4241").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("New connection ");
        // handle_connection(stream);
    }
}

fn main() -> Result<()> {
    // let mut config: Config = Config::new();
    // config.parse_config_file(String::from("config.ini"))?;
    init_server();
    Ok(())
}

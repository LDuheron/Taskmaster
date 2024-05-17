mod error;
mod parser;

// https://doc.rust-lang.org/std/net/struct.TcpListener.html
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

pub use self::error::{Error, Result};
use parser::Config;

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    println!("In handle connection");
    loop {
        let mut data = [0; 512];
        let bytes_read = stream
            .read(&mut data)
            .map_err(|e| Error::Default(e.to_string()))?;
        if (bytes_read == 0) {
            println!("Client is disconnected");
            break;
        }
        let formatted = String::from_utf8_lossy(&data[..bytes_read]);
        println!("Received: {}", formatted);
    }
    Ok(())
}

fn init_server() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4241").unwrap_or_else(|err| {
        panic!("Failed to bind");
    });

    for stream in listener.incoming() {
        let stream = stream.unwrap_or_else(|err| panic!("Connexion error")); // faire match
        println!("New connection ");
        handle_connection(stream)?;
        println!("has skipped handle connection");
    }

    Ok(())
}

fn main() -> Result<()> {
    // let mut config: Config = Config::new();
    // config.parse_config_file(String::from("config.ini"))?;
    init_server()?;
    Ok(())
}

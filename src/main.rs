mod error;
mod parser;

// https://doc.rust-lang.org/std/net/struct.TcpListener.html
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html

// Erreurs : si le server exit, le client continue d'envoyer des messages
// Gerer le controle c cote serveur
// gerer plusieurs clients -> bloquer les autres

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
        if bytes_read == 0 {
            println!("Client is disconnected");
            break;
        }
        let formatted = String::from_utf8_lossy(&data[..bytes_read]);
		if (formatted == "PING") {
			println!("received a ping");
		}
        println!("Received: {}", formatted);
    }
    Ok(())
}

fn init_server() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4243").unwrap_or_else(|err| {
        panic!("Failed to bind");
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Info : New connection");
                let _ = handle_connection(stream);
            }
            Err(_) => {
                println!("Error during connection");
            }
        }
        {}
        println!("Info : Client disconnected");
    }

    Ok(())
}

fn main() -> Result<()> {
    // let mut config: Config = Config::new();
    // config.parse_config_file(String::from("config.ini"))?;
    init_server()?;
    Ok(())
}

mod error;
mod parser;

// https://doc.rust-lang.org/std/net/struct.TcpListener.html
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html

use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;

pub use self::error::{Error, Result};
use parser::Config;

fn handle_connection(mut stream: TcpStream) -> Result<()> {
	println!("In handle connection");
	let mut data = [0; 512];
   	let bytes_read = stream.read(&mut data).map_err(|e| Error::Default(e.to_string()))?;
	let formatted = String::from_utf8_lossy(&data[..bytes_read]); 
	println!("Received: {}", formatted);
	Ok(())
}
fn init_server()  -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4241").unwrap_or_else(|err| {
		panic!("Failed to bind"); // pas de panic
	});

	println!("test");
	loop
	{
		for stream in listener.incoming() {
			println!("test 2");
			let stream = stream.unwrap_or_else(|err| {panic!("Connexion error")}); // faire match 
			println!("New connection ");

			handle_connection(stream)?;
			println!("has skipped handle connection");
		}
	}
	Ok(())
}

fn main() -> Result<()> {
    // let mut config: Config = Config::new();
    // config.parse_config_file(String::from("config.ini"))?;
    init_server()?;
    Ok(())
}


// fn handle_connection(mut stream: TcpStream) {
// 	println!("In handle connection");
// 	let mut data = [0; 512];
// 	match stream.read(&mut data) {
// 		Ok(bytes_read)=> {
// 		let formatted = String::from_utf8_lossy(&data[..bytes_read]); 
// 		println!("Received: {}", formatted);
// 		},
// 		Err(_) => {
// 		println!("Error");
// 		}
// 	} {}
// }


// // // create socket, biend, listen, accept
// // // proteger le bind qui peut fail
// fn init_server() {
//     let listener = TcpListener::bind("127.0.0.1:4241").unwrap();

// 	println!("test");
// 	// loop
// 	// {
// 	for stream in listener.incoming() {
// 		println!("test 2");
// 		let stream = stream.unwrap();
// 		println!("New connection ");
// 		handle_connection(stream);
// 		println!("has skipped handle connection");
// 	}
// 	// }
// 	// Ok(())
// }


use std::io::{Write, stdout, stdin};

use user_input::get_input;

mod user_input {
    use std::io;

    pub fn get_input(history: &mut Vec<String>) {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_ok_) => {
                let _ = &history.push(input.clone());
            }
            Err(_not_ok_) => {}
        }
        input.trim().to_string();
    }
}

fn main() {
    let mut history: Vec<String> = vec![];

    loop {
        get_input(&mut history);
        for input in &history {
            println!("{}", input);
        }
        println!("\n");
		// "parse command"
		// renvoyer vers la focntion qui correspond 
    }
}



// Deux modes pour le terminal de rust :
// canonical = process l'input line par line
// raw mode : process byte par byte
// https://packt.medium.com/implementing-terminal-i-o-in-rust-4a44652b0f11


// fn treat_input() {
// 	if start
// 		todo!("Start management");
// 	else if stop
// 		todo!("Stop management");
// 	else if restart
// 		todo!("Restart management");
// }


// Arrows
// up : ^[[A
// down : ^[[D

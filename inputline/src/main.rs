use user_input::get_input;

fn main() {
    let mut history: Vec<String> = vec![];

    loop {
        let _input: String = get_input(&mut history);
        for input in &history {
            println!("{}", input);
        }
        println!("\n");
    }
}

mod user_input {
    use std::io;

    pub fn get_input(history: &mut Vec<String>) -> String {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_ok_) => {
                let _ = &history.push(input.clone());
            }
            Err(_not_ok_) => {}
        }
        input.trim().to_string()
    }
}

// Arrows
// up : ^[[A
// down : ^[[D

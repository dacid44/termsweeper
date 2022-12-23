use std::io::{stdin, stdout, Write};
use crate::game::Field;

mod game;

fn main() {
    println!("Hello, world!");

    let mut field = Field::new((25, 25), 40).unwrap();

    println!("{}", field.render().join("\n"));

    loop {
        print!("Guess: ");
        stdout().flush().unwrap();
        let mut s = String::new();
        stdin().read_line(&mut s).unwrap();
        let nums = s.trim().split_whitespace().collect::<Vec<_>>();
        let row = nums[0].parse::<usize>().unwrap();
        let col = nums[1].parse::<usize>().unwrap();
        println!("{:?}", field.clear_cell((row, col)));
        println!("{}", field.render().join("\n"));
    }
}

// TODO next: resolve borrow checker issues, display the board with crossterm (and then, controls)
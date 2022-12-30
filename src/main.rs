use crate::game::{Field, Game};

mod game;
mod tui;

fn main() {
    println!("Hello, world!");

    let mut game = Game::new(Field::new((25, 25), 40).unwrap()).unwrap();

    game.render().unwrap();

    while game.handle_event(crossterm::event::read().unwrap()).unwrap() {
//        print!("Guess: ");
//        stdout().flush().unwrap();
//        let mut s = String::new();
//        stdin().read_line(&mut s).unwrap();
//        let nums = s.trim().split_whitespace().collect::<Vec<_>>();
//        let row = nums[0].parse::<usize>().unwrap();
//        let col = nums[1].parse::<usize>().unwrap();
//        println!("{:?}", game.field.clear_cell((row, col)));
//        game.render().unwrap();

//        game.handle_event(crossterm::event::read().unwrap()).unwrap();
        game.render().unwrap();
    }
}

// TODO: Make relocatable
use std::fmt::{Display, Formatter};
use std::io::{stdout, stderr, Write};
use rand::{Rng, thread_rng};
use rand::distributions::Uniform;
use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    event::{Event, KeyEvent, KeyEventKind, KeyCode},
    cursor::{MoveTo},
};

use crate::tui::{Component, BoxedComponent, Controls, Title};

type IoResult<T> = std::io::Result<T>;

//#[derive(Debug)]
pub(crate) struct Game {
    pub(crate) field: Field,
    field_loc: (u16, u16),
    cursor: (u16, u16),
    terminal_size: (u16, u16),
    game_ended: bool,
    closed: bool,
}

impl Game {
    pub(crate) fn new(field: Field) -> IoResult<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
//        crossterm::terminal::enable_raw_mode()?;
        Ok(Self {
            field,
            field_loc: (1, 1),
            cursor: (0, 0),
            terminal_size: terminal::size()?,
            game_ended: false,
            closed: false
        })
    }

    pub(crate) fn close(&mut self) -> IoResult<()> {
//        crossterm::terminal::disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        self.closed = true;
        Ok(())
    }

    pub(crate) fn render(&self) -> IoResult<()> {
        let mut buffer = vec![String::new(); self.terminal_size.1 as usize];
        let buf = BoxedComponent(&self.field).render_at(&mut buffer);
        let buf = BoxedComponent(&Controls).render_at(buf);
        if self.game_ended {
            Title::new("Game Over").render_at(buf);
        }
        execute!(stdout(), MoveTo(0, 0))?;
        write!(stdout(), "{}", buffer.into_iter()
            .collect::<Vec<_>>()
            .join("\n")
        )?;

        execute!(stdout(), MoveTo(self.cursor.0 + self.field_loc.0, self.cursor.1 + self.field_loc.1))?;
        write!(stdout(), "◎")?;
        execute!(stdout(), MoveTo(0, self.field.board.len() as u16 + 1))
    }

    // Returned bool indicates whether to continue (true for continue, false for exit)
    pub(crate) fn handle_event(&mut self, event: Event) -> IoResult<bool> {
        match event {
            Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) => match code {
                KeyCode::Left => self.step_cursor(Direction::Left),
                KeyCode::Right => self.step_cursor(Direction::Right),
                KeyCode::Up => self.step_cursor(Direction::Up),
                KeyCode::Down => self.step_cursor(Direction::Down),
                KeyCode::Char(' ') => {
                    let r = self.field.clear_cell((self.cursor.1 as usize, self.cursor.0 as usize));
                    if matches!(r, Some(true)) {
                        self.game_ended = true;
                    }
                },
                KeyCode::Char('f') => {
                    let _ = self.field.toggle_flag((self.cursor.1 as usize, self.cursor.0 as usize));
                }
                KeyCode::Char('q') => return Ok(false),
                _ => { },
            }
            Event::Resize(width, height) => self.terminal_size = (width, height),
            _ => { },
        }

        Ok(true)
    }

    fn move_cursor(&mut self, pos: (u16, u16)) {
        if pos.0 >= self.field_loc.0
            && pos.1 >= self.field_loc.1
            && pos.0 < self.field_loc.0 + self.field.width() as u16
            && pos.1 < self.field_loc.1 + self.field.height() as u16
        {
            self.cursor.0 = pos.0 - self.field_loc.0;
            self.cursor.1 = pos.1 - self.field_loc.1;
        }
    }

    fn step_cursor(&mut self, direction: Direction) {
        let new_pos = direction.offset(self.cursor);
        if new_pos.0 < self.field.width() as u16 && new_pos.1 < self.field.height() as u16 {
            self.cursor = new_pos;
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        use std::thread::panicking;
        if !self.closed {
            if let Err(e) = self.close() {
                if panicking() {
                    let _ = writeln!(stderr(), "{}", e);
                } else {
                    Err::<(), std::io::Error>(e).unwrap();
                }
            }
        }
    }
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn offset(&self, pos: (u16, u16)) -> (u16, u16) {
        match self {
            Direction::Left => (if pos.0 > 0 { pos.0 - 1 } else { 0 }, pos.1),
            Direction::Right => (pos.0 + 1, pos.1),
            Direction::Up => (pos.0, if pos.1 > 0 { pos.1 - 1 } else { 0 }),
            Direction::Down => (pos.0, pos.1 + 1),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) board: Vec<Vec<Cell>>,
}

impl Field {
    /// Returns None if either dimension was zero, or too many mines were specified than can (reasonably)
    /// fit on the board.
    pub(crate) fn new(size: (usize, usize), mines: usize) -> Option<Self> {
        if size.0 == 0 || size.1 == 0 || mines > (size.0 * size.1 + 1) / 2 {
            return None;
        }

        let mut board = vec![vec![Cell::default(); size.1]; size.0];

        let mut rng = thread_rng();
        let row_d = Uniform::new(0, size.0);
        let col_d = Uniform::new(0, size.1);

        let mut placed_mines = 0;
        while placed_mines < mines {
            let mine_row = rng.sample(row_d);
            let mine_col = rng.sample(col_d);
            if board[mine_row][mine_col].mine {
                continue;
            }

            board[mine_row][mine_col].mine = true;

            let top_edge = mine_row == 0;
            let bottom_edge = mine_row == board.len() - 1;
            let left_edge = mine_col == 0;
            let right_edge = mine_col == board[0].len() - 1;

            if !left_edge && !top_edge { board[mine_row - 1][mine_col - 1].neighbors += 1 }
            if !left_edge { board[mine_row][mine_col - 1].neighbors += 1 }
            if !left_edge && !bottom_edge { board[mine_row + 1][mine_col - 1].neighbors += 1 }
            if !bottom_edge { board[mine_row + 1][mine_col].neighbors += 1 }
            if !right_edge && !bottom_edge { board[mine_row + 1][mine_col + 1].neighbors += 1 }
            if !right_edge { board[mine_row][mine_col + 1].neighbors += 1 }
            if !right_edge && !top_edge { board[mine_row - 1][mine_col + 1].neighbors += 1 }
            if !top_edge { board[mine_row - 1][mine_col].neighbors += 1 }

            placed_mines += 1;
        }

        Some(Self { board })
    }

    /// Returns a bool signifying if a mine has exploded. Returns None if the given cell has already
    /// been cleared or flagged, or if the given cell is invalid.
    pub(crate) fn clear_cell(&mut self, pos: (usize, usize)) -> Option<bool> {
        match self.board.get_mut(pos.0)?
            .get_mut(pos.1)?
            .reveal()?
        {
            RevealStatus::Exploded => return Some(true),
            RevealStatus::Safe => return Some(false),
            RevealStatus::Empty => {}
        }

        let mut check = Vec::new();

        fn add_neighbors(check: &mut Vec<(usize, usize)>, board_size: (usize, usize), pos: (usize, usize)) {
            let top_edge = pos.0 == 0;
            let bottom_edge = pos.0 == board_size.0 - 1;
            let left_edge = pos.1 == 0;
            let right_edge = pos.1 == board_size.1 - 1;

            if !left_edge && !top_edge { check.push((pos.0 - 1, pos.1 - 1)) }
            if !left_edge { check.push((pos.0, pos.1 - 1)) }
            if !left_edge && !bottom_edge { check.push((pos.0 + 1, pos.1 - 1)) }
            if !bottom_edge { check.push((pos.0 + 1, pos.1)) }
            if !right_edge && !bottom_edge { check.push((pos.0 + 1, pos.1 + 1)) }
            if !right_edge { check.push((pos.0, pos.1 + 1)) }
            if !right_edge && !top_edge { check.push((pos.0 - 1, pos.1 + 1)) }
            if !top_edge { check.push((pos.0 - 1, pos.1)) }
        }

        add_neighbors(&mut check, (self.board.len(), self.board[0].len()), pos);

        while !check.is_empty() {
            let (next_row, next_col) = check.pop().expect("We shouldn't have started another iteration if the stack is empty");
            if matches!(self.board[next_row][next_col].reveal(), Some(RevealStatus::Empty)) {
                add_neighbors(&mut check, (self.board.len(), self.board[0].len()), (next_row, next_col));
            }
        }

        Some(false)
    }

    /// Returns a bool signifying that the flag was valid (i.e., that the cell was not already
    /// revealed). Returns None if the cell was invalid.
    fn toggle_flag(&mut self, pos: (usize, usize)) -> Option<bool> {
        Some(
            self.board.get_mut(pos.0)?
                .get_mut(pos.1)?
                .toggle_flag()
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Cell {
    state: CellState,
    neighbors: u8,
    mine: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self { state: CellState::Unrevealed, neighbors: 0, mine: false }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self.state {
            CellState::Unrevealed => "█".to_string(),
//            CellState::Unrevealed => "▓".to_string(),
            CellState::Flagged => "⚑".to_string(),
            CellState::Revealed => self.neighbors.to_string(),
            CellState::Exploded => "✲".to_string(),
            CellState::Empty => "░".to_string(),
        })
    }
}

impl Cell {
    /// Returns None if the cell has already been cleared or flagged.
    fn reveal(&mut self) -> Option<RevealStatus> {
        match self.state {
            CellState::Unrevealed if self.mine => {
                self.state = CellState::Exploded;
                Some(RevealStatus::Exploded)
            },
            CellState::Unrevealed if self.neighbors == 0 => {
                self.state = CellState::Empty;
                Some(RevealStatus::Empty)
            },
            CellState::Unrevealed => {
                self.state = CellState::Revealed;
                Some(RevealStatus::Safe)
            }
            _ => None,
        }
    }

    /// Returns a bool signifying that the flag was valid (i.e., that the cell was not already
    /// revealed).
    fn toggle_flag(&mut self) -> bool {
        match self.state {
            CellState::Unrevealed => {
                self.state = CellState::Flagged;
                true
            },
            CellState::Flagged => {
                self.state = CellState::Unrevealed;
                true
            },
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum CellState {
    Unrevealed, // Initial state
    Flagged,    // Flagged
    Revealed,   // Clicked on, showing a number
    Exploded,   // Clicked on, was a mine
    Empty,      // Clicked on, no mines
}

enum RevealStatus {
    Exploded,
    Safe,
    Empty,
}

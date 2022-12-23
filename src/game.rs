use std::fmt::{Display, Formatter};
use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

#[derive(Debug)]
struct Game {

}

#[derive(Debug)]
pub(crate) struct Field {
    board: Vec<Vec<Cell>>,
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

    pub(crate) fn render(&self) -> Vec<String> {
        self.board.iter()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect())
            .collect()
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
}

#[derive(Copy, Clone, Debug)]
struct Cell {
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
    Unrevealed,
    Flagged,
    Revealed,
    Exploded,
    Empty,
}

enum RevealStatus {
    Exploded,
    Safe,
    Empty,
}

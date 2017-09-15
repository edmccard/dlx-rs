extern crate dlx;

use dlx::{Index, Row};

/// Reads 9x9 sudoku clues in .sdm text format (one grid per
/// 81-character line) and prints the solutions.
fn main() {
    let mut solver = dlx::Solver::new(324, SudokuMatrix::new());
    let stdin = ::std::io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        if stdin.read_line(&mut line).unwrap() == 0 {
            return;
        }
        let mut solutions = Solutions { solved: false };
        solver.solve(rows_from_line(line.trim()), &mut solutions);
        if !solutions.solved {
            println!("No solution");
        }
    }
}

const SIZE_RT: Index = 3;
const SIZE: Index = 9;
const SIZE_SQ: Index = 81;

/// Build a matrix that encodes 9x9 sudoku as an exact cover problem.
///
/// The matrix has 324 columns:
///
/// * 81 for "cell (x, y) is occupied"
/// * 81 for "num n present in col c"
/// * 81 for "num n present in row r"
/// * 81 for "num n present in box b"
struct SudokuMatrix {
    num: Index,
    row: Index,
    col: Index,
}

impl SudokuMatrix {
    fn new() -> SudokuMatrix {
        SudokuMatrix {
            num: 0,
            row: 0,
            col: 0,
        }
    }
}

impl Iterator for SudokuMatrix {
    type Item = Row;
    fn next(&mut self) -> Option<Self::Item> {
        if self.num >= SIZE {
            return None;
        }
        let row = sudoku_cover_row(self.num, self.row, self.col);
        self.col += 1;
        if self.col == SIZE {
            self.col = 0;
            self.row += 1;
            if self.row == SIZE {
                self.row = 0;
                self.num += 1;
            }
        }
        Some(row)
    }
}

/// Return the row representing the constraints satisfied by the
/// number num in the cell (row, col).
fn sudoku_cover_row(num: Index, row: Index, col: Index) -> Vec<Index> {
    let bx = ((row / SIZE_RT) * SIZE_RT) + (col / SIZE_RT);
    vec![
        row * SIZE + col,
        SIZE_SQ + num * SIZE + col,
        SIZE_SQ * 2 + num * SIZE + row,
        SIZE_SQ * 3 + num * SIZE + bx,
    ]
}

/// Return the rows for the constraints satisfied by the entries in a
/// sudoku clue.
#[cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]
fn rows_from_line(line: &str) -> Vec<Row> {
    if line.len() != SIZE_SQ {
        panic!("unknown format");
    }
    let mut rows = Vec::new();
    let line = line.as_bytes();
    for row in 0..SIZE {
        for col in 0..SIZE {
            let c = line[row * SIZE + col];
            if c < b'1' || c > b'9' {
                continue;
            }
            let num = c - b'1';
            rows.push(sudoku_cover_row(num as Index, row, col));
        }
    }
    rows
}

/// A record of the solution for a single sudoku puzzle.
struct Solutions {
    solved: bool,
}

impl dlx::Solutions for Solutions {
    /// Print the first solution and stop.
    fn push(&mut self, sol: dlx::Solution) -> bool {
        self.solved = true;
        // Convert the rows from the exact cover matrix into a
        // standard sudoku grid.
        let mut grid = vec![b'0'; SIZE_SQ];
        for row in sol {
            let cell_row = row[0] / SIZE;
            let dlx_col = row[1] - SIZE_SQ;
            let cell_num = dlx_col / SIZE;
            let cell_col = dlx_col % SIZE;
            grid[cell_row * SIZE + cell_col] = b'1' + cell_num as u8;
        }
        println!("{}", ::std::str::from_utf8(&grid).unwrap());
        // Stop after finding one solution.
        false
    }
}

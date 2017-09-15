#[cfg(test)]
#[macro_use]
extern crate lazy_static;


pub type Index = usize;

// TODO: const generics
pub type Row = Vec<Index>;

struct Entry {
    left: Index,
    right: Index,
    up: Index,
    down: Index,
    // For headers, x1 is the column_number.
    // For data, x1 is the index of the header.
    x1: Index,
    // For headers, x2 is the size (1-count).
    // For data, x2 is 1 for the start of a row, 0 otherwise.
    x2: Index,
}

impl Default for Entry {
    fn default() -> Entry {
        Entry {
            left: 0,
            right: 0,
            up: 0,
            down: 0,
            x1: 0,
            x2: 0,
        }
    }
}

impl Entry {
    fn new() -> Entry {
        Default::default()
    }
}

pub struct Solver {
    es: Vec<Entry>,
    sol_rows: Vec<Index>,
    finished: bool,
}

impl Solver {
    pub fn new<I: Iterator<Item = Row>>(ncols: usize, rows: I) -> Solver {
        let mut es = Vec::new();
        Solver::add_headers(&mut es, ncols);
        // Bottoms keeps track of the index of the "bottom" entry in
        // each column, updated as rows are added.
        let mut bottoms = Vec::new();
        bottoms.extend(2..ncols + 2);
        for row in rows {
            Solver::add_row(&mut es, &row, &mut bottoms);
        }
        // Set the "tops" to point up at the "bottoms", and vice
        // versa.
        let mut idx = 2;
        while idx != 1 {
            es[idx].up = bottoms[idx - 2];
            es[bottoms[idx - 2]].down = idx;
            idx = es[idx].right;
        }
        Solver {
            es,
            sol_rows: Vec::new(),
            finished: false,
        }
    }

    fn add_headers(es: &mut Vec<Entry>, ncols: usize) {
        // Skip entry 0 since index 0 corresponds to a null pointer.
        es.push(Entry::new());
        // The first element is the root.
        es.push(Entry {
            right: 2,
            ..Default::default()
        });
        // The next ncols elements are the list headers.
        for i in 1..ncols + 1 {
            es.push(Entry {
                left: i,
                right: i + 2,
                x1: i - 1,
                ..Default::default()
            });
        }
        // Connect the rightmost header to the root.
        es[1].left = ncols + 1;
        es[ncols + 1].right = 1;
    }

    fn add_row(es: &mut Vec<Entry>, row: &[Index], bottoms: &mut Vec<Index>) {
        let row_start = es.len();
        for &col_num in row {
            let idx = es.len();
            // Add an entry linked to the correct column and pointing
            // up to the current bottom.
            es.push(Entry {
                x1: col_num + 2,
                left: idx - 1,
                right: idx + 1,
                up: bottoms[col_num],
                ..Default::default()
            });
            // Update the bottom of the column to point to the new
            // entry.
            es[bottoms[col_num]].down = idx;
            bottoms[col_num] = idx;
            // Increment the count of non-zero entries in the column.
            es[col_num + 2].x2 += 1;
        }
        // Connect the leftmost and rightmost entries in the row.
        let row_end = es.len() - 1;
        es[row_end].right = row_start;
        es[row_start].left = row_end;
        // Mark the leftmost entry as the start of a row.
        es[row_start].x2 = 1;
    }

    // Choose the column that results in the least amount of recursive
    // calls.
    fn choose_column(&self) -> Index {
        let mut min_size = Index::max_value();
        let mut min_hdr = 0;
        let mut hdr = self.es[1].right;
        while hdr != 1 {
            if self.es[hdr].x2 < min_size {
                min_hdr = hdr;
                min_size = self.es[hdr].x2;
            }
            hdr = self.es[hdr].right;
        }
        min_hdr
    }

    fn cover(&mut self, hdr: Index) {
        let hdr_right = self.es[hdr].right;
        self.es[hdr_right].left = self.es[hdr].left;
        let hdr_left = self.es[hdr].left;
        self.es[hdr_left].right = self.es[hdr].right;
        let mut row = self.es[hdr].down;
        while row != hdr {
            let mut col = self.es[row].right;
            while col != row {
                let col_hdr = self.es[col].x1;
                self.es[col_hdr].x2 -= 1;
                let col_up = self.es[col].up;
                self.es[col_up].down = self.es[col].down;
                let col_down = self.es[col].down;
                self.es[col_down].up = self.es[col].up;
                col = self.es[col].right;
            }
            row = self.es[row].down;
        }
    }

    fn uncover(&mut self, hdr: Index) {
        let mut row = self.es[hdr].up;
        while row != hdr {
            let mut col = self.es[row].left;
            while col != row {
                let col_hdr = self.es[col].x1;
                self.es[col_hdr].x2 += 1;
                let col_up = self.es[col].up;
                self.es[col_up].down = col;
                let col_down = self.es[col].down;
                self.es[col_down].up = col;
                col = self.es[col].left;
            }
            row = self.es[row].up;
        }
        let hdr_right = self.es[hdr].right;
        self.es[hdr_right].left = hdr;
        let hdr_left = self.es[hdr].left;
        self.es[hdr_left].right = hdr;
    }

    pub fn solve(&mut self, partial: Vec<Row>, sols: &mut Solutions) {
        self.finished = false;
        self.sol_rows.clear();

        for row in &partial {
            for col_num in row {
                self.cover(col_num + 2);
            }
        }

        self.solveception(&partial, sols);

        for row in partial.iter().rev() {
            for col_num in row.iter().rev() {
                self.uncover(col_num + 2);
            }
        }
    }

    fn solveception(&mut self, partial: &[Row], sols: &mut Solutions) {
        // We have a solution if we've successfully covered all the
        // columns.
        if self.es[1].right == 1 {
            self.finished = {
                let sol = Solution {
                    solver: self,
                    rows: self.sol_rows.iter(),
                    partial: partial.iter(),
                };
                !sols.push(sol)
            };
            return;
        }

        let hdr = self.choose_column();
        self.cover(hdr);

        // For each row with an entry in the chosen column:
        let mut row_idx = self.es[hdr].down;
        while row_idx != hdr {
            // add it to the partial solution,
            self.sol_rows.push(row_idx);
            // cover all the other rows with overlapping entries,
            let mut col_idx = self.es[row_idx].right;
            while col_idx != row_idx {
                let col_hdr = self.es[col_idx].x1;
                let col_num = self.es[col_hdr].x1;
                self.cover(col_num + 2);
                col_idx = self.es[col_idx].right;
            }
            // recurse,
            self.solveception(partial, sols);
            // then undo.
            col_idx = self.es[row_idx].left;
            while col_idx != row_idx {
                let col_hdr = self.es[col_idx].x1;
                let col_num = self.es[col_hdr].x1;
                self.uncover(col_num + 2);
                col_idx = self.es[col_idx].left;
            }
            self.sol_rows.pop();
            if self.finished {
                break;
            }
            row_idx = self.es[row_idx].down;
        }
        self.uncover(hdr);
    }
}

/// An iterator over the rows of a solution.
pub struct Solution<'a, 'b> {
    solver: &'a Solver,
    partial: ::std::slice::Iter<'b, Row>,
    rows: ::std::slice::Iter<'a, Index>,
}

impl<'a, 'b> Iterator for Solution<'a, 'b> {
    type Item = Row;
    // First returns each row from the inital partial solution, then
    // returns each row from the rest of the solution.
    fn next(&mut self) -> Option<Self::Item> {
        match self.partial.next() {
            Some(row) => Some(row.clone()),
            None => match self.rows.next() {
                None => None,
                Some(&row_idx) => Some(self.get_row(row_idx)),
            },
        }
    }
}

impl<'a, 'b> Solution<'a, 'b> {
    // Returns the indexes for all entries in the same row as the one
    // at row_idx.
    fn get_row(&self, row_idx: Index) -> Row {
        let mut row_idx = row_idx;
        let es = &self.solver.es;
        while es[row_idx].x2 == 0 {
            row_idx = es[row_idx].left;
        }
        let mut row = vec![es[es[row_idx].x1].x1];
        let mut col_idx = es[row_idx].right;
        while col_idx != row_idx {
            row.push(es[es[col_idx].x1].x1);
            col_idx = es[col_idx].right;
        }
        row
    }
}

pub trait Solutions {
    /// Handle a solution.
    ///
    /// Return true to keep looking for more solutions, or false to
    /// quit.
    fn push(&mut self, sol: Solution) -> bool;
}

#[cfg(test)]
mod test {
    use {Row, Solution, Solutions, Solver};

    lazy_static! {
        // Rows 0-26 ((num-1)*9 + row*3 + col): num in cell(row,col)
        //
        // Columns 0-8   (row*3 + col):      cell(row,col) is filled
        // Columns 9-17  (9 + (n-1)*3 + r):  num n in row r
        // Columns 18-16 (18 + (n-1)*3 + c): num n in col c
        static ref LATIN_3X3_MATRIX: Vec<Row> = vec![
            vec![0, 9, 18],
            vec![1, 9, 19],
            vec![2, 9, 20],
            vec![3, 10, 18],
            vec![4, 10, 19],
            vec![5, 10, 20],
            vec![6, 11, 18],
            vec![7, 11, 19],
            vec![8, 11, 20],
            vec![0, 12, 21],
            vec![1, 12, 22],
            vec![2, 12, 23],
            vec![3, 13, 21],
            vec![4, 13, 22],
            vec![5, 13, 23],
            vec![6, 14, 21],
            vec![7, 14, 22],
            vec![8, 14, 23],
            vec![0, 15, 24],
            vec![1, 15, 25],
            vec![2, 15, 26],
            vec![3, 16, 24],
            vec![4, 16, 25],
            vec![5, 16, 26],
            vec![6, 17, 24],
            vec![7, 17, 25],
            vec![8, 17, 26],
        ];

        // For compactness, solutions are represented here as vectors
        // of sorted indexes into LATIN_3X3_MATRIX.
        static ref LATIN_3X3_SOLS: Vec<Row> = vec![
            vec![0, 4, 8, 10, 14, 15, 20, 21, 25], // 123312231
            vec![0, 4, 8, 11, 12, 16, 19, 23, 24], // 132213321
            vec![0, 5, 7, 10, 12, 17, 20, 22, 24], // 123231312
            vec![0, 5, 7, 11, 13, 15, 19, 21, 26], // 132321213
            vec![1, 3, 8, 9, 14, 16, 20, 22, 24], //  213132321
            vec![1, 3, 8, 11, 13, 15, 18, 23, 25], // 312123231
            vec![1, 5, 6, 9, 13, 17, 20, 21, 25], //  213321132
            vec![1, 5, 6, 11, 12, 16, 18, 22, 26], // 312231123
            vec![2, 3, 7, 9, 13, 17, 19, 23, 24], //  231123312
            vec![2, 3, 7, 10, 14, 15, 18, 22, 26], // 321132213
            vec![2, 4, 6, 9, 14, 16, 19, 21, 26], //  231312123
            vec![2, 4, 6, 10, 12, 17, 18, 23, 25], // 321213132
        ];
    }

    struct LS3x3Solutions {
        count: usize,
        expected_count: usize,
        sol_rows: Vec<Row>,
    }

    impl LS3x3Solutions {
        fn new(expected_count: usize) -> LS3x3Solutions {
            LS3x3Solutions {
                count: 0,
                sol_rows: Vec::new(),
                expected_count,
            }
        }
    }

    impl Solutions for LS3x3Solutions {
        fn push(&mut self, sols: Solution) -> bool {
            let mut idxs = Vec::new();
            for sol in sols {
                let idx = LATIN_3X3_MATRIX.iter().position(|s| *s == sol);
                assert!(idx.is_some(), format!("invalid row {:?}", sol));
                idxs.push(idx.unwrap());
            }
            idxs.sort();
            self.sol_rows.push(idxs.clone());
            let ok = LATIN_3X3_SOLS.iter().find(|&i| *i == idxs);
            assert!(ok.is_some(), format!("invalid rows {:?}", idxs));
            self.count += 1;
            if self.count > self.expected_count {
                return false;
            }
            true
        }
    }

    #[test]
    fn latin_squares_3x3_all() {
        let mut solver = Solver::new(27, LATIN_3X3_MATRIX.clone().into_iter());
        let mut sols = LS3x3Solutions::new(12);
        solver.solve(Vec::new(), &mut sols);
        assert_eq!(12, sols.count);
    }

    #[test]
    fn latin_squares_3x3_invalid_clue() {
        let mut solver = Solver::new(27, LATIN_3X3_MATRIX.clone().into_iter());
        let mut sols = LS3x3Solutions::new(0);
        let clues = vec![
            vec![0, 9, 18],  // 1--
            vec![4, 13, 22], // -2-
            vec![8, 11, 20], // --1
        ];
        solver.solve(clues, &mut sols);
        assert_eq!(0, sols.count);
    }

    #[test]
    fn latin_squares_3x3_single() {
        let mut solver = Solver::new(27, LATIN_3X3_MATRIX.clone().into_iter());
        let mut sols = LS3x3Solutions::new(1);
        let clues = vec![
            vec![0, 9, 18], //  1--
            //                  ---
            vec![8, 14, 23], // --2
        ];
        solver.solve(clues, &mut sols);
        assert_eq!(1, sols.count);
        assert_eq!(vec![0, 5, 7, 10, 12, 17, 20, 22, 24], sols.sol_rows[0]);
    }

    #[test]
    fn latin_squares_3x3_solver_reuse() {
        let mut solver = Solver::new(27, LATIN_3X3_MATRIX.clone().into_iter());
        let mut sols1 = LS3x3Solutions::new(1);
        let clues1 = vec![
            vec![0, 9, 18], //  1--
            //                  ---
            vec![8, 14, 23], // --2
        ];
        let mut sols2 = LS3x3Solutions::new(1);
        let clues2 = vec![
            vec![0, 12, 21], // 2--
            //                  ---
            vec![8, 11, 20], // --1
        ];
        solver.solve(clues1, &mut sols1);
        solver.solve(clues2, &mut sols2);
        assert_eq!(1, sols1.count);
        assert_eq!(1, sols2.count);
        assert_eq!(vec![0, 5, 7, 10, 12, 17, 20, 22, 24], sols1.sol_rows[0]);
        assert_eq!(vec![1, 3, 8, 9, 14, 16, 20, 22, 24], sols2.sol_rows[0]);
    }
}

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

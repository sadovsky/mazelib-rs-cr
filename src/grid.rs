//! A rectangular wall/passage grid stored row-major in a `Vec<u8>`.

/// A `(row, col)` position, row 0 at the top.
pub type Cell = (usize, usize);

/// A path: a list of cells.
pub type Path = Vec<Cell>;

/// Wall value.
pub const WALL: u8 = 1;
/// Passage value.
pub const OPEN: u8 = 0;

/// A maze grid. A maze of `h` x `w` cells is a grid of `2h+1` rows by `2w+1` columns;
/// odd/odd positions are cells, everything else is a wall slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Grid {
    rows: usize,
    cols: usize,
    data: Vec<u8>,
}

impl Grid {
    /// A grid of `rows` x `cols`, every position set to `fill`.
    pub fn new(rows: usize, cols: usize, fill: u8) -> Self {
        Grid {
            rows,
            cols,
            data: vec![fill; rows * cols],
        }
    }

    /// A grid of all walls.
    pub fn walls(rows: usize, cols: usize) -> Self {
        Self::new(rows, cols, WALL)
    }

    /// Number of rows.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// `(rows, cols)`.
    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Whether `(r, c)` lies inside the grid.
    pub fn in_bounds(&self, r: usize, c: usize) -> bool {
        r < self.rows && c < self.cols
    }

    /// The value at `(r, c)`. Panics when out of bounds.
    pub fn get(&self, r: usize, c: usize) -> u8 {
        assert!(self.in_bounds(r, c), "grid index out of bounds");
        self.data[r * self.cols + c]
    }

    /// The value at `(r, c)`, or `None` when out of bounds.
    pub fn try_get(&self, r: usize, c: usize) -> Option<u8> {
        self.in_bounds(r, c).then(|| self.data[r * self.cols + c])
    }

    /// Set the value at `(r, c)`. Panics when out of bounds.
    pub fn set(&mut self, r: usize, c: usize, v: u8) {
        assert!(self.in_bounds(r, c), "grid index out of bounds");
        self.data[r * self.cols + c] = v;
    }

    /// Whether `(r, c)` is a wall. Out-of-bounds positions count as walls.
    pub fn is_wall(&self, r: usize, c: usize) -> bool {
        self.try_get(r, c).is_none_or(|v| v != OPEN)
    }

    /// Whether `(r, c)` is an in-bounds passage.
    pub fn is_open(&self, r: usize, c: usize) -> bool {
        self.try_get(r, c) == Some(OPEN)
    }

    /// The raw row-major data.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Parse the protocol encoding: one string per row, `#` wall, anything else open.
    pub fn from_rows<S: AsRef<str>>(rows: &[S]) -> Self {
        let r = rows.len();
        let c = rows.first().map_or(0, |s| s.as_ref().chars().count());
        let mut g = Grid::walls(r, c);
        for (i, row) in rows.iter().enumerate() {
            for (j, ch) in row.as_ref().chars().enumerate().take(c) {
                g.set(i, j, if ch == '#' { WALL } else { OPEN });
            }
        }
        g
    }

    /// The protocol encoding: one string per row, `#` wall, `.` open.
    pub fn to_rows(&self) -> Vec<String> {
        (0..self.rows)
            .map(|r| {
                (0..self.cols)
                    .map(|c| if self.get(r, c) == OPEN { '.' } else { '#' })
                    .collect()
            })
            .collect()
    }

    /// In-bounds 4-neighbours of `(r, c)` at distance `step` (N, S, W, E order).
    pub fn neighbors(&self, (r, c): Cell, step: usize) -> Vec<Cell> {
        let mut out = Vec::with_capacity(4);
        if r >= step {
            out.push((r - step, c));
        }
        if r + step < self.rows {
            out.push((r + step, c));
        }
        if c >= step {
            out.push((r, c - step));
        }
        if c + step < self.cols {
            out.push((r, c + step));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_rows() {
        let rows = ["###", "#.#", "###"];
        let g = Grid::from_rows(&rows);
        assert_eq!(g.shape(), (3, 3));
        assert!(g.is_open(1, 1));
        assert!(g.is_wall(0, 0));
        assert!(g.is_wall(9, 9));
        assert_eq!(g.to_rows(), rows);
    }

    #[test]
    fn neighbors_clip_at_edges() {
        let g = Grid::walls(5, 5);
        assert_eq!(g.neighbors((1, 1), 2), vec![(3, 1), (1, 3)]);
        assert_eq!(g.neighbors((2, 2), 1).len(), 4);
    }
}

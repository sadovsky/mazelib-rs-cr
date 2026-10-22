use super::*;
use crate::generate::Prims;
use std::collections::VecDeque;

const G1: [&str; 7] = [
    "#######", "#.....#", "#.#####", "#...#.#", "#.###.#", "#.....#", "#######",
];

const G2: [&str; 13] = [
    "#############",
    "#...........#",
    "#.###.#####.#",
    "#.#.....#...#",
    "###.#####.#.#",
    "#.....#...#.#",
    "#.#.###.###.#",
    "#.#...#...#.#",
    "#.#########.#",
    "#.........#.#",
    "#.#####.###.#",
    "#.#.......#.#",
    "#############",
];

fn maze_with(rows: &[&str]) -> Maze {
    let mut m = Maze::new();
    m.grid = Some(Grid::from_rows(rows));
    m
}

fn g1_path() -> Path {
    vec![(3, 1), (2, 1), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5)]
}

#[test]
fn defaults() {
    let m = Maze::new();
    assert!(m.grid.is_none() && m.start.is_none() && m.end.is_none());
    assert!(m.generator.is_none() && m.solver.is_none());
    assert!(m.transmuters.is_empty() && m.solutions.is_none());
    assert!(m.prune);
    assert_eq!(m.tostring(true, true), "");
}

#[test]
fn bad_seed_is_value_error() {
    assert!(matches!(
        Maze::with_seed(-7),
        Err(MazeError::InvalidValue(_))
    ));
    assert!(matches!(
        Maze::with_seed(4_294_967_301),
        Err(MazeError::InvalidValue(_))
    ));
    assert!(Maze::with_seed(0).is_ok());
}

#[test]
fn tostring_g1_oo() {
    let mut m = maze_with(&G1);
    m.start = Some((3, 0));
    m.end = Some((1, 6));
    m.solutions = Some(vec![g1_path()]);
    let plain = "#######\n#     #\n# #####\n#   # #\n# ### #\n#     #\n#######";
    assert_eq!(m.tostring(false, false), plain);
    assert_eq!(
        m.tostring(false, true),
        "#######\n#+++++#\n#+#####\n#+  # #\n# ### #\n#     #\n#######"
    );
    assert_eq!(
        m.tostring(true, false),
        "#######\n#     E\n# #####\nS   # #\n# ### #\n#     #\n#######"
    );
    assert_eq!(
        m.tostring(true, true),
        "#######\n#+++++E\n#+#####\nS+  # #\n# ### #\n#     #\n#######"
    );
    assert_eq!(m.to_string(), m.tostring(true, true));
    m.start = None;
    m.end = None;
    m.solutions = None;
    assert_eq!(m.tostring(true, true), plain);
}

#[test]
fn tostring_g1_ii_and_partial_entrances() {
    let mut m = maze_with(&G1);
    m.start = Some((5, 3));
    m.end = Some((3, 1));
    m.solutions = Some(vec![]);
    let want = "#######\n#     #\n# #####\n#E  # #\n# ### #\n#  S  #\n#######";
    assert_eq!(m.tostring(true, true), want);
    assert_eq!(m.tostring(true, false), want);
    m.end = None;
    assert!(!m.tostring(true, false).contains('S'));
}

#[test]
fn tostring_only_first_path_and_plus_wins() {
    let mut m = maze_with(&G1);
    m.start = Some((1, 1));
    m.end = Some((1, 1));
    m.solutions = Some(vec![vec![], vec![(5, 5)]]);
    let s = m.tostring(true, true);
    assert!(s.contains('E') && !s.contains('S') && !s.contains('+'));
    m.solutions = Some(vec![vec![(1, 1), (99, 99)], vec![(5, 5)]]);
    let s = m.tostring(true, true);
    assert_eq!(s.matches('+').count(), 1);
    assert!(!s.contains('E'));
}

#[test]
fn tostring_g2() {
    let mut m = maze_with(&G2);
    m.start = Some((11, 12));
    m.end = Some((7, 0));
    m.solutions = Some(vec![vec![
        (11, 11),
        (10, 11),
        (9, 11),
        (8, 11),
        (7, 11),
        (6, 11),
        (5, 11),
        (4, 11),
        (3, 11),
        (2, 11),
        (1, 11),
        (1, 10),
        (1, 9),
        (1, 8),
        (1, 7),
        (1, 6),
        (1, 5),
        (2, 5),
        (3, 5),
        (3, 4),
        (3, 3),
        (4, 3),
        (5, 3),
        (5, 2),
        (5, 1),
        (6, 1),
        (7, 1),
    ]]);
    let want = [
        "#############",
        "#    +++++++#",
        "# ###+#####+#",
        "# #+++  #  +#",
        "###+##### #+#",
        "#+++  #   #+#",
        "#+# ### ###+#",
        "E+#   #   #+#",
        "# #########+#",
        "#         #+#",
        "# ##### ###+#",
        "# #       #+S",
        "#############",
    ]
    .join("\n");
    assert_eq!(m.tostring(true, true), want);
}

// ---- golden file test (skipped when the goldens checkout is absent) ----

/// A minimal JSON value, enough for the tostring golden lines.
#[derive(Debug, Clone)]
enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    fn get(&self, k: &str) -> &J {
        match self {
            J::Obj(kv) => kv
                .iter()
                .find(|(key, _)| key == k)
                .map_or(&J::Null, |p| &p.1),
            _ => &J::Null,
        }
    }
    fn arr(&self) -> &[J] {
        match self {
            J::Arr(a) => a,
            _ => &[],
        }
    }
    fn str(&self) -> &str {
        match self {
            J::Str(s) => s,
            _ => "",
        }
    }
    fn cell(&self) -> Option<Cell> {
        match self {
            J::Arr(a) if a.len() == 2 => match (&a[0], &a[1]) {
                (J::Num(r), J::Num(c)) => Some((*r as usize, *c as usize)),
                _ => None,
            },
            _ => None,
        }
    }
    fn flag(&self) -> bool {
        matches!(self, J::Bool(true))
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn value(&mut self) -> J {
        self.ws();
        match self.s[self.i] {
            b'n' => {
                self.i += 4;
                J::Null
            }
            b't' => {
                self.i += 4;
                J::Bool(true)
            }
            b'f' => {
                self.i += 5;
                J::Bool(false)
            }
            b'"' => J::Str(self.string()),
            b'[' => {
                self.i += 1;
                let mut v = Vec::new();
                loop {
                    self.ws();
                    if self.s[self.i] == b']' {
                        self.i += 1;
                        break;
                    }
                    v.push(self.value());
                    self.ws();
                    if self.s[self.i] == b',' {
                        self.i += 1;
                    }
                }
                J::Arr(v)
            }
            b'{' => {
                self.i += 1;
                let mut v = Vec::new();
                loop {
                    self.ws();
                    if self.s[self.i] == b'}' {
                        self.i += 1;
                        break;
                    }
                    let k = self.string();
                    self.ws();
                    self.i += 1; // ':'
                    v.push((k, self.value()));
                    self.ws();
                    if self.s[self.i] == b',' {
                        self.i += 1;
                    }
                }
                J::Obj(v)
            }
            _ => {
                let st = self.i;
                while self.i < self.s.len() && b"+-.eE0123456789".contains(&self.s[self.i]) {
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.s[st..self.i]).unwrap();
                J::Num(t.parse().unwrap())
            }
        }
    }
    fn string(&mut self) -> String {
        self.i += 1; // opening quote
        let mut out = String::new();
        loop {
            let b = self.s[self.i];
            self.i += 1;
            match b {
                b'"' => break,
                b'\\' => {
                    let e = self.s[self.i];
                    self.i += 1;
                    match e {
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let h = std::str::from_utf8(&self.s[self.i..self.i + 4]).unwrap();
                            self.i += 4;
                            let cp = u32::from_str_radix(h, 16).unwrap();
                            out.push(char::from_u32(cp).unwrap_or('?'));
                        }
                        other => out.push(other as char),
                    }
                }
                _ => {
                    // Multi-byte UTF-8: copy the full sequence.
                    let st = self.i - 1;
                    let mut en = self.i;
                    while en < self.s.len() && (self.s[en] & 0xC0) == 0x80 {
                        en += 1;
                    }
                    out.push_str(std::str::from_utf8(&self.s[st..en]).unwrap());
                    self.i = en;
                }
            }
        }
        out
    }
}

fn parse(line: &str) -> J {
    Parser {
        s: line.as_bytes(),
        i: 0,
    }
    .value()
}

fn goldens_dir() -> Option<std::path::PathBuf> {
    let dir = std::env::var_os("MAZELIB_GOLDENS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .map(|p| p.join("slopfork-mazelib-goldens"))
                .unwrap_or_default()
        });
    dir.join("cases").is_dir().then_some(dir)
}

fn read_lines(dir: &std::path::Path, sub: &str, set: &str) -> Vec<J> {
    let p = dir.join(sub).join(format!("{set}.jsonl"));
    std::fs::read_to_string(p)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(parse)
        .collect()
}

#[test]
fn tostring_goldens() {
    let Some(dir) = goldens_dir() else {
        eprintln!("goldens not found; skipping");
        return;
    };
    let cases = read_lines(&dir, "cases", "tostring");
    let expected = read_lines(&dir, "expected", "tostring");
    let mut checked = 0;
    for case in &cases {
        let id = case.get("id").str();
        let want = expected
            .iter()
            .find(|e| e.get("id").str() == id)
            .expect("expected entry");
        let a = case.get("args");
        let rows: Vec<&str> = a.get("grid").arr().iter().map(J::str).collect();
        let mut m = maze_with(&rows);
        m.start = a.get("start").cell();
        m.end = a.get("end").cell();
        m.solutions = match a.get("solutions") {
            J::Arr(paths) => Some(
                paths
                    .iter()
                    .map(|p| p.arr().iter().filter_map(J::cell).collect())
                    .collect(),
            ),
            _ => None,
        };
        let got = m.tostring(a.get("entrances").flag(), a.get("show_solutions").flag());
        let text = want.get("result").get("value").get("text").str();
        assert_eq!(got, text, "case {id}");
        checked += 1;
    }
    assert_eq!(checked, cases.len());
    assert!(checked >= 20);
}

// ---- entrances ----

fn legal(grid: &Grid, c: Cell, outer: bool) -> bool {
    let (h, w) = grid.shape();
    let (r, col) = c;
    if r >= h || col >= w {
        return false;
    }
    let border = r == 0 || r == h - 1 || col == 0 || col == w - 1;
    if outer {
        border
            && (((r == 0 || r == h - 1) && col % 2 == 1)
                || ((col == 0 || col == w - 1) && r % 2 == 1))
    } else {
        r % 2 == 1 && col % 2 == 1 && !border
    }
}

fn opposite_sides(grid: &Grid, s: Cell, e: Cell) -> bool {
    let (h, w) = grid.shape();
    (s.0 == 0 && e.0 == h - 1)
        || (s.0 == h - 1 && e.0 == 0)
        || (s.1 == 0 && e.1 == w - 1)
        || (s.1 == w - 1 && e.1 == 0)
}

fn check_entrances(m: &Maze, so: bool, eo: bool) {
    let g = m.grid.as_ref().unwrap();
    let (s, e) = (m.start.unwrap(), m.end.unwrap());
    assert!(legal(g, s, so), "start {s:?}");
    assert!(legal(g, e, eo), "end {e:?}");
    assert!(manhattan(s, e) >= 2);
    if so && eo {
        assert!(opposite_sides(g, s, e), "{s:?} {e:?}");
    }
}

#[test]
fn entrances_all_modes_many_seeds() {
    let small = ["#####", "#...#", "#####"]; // 1x2 cells
    for rows in [&G1[..], &G2[..], &small[..]] {
        for seed in 0..60 {
            for (so, eo) in [(true, true), (true, false), (false, true), (false, false)] {
                let mut m = maze_with(rows);
                m.set_seed(seed).unwrap();
                let before = m.grid.clone();
                m.solutions = Some(vec![vec![(1, 1)]]);
                m.generate_entrances(so, eo).unwrap();
                check_entrances(&m, so, eo);
                assert_eq!(m.grid, before);
                assert!(m.solutions.is_some());
            }
        }
    }
}

#[test]
fn entrances_goldens() {
    let Some(dir) = goldens_dir() else {
        return;
    };
    for case in read_lines(&dir, "cases", "entrances") {
        let a = case.get("args");
        let rows: Vec<&str> = a.get("grid").arr().iter().map(J::str).collect();
        let (so, eo) = (a.get("start_outer").flag(), a.get("end_outer").flag());
        let mut m = maze_with(&rows);
        if let J::Num(s) = case.get("seed") {
            m.set_seed(*s as i128).unwrap();
        }
        m.generate_entrances(so, eo).unwrap();
        check_entrances(&m, so, eo);
    }
}

#[test]
fn entrances_errors_on_degenerate_grids() {
    let mut m = Maze::new();
    assert!(m.generate_entrances(true, true).is_err());
    let one = ["###", "#.#", "###"];
    for (so, eo) in [(true, false), (false, true), (false, false)] {
        let mut m = maze_with(&one);
        assert!(m.generate_entrances(so, eo).is_err());
    }
    let mut m = maze_with(&one);
    m.generate_entrances(true, true).unwrap();
    check_entrances(&m, true, true);
    let mut m = maze_with(&["##", "##"]);
    assert!(m.generate_entrances(true, true).is_err());
}

// ---- solve / generate / transmute / monte carlo ----

/// BFS shortest path; returns the cells strictly between start and end.
struct Bfs;

impl Solver for Bfs {
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, _rng: &mut Rng) -> Result<Vec<Path>> {
        let (h, w) = grid.shape();
        let mut prev = vec![None; h * w];
        let mut seen = vec![false; h * w];
        let mut q = VecDeque::from([start]);
        seen[start.0 * w + start.1] = true;
        while let Some(c) = q.pop_front() {
            if c == end {
                break;
            }
            for n in grid.neighbors(c, 1) {
                let i = n.0 * w + n.1;
                if !seen[i] && (grid.is_open(n.0, n.1) || n == end) {
                    seen[i] = true;
                    prev[i] = Some(c);
                    q.push_back(n);
                }
            }
        }
        let mut path = Vec::new();
        let mut cur = prev[end.0 * w + end.1];
        while let Some(c) = cur {
            if c == start {
                break;
            }
            path.push(c);
            cur = prev[c.0 * w + c.1];
        }
        path.reverse();
        Ok(vec![path])
    }
}

/// A solver returning fixed raw paths (to observe pruning).
struct Fixed(Vec<Path>);

impl Solver for Fixed {
    fn solve(&mut self, _: &Grid, _: Cell, _: Cell, _: &mut Rng) -> Result<Vec<Path>> {
        Ok(self.0.clone())
    }
}

/// Opens every wall slot touching `start`, to show transmuters get entrances.
struct OpenAroundStart;

impl Transmuter for OpenAroundStart {
    fn transmute(
        &mut self,
        grid: &mut Grid,
        start: Option<Cell>,
        _end: Option<Cell>,
        _rng: &mut Rng,
    ) -> Result<()> {
        if let Some(s) = start {
            grid.set(s.0, s.1, 0);
        }
        Ok(())
    }
}

#[test]
fn errors_are_invalid_input() {
    let mut m = Maze::new();
    assert!(matches!(m.generate(), Err(MazeError::InvalidInput(_))));
    assert!(matches!(m.transmute(), Err(MazeError::InvalidInput(_))));
    assert!(matches!(m.solve(), Err(MazeError::InvalidInput(_))));
    m.solver = Some(Box::new(Bfs));
    assert!(matches!(m.solve(), Err(MazeError::InvalidInput(_))));
    m.start = Some((0, 1));
    m.end = Some((6, 1));
    assert!(matches!(m.solve(), Err(MazeError::InvalidInput(_))));
    m.grid = Some(Grid::from_rows(&G1));
    m.end = Some((7, 1));
    assert!(matches!(m.solve(), Err(MazeError::InvalidInput(_))));
    for d in [1.5, -0.1, f64::NAN] {
        assert!(matches!(
            m.generate_monte_carlo(1, 1, d),
            Err(MazeError::InvalidInput(_))
        ));
    }
    m.generator = Some(Box::new(Prims::new(3, 3).unwrap()));
    assert!(m.generate_monte_carlo(0, 1, 0.5).is_err());
    assert!(m.generate_monte_carlo(1, 0, 0.5).is_err());
}

#[test]
fn solve_g1_with_bfs() {
    let mut m = maze_with(&G1);
    m.solver = Some(Box::new(Bfs));
    m.start = Some((3, 0));
    m.end = Some((1, 6));
    m.solve().unwrap();
    assert_eq!(m.solutions, Some(vec![g1_path()]));
    assert_eq!(m.grid, Some(Grid::from_rows(&G1)));
}

#[test]
fn solve_prune_flag() {
    let raw = vec![
        vec![(3, 0), (3, 1), (3, 2), (3, 1), (2, 1), (1, 1), (1, 6)],
        vec![],
        vec![(3, 0)],
    ];
    let mut m = maze_with(&G1);
    m.solver = Some(Box::new(Fixed(raw.clone())));
    m.start = Some((3, 0));
    m.end = Some((1, 6));
    m.prune = false;
    m.solve().unwrap();
    assert_eq!(m.solutions.as_ref(), Some(&raw));
    m.prune = true;
    m.solve().unwrap();
    assert_eq!(
        m.solutions,
        Some(vec![vec![(3, 1), (2, 1), (1, 1)], vec![], vec![(3, 0)]])
    );
}

#[test]
fn generate_and_transmute() {
    let mut m = Maze::with_seed(3).unwrap();
    m.generator = Some(Box::new(Prims::new(4, 5).unwrap()));
    m.start = Some((0, 1));
    m.solutions = Some(vec![]);
    m.generate().unwrap();
    assert_eq!(m.grid.as_ref().unwrap().shape(), (9, 11));
    assert!(m.start.is_none() && m.end.is_none() && m.solutions.is_none());
    m.transmute().unwrap();
    m.transmuters.push(Box::new(OpenAroundStart));
    m.generate_entrances(true, true).unwrap();
    let s = m.start.unwrap();
    assert!(m.grid.as_ref().unwrap().is_wall(s.0, s.1));
    m.transmute().unwrap();
    assert!(m.grid.as_ref().unwrap().is_open(s.0, s.1));
}

fn check_mc(m: &Maze, shape: (usize, usize)) -> usize {
    let g = m.grid.as_ref().unwrap();
    assert_eq!(g.shape(), shape);
    let (s, e) = (m.start.unwrap(), m.end.unwrap());
    check_entrances(m, true, true);
    let sols = m.solutions.as_ref().unwrap();
    let p = &sols[0];
    assert!(!p.is_empty());
    let mut chain = vec![s];
    chain.extend(p.iter().copied());
    chain.push(e);
    for win in chain.windows(2) {
        assert_eq!(manhattan(win[0], win[1]), 1);
    }
    assert!(p.iter().all(|&(r, c)| g.is_open(r, c)));
    p.len()
}

#[test]
fn monte_carlo_with_prims_and_bfs() {
    for seed in 0..5 {
        let mut lens = Vec::new();
        for d in [0.0, 0.5, 1.0] {
            let mut m = Maze::with_seed(seed).unwrap();
            m.generator = Some(Box::new(Prims::new(4, 4).unwrap()));
            m.solver = Some(Box::new(Bfs));
            m.generate_monte_carlo(3, 3, d).unwrap();
            lens.push(check_mc(&m, (9, 9)));
        }
        assert!(lens.windows(2).all(|w| w[0] <= w[1]), "{lens:?}");
    }
    let mut m = Maze::with_seed(1).unwrap();
    m.generator = Some(Box::new(Prims::new(3, 3).unwrap()));
    m.solver = Some(Box::new(Bfs));
    m.generate_monte_carlo(1, 1, 1.0).unwrap();
    check_mc(&m, (7, 7));
}

#[test]
fn monte_carlo_custom_reducer() {
    let mut m = Maze::with_seed(5).unwrap();
    m.generator = Some(Box::new(Prims::new(5, 5).unwrap()));
    m.solver = Some(Box::new(Bfs));
    m.generate_monte_carlo_with(4, 2, 0.0, |p| -(p.len() as f64))
        .unwrap();
    check_mc(&m, (11, 11));
}

#[test]
fn monte_carlo_with_real_shortest_path() {
    for seed in 0..3 {
        for d in [0.0, 0.5, 1.0] {
            let mut m = Maze::with_seed(seed).unwrap();
            m.generator = Some(Box::new(Prims::new(4, 4).unwrap()));
            m.solver = Some(Box::new(crate::solve::ShortestPath::new()));
            m.generate_monte_carlo(3, 3, d).unwrap();
            check_mc(&m, (9, 9));
        }
    }
}

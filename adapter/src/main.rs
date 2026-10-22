//! JSON-lines adapter for the slopfork mazelib goldens protocol.
//!
//! Reads one case per line on stdin, writes one result per line on stdout.
//! Library errors are mapped onto Python exception kind names here, and only here.
#![forbid(unsafe_code)]

use std::io::{self, BufRead, BufWriter, Write};

use mazelib::generate::{
    AldousBroder, BacktrackingGenerator, BinaryTree, CellularAutomaton, Division, DungeonRooms,
    Ellers, GrowingTree, HuntAndKill, HuntOrder, Kruskal, MazeType, Prims, Room, Sidewinder, Skew,
    TrivialMaze, Wilsons,
};
use mazelib::solve::{
    BacktrackingSolver, Chain, Collision, RandomMouse, ShortestPath, ShortestPaths, Tremaux, Turn,
};
use mazelib::transmute::{CuldeSacFiller, DeadEndFiller, Perturbation};
use mazelib::{Cell, Generator, Grid, Maze, MazeError, Path, Rng, Solver, Transmuter};
use serde_json::{json, Value};

/// A failed case: Python exception kind plus message.
struct Failure {
    kind: &'static str,
    message: String,
}

impl From<MazeError> for Failure {
    fn from(e: MazeError) -> Self {
        let kind = match e {
            MazeError::InvalidInput(_) => "AssertionError",
            MazeError::InvalidValue(_) => "ValueError",
            MazeError::Empty(_) => "IndexError",
        };
        Failure {
            kind,
            message: e.message().to_string(),
        }
    }
}

/// A malformed case (not something the original would see).
fn bad(msg: impl Into<String>) -> Failure {
    Failure {
        kind: "TypeError",
        message: msg.into(),
    }
}

type Out<T> = std::result::Result<T, Failure>;

fn get<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.get(key).filter(|x| !x.is_null())
}

fn int(v: &Value, key: &str) -> Out<i64> {
    get(v, key)
        .and_then(Value::as_i64)
        .ok_or_else(|| bad(format!("missing integer `{key}`")))
}

fn float_or(v: &Value, key: &str, default: f64) -> f64 {
    get(v, key).and_then(Value::as_f64).unwrap_or(default)
}

fn str_or<'a>(v: &'a Value, key: &str, default: &'a str) -> &'a str {
    get(v, key).and_then(Value::as_str).unwrap_or(default)
}

fn bool_or(v: &Value, key: &str, default: bool) -> bool {
    get(v, key).and_then(Value::as_bool).unwrap_or(default)
}

fn grid_in(v: &Value) -> Out<Grid> {
    let rows = v.as_array().ok_or_else(|| bad("grid must be a list"))?;
    let rows: Vec<&str> = rows
        .iter()
        .map(|r| r.as_str().ok_or_else(|| bad("grid rows must be strings")))
        .collect::<Out<_>>()?;
    Ok(Grid::from_rows(&rows))
}

fn grid_out(g: &Grid) -> Value {
    json!(g.to_rows())
}

fn cell_in(v: Option<&Value>) -> Out<Option<Cell>> {
    let Some(v) = v.filter(|x| !x.is_null()) else {
        return Ok(None);
    };
    let pair = v.as_array().filter(|a| a.len() == 2);
    let coords = pair.and_then(|a| Some((a[0].as_i64()?, a[1].as_i64()?)));
    match coords {
        Some((r, c)) if r >= 0 && c >= 0 => Ok(Some((r as usize, c as usize))),
        Some(_) => Err(Failure {
            kind: "AssertionError",
            message: "cell outside the grid".into(),
        }),
        None => Err(bad("cell must be [row, col]")),
    }
}

fn cell_out(c: Option<Cell>) -> Value {
    c.map_or(Value::Null, |(r, c)| json!([r, c]))
}

fn solutions_out(s: Option<&Vec<Path>>) -> Value {
    s.map_or(Value::Null, |paths| {
        Value::Array(
            paths
                .iter()
                .map(|p| Value::Array(p.iter().map(|&c| cell_out(Some(c))).collect()))
                .collect(),
        )
    })
}

fn rooms_in(v: &Value) -> Out<Vec<Room>> {
    let corner = |c: &Value| -> Out<(i64, i64)> {
        let a = c.as_array().filter(|a| a.len() == 2);
        a.and_then(|a| Some((a[0].as_i64()?, a[1].as_i64()?)))
            .ok_or_else(|| bad("room corner must be [row, col]"))
    };
    v.as_array()
        .ok_or_else(|| bad("rooms must be a list"))?
        .iter()
        .map(|room| {
            let r = room
                .as_array()
                .filter(|a| a.len() == 2)
                .ok_or_else(|| bad("room must be two corners"))?;
            Ok((corner(&r[0])?, corner(&r[1])?))
        })
        .collect()
}

fn make_generator(spec: &Value) -> Out<Box<dyn Generator>> {
    let algo = str_or(spec, "algo", "");
    let h = int(spec, "h")?;
    let w = int(spec, "w")?;
    let empty = json!({});
    let p = get(spec, "params").unwrap_or(&empty);
    let hunt = || HuntOrder::parse(str_or(p, "hunt_order", "random"));
    let g: Box<dyn Generator> = match algo {
        "AldousBroder" => Box::new(AldousBroder::new(h, w)?),
        "BacktrackingGenerator" => Box::new(BacktrackingGenerator::new(h, w)?),
        "BinaryTree" => {
            let skew = get(p, "skew").and_then(Value::as_str).and_then(Skew::parse);
            Box::new(BinaryTree::new(h, w, skew)?)
        }
        "CellularAutomaton" => Box::new(CellularAutomaton::new(
            h,
            w,
            float_or(p, "complexity", 1.0),
            float_or(p, "density", 1.0),
        )?),
        "Division" => Box::new(Division::new(h, w)?),
        "DungeonRooms" => {
            let rooms = get(p, "rooms").map(rooms_in).transpose()?;
            let grid = get(p, "grid").map(grid_in).transpose()?;
            Box::new(DungeonRooms::new(h, w, rooms, grid, hunt())?)
        }
        "Ellers" => Box::new(Ellers::new(
            h,
            w,
            float_or(p, "xskew", 0.5),
            float_or(p, "yskew", 0.5),
        )?),
        "GrowingTree" => Box::new(GrowingTree::new(
            h,
            w,
            float_or(p, "backtrack_chance", 1.0),
        )?),
        "HuntAndKill" => Box::new(HuntAndKill::new(h, w, hunt())?),
        "Kruskal" => Box::new(Kruskal::new(h, w)?),
        "Prims" => Box::new(Prims::new(h, w)?),
        "Sidewinder" => Box::new(Sidewinder::new(h, w, float_or(p, "skew", 0.5))?),
        "TrivialMaze" => Box::new(TrivialMaze::new(
            h,
            w,
            MazeType::parse(str_or(p, "maze_type", "spiral")),
        )?),
        "Wilsons" => Box::new(Wilsons::new(h, w, hunt())?),
        other => return Err(bad(format!("unknown generator `{other}`"))),
    };
    Ok(g)
}

fn op_generate(a: &Value, rng: &mut Rng) -> Out<Value> {
    let grid = make_generator(a)?.generate(rng)?;
    Ok(json!({ "grid": grid_out(&grid) }))
}

fn op_rng(a: &Value, rng: &mut Rng) -> Out<Value> {
    let calls = get(a, "calls")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing `calls`"))?;
    let mut out = Vec::with_capacity(calls.len());
    for call in calls {
        let call = call.as_array().ok_or_else(|| bad("call must be a list"))?;
        let name = call.first().and_then(Value::as_str).unwrap_or("");
        let args: Vec<i128> = call[1..]
            .iter()
            .map(|x| x.as_i64().map(i128::from).ok_or_else(|| bad("int args")))
            .collect::<Out<_>>()?;
        let arg = |i: usize| args.get(i).copied().ok_or_else(|| bad("missing arg"));
        let v = match name {
            "random" => json!(rng.py.random()),
            "randrange" => {
                let r = match args.len() {
                    1 => rng.py.randrange(args[0], None, 1)?,
                    2 => rng.py.randrange(args[0], Some(args[1]), 1)?,
                    _ => rng.py.randrange(arg(0)?, Some(arg(1)?), arg(2)?)?,
                };
                json!(r as i64)
            }
            "randint" => json!(rng.py.randint(arg(0)?, arg(1)?)? as i64),
            "choice" => json!(rng.py.choice_index(arg(0)?.max(0) as usize)?),
            "shuffle" => {
                let mut x: Vec<usize> = (0..arg(0)?.max(0) as usize).collect();
                rng.py.shuffle(&mut x);
                json!(x)
            }
            "np_shuffle" => {
                let mut x: Vec<usize> = (0..arg(0)?.max(0) as usize).collect();
                rng.np.shuffle(&mut x);
                json!(x)
            }
            other => return Err(bad(format!("unknown rng call `{other}`"))),
        };
        out.push(v);
    }
    Ok(json!({ "out": out }))
}

fn make_solver(algo: &str, p: &Value) -> Out<Box<dyn Solver>> {
    let s: Box<dyn Solver> = match algo {
        "BacktrackingSolver" => Box::new(BacktrackingSolver::new()),
        "Chain" => Box::new(Chain::new(Turn::parse(str_or(p, "turn", "right")))),
        "Collision" => Box::new(Collision::new()),
        "RandomMouse" => Box::new(RandomMouse::new()),
        "ShortestPath" => Box::new(ShortestPath::new()),
        "ShortestPaths" => Box::new(ShortestPaths::new()),
        "Tremaux" => Box::new(Tremaux::new()),
        other => return Err(bad(format!("unknown solver `{other}`"))),
    };
    Ok(s)
}

fn make_transmuter(algo: &str, p: &Value) -> Out<Box<dyn Transmuter>> {
    let int_or = |key: &str, default: i64| get(p, key).and_then(Value::as_i64).unwrap_or(default);
    let t: Box<dyn Transmuter> = match algo {
        "CuldeSacFiller" => Box::new(CuldeSacFiller::new()),
        "DeadEndFiller" => Box::new(match get(p, "iterations") {
            Some(v) if v.is_i64() => DeadEndFiller::new(int_or("iterations", 1)),
            Some(v) => DeadEndFiller::from_f64(v.as_f64().unwrap_or(1.0)),
            None => DeadEndFiller::new(1),
        }),
        "Perturbation" => Box::new(Perturbation::new(
            int_or("repeat", 1),
            int_or("new_walls", 1),
        )),
        other => return Err(bad(format!("unknown transmuter `{other}`"))),
    };
    Ok(t)
}

fn params(a: &Value) -> Value {
    get(a, "params").cloned().unwrap_or_else(|| json!({}))
}

/// Run `f` on a fresh [`Maze`] that borrows the adapter's RNG streams.
fn with_maze<T>(rng: &mut Rng, f: impl FnOnce(&mut Maze) -> Out<T>) -> Out<T> {
    let mut m = Maze::new();
    m.rng = std::mem::take(rng);
    let r = f(&mut m);
    *rng = std::mem::take(&mut m.rng);
    r
}

fn op_entrances(a: &Value, rng: &mut Rng) -> Out<Value> {
    let grid = grid_in(get(a, "grid").ok_or_else(|| bad("missing grid"))?)?;
    with_maze(rng, |m| {
        m.grid = Some(grid);
        m.generate_entrances(
            bool_or(a, "start_outer", true),
            bool_or(a, "end_outer", true),
        )?;
        Ok(json!({ "start": cell_out(m.start), "end": cell_out(m.end) }))
    })
}

fn op_solve(a: &Value, rng: &mut Rng) -> Out<Value> {
    let grid = grid_in(get(a, "grid").ok_or_else(|| bad("missing grid"))?)?;
    let solver = make_solver(str_or(a, "algo", ""), &params(a))?;
    let (start, end) = (cell_in(a.get("start"))?, cell_in(a.get("end"))?);
    with_maze(rng, |m| {
        m.grid = Some(grid);
        m.start = start;
        m.end = end;
        m.solver = Some(solver);
        m.prune = bool_or(a, "prune", true);
        m.solve()?;
        Ok(json!({ "solutions": solutions_out(m.solutions.as_ref()) }))
    })
}

fn op_transmute(a: &Value, rng: &mut Rng) -> Out<Value> {
    let grid = grid_in(get(a, "grid").ok_or_else(|| bad("missing grid"))?)?;
    let t = make_transmuter(str_or(a, "algo", ""), &params(a))?;
    let (start, end) = (cell_in(a.get("start"))?, cell_in(a.get("end"))?);
    with_maze(rng, |m| {
        m.grid = Some(grid);
        m.start = start;
        m.end = end;
        m.transmuters = vec![t];
        m.transmute()?;
        let g = m.grid.as_ref().ok_or_else(|| bad("grid vanished"))?;
        Ok(json!({ "grid": grid_out(g) }))
    })
}

fn op_tostring(a: &Value, rng: &mut Rng) -> Out<Value> {
    let grid = grid_in(get(a, "grid").ok_or_else(|| bad("missing grid"))?)?;
    let (start, end) = (cell_in(a.get("start"))?, cell_in(a.get("end"))?);
    let solutions = match get(a, "solutions") {
        None => None,
        Some(v) => Some(
            v.as_array()
                .ok_or_else(|| bad("solutions must be a list"))?
                .iter()
                .map(|p| {
                    p.as_array()
                        .ok_or_else(|| bad("path must be a list"))?
                        .iter()
                        .map(|c| cell_in(Some(c))?.ok_or_else(|| bad("null cell")))
                        .collect::<Out<Path>>()
                })
                .collect::<Out<Vec<Path>>>()?,
        ),
    };
    with_maze(rng, |m| {
        m.grid = Some(grid);
        m.start = start;
        m.end = end;
        m.solutions = solutions;
        let text = m.tostring(
            bool_or(a, "entrances", false),
            bool_or(a, "show_solutions", false),
        );
        Ok(json!({ "text": text }))
    })
}

fn op_monte_carlo(a: &Value, rng: &mut Rng) -> Out<Value> {
    let empty = json!({});
    let solver_spec = get(a, "solver").unwrap_or(&empty);
    let solver = make_solver(str_or(solver_spec, "algo", ""), &params(solver_spec))?;
    let count = |key: &str| -> Out<usize> {
        let n = int(a, key)?;
        Ok(n.max(0) as usize)
    };
    let (repeat, entrances) = (count("repeat")?, count("entrances")?);
    let difficulty = get(a, "difficulty")
        .and_then(Value::as_f64)
        .ok_or_else(|| bad("missing difficulty"))?;
    // Python validates difficulty before building the generator.
    if !(0.0..=1.0).contains(&difficulty) {
        return Err(
            MazeError::InvalidInput("Maze difficulty must be between 0 and 1.".into()).into(),
        );
    }
    let generator = make_generator(get(a, "generator").unwrap_or(&empty))?;
    with_maze(rng, |m| {
        m.generator = Some(generator);
        m.solver = Some(solver);
        m.generate_monte_carlo(repeat, entrances, difficulty)?;
        let g = m.grid.as_ref().ok_or_else(|| bad("no grid"))?;
        Ok(json!({
            "grid": grid_out(g),
            "start": cell_out(m.start),
            "end": cell_out(m.end),
            "solutions": solutions_out(m.solutions.as_ref()),
        }))
    })
}

fn run(case: &Value, rng: &mut Rng) -> Out<Value> {
    if let Some(seed) = get(case, "seed") {
        let seed = seed
            .as_i64()
            .ok_or_else(|| bad("seed must be an integer"))?;
        rng.set_seed(i128::from(seed))?;
    }
    let empty = json!({});
    let a = get(case, "args").unwrap_or(&empty);
    match str_or(case, "op", "") {
        "generate" => op_generate(a, rng),
        "entrances" => op_entrances(a, rng),
        "solve" => op_solve(a, rng),
        "transmute" => op_transmute(a, rng),
        "tostring" => op_tostring(a, rng),
        "monte_carlo" => op_monte_carlo(a, rng),
        "rng" => op_rng(a, rng),
        other => Err(bad(format!("unknown op `{other}`"))),
    }
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut out = BufWriter::new(io::stdout().lock());
    let mut rng = Rng::new();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let result = match serde_json::from_str::<Value>(&line) {
            Ok(case) => run(&case, &mut rng),
            Err(e) => Err(bad(format!("bad JSON: {e}"))),
        };
        let v = match result {
            Ok(value) => json!({ "ok": true, "value": value }),
            Err(f) => json!({ "ok": false, "error": { "kind": f.kind, "message": f.message } }),
        };
        writeln!(out, "{v}")?;
        out.flush()?;
    }
    Ok(())
}

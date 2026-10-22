//! Generate, solve and print a maze (the README example).

use mazelib::{generate::Prims, solve::ShortestPath, Maze};

fn main() -> Result<(), mazelib::MazeError> {
    let mut m = Maze::with_seed(42)?;
    m.generator = Some(Box::new(Prims::new(10, 10)?));
    m.generate()?;
    m.generate_entrances(true, true)?;
    m.solver = Some(Box::new(ShortestPath::new()));
    m.solve()?;
    println!("{}", m.tostring(true, true));
    Ok(())
}

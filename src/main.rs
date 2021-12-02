mod graph;
mod io;
use graph::Graph;
use io::read;

fn main() {
    let graph = read();

    // let graph = Graph::new(4);

    // graph.has_cycle(&vec![0, 2, 3]);

    println!("sup!");
}

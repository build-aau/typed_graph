use super::TestGraph;
use fake::*;
use rand::Rng;

/// Create a complete graph comprised of nodes and edges of random types
pub struct CompleteGraph {
    pub nodes: usize,
    pub node_types: usize,
    pub edge_types: usize,
}

impl Dummy<CompleteGraph> for TestGraph {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &CompleteGraph, rng: &mut R) -> Self {
        let mut g = TestGraph::default();
        for i in 0..config.nodes {
            g.add_node((i, rng.gen_range(0..config.node_types)))
                .unwrap();
        }

        for x in 0..config.nodes {
            for y in 0..config.nodes {
                if x == y {
                    continue;
                }

                g.add_edge(
                    x,
                    y,
                    (x + y * config.nodes, rng.gen_range(0..config.edge_types)),
                )
                .unwrap();
            }
        }

        g
    }
}

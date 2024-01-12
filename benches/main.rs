use std::ops::Range;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, BatchSize, Throughput};
use rand::seq::IteratorRandom;
use typed_graph::generic_graph::{GenericGraph, GenericResult};
use fake::*;

type TestGraph = GenericGraph<usize, usize, usize, usize>;
type TestResult<T> = GenericResult<T, usize, usize, usize, usize>;

/// Create a complete graph comprised of nodes and edges of random types
pub struct CompleteGraph {
    /// Width of the graph
    pub width: usize,
    /// Height of the graph
    pub height: usize,
    /// Number of node types (types start at 0)
    pub node_types: usize,
    /// Number of edge types (types start at 0)
    pub edge_types: usize
}

impl Dummy<CompleteGraph> for TestGraph {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &CompleteGraph, rng: &mut R) -> Self {
        let mut g = TestGraph::default();
        let node_count = config.width * config.height;
        for i in 0..node_count {
            g.add_node((i, rng.gen_range(0..config.node_types))).unwrap();
        }
        
        if config.edge_types != 0 {
            for x in 0..config.width {
                for y in 0..config.height {
                    if x == y  {
                        continue;
                    }
    
                    g.add_edge(
                        x, 
                        y, 
                        (x + y * config.width, rng.gen_range(0..config.edge_types))
                    ).unwrap();
                }
            }
        }

        g
    }
}

/// Create a complete graph comprised of nodes and edges of random types
pub struct SparseGraph {
    /// Width of the graph
    pub width: usize,
    /// Height of the graph
    pub height: usize,
    /// To how many other nodes should each node be connected to
    pub node_connections: Range<usize>,
    /// Number of node types (types start at 0)
    pub node_types: usize,
    /// Number of edge types (types start at 0)
    pub edge_types: usize
}

impl Dummy<SparseGraph> for TestGraph {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &SparseGraph, rng: &mut R) -> Self {
        let mut g = TestGraph::default();
        let node_count = config.width * config.height;
        for i in 0..node_count {
            g.add_node((i, rng.gen_range(0..config.node_types))).unwrap();
        }

        for i in 0..node_count {
            let edge_count = config.node_connections.clone().choose(rng).unwrap_or_default();

            for _ in 0..edge_count {
                // Figure out which node to connect to
                let target = (0..node_count).choose(rng).unwrap_or_default();

                // Do not include self loops
                // Otherwise a 1x2 graph would be very strongly connected
                if target == i {
                    continue;
                }

                g.add_edge(i, target, (g.edge_count(), rng.gen_range(0..config.edge_types))).unwrap();
            }
        }

        g
    }
}

fn add_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("Node+");
    
    for side_length in (0..=100).step_by(25) {
        group.throughput(Throughput::Elements((side_length*side_length) as u64));
        group.bench_with_input::<_, _, usize>(
            BenchmarkId::from_parameter(side_length*side_length),
            &side_length, 
            |b, side_length| {
        
            b.iter_batched_ref(
                || {
                    TestGraph::default()
                }, 
                |g| {
                    for i in 0..(side_length*side_length) {
                        g.add_node((i, i))?;
                    }

                    TestResult::Ok(())
                }, 
                BatchSize::SmallInput
            )
        });
    }
    group.finish();
}

fn remove_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("Node-");
    
    for side_length in (0..=100).step_by(25) {
        group.throughput(Throughput::Elements((side_length*side_length) as u64));
        group.bench_with_input::<_, _, usize>(
            BenchmarkId::from_parameter(side_length*side_length),
            &side_length, 
            |b, side_length| {
        
            b.iter_batched_ref(
                || {
                    CompleteGraph {
                        width: *side_length,
                        height: *side_length,
                        node_types: side_length*side_length,
                        edge_types: side_length*side_length
                    }.fake::<TestGraph>()
                }, 
                |g| {
                    let node_count = g.node_count();
                    for i in 0..node_count {
                        g.remove_node(i)?;
                    }

                    TestResult::Ok(())
                }, 
                BatchSize::SmallInput
            )
        });
    }
    group.finish();
}

fn add_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("Edge+");
    
    for side_length in (0..=100).step_by(25) {
        group.throughput(Throughput::Elements((side_length*side_length) as u64));
        group.bench_with_input::<_, _, usize>(
            BenchmarkId::from_parameter(side_length*side_length),
            &side_length, 
            |b, side_length| {
        
            b.iter_batched_ref(
                || {
                    CompleteGraph {
                        width: *side_length,
                        height: *side_length,
                        node_types: side_length*side_length,
                        edge_types: 0
                    }.fake::<TestGraph>()
                }, 
                |g| {
                    for x in 0..*side_length {
                        for y in 0..*side_length {
                            g.add_edge(
                                x, 
                                y, 
                                (x + y * 100, 0)
                            )?;
                        }
                    }

                    TestResult::Ok(())
                }, 
                BatchSize::SmallInput
            )
        });
    }
    group.finish();
}

fn remove_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("Edge-");
    
    for side_length in (0..=100).step_by(25) {
        group.throughput(Throughput::Elements((side_length*side_length) as u64));
        group.bench_with_input::<_, _, usize>(
            BenchmarkId::from_parameter(side_length*side_length),
            &side_length, 
            |b, side_length| {
                b.iter_batched_ref(
                    || {
                        CompleteGraph {
                            width: *side_length,
                            height: *side_length,
                            node_types: side_length*side_length,
                            edge_types: side_length*side_length
                        }.fake::<TestGraph>()
                    }, 
                    |g| {
                        let edge_count = g.edge_count();
                        for i in 0..edge_count {
                            g.remove_edge(i)?;
                        }
        
                        TestResult::Ok(())
                    }, 
                    BatchSize::SmallInput
                )
            }
        );

    }
    group.finish();
}

criterion_group!(
    benches, 
    add_node,
    remove_node,
    add_edge,
    remove_edge
);
criterion_main!(benches);
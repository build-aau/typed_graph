use std::collections::HashSet;

use fake::{Dummy, Fake, Faker};
use pyo3::types::PyModule;
use pyo3::*;
use rand::seq::{IteratorRandom, SliceRandom};
use serde::{Deserialize, Serialize};

use crate::Id;

use super::{TestGraph, TestResult, TestSchema};

const TYPED_GRAPH_PY: &'static str = include_str!("rust_test.py");

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Action {
    AddNode {
        id: usize,
        ty: usize,
    },
    AddEdge {
        id: usize,
        ty: usize,
        source: usize,
        target: usize,
    },
    RemoveNode {
        id: usize,
    },
    RemoveEdge {
        id: usize,
    },
}

impl Action {
    pub fn apply(&self, g: &mut TestGraph) -> TestResult<()> {
        match self {
            Action::AddNode { id, ty } => {
                g.add_node((*id, *ty))?;
            }
            Action::AddEdge {
                id,
                ty,
                source,
                target,
            } => {
                g.add_edge(*source, *target, (*id, *ty))?;
            }
            Action::RemoveNode { id } => {
                g.remove_node(*id)?;
            }
            Action::RemoveEdge { id } => {
                g.remove_edge(*id)?;
            }
        }

        Ok(())
    }
}

struct TestProject {
    g: TestGraph,
    actions: Vec<Action>,
}

impl Dummy<Faker> for TestProject {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        // Create 5-10 node types
        let node_whitelist: Vec<_> = (0..rng.gen_range(5..10)).collect();
        let node_whitelist_len = node_whitelist.len();
        let max_edge_len = node_whitelist_len * node_whitelist_len;

        // Create 5-10 edge types going between none, some or all the node types
        let mut edge_whitelist: HashSet<(usize, usize, usize)> = HashSet::new();
        for edge_type in 0..rng.gen_range(5..10) {
            for _ in 0..rng.gen_range(0..max_edge_len) {
                edge_whitelist.insert((
                    *node_whitelist.choose(rng).unwrap(),
                    *node_whitelist.choose(rng).unwrap(),
                    edge_type,
                ));
            }
        }

        // Create the schema
        let schema = TestSchema::new()
            .node_whitelist(Some(node_whitelist.clone()))
            .endpoint_whitelist(Some(edge_whitelist.iter().cloned().collect()));

        let mut g = TestGraph::new(schema);
        let mut actions = Vec::new();

        let mut next_node_id = 0;
        let mut next_edge_id = 0;

        // Begin creating actions
        let action_count = rng.gen_range(10..40);
        for _ in 0..action_count {
            // First figure out which actions are possible
            let mut possible_actions: Vec<Action> = Vec::new();

            let add_node = Action::AddNode {
                id: next_node_id,
                ty: node_whitelist.choose(rng).unwrap().clone(),
            };
            possible_actions.push(add_node.clone());
            possible_actions.push(add_node);

            if g.node_count() > 0 {
                let all_nodes = g.node_ids();
                let remove_node = Action::RemoveNode {
                    id: all_nodes.choose(rng).unwrap().clone(),
                };
                possible_actions.push(remove_node);

                let current_node_types: HashSet<usize> = g.nodes().map(|n| n.1).collect();

                let possible_edge_types: HashSet<&(usize, usize, usize)> = edge_whitelist
                    .iter()
                    .filter(|(source, target, _)| {
                        current_node_types.contains(source) && current_node_types.contains(target)
                    })
                    .collect();
                if let Some((source, target, edge_type)) = possible_edge_types.iter().choose(rng) {
                    let source_id = g
                        .nodes()
                        .filter(|n| &n.1 == source)
                        .choose(rng)
                        .map(|n| n.0)
                        .unwrap();
                    let target_id = g
                        .nodes()
                        .filter(|n| &n.1 == target)
                        .choose(rng)
                        .map(|n| n.0)
                        .unwrap();

                    let add_edge = Action::AddEdge {
                        id: next_edge_id,
                        ty: *edge_type,
                        source: source_id,
                        target: target_id,
                    };
                    possible_actions.push(add_edge.clone());
                    possible_actions.push(add_edge);
                }
            }

            if g.edge_count() > 0 {
                let remove_edge = Action::RemoveEdge {
                    id: g.edge_ids().choose(rng).unwrap(),
                };
                possible_actions.push(remove_edge)
            }

            // Finally pick one of the actions to do
            if let Some(action) = possible_actions.choose(rng) {
                if matches!(action, Action::AddNode { .. }) {
                    next_node_id += 1;
                }

                if matches!(action, Action::AddEdge { .. }) {
                    next_edge_id += 1;
                }

                action.apply(&mut g).unwrap();
                actions.push(action.clone());
            }
        }

        TestProject { g, actions }
    }
}

fn run_py_test(json_schema: String, json_actions: String) -> String {
    Python::with_gil(|py| -> PyResult<String> {
        let rust_test_mod: &PyModule = PyModule::from_code(py, TYPED_GRAPH_PY, "", "")?;

        let json_py_graph = rust_test_mod
            .call_method("run", (json_schema, json_actions), None)?
            .extract()?;
        Ok(json_py_graph)
    })
    .unwrap()
}

#[test]
fn test_typed_graph_py() {
    for _ in 0..100 {
        let prj: TestProject = Faker.fake();

        let json_schema = serde_json::to_string(prj.g.get_schema()).unwrap();
        let json_actions = serde_json::to_string(&prj.actions).unwrap();

        println!("let json_schema = r#\"{}\"#;", json_schema);
        println!("let json_actions = r#\"{}\"#;", json_actions);
        println!();
        println!();

        let json_py_graph = run_py_test(json_schema, json_actions);

        let py_graph: TestGraph = serde_json::from_str(&json_py_graph).unwrap();
        prj.g.assert_eq(&py_graph).unwrap();
    }
}

/// Takes output from test_typed_graph_py and run it
#[allow(unused)]
// #[test]
fn run_single() {
    let json_schema = r#"{"node_whitelist":[0,1,2,3,4,5,6,7,8],"node_blacklist":null,"edge_whitelist":null,"edge_blacklist":null,"edge_endpoint_whitelist":[[4,6,2],[1,3,8],[6,5,2],[5,5,8],[1,8,6],[1,0,5],[4,3,2],[6,6,0],[2,7,0],[1,2,8],[2,2,8],[2,1,6],[1,5,2],[1,6,4],[1,1,6],[5,5,3],[6,8,7],[4,3,8],[6,3,0],[1,0,8],[1,3,2],[1,7,5],[5,0,5],[5,8,6],[1,8,1],[5,4,8],[1,0,7],[2,5,3],[4,3,1],[1,6,3],[1,1,5],[1,7,6],[2,5,1],[2,3,3],[5,1,5],[6,8,6],[6,6,7],[4,3,0],[6,1,8],[6,8,0],[1,4,5],[3,3,1],[2,4,3],[4,0,7],[6,3,3],[1,2,0],[1,1,7],[2,4,8],[2,3,0],[2,8,8],[6,2,3],[5,0,3],[2,5,5],[5,2,6],[1,4,3],[4,0,5],[4,2,3],[4,3,6],[2,2,7],[2,6,2],[6,0,0],[6,7,4],[6,7,5],[5,5,0],[4,2,5],[6,8,3],[0,8,4],[1,3,6],[1,1,8],[1,5,1],[2,7,7],[4,8,7],[4,0,3],[5,6,0],[5,2,0],[2,6,6],[4,6,5],[6,0,4],[6,6,4],[5,3,6],[0,3,0],[1,7,1],[2,7,2],[4,1,6],[4,0,4],[1,8,3],[5,3,1],[1,6,5],[5,5,2],[6,8,2],[4,2,8],[6,1,6],[1,6,0],[6,4,7],[1,7,7],[5,7,8],[1,2,1],[1,7,2],[1,4,4],[1,6,1],[6,7,1],[4,5,3],[2,5,8],[1,0,2],[6,3,6],[6,0,2],[5,4,6],[6,6,5],[6,3,2],[2,2,6],[5,0,6],[4,0,1],[1,8,8],[4,5,4],[5,2,2],[6,1,4],[1,0,0],[2,7,4],[5,1,3],[2,7,1],[4,4,1],[5,1,8],[6,7,0],[4,0,8],[2,1,1],[5,0,8],[1,2,4],[2,0,5],[5,0,7],[6,4,3],[4,5,2],[5,0,1],[1,3,3],[5,2,7],[4,4,7],[4,1,2],[6,5,6],[1,5,8],[1,5,6],[2,0,4],[0,3,7],[1,5,5],[5,1,1],[6,0,3],[0,6,0],[5,4,3],[2,5,4],[4,8,3],[2,2,2],[4,2,1],[4,3,3],[0,4,2],[1,5,0],[1,4,1],[6,2,7],[1,6,6],[1,8,5],[5,7,0],[1,3,0],[4,7,5],[6,2,8],[1,2,2],[1,1,1],[1,5,7],[1,1,2],[2,1,0],[2,8,1],[4,4,8],[5,3,8],[5,7,4],[6,1,0],[6,5,4],[4,8,6],[1,7,0],[2,0,7],[1,8,4],[6,6,2],[4,8,8],[1,0,3],[0,7,2]],"edge_endpoint_blacklist":null,"edge_endpoint_max_quantity":null}"#;
    let json_actions = r#"[{"AddNode":{"id":0,"ty":4}},{"AddNode":{"id":1,"ty":6}},{"RemoveNode":{"id":1}},{"RemoveNode":{"id":0}},{"AddNode":{"id":2,"ty":3}},{"AddEdge":{"id":0,"ty":2,"source":2,"target":2}},{"AddEdge":{"id":1,"ty":4,"source":2,"target":2}},{"AddEdge":{"id":2,"ty":4,"source":2,"target":2}},{"AddEdge":{"id":3,"ty":4,"source":2,"target":2}},{"AddNode":{"id":3,"ty":3}},{"AddEdge":{"id":4,"ty":4,"source":3,"target":2}},{"AddNode":{"id":4,"ty":8}},{"RemoveEdge":{"id":1}},{"AddNode":{"id":5,"ty":5}},{"RemoveNode":{"id":4}},{"AddEdge":{"id":5,"ty":1,"source":5,"target":5}},{"AddEdge":{"id":6,"ty":1,"source":2,"target":2}},{"AddEdge":{"id":7,"ty":2,"source":5,"target":3}},{"AddNode":{"id":6,"ty":0}}]"#;

    let s: TestSchema = serde_json::from_str(json_schema).unwrap();
    let mut g: TestGraph = TestGraph::new(s);
    let actions: Vec<Action> = serde_json::from_str(json_actions).unwrap();
    for action in actions {
        action.apply(&mut g).unwrap();
    }

    let json_py_graph = run_py_test(json_schema.to_string(), json_actions.to_string());

    let py_graph: TestGraph = serde_json::from_str(&json_py_graph).unwrap();

    let out: Vec<usize> = py_graph
        .get_outgoing(2)
        .unwrap()
        .map(|e| e.get_id())
        .collect();
    dbg!(out);
    g.assert_eq(&py_graph).unwrap();
}

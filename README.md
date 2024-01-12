# typed_graph
Graph data structure with an enforcable schema

[![Crates.io](https://img.shields.io/crates/v/typed_graph.svg)](https://crates.io/crates/typed_graph)
[![docs.rs](https://img.shields.io/docsrs/typed_graph.svg)](https://docs.rs/petgraph/latest/typed_graph/)

typed_graph is an alternative to petgraph that focuses more on functionality rather than performance.

The main guarantees typed_graph provides:
 - Consistent node and edge weight types
 - Custom keys for all nodes and edges
 - Type safe convertion from one schema to another
 - Simple traversal API
 - Preserves outgoing edge order 

## Getting Started
For a comprehensive overview on how to setup and use typed_graph see the json_graph example in the example folder

## Other crates 
 - migration_handler: Provides a language for auto generating staticly typed schemas
 - typed_graph_py: Port of typed_graph to python
use super::GenericWeight;
use crate::{
    DisAllowedEdge, DisAllowedNode, EdgeExt, Id, Key, NodeExt, SchemaExt, SchemaResult,
    TypeIdentifier, Typed, TypedGraph,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

// Define a node and edge type
pub type GenericNode<K, T> = GenericWeight<K, T>;
pub type GenericEdge<K, T> = GenericWeight<K, T>;

impl<K: Key, T: GenericTypeIdentifier> NodeExt<K> for GenericNode<K, T> {}
impl<K: Key, T: GenericTypeIdentifier> EdgeExt<K> for GenericEdge<K, T> {}

// Create a graph and result over a generic id, edge and node type
pub type GenericGraph<NK, EK, NT, ET> = TypedGraph<NK, EK, GenericSchema<NT, ET>>;
pub type GenericResult<T, NK, EK, NT, ET> = SchemaResult<T, NK, EK, GenericSchema<NT, ET>>;

/// Common trait for all generic node or edge types
///
/// This has further requirements than TypeIdentifier due to the GenericSchema
pub trait GenericTypeIdentifier: TypeIdentifier + Eq + Hash {}

impl<T> GenericTypeIdentifier for T where T: TypeIdentifier + Eq + Hash {}

/// Schema capable of controlling all aspects of the graph
///
/// The schema is build
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GenericSchema<NT: GenericTypeIdentifier, ET: GenericTypeIdentifier> {
    node_whitelist: Option<Vec<NT>>,
    node_blacklist: Option<Vec<NT>>,
    edge_whitelist: Option<Vec<ET>>,
    edge_blacklist: Option<Vec<ET>>,
    endpoint_whitelist: Option<Vec<(NT, NT, ET)>>,
    endpoint_blacklist: Option<Vec<(NT, NT, ET)>>,
    endpoint_max_quantity: Option<HashMap<(NT, NT, ET), usize>>,
}

impl<NT: GenericTypeIdentifier, ET: GenericTypeIdentifier> GenericSchema<NT, ET> {
    pub fn new() -> Self
    where
        NT: Default,
        ET: Default,
    {
        Default::default()
    }

    /// Node filter: NodeType
    pub fn node_whitelist(mut self, node_whitelist: Option<Vec<NT>>) -> Self {
        self.node_whitelist = node_whitelist;
        self
    }

    /// Node filter: NodeType
    pub fn node_blacklist(mut self, node_blacklist: Option<Vec<NT>>) -> Self {
        self.node_blacklist = node_blacklist;
        self
    }

    /// Edge filter: EdgeType
    pub fn edge_whitelist(mut self, edge_whitelist: Option<Vec<ET>>) -> Self {
        self.edge_whitelist = edge_whitelist;
        self
    }

    /// Edge filter: EdgeType
    pub fn edge_blacklist(mut self, edge_blacklist: Option<Vec<ET>>) -> Self {
        self.edge_blacklist = edge_blacklist;
        self
    }

    /// Edge filter: (EdgeType, NodeType, NodeType)
    pub fn endpoint_whitelist(
        mut self,
        edge_endpoint_whitelist: Option<Vec<(NT, NT, ET)>>,
    ) -> Self {
        self.endpoint_whitelist = edge_endpoint_whitelist;
        self
    }

    /// Edge filter: (EdgeType, NodeType, NodeType)
    pub fn endpoint_blacklist(mut self, endpoint_blacklist: Option<Vec<(NT, NT, ET)>>) -> Self {
        self.endpoint_blacklist = endpoint_blacklist;
        self
    }

    pub fn endpoint_max_quantity(
        mut self,
        endpoint_max_quantity: Option<HashMap<(NT, NT, ET), usize>>,
    ) -> Self {
        self.endpoint_max_quantity = endpoint_max_quantity;
        self
    }
}

impl<NK, EK, NT, ET> SchemaExt<NK, EK> for GenericSchema<NT, ET>
where
    NK: Key,
    EK: Key,
    NT: GenericTypeIdentifier,
    ET: GenericTypeIdentifier,
{
    type N = GenericWeight<NK, NT>;
    type E = GenericWeight<EK, ET>;

    fn name(&self) -> String {
        "GenericSchema".to_string()
    }

    fn allow_edge(
        &self,
        new_edge_count: usize,
        edge_ty: <Self::E as Typed>::Type,
        source: <Self::N as Typed>::Type,
        target: <Self::N as Typed>::Type,
    ) -> Result<(), crate::DisAllowedEdge> {
        let is_whitelist = self
            .edge_whitelist
            .as_ref()
            .map_or(true, |l| l.contains(&edge_ty));
        let is_blacklist = self
            .edge_blacklist
            .as_ref()
            .map_or(true, |l| !l.contains(&edge_ty));

        let endpoint = (source.clone(), target.clone(), edge_ty.clone());

        let is_endpoint_whitelist = self
            .endpoint_whitelist
            .as_ref()
            .map_or(true, |l| l.contains(&endpoint));

        let is_endpoint_blacklist = self
            .endpoint_blacklist
            .as_ref()
            .map_or(true, |l| !l.contains(&endpoint));

        let is_allowed_type =
            is_whitelist && is_blacklist && is_endpoint_whitelist && is_endpoint_blacklist;

        if !is_allowed_type {
            return Err(DisAllowedEdge::InvalidType);
        }

        let is_endpoint_quantity = self.endpoint_max_quantity.as_ref().map_or(true, |l| {
            l.get(&endpoint)
                .map_or(true, |quantity| new_edge_count <= *quantity)
        });
        let is_allowed_quantity = is_endpoint_quantity;

        if !is_allowed_quantity {
            return Err(DisAllowedEdge::ToMany);
        }

        Ok(())
    }

    fn allow_node(&self, node_ty: <Self::N as Typed>::Type) -> Result<(), crate::DisAllowedNode> {
        let is_whitelist = self
            .node_whitelist
            .as_ref()
            .map_or(true, |l| l.contains(&node_ty));
        let is_blacklist = self
            .node_blacklist
            .as_ref()
            .map_or(true, |l| !l.contains(&node_ty));
        let is_allowed = is_whitelist && is_blacklist;

        if !is_allowed {
            return Err(DisAllowedNode::InvalidType);
        }

        Ok(())
    }
}

impl<NK: Key, EK: Key, NT: GenericTypeIdentifier, ET: GenericTypeIdentifier>
    GenericGraph<NK, EK, NT, ET>
{
    pub fn assert_eq(&self, other: &Self) -> GenericResult<(), NK, EK, NT, ET> {
        assert_eq!(
            self.node_count(),
            other.node_count(),
            "Inconsistent node count"
        );
        assert_eq!(
            self.edge_count(),
            other.edge_count(),
            "Inconsistent edge count"
        );

        for node in self.nodes() {
            let other_node = other.get_node(node.get_id())?;
            assert_eq!(
                node.get_type(),
                other_node.get_type(),
                "Inconsistent node type"
            );
        }

        for edge in self.edges_full() {
            let other_edge = other.get_edge_full(edge.get_id())?;
            assert_eq!(
                edge.get_type(),
                other_edge.get_type(),
                "Inconsistent edge type"
            );
            assert_eq!(
                edge.get_target(),
                other_edge.get_target(),
                "Inconsistent edge target"
            );
            assert_eq!(
                edge.get_source(),
                other_edge.get_source(),
                "Inconsistent edge source"
            );
        }

        for node in self.node_ids() {
            let edges = self.get_outgoing(node)?;
            let other_edges = other.get_outgoing(node)?;

            let zipped = edges.zip(other_edges);
            for (edge, other_edge) in zipped {
                // Compares the id of the weight
                assert_eq!(edge.weight.0, other_edge.0, "Inconsistent edge order");
            }
        }

        Ok(())
    }
}

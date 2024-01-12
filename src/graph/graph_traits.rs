use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::SchemaResult;

/// Trait shared by all nodes in a graph
pub trait NodeExt<NK: Key>: Typed + Id<NK> + Clone + Debug {}
/// Trait shared by all edges in a graph
pub trait EdgeExt<EK: Key>: Typed + Id<EK> + Clone + Debug {}

pub trait SchemaExt<NK, EK>: Sized
where
    NK: Key,
    EK: Key
{
    /// Type of node weights used by the schema
    type N: NodeExt<NK>;
    /// Type of edge weights used by the schema
    type E: EdgeExt<EK>;

    /// Get the name of the schema in order to provide better error messages
    fn name(&self) -> String;
    
    /// Before adding a new node, check if the new node is allowed
    /// 
    /// Upon encountering an invalid edge return Err(InvalidType)
    fn allow_node(
        &self, 
        node_ty: <Self::N as Typed>::Type
    ) -> Result<(), DisAllowedNode>;

    /// Before adding a new edge check if the new edge is allowed
    /// 
    /// Upon encountering an invalid edge return Err(InvalidType)
    /// If the quantity limit is reached return Err(TomMny)
    fn allow_edge(
        &self, 
        new_edge_count: usize,
        edge_ty: <Self::E as Typed>::Type, 
        source: <Self::N as Typed>::Type, 
        target: <Self::N as Typed>::Type,
    ) -> Result<(), DisAllowedEdge>;
}

#[derive(Debug)]
pub enum DisAllowedNode {
    InvalidType
}

#[derive(Debug)]
pub enum DisAllowedEdge {
    ToMany,
    InvalidType
}

/// Trait indicating a type can be used as a key in the graph
/// 
/// Mostly common key types is integers and uuid's. 
/// By implementing this trait more exotic types can be used aswell
pub trait Key: Hash + Debug + PartialEq + Eq + Copy{}

impl<K> Key for K 
where
    K: Hash + Debug + Eq + Copy
{}

/// Provide a getter and setter for the id of a node or edge
pub trait Id<K: Key> {
    fn get_id(&self) -> K;
    fn set_id(&mut self, new_id: K);
}

impl<T: Key> Id<T> for T {
    fn get_id(&self) -> T {
        *self
    }

    fn set_id(&mut self, new_id: T) {
        *self = new_id;
    }
}

pub trait Name {
    type Name;
    
    /// Retrieve the name of a node or edge
    fn get_name(&self) -> Option<&Self::Name>;
}

pub trait TypeIdentifier: PartialEq + Display + Debug + Clone {}

impl<T> TypeIdentifier for T
where
    T: PartialEq + Display + Debug + Clone
{}

pub trait Typed: PartialEq<Self::Type> {
    type Type: TypeIdentifier;

    /// Retrieve a runtime representation of the type of the node or edge
    /// 
    /// PartialEq can then be used on the returned type to check if other nodes has the same type
    fn get_type(&self) -> Self::Type;
}

pub trait Downcast<NK, EK, T, S> 
where
NK: Key,
EK: Key,
S: SchemaExt<NK, EK>,
{
    /// Cast a node or edge into a more specific type
    /// 
    /// The call will fail if the requested type is not a suptype of the current one
    fn downcast(&self) -> SchemaResult<&T, NK, EK, S>;
}

pub trait DowncastMut<NK, EK, T, S> 
where
NK: Key,
EK: Key,
S: SchemaExt<NK, EK>
{
    /// Cast a node or edge into a more specific type
    /// 
    /// The call will fail if the requested type is not a suptype of the current one
    fn downcast_mut(&mut self) -> SchemaResult<&mut T, NK, EK, S>;
}
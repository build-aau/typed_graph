use std::any::type_name;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use either::Either;

use crate::{SchemaError, SchemaResult};

/// Trait shared by all nodes in a graph
pub trait NodeExt<NK: Key>: Typed + Id<NK> + Clone + Debug {}
/// Trait shared by all edges in a graph
pub trait EdgeExt<EK: Key>: Typed + Id<EK> + Clone + Debug {}

pub trait SchemaExt<NK, EK>: Sized
where
    NK: Key,
    EK: Key,
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
    fn allow_node(&self, node_ty: <Self::N as Typed>::Type) -> Result<(), DisAllowedNode>;

    /// Before adding a new edge check if the new edge is allowed
    ///
    /// Upon encountering an invalid edge return Err(InvalidType)
    /// If the quantity limit is reached return Err(TomMny)
    fn allow_edge(
        &self,
        outgoing_edge_count: usize, 
        incoming_edge_count: usize, 
        edge_ty: <Self::E as Typed>::Type,
        source: <Self::N as Typed>::Type,
        target: <Self::N as Typed>::Type,
    ) -> Result<(), DisAllowedEdge>;
}

#[derive(Debug)]
pub enum DisAllowedNode {
    InvalidType,
}

#[derive(Debug)]
pub enum DisAllowedEdge {
    ToManyOutgoing,
    ToManyIncoming,
    InvalidType,
}

/// Trait indicating a type can be used as a key in the graph
///
/// Mostly common key types is integers and uuid's.
/// By implementing this trait more exotic types can be used aswell
pub trait Key: Hash + Debug + PartialEq + Eq + Copy {}

impl<K> Key for K where K: Hash + Debug + Eq + Copy {}

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

impl<T> TypeIdentifier for T where T: PartialEq + Display + Debug + Clone {}

pub trait Typed: PartialEq<Self::Type> {
    type Type: TypeIdentifier;

    /// Retrieve a runtime representation of the type of the node or edge
    ///
    /// PartialEq can then be used on the returned type to check if other nodes has the same type
    fn get_type(&self) -> Self::Type;
}

pub trait Downcast<'b, NK, EK, T, S>
where
    NK: Key,
    EK: Key,
    T: 'b,
    S: SchemaExt<NK, EK>,
{
    /// Cast a node or edge into a more specific type
    ///
    /// The call will fail if the requested type is not a suptype of the current one
    fn downcast<'a: 'b>(&'a self) -> SchemaResult<T, NK, EK, S>;
}

pub trait DowncastMut<'b, NK, EK, T, S>
where
    NK: Key,
    EK: Key,
    T: 'b,
    S: SchemaExt<NK, EK>,
{
    /// Cast a node or edge into a more specific type
    ///
    /// The call will fail if the requested type is not a suptype of the current one
    fn downcast_mut<'a: 'b>(&'a mut self) -> SchemaResult<T, NK, EK, S>;
}

impl<'a, O1, O2, NK, EK, S, T> Downcast<'a, NK, EK, Either<&'a O1, &'a O2>, S> for T
    where
        T: Downcast<'a, NK, EK, &'a O1, S> + Downcast<'a, NK, EK, &'a O2, S> + Typed,
        O1: Typed,
        O2: Typed,
        NK: Key,
        EK: Key,
        S: SchemaExt<NK, EK>
{
    fn downcast<'b: 'a>(&'a self) -> SchemaResult<Either<&'a O1, &'a O2>, NK, EK, S> {
        let n1 = Downcast::<'a, NK, EK, &'a O1, S>::downcast(self);

        if let Ok(n1) = n1 {
            return Ok(Either::Left(n1));
        }

        let n2 = Downcast::<'a, NK, EK, &'a O2, S>::downcast(self);

        if let Ok(n2) = n2 {
            return Ok(Either::Right(n2));
        }

        Err(SchemaError::<NK, EK, S>::DownCastFailed(format!("{:?} or {:?}", type_name::<O1>(), type_name::<O2>()), self.get_type().to_string()))
    }
}

/*impl<'a, O1, O2, NK, EK, S, T> DowncastMut<'a, NK, EK, Either<&'a mut O1, &'a mut O2>, S> for T
    where
        T: DowncastMut<'a, NK, EK, &'a mut O1, S> + DowncastMut<'a, NK, EK, &'a mut O2, S> + Typed,
        O1: Typed,
        O2: Typed,
        NK: Key,
        EK: Key,
        S: SchemaExt<NK, EK>
{
    fn downcast_mut<'b: 'a>(&'a mut self) -> SchemaResult<Either<&'a mut O1, &'a mut O2>, NK, EK, S> {
        let n1 = DowncastMut::<'a, NK, EK, &'a mut O1, S>::downcast_mut(self);

        if let Ok(n1) = n1 {
            return Ok(Either::Left(n1));
        }

        drop(n1);

        let n2 = DowncastMut::<'a, NK, EK, &'a mut O2, S>::downcast_mut(self);

        if let Ok(n2) = n2 {
            return Ok(Either::Right(n2));
        }

        drop(n2);

        Err(SchemaError::<NK, EK, S>::DownCastFailed(format!("{:?} or {:?}", type_name::<O1>(), type_name::<O2>()), self.get_type().to_string()))
    }
}*/
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::{DerefMut, Deref};
use crate::{NodeKey, EdgeKey};


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct EdgeMetadata<E> {
    pub(crate) weight: E,
    pub(crate) source: NodeKey,
    pub(crate) target: NodeKey,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct NodeMetada<N> {
    pub(crate) weight: N,
    /// Look Up Table to help quickly find the incoming edges of a given node.
    /// Notice that the contained information can be produced from `edges`.
    /// Since this is duplicate information, no external mutation must be allowed.
    /// No edge order is guaranteed.
    pub(crate) incoming_edges: IndexSet<EdgeKey>,

    /// Same as `incoming_edges` (just for outgoing edges) with a notable exception:
    /// This LUT controls the order of the outgoing edges from a given node.
    /// Iteration, walkers and so on should respect this order.
    /// External manipulation of this order is to be supported.
    /// Furthermore this order can not be reconstructed from `edges`.
    /// This means that `incoming_edges` can be seen as an expendable cache, but `outgoing_edges`
    /// can not!
    pub(crate) outgoing_edges: IndexSet<EdgeKey>,
}

impl<N> AsRef<N> for NodeMetada<N> {
    fn as_ref(&self) -> &N {
        &self.weight
    }
}

impl<N> Deref for NodeMetada<N> {
    type Target = N;
    fn deref(&self) -> &Self::Target {
        &self.weight
    }
}

impl<N> DerefMut for NodeMetada<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.weight
    }
}

impl<E> EdgeMetadata<E> {

}

impl<E> AsRef<E> for EdgeMetadata<E> {
    fn as_ref(&self) -> &E {
        &self.weight
    }
}

impl<E> Deref for EdgeMetadata<E> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.weight
    }
}

impl<E> DerefMut for EdgeMetadata<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.weight
    }
}
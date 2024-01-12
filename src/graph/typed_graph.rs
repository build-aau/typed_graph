use super::*;
use crate::{TypedError, SchemaResult, GenericTypedResult};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, new_key_type};
use std::fmt::{Debug, self, Display};
use std::hash::Hash;

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Outgoing,
    Incoming
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InsertPosition {
    Before,
    After
}

new_key_type! {
    pub struct NodeKey;
    pub struct EdgeKey;
}

#[derive(Debug, Clone)]
pub struct TypedGraph<NK, EK, S: SchemaExt<NK, EK>>
where
    NK: Key,
    EK: Key
{
    /// Mapping from node ids to node keys
    node_lut: HashMap<NK, NodeKey>,
    /// Mapping from edge ids to edge keys
    edge_lut: HashMap<EK, EdgeKey>,
    /// Contains the node weights and adjecency list
    /// 
    /// Since the nodes stores its own id this can be used to convert node keys to node ids
    nodes: HopSlotMap<NodeKey, NodeMetada<S::N>>,
    /// Contains the edge weights, and edge endpoints
    /// 
    /// Since the edges stores its own id this can be used to convert edge keys to edge ids
    edges: HopSlotMap<EdgeKey, EdgeMetadata<S::E>>,

    schema: S
}

impl<NK, EK, S> TypedGraph<NK, EK, S> 
where
    NK: Key,
    EK: Key,
    S: SchemaExt<NK, EK>
{
    pub fn new(schema: S) -> Self {
        TypedGraph {
            node_lut: Default::default(),
            edge_lut: Default::default(), 
            nodes: HopSlotMap::with_key(), 
            edges: HopSlotMap::with_key(),  
            schema: schema
        }
    }

    pub fn get_schema(&self) -> &S {
        &self.schema
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn get_node_key(&self, node_id: NK) -> SchemaResult<NodeKey, NK, EK, S> {
        self.node_lut.get(&node_id).copied().ok_or_else(|| TypedError::MissingNode(node_id))
    }

    fn get_edge_key(&self, edge_id: EK) -> SchemaResult<EdgeKey, NK, EK, S> {
        self.edge_lut.get(&edge_id).copied().ok_or_else(|| TypedError::MissingEdge(edge_id))
    }

    fn get_node_internal(&self, node_key: NodeKey) -> SchemaResult<&NodeMetada<S::N>, NK, EK, S> {
        self.nodes.get(node_key).ok_or_else(|| TypedError::MissingNodeKey(node_key))
    }

    fn get_edge_internal(&self, edge_key: EdgeKey) -> SchemaResult<&EdgeMetadata<S::E>, NK, EK, S> {
        self.edges.get(edge_key).ok_or_else(|| TypedError::MissingEdgeKey(edge_key))
    }

    fn get_node_mut_internal(&mut self, node_key: NodeKey) -> SchemaResult<&mut NodeMetada<S::N>, NK, EK, S> {
        self.nodes.get_mut(node_key).ok_or_else(|| TypedError::MissingNodeKey(node_key))
    }

    fn get_edge_mut_internal(&mut self, edge_key: EdgeKey) -> SchemaResult<&mut EdgeMetadata<S::E>, NK, EK, S> {
        self.edges.get_mut(edge_key).ok_or_else(|| TypedError::MissingEdgeKey(edge_key))
    }

    pub fn get_nodes(&self) -> impl Iterator<Item = NK> + '_ {
        self.nodes.values().into_iter().map(|n| n.get_id())
    }

    pub fn get_edges(&self) -> impl Iterator<Item = EK> + '_ {
        self.edges.values().into_iter().map(|e| e.get_id())
    }

    pub fn get_node_safe(&self, node_id: NK) -> Option<&S::N> {
        let key = self.node_lut.get(&node_id)?;
        let node = self.nodes.get(*key)?;
        Some(&node.weight)
    }

    pub fn get_node_safe_mut(&mut self, node_id: NK) -> Option<&mut S::N> {
        let key = self.node_lut.get(&node_id)?;
        let node = self.nodes.get_mut(*key)?;
        Some(&mut node.weight)
    }

    pub fn get_edge_safe(&self, edge_id: EK) -> Option<&S::E> {
        let key = self.edge_lut.get(&edge_id)?;
        self.edges.get(*key).map(|x| &x.weight)
    }

    pub fn get_edge_safe_mut(&mut self, edge_id: EK) -> Option<&mut S::E> {
        let key = self.edge_lut.get(&edge_id)?;
        self.edges.get_mut(*key).map(|x| &mut x.weight)
    }

    pub fn get_node(&self, node_id: NK) -> SchemaResult<&S::N, NK, EK, S> {
        self.get_node_safe(node_id).ok_or_else(|| TypedError::MissingNode(node_id))
    }

    pub fn get_edge(&self, edge_id: EK) -> SchemaResult<&S::E, NK, EK, S> {
        self.get_edge_safe(edge_id).ok_or_else(|| TypedError::MissingEdge(edge_id))
    }

    pub fn get_edge_full(&self, edge_id: EK) -> SchemaResult<EdgeRef<'_, NK, EK, S>, NK, EK, S> {
        let edge_key = self.get_edge_key(edge_id)?;
        let edge = self.get_edge_internal(edge_key)?;
        Ok(EdgeRef {
            weight: &edge.weight,
            source: self.nodes.get(edge.source).unwrap().get_id(),
            target: self.nodes.get(edge.target).unwrap().get_id(),
            direction: Direction::Outgoing
        })
    }

    pub fn get_node_mut(&mut self, node_id: NK) -> SchemaResult<&mut S::N, NK, EK, S> {
        self.get_node_safe_mut(node_id).ok_or_else(|| TypedError::MissingNode(node_id))
    }

    pub fn get_edge_mut(&mut self, edge_id: EK) -> SchemaResult<&mut S::E, NK, EK, S> {
        self.get_edge_safe_mut(edge_id).ok_or_else(|| TypedError::MissingEdge(edge_id))
    }

    pub fn get_node_downcast<N>(&self, node_id: NK) -> SchemaResult<&N, NK, EK, S> 
    where
        S::N: Downcast<NK, EK, N, S>
    {
        self.get_node(node_id).and_then(|n| n.downcast())
    }

    pub fn get_node_downcast_mut<N>(&mut self, node_id: NK) -> SchemaResult<&mut N, NK, EK, S> 
    where
        S::N: DowncastMut<NK, EK, N, S>
    {
        self.get_node_mut(node_id).and_then(|n| n.downcast_mut())
    }

    pub fn get_edge_downcast<E>(&self, edge_id: EK) -> SchemaResult<&E, NK, EK, S> 
    where
        S::E: Downcast<NK, EK, E, S>
    {
        self.get_edge(edge_id).and_then(|e| e.downcast())
    }

    pub fn get_edge_downcast_mut<E>(&mut self, edge_id: EK) -> SchemaResult<&mut E, NK, EK, S> 
    where
        S::E: DowncastMut<NK, EK, E, S>
    {
        self.get_edge_mut(edge_id).and_then(|e| e.downcast_mut())
    }

    pub fn has_node(&self, node_id: NK) -> bool {
        self.node_lut.contains_key(&node_id)
    }

    pub fn has_edge(&self, edge_id: EK) -> bool {
        self.edge_lut.contains_key(&edge_id)
    }

    /// Reorder the outgoing edge order by inserting the source edge and shifting all other edges
    /// 
    /// This will fail if both edges do not have the same source node
    /// 
    /// InsertPosition::Before the source edge will have the same index as the target edge currently has
    /// InsertPosition::After the source edge will be placed at target_idx + 1 instead
    pub fn move_edge_order(&mut self, source_id: EK, target_id: EK, insert_position: InsertPosition) -> SchemaResult<(), NK, EK, S> {
        if source_id == target_id {
            return Ok(());
        }

        let source_edge = self.get_edge_full(source_id)?;

        let source_key = self.get_edge_key(source_id)?;
        let target_key = self.get_edge_key(target_id)?;

        // We base the search on the source node of the source edge
        // Since both edges have the same source node this is fine
        let node_key = self.get_node_key(source_edge.source)?;
        let node = self.get_node_mut_internal(node_key)?;

        // Somehow an edge was created without it being registered with the node
        if node.outgoing_edges.is_empty() {
            return Err(TypedError::InvalidInternalState);
        }

        if !node.outgoing_edges.contains(&target_key) {
            return Err(TypedError::InvalidEdgeMove(source_id, target_id));
        }

        let source_idx = node.outgoing_edges
            .get_index_of(&source_key)
            .ok_or_else(|| TypedError::InvalidEdgeMove(source_id, target_id))?;

        // Then we find out where in the order we want the node to be places
        let mut target_idx = node.outgoing_edges
            .get_index_of(&target_key)
            .ok_or_else(|| TypedError::InvalidEdgeMove(source_id, target_id))?;

        // Figure out where to place the source relative to the target
        // The position has to be adjusted as to not go out of bounds and 
        // play nicely with the behaviour of move_index
        match insert_position {
            InsertPosition::After => {
                if target_idx + 1 != node.outgoing_edges.len() && source_idx > target_idx {
                    target_idx += 1;
                }
            },
            InsertPosition::Before => {
                if target_idx != 0 && source_idx < target_idx {
                    target_idx -= 1;
                }
            }
        }

        // We then place the edge at the index and shift all the other edges to the right
        node.outgoing_edges.move_index(source_idx, target_idx);

        Ok(())
    }

    /// Add a node or update an existing one
    /// Updating a node is only allowed if all the connected edges allows for the new type
    pub fn add_node<N>(&mut self, node: N) -> SchemaResult<NK, NK, EK, S> 
    where
        N: Into<S::N>
    {
        let weight: S::N = node.into();

        // Check that the schema allows the type of the node
        let weight_type = weight.get_type();
        let allowed = self.schema.allow_node(weight.get_type());
        if let Err(e) = allowed {
            return Err(TypedError::InvalidNodeType(weight_type, e))
        }

        let node_id = weight.get_id();
        // Check if there already exists a node at the given id
        if let Ok(node_key) = self.get_node_key(node_id) {
            let node = self.get_node_internal(node_key)?;

            // Check if the existing node has the same type as the new one
            if node.get_type() != weight_type {
                // We now check if the new node is a replacement for the old one

                // Join incoming and outgoing edges
                let edge_keys = node.incoming_edges.iter().chain(node.outgoing_edges.iter());

                // Check if the new type is compatible with all the existing edges
                for edge_key in edge_keys {
                    let edge = self.get_edge_internal(*edge_key)?;
                    let weight_type = edge.weight.get_type();

                    // Replace the source node with the new node if it is an outgoing edge
                    let source_node = if edge.source == node_key {
                        &weight
                    } else {
                        self.get_node_internal(edge.source)?
                    };

                    // Replace the target node with the new node if it is an incoming edge
                    let target_node = if edge.target == node_key {
                        &weight
                    } else {
                        self.get_node_internal(edge.target)?
                    };

                    // Count the number of other edges going in the same direction
                    let mut quantity = 0;
                    let outgoing = self.get_outgoing(source_node.get_id())?;
                    for out_edge in outgoing {
                        // Only look at edge with the same type as the focused one
                        if out_edge.weight.get_type() != weight_type {
                            continue;
                        }

                        // Only look at edges going to nodes of the same type
                        let out_target_node = self.get_node(out_edge.target)?;
                        if out_target_node.get_type() != target_node.get_type() {
                            continue;
                        }
                        
                        quantity += 1;
                    }
                    
                    let allowed = self.schema.allow_edge(
                        // Account for the new type adding a new edge
                        quantity + 1,
                        weight_type.clone(), 
                        source_node.get_type(), 
                        target_node.get_type(),

                    );
                    if let Err(e) = allowed {
                        return Err(TypedError::InvalidEdgeType(weight_type, source_node.get_type(), target_node.get_type(), e))
                    }
                }

                // Update the node
                let node = self.get_node_mut_internal(node_key)?;
                node.weight = weight;
            } else {
                // Just replace the node
                let node = self.get_node_mut_internal(node_key)?;
                node.weight = weight;

            }
        } else {
            // Add the node to the graph
            let node_key = self.nodes.insert(NodeMetada {
                weight: weight,
                outgoing_edges: Default::default(),
                incoming_edges: Default::default()
            });
            self.node_lut.insert(node_id, node_key);
        }

        Ok(node_id)
    }

    /// Add an edge and if it already exists update the weight and enpoints of the edge
    /// The edge will preserve the order of endpoints that does not change
    pub fn add_edge<E>(&mut self, source: NK, target: NK, edge: E) -> SchemaResult<EK, NK, EK, S> 
    where
        E: Into<S::E>
    {
        let weight: S::E = edge.into();
        let edge_id = weight.get_id();

        let source_key = self.get_node_key(source)?;
        let target_key = self.get_node_key(target)?;

        let weight_type = weight.get_type();
        let source_node = self.get_node_internal(source_key)?;
        let target_node = self.get_node_internal(target_key)?;

        let mut quantity = 0;
        let edges = self.get_outgoing(source_node.get_id())?;
        for edge in edges {
            // Only look at edges of the same type
            if edge.get_type() != weight.get_type() {
                continue;
            }

            // Only look at edges going to nodes of the same type
            let out_target_node = self.get_node(edge.target)?;
            if out_target_node.get_type() != target_node.get_type() {
                continue;
            }

            quantity += 1;
        }

        let allowed = self.schema.allow_edge(
            quantity + 1,
            weight_type.clone(), 
            source_node.get_type(), 
            target_node.get_type(),
        );
        if let Err(e) = allowed {
            return Err(TypedError::InvalidEdgeType(weight_type, source_node.get_type(), target_node.get_type(), e))
        }

        if let Some(edge_key) = self.edge_lut.get(&edge_id).copied() {
            let edge = self.get_edge_mut_internal(edge_key)?;
            edge.weight = weight;

            let source_key = edge.source;
            let target_key = edge.target;

            // Update the source endpoint
            let old_source = self.get_node_internal(source_key)?.get_id();
            if old_source != source {
                let old_source_key = self.get_node_key(old_source)?;
                self.get_node_mut_internal(old_source_key)?
                    .outgoing_edges
                    .shift_remove(&edge_key);

                self.get_node_mut_internal(source_key)?
                    .outgoing_edges
                    .shift_remove(&edge_key);
            }

            // Update the target
            let old_target = self.get_node_internal(target_key)?.get_id();
            if old_target != target {
                let old_target_key = self.get_node_key(old_target)?;
                self.get_node_mut_internal(old_target_key)?
                    .incoming_edges
                    .remove(&edge_key);

                self.get_node_mut_internal(target_key)?
                    .incoming_edges
                    .remove(&edge_key);
            }
        } else {
            // Insert the edge
            let full_weight = EdgeMetadata {
                weight,
                source: source_key,
                target: target_key
            };
            let edge_key = self.edges.insert(full_weight);
            self.edge_lut.insert(edge_id, edge_key);
    
            // Add the edge to the source
            self.get_node_mut_internal(source_key)?
                .outgoing_edges
                .insert(edge_key);
        
            // Add the edge to the target
            self.get_node_mut_internal(target_key)?
                .incoming_edges
                .insert(edge_key);

        }


        Ok(edge_id)
    }

    /// Remove a node and all edges to and from it
    pub fn remove_node(&mut self, node_id: NK) -> SchemaResult<S::N, NK, EK, S> {
        let node_key = self.node_lut.remove(&node_id).ok_or_else(|| TypedError::NodeIdMissing(node_id))?;
        let node = self.nodes.remove(node_key).unwrap();

        for edge_key in node.outgoing_edges {
            let edge = self.edges.remove(edge_key).ok_or_else(|| TypedError::InvalidInternalState)?;
            self.edge_lut.remove(&edge.weight.get_id());
            if edge.target != node_key {
                self.get_node_mut_internal(edge.target)?.incoming_edges.remove(&edge_key);
            }
        }

        for edge_key in node.incoming_edges {
            // edge loops will show up in both incoming and outgoing edges
            // So the egde might already have been deleted
            if !self.edges.contains_key(edge_key) {
                continue;
            }

            let edge = self.edges.remove(edge_key).ok_or_else(|| TypedError::InvalidInternalState)?;
            self.edge_lut.remove(&edge.weight.get_id());
            if edge.source != node_key {
                self.get_node_mut_internal(edge.source)?.outgoing_edges.shift_remove(&edge_key);
            }
        }

        Ok(node.weight)
    }

    /// Remove an edge.
    pub fn remove_edge(&mut self, edge_id: EK) -> SchemaResult<S::E, NK, EK, S> {
        let edge_key = self.edge_lut.remove(&edge_id).ok_or_else(|| TypedError::EdgeIdMissing(edge_id))?;

        // Remove the edge itself.
        let edge = self.edges.remove(edge_key).ok_or_else(|| TypedError::InvalidInternalState)?;
        self.get_node_mut_internal(edge.source)?.outgoing_edges.shift_remove(&edge_key);
        self.get_node_mut_internal(edge.target)?.incoming_edges.remove(&edge_key);

        Ok(edge.weight)
    }

    /// Get all incoming edges 
    pub fn get_incoming<'a>(&'a self, node_id: NK) -> SchemaResult<impl Iterator<Item = EdgeRef<'a, NK, EK, S>>, NK, EK, S> {
        let node_key = *self.node_lut.get(&node_id).ok_or_else(|| TypedError::NodeIdMissing(node_id))?;
        Ok(self
            .get_node_internal(node_key)?
            .incoming_edges
            .iter()
            .map(|edge_key| {
                let edge = self.edges.get(*edge_key).unwrap();
                EdgeRef {
                    weight: &edge.weight,
                    source: self.nodes.get(edge.source).unwrap().get_id(),
                    target: self.nodes.get(edge.target).unwrap().get_id(),
                    direction: Direction::Incoming
                }
            })
        )
    }

    pub fn get_outgoing<'a>(&'a self, node_id: NK) -> SchemaResult<impl Iterator<Item = EdgeRef<'a, NK, EK, S>>, NK, EK, S> {
        let node_key = *self.node_lut.get(&node_id).ok_or_else(|| TypedError::NodeIdMissing(node_id))?;
        Ok(self
            .get_node_internal(node_key)?
            .outgoing_edges
            .iter()
            .map(|edge_key| {
                let edge = self.edges.get(*edge_key).unwrap();
                EdgeRef {
                    weight: &edge.weight,
                    source: self.nodes.get(edge.source).unwrap().get_id(),
                    target: self.nodes.get(edge.target).unwrap().get_id(),
                    direction: Direction::Outgoing
                }
            })
        )
    }

    pub fn get_incoming_and_outgoing<'a>(&'a self, node_id: NK) -> SchemaResult<impl Iterator<Item = EdgeRef<'a, NK, EK, S>>, NK, EK, S> {
        self
            .get_incoming(node_id)
            .and_then(|inc| self
                .get_outgoing(node_id)
                .map(|out|inc.chain(out))
            )
    }

    pub fn get_outgoing_filter_edge<'a, F>(&'a self, node_id: NK, filter: F) -> SchemaResult<impl Iterator<Item = EdgeRef<'a, NK, EK, S>>, NK, EK, S> 
    where
        F: Fn(&S::E) -> bool
    { 
        Ok(
            self
                .get_outgoing(node_id)?
                .filter(move |e|  filter(&e.weight))
        )
    }

    
    pub fn get_incoming_filter_edge<'a, F>(&'a self, node_id: NK, filter: F) -> SchemaResult<impl Iterator<Item = EdgeRef<'a, NK, EK, S>>, NK, EK, S>
    where
        F: Fn(&S::E) -> bool
    { 
        Ok(
            self
                .get_incoming(node_id)?
                .filter(move |e|  filter(&e.weight))
        )
    }

    pub fn nodes<'a>(&'a self) -> impl Iterator<Item = &S::N> + 'a {
        self.nodes.values().map(Deref::deref)
    }

    pub fn edges<'a>(&'a self) -> impl Iterator<Item = &S::E> + 'a {
        self.edges.values().map(Deref::deref)
    }

    pub fn edges_full<'a>(&'a self) -> impl Iterator<Item = EdgeRef<'a, NK, EK, S>> + 'a {
        self
            .edges
            .values()
            .map(|edge| EdgeRef {
                weight: &edge.weight,
                source: self.nodes.get(edge.source).unwrap().get_id(),
                target: self.nodes.get(edge.target).unwrap().get_id(),
                direction: Direction::Outgoing
            })
    }

    pub fn node_ids<'a>(&'a self) -> impl Iterator<Item = NK> + 'a {
        self.nodes.values().map(Deref::deref).map(Id::get_id)
    }

    pub fn edge_ids<'a>(&'a self) -> impl Iterator<Item = EK> + 'a {
        self.edges.values().map(Deref::deref).map(Id::get_id)
    }

    /// Apply a Migration to the current graph
    pub fn migrate<NS>(self, new_schema: NS, handler: &S::Handler) -> GenericTypedResult<TypedGraph<NK, EK, NS>, NK, EK>
    where
        S: Migration<NK, EK, NS>,
        NS: SchemaExt<NK, EK> + Clone,
    {
        Migration::migrate(self, handler, new_schema)
    }
    
    /// Migrate directly from one version to another
    pub fn migrate_direct<NS>(self) -> GenericTypedResult<TypedGraph<NK, EK, NS>, NK, EK>
    where
        S: DirectMigration<NK, EK, NS>,
        NS: SchemaExt<NK, EK>,
    {
        DirectMigration::migrate(self)
    }

    /// Convert the graph from one schema to another using two mapping functions
    /// The mapping functions are not allowed to change the id of any of the nodes only their data
    /// 
    /// 
    /// Returning None from a mapping function will delete the node from the resulting graph
    /// 
    /// 
    /// When mapping to a schema with lower bounds on the number of edges allowed from a node. 
    /// Edges higher in the outgoing edge order will be removed
    pub fn update_schema<NS, NF, EF>(mut self, schema: NS, node_map: NF, edge_map: EF) -> SchemaResult<TypedGraph<NK, EK, NS>, NK, EK, NS>
    where
        NS: SchemaExt<NK, EK>,
        NF: Fn(&S, &NS, S::N) -> Option<NS::N>,
        EF: Fn(&S, &NS, S::E) -> Option<NS::E>,
    {
        let old_schema = self.schema;
        let mut new_graph = TypedGraph::new(schema);
        
        // Create a list of all the edges that stores them in outgoing order
        let mut edges = Vec::new();
        for (_, node) in &self.nodes {
            for e in &node.outgoing_edges {
                edges.push(self.edges.remove(*e).ok_or_else(|| TypedError::InvalidInternalState)?);
            }
        }

        let mut node_id_lut = HashMap::new();

        for (nk, node) in self.nodes {
            let old_id = node.get_id();
            node_id_lut.insert(nk, old_id);

            // Remove the node if it is not part of the new schema
            if let Some(n) = node_map(&old_schema, new_graph.get_schema(), node.weight) {
                // Check that the mapping function is not changing the id
                if n.get_id() != old_id {
                    return Err(TypedError::InconsistentNodeIds(old_id, n.get_id()));
                }

                new_graph.add_node(n)?;
            }
        }

        // Update the edges in outgoing order
        for edge in edges {
            let old_id = edge.weight.get_id();

            // Remove the edge if it is not part of the new schema
            if let Some(e) = edge_map(&old_schema, new_graph.get_schema(), edge.weight) {
                // Check that the mapping function is not changing the id
                if e.get_id() != old_id {
                    return Err(TypedError::InconsistentEdgeIds(old_id, e.get_id()));
                }

                let source_id = *node_id_lut.get(&edge.source).ok_or_else(|| TypedError::InvalidInternalState)?;
                let target_id = *node_id_lut.get(&edge.target).ok_or_else(|| TypedError::InvalidInternalState)?;

                // Don't include the edge if the source or target has been removed
                if new_graph.has_node(source_id) && new_graph.has_node(target_id) {
                    let e = new_graph.add_edge(source_id, target_id, e);

                    match e {
                        // Any excess edges are removed
                        // Since egdes are updated in outgoing order this will remove the last edges in the outgoing order
                        Err(TypedError::InvalidEdgeType(_, _, _, DisAllowedEdge::ToMany)) => (),
                        Err(e) => Err(e)?,
                        Ok(_) => ()
                    }
                }
            }
        }

        Ok(new_graph)
    }
}

impl<NK, EK, S> Default for TypedGraph<NK, EK, S> 
where
    NK: Key,
    EK: Key,
    S: SchemaExt<NK, EK> + Default
{
    fn default() -> Self {
        TypedGraph { 
            node_lut: Default::default(),
            edge_lut: Default::default(),
            nodes: HopSlotMap::with_key(), 
            edges: HopSlotMap::with_key(), 
            schema: S::default()
        }
    }
}

use serde::ser::*;
use serde::de::*;
use serde::de::Error;

/// A reference to an edge with its source and target id
#[derive(Serialize)]
struct EdgeWriteDTO<'a, NK, E> {
    weight: &'a E,
    source: NK,
    target: NK
}

// This is what #[derive(Serialize)] would generate.
impl<NK, EK, N, E, S> Serialize for TypedGraph<NK, EK, S> 
where
    NK: Key + Serialize,
    EK: Key + Serialize,
    N: Serialize + NodeExt<NK>,
    E: Serialize + EdgeExt<EK>,
    S: SchemaExt<NK, EK, N = N, E = E> + Serialize
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        // Serialize the nodes as is
        let nodes: Vec<_> = self
            .nodes
            .values()
            .map(|n| &n.weight)
            .collect();

        // Wrap the edges in write dto
        let edges: Vec<_> = self
            .nodes
            .values()
            // Export the edges in outgoing order
            // This ensure that the will be imported in the correct order
            .flat_map(|n| n.outgoing_edges.iter())
            .map(|ek| self.edges.get(*ek).unwrap())
            .map(|e| EdgeWriteDTO {
                weight: &e.weight,
                source: self.nodes.get(e.source).unwrap().get_id(),
                target: self.nodes.get(e.target).unwrap().get_id(),
            })
            .collect();
        
        // Serialize the graph as a map with 3 fields
        let mut s = serializer.serialize_map(Some(3))?;
        s.serialize_entry("schema", &self.schema)?;
        s.serialize_entry("nodes", &nodes)?;
        s.serialize_entry("edges", &edges)?;
        s.end()
    }
}

/// An owned reference to aedge with its source and target id
#[derive(Deserialize)]
struct EdgeReadDTO<NK, E> {
    weight: E,
    source: NK,
    target: NK
}

/// A deserialize visitor that can generate a TypedGraph
/// 
/// this contains all the generics used by the TypeGraph since they would otherwise be seen as not used 
#[derive(Default)]
struct TypedGraphVisitor<NK, EK, N, E, S> 
where
    NK: Key,
    EK: Key,
    N: NodeExt<NK>,
    E: EdgeExt<EK>,
    S: SchemaExt<NK, EK, N = N, E = E>,
{
    nk: PhantomData<NK>,
    ek: PhantomData<EK>,
    n: PhantomData<N>,
    e: PhantomData<E>,
    s: PhantomData<S>,
}

impl<'de, NK, EK, N, E, S> Visitor<'de> for TypedGraphVisitor<NK, EK, N, E, S>
where
    NK: Key + Display + Deserialize<'de>,
    EK: Key + Display + Deserialize<'de>,
    N: NodeExt<NK> + Deserialize<'de>,
    E: EdgeExt<EK> + Deserialize<'de>,
    S: SchemaExt<NK, EK, N = N, E = E> + Deserialize<'de>,
{
    /// Produce a typed graph
    type Value = TypedGraph<NK, EK, S>;

    /// Message in case it all goes wrong
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("TypedGraph")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {

        // Step 1: Deserialize the schema
        let (schema_field, schema): (&'de str, S) = access.next_entry()?.ok_or_else(|| M::Error::missing_field("schema"))?;
        if schema_field != "schema" {
            return Err(M::Error::unknown_field(schema_field, &["schema"]));
        }
        
        let mut g = TypedGraph::new(schema);

        // Step 2: Deserialize the nodes
        let (nodes_field, nodes): (&'de str, Vec<N>) = access.next_entry()?.ok_or_else(|| M::Error::missing_field("nodes"))?;
        if nodes_field != "nodes" {
            return Err(M::Error::unknown_field(nodes_field, &["nodes"]));
        }

        // Check for id collisions and propper node types
        for n in nodes {
            g.add_node(n).map_err(|e| M::Error::custom(e))?;
        }

        // Step 3: Deserialize the edges
        let (edges_field, edges): (&'de str, Vec<EdgeReadDTO<NK, E>>) = access.next_entry()?.ok_or_else(|| M::Error::missing_field("edges"))?;
        if edges_field != "edges" {
            return Err(M::Error::unknown_field(edges_field, &["edges"]));
        }

        // Check for id collisions and propper edge types
        for e in edges {
            g.add_edge(e.source, e.target, e.weight).map_err(|e| M::Error::custom(e))?;
        }

        Ok(g)

    }
}

/// Use the visitor to deserialize the TypedGraph
impl<'de, NK, EK, N, E, S> Deserialize<'de> for TypedGraph<NK, EK, S> 
where
    NK: Key + Display + Deserialize<'de>,
    EK: Key + Display + Deserialize<'de>,
    N: NodeExt<NK> + Deserialize<'de>,
    E: EdgeExt<EK> + Deserialize<'de>,
    S: SchemaExt<NK, EK, N = N, E = E> + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TypedGraphVisitor {
            nk: PhantomData,
            ek: PhantomData,
            n: PhantomData,
            e: PhantomData,
            s: PhantomData
        })
    }
}

#[test]
fn graph_compose_test() -> crate::test::TestResult<()> {
    use crate::test::*;
    use fake::Dummy;

    let config = CompleteGraph {
        nodes: 100,
        node_types: 5,
        edge_types: 5
    };

    let g = TestGraph::dummy(&config);
    let s = serde_json::to_string_pretty(&g)?;
    let ng: TestGraph = serde_json::from_str(&s)?;
    g.assert_eq(&ng)?;

    Ok(())
}

#[test]
fn graph_quantity_test() -> crate::test::TestResult<()> {
    use crate::test::*;

    const NODE_TYPE: usize = 0;
    const NODE_TYPE1: usize = 1;
    const NODE_TYPE2: usize = 2;
    const EDGE_TYPE1: usize = 0;
    const EDGE_TYPE2: usize = 2;

    let s = TestSchema::new()
        .endpoint_max_quantity(Some(vec![
            // Set a max capacity of 2 edges of type EDGE_TYPE1 from NODE_TYPE to NODE_TYPE1
            ((EDGE_TYPE1, NODE_TYPE, NODE_TYPE1), 2)
        ].into_iter().collect()));
    let mut g = TestGraph::new(s);

    let source_id = g.add_node((0, NODE_TYPE))?;
    let target_id = g.add_node((1, NODE_TYPE1))?;

    // First outgoing edge
    g.add_edge(source_id, target_id, (0, EDGE_TYPE1))?;
    // Second outgoing edge
    g.add_edge(source_id, target_id, (1, EDGE_TYPE1))?;
    // Third outgoing edge
    let e = g.add_edge(source_id, target_id, (2, EDGE_TYPE1));
    assert!(e.is_err(), "Added third edge");

    // Try to add third edge by updating an existing edge of another type
    g.add_edge(source_id, target_id, (2, EDGE_TYPE2))?;

    let e = g.add_edge(source_id, target_id, (2, EDGE_TYPE1));
    assert!(e.is_err(), "Updating edge");
    g.remove_edge(2)?;

    // Try to add third edge by updating an existing edge of another type going in the opposit direction
    g.add_edge(target_id, source_id, (2, EDGE_TYPE2))?;

    let e = g.add_edge(source_id, target_id, (2, EDGE_TYPE1));
    assert!(e.is_err(), "Updating edge reversed");
    g.remove_edge(2)?;

    // Try to add third edge by updating an existing edge of another type coming from different nodes
    let source_id2 = g.add_node((2, NODE_TYPE2))?;
    let target_id2 = g.add_node((3, NODE_TYPE2))?;
    g.add_edge(target_id2, source_id2, (2, EDGE_TYPE2))?;
    
    let e = g.add_edge(source_id, target_id, (2, EDGE_TYPE1));
    assert!(e.is_err(), "Updating completly different edge");
    g.remove_edge(2)?;

    // Make sure the limit only works in one direction
    // First incoming edge
    g.add_edge(target_id, source_id, (10, EDGE_TYPE1))?;
    // Second incoming edge
    g.add_edge(target_id, source_id, (11, EDGE_TYPE1))?;
    // Third incoming edge
    g.add_edge(target_id, source_id, (12, EDGE_TYPE1))?;
    // Fourth incoming edge
    g.add_edge(target_id, source_id, (12, EDGE_TYPE1))?;

    Ok(())
}

#[test]
fn edge_order() -> crate::test::TestResult<()> {
    use crate::test::*;

    let s = TestSchema::new();
    let mut g = TestGraph::new(s);

    let a = g.add_node((0, 0))?;
    let b = g.add_node((1, 0))?;

    for i in 0..5 {
        g.add_edge(a, b, (i, i))?;
    }

    let ids: Vec<usize> = g.get_outgoing(a)?.map(|e| e.get_type()).collect();
    assert_eq!(ids, &[0, 1, 2, 3, 4]);

    // Swap before when after
    let mut ng = g.clone();
    ng.move_edge_order(3, 1, InsertPosition::Before)?;
    let ids: Vec<usize> = ng.get_outgoing(0)?.map(|e| e.get_type()).collect();
    assert_eq!(ids, &[0, 3, 1, 2, 4]);

    // Swap after when after
    let mut ng = g.clone();
    ng.move_edge_order(3, 1, InsertPosition::After)?;
    let ids: Vec<usize> = ng.get_outgoing(a)?.map(|e| e.get_type()).collect();
    assert_eq!(ids, &[0, 1, 3, 2, 4]);

    // Swap before start
    let mut ng = g.clone();
    ng.move_edge_order(3, 0, InsertPosition::Before)?;
    let ids: Vec<usize> = ng.get_outgoing(a)?.map(|e| e.get_type()).collect();
    assert_eq!(ids, &[3, 0, 1, 2, 4]);

    // Swap after end
    let mut ng = g.clone();
    ng.move_edge_order(3, 4, InsertPosition::After)?;
    let ids: Vec<usize> = ng.get_outgoing(a)?.map(|e| e.get_type()).collect();
    assert_eq!(ids, &[0, 1, 2, 4, 3]);



    // Swap before self is identity
    let mut ng = g.clone();
    ng.move_edge_order(2, 2, InsertPosition::Before)?;
    let ids: Vec<usize> = ng.get_outgoing(a)?.map(|e| e.get_type()).collect();
    assert_eq!(ids, &[0, 1, 2, 3, 4]);

    // Swap after self is identity
    let mut ng = g.clone();
    ng.move_edge_order(2, 2, InsertPosition::After)?;
    let ids: Vec<usize> = ng.get_outgoing(a)?.map(|e| e.get_type()).collect();
    assert_eq!(ids, &[0, 1, 2, 3, 4]);


    // Swap before when before
    let mut ng = g.clone();
    ng.move_edge_order(1, 2, InsertPosition::Before)?;
    let ids: Vec<usize> = ng.get_outgoing(0)?.map(|e| e.get_type()).collect();
    assert_eq!(ids, &[0, 1, 2, 3, 4]);

    // Swap after when before
    let mut ng = g.clone();
    ng.move_edge_order(1, 2, InsertPosition::After)?;
    let ids: Vec<usize> = ng.get_outgoing(a)?.map(|e| e.get_type()).collect();
    assert_eq!(ids, &[0, 2, 1, 3, 4]);

    Ok(())
}
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::value::Value;
///! In this example we will implement a json backed graph using serde_json
///!
///! ## Implementation Details
///! The graph represents nodes and edges as json objects
///! The type and id of a node or edge are found in the "id" and "type" fields
///! The schema stores a list of allowed node types, edge types and relationships between edge types and node types
///! Migration of the schema is done by discarding any node or edge that does not have an allowed type by the new schema
///!
///! ## Implementation Process
///! ### Define node and edge type
///! First we define the struct Weight which is a light wrapper around serde_json::Value
///! ```rust
///! pub struct Weight(Value);
///! ```
///!
///! ### Implementing graph traits
///! Then to use the Weight as a node and edge we need to implement NodeExt and EdgeExt
///! However before doing so we need to implement Id, Typed and PartialEq<String> for Weight
///!
///! The requirement for PartialEq<String> comes from the fact that we set the Type for Typed as String.
///! The Type used in Typed is quick way of checking if a given weight has a specific type.
///!
///! ### Implementing schema
///! Finally we define the schema
///! ```rust
///! struct JsonSchema {
///!     /// Name used for errors and debugging
///!     version: String,
///!     /// List of allowed node tyes
///!     nodes: Vec<String>,
///!     /// List of allowed edge types, source node types and target node types
///!     edges: Vec<(String, String, String)>
///! }
///! ```
///!
///! Then to use the schema we implement SchemaExt and set the node and edge to Weight
///!
///! ### Implementing migrations
///! Additionally we implement a migration strategy from one schema to another
///! For this example we just discard any node or edges that have types which are not allowed in the new schema
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use typed_graph::*;

type WeightId = u64;

#[derive(Debug, Clone)]
pub struct Weight(Value);

/// The Weight is actually just a light wrapper around serde_json::Value
impl Deref for Weight {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The Weight is actually just a light wrapper around serde_json::Value
impl DerefMut for Weight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Make it easier to create Weight from Value
impl From<Value> for Weight {
    fn from(value: Value) -> Self {
        Weight(value)
    }
}

impl Weight {
    /// Get the id field from json and return it as a u64
    ///
    /// If the id is not available this will return 0
    fn get_id_from_json(&self) -> u64 {
        self.as_object()
            .and_then(|obj| obj.get("id"))
            .and_then(|ty| ty.as_u64())
            .unwrap_or_default()
    }

    /// Set the id if possible
    fn set_id_for_json(&mut self, id: u64) {
        self.as_object_mut()
            .and_then(|obj| obj.insert("id".to_string(), id.into()));
    }

    /// Get the type and return it as a string refference
    ///
    /// If the type is not available this will return an empty string
    fn get_type_from_json(&self) -> &str {
        self.as_object()
            .and_then(|obj| obj.get("type"))
            .and_then(|ty| ty.as_str())
            .unwrap_or_default()
    }
}

/// Make the id available to the graph
impl Id<WeightId> for Weight {
    fn get_id(&self) -> WeightId {
        self.get_id_from_json()
    }

    fn set_id(&mut self, new_id: WeightId) {
        self.set_id_for_json(new_id)
    }
}

/// Tell the graph that as type indicators we are using Strings
/// Normally you would have a dedicated type for this, however since we do not know which types exists
/// We just default to sending the name of the type around
impl Typed for Weight {
    type Type = String;
    fn get_type(&self) -> Self::Type {
        self.get_type_from_json().to_string()
    }
}

/// Check a given type to see if it is the same as the one for the weight
impl PartialEq<String> for Weight {
    fn eq(&self, other: &String) -> bool {
        self.get_type_from_json() == other
    }
}

/// Here we use the weight for both nodes and edges
/// Often you would want to store different values
/// on the nodes and edges and therefore seperate the tow
impl NodeExt<WeightId> for Weight {}
impl EdgeExt<WeightId> for Weight {}

/// As a schema we store the name of the schema
/// Which node types it allows
/// And which edges between these nodes it allows
#[derive(Serialize, Deserialize, Clone, Debug)]
struct JsonSchema {
    /// Name used for errors and debugging
    version: String,
    /// List of allowed node tyes
    nodes: Vec<String>,
    /// List of allowed edge types, source node types and target node types
    edges: Vec<(String, String, String)>,
}

/// Here we define the rules for the schema
///
/// This is what is used to check if a given type is allowed
impl SchemaExt<WeightId, WeightId> for JsonSchema {
    type E = Weight;
    type N = Weight;

    /// The name of the schema is just its version number
    fn name(&self) -> String {
        self.version.clone()
    }

    /// Only let nodes in the whitelist through
    fn allow_node(&self, node_ty: <Self::N as Typed>::Type) -> Result<(), DisAllowedNode> {
        if self.nodes.contains(&node_ty) {
            Ok(())
        } else {
            Err(DisAllowedNode::InvalidType)
        }
    }

    /// Only let edges in the whitelist through
    ///
    /// This could be modified to also check for quantity by returning
    /// DisAllowedEdge::ToMany if the count exceeds a specified amount
    fn allow_edge(
        &self,
        _outgoing_edge_count: usize, 
        _incoming_edge_count: usize, 
        edge_ty: <Self::E as Typed>::Type,
        source: <Self::N as Typed>::Type,
        target: <Self::N as Typed>::Type,
    ) -> Result<(), DisAllowedEdge> {
        // an edge is allowed if it is present in the schema
        let endpoint = (edge_ty, source, target);
        if self.edges.contains(&endpoint) {
            Ok(())
        } else {
            Err(DisAllowedEdge::InvalidType)
        }
    }
}

// A migration of the json graph is done by going from one instance of the JsonSchema to another
// Since each schema contains which nodes and edges are allowed
// updating the data is just a question of discarding any types that are not in the new schema
impl MigrateSchema<WeightId, WeightId, JsonSchema> for JsonSchema {
    /// Update an edge from the previous schema to the new one
    fn update_edge(&self, new_schema: &JsonSchema, edge: Self::E) -> Option<Weight> {
        let edge_type = edge.get_type();
        // Check if the edge type also exist in the new schema
        let is_allowed = new_schema.edges.iter().any(|(ty, _, _)| &edge_type == ty);
        // If it is not allowed then it is marked as not present and will be removed as part of the migration process
        is_allowed.then(|| edge)
    }

    /// The same process is used for the nodes
    fn update_node(&self, new_schema: &JsonSchema, node: Self::N) -> Option<Weight> {
        let node_type = node.get_type();
        let is_allowed = new_schema.nodes.iter().any(|ty| &node_type == ty);
        is_allowed.then(|| node)
    }

    /// Update of edge types and node types should be consistent with that of update node and edge
    /// We achieve this here by just copy pasting the check we did for nodes and edges above
    fn update_edge_type(
        &self,
        new_schema: &JsonSchema,
        edge_type: <Self::E as Typed>::Type,
    ) -> Option<<Weight as Typed>::Type> {
        let is_allowed = new_schema.edges.iter().any(|(ty, _, _)| &edge_type == ty);
        is_allowed.then(|| edge_type)
    }

    fn update_node_type(
        &self,
        new_schema: &JsonSchema,
        node_type: <Self::N as Typed>::Type,
    ) -> Option<<Weight as Typed>::Type> {
        let is_allowed = new_schema.nodes.iter().any(|ty| &node_type == ty);
        is_allowed.then(|| node_type)
    }
}

impl Migration<WeightId, WeightId, JsonSchema> for JsonSchema {
    // Here we can set a migration handler
    // A migration handler allows us to change the graph manually while the migration is being done
    // In this example we are only interested in
    type Handler = DefaultMigrationHandler;
}

fn main() -> SchemaResult<(), WeightId, WeightId, JsonSchema> {
    // Create a schema for the graph
    //        AC
    // ---------------
    // v             |
    // A ---> B ---> C--
    //    AB    BC   ^  | CC
    //                --
    let schemav0 = JsonSchema {
        version: "V0".to_string(),
        nodes: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        edges: vec![
            ("AB".to_string(), "A".to_string(), "B".to_string()),
            ("BC".to_string(), "B".to_string(), "C".to_string()),
            ("CA".to_string(), "C".to_string(), "A".to_string()),
            ("CC".to_string(), "C".to_string(), "C".to_string()),
        ],
    };

    let mut gv0 = TypedGraph::new(schemav0);

    // We can then add nodes and edges to the graph
    // As long as their types match what was described in the schema
    let a_id = gv0.add_node(json!({"id": 0, "type": "A"}))?;
    let b_id = gv0.add_node(json!({"id": 1, "type": "B"}))?;
    let c_id = gv0.add_node(json!({"id": 2, "type": "C"}))?;
    let ab_id = gv0.add_edge(a_id, b_id, json!({"id": 0, "type": "AB"}))?;
    let bc_id = gv0.add_edge(b_id, c_id, json!({"id": 1, "type": "BC"}))?;
    let ca_id = gv0.add_edge(c_id, a_id, json!({"id": 2, "type": "CA"}))?;

    // trying to add a type that is not part of the schema will result in  an error
    let new_node_id = gv0.add_node(json!({"id": 2, "type": "D"}));
    assert!(new_node_id.is_err());
    println!("Adding node D");
    println!("{:?}", new_node_id);

    // The same thing happens when trying to add an edge with a type that is not allowed
    let new_edge_id = gv0.add_edge(c_id, a_id, json!({"id": 2, "type": "AB"}));
    assert!(new_edge_id.is_err());
    println!("Adding edge AC");
    println!("{:?}", new_edge_id);

    // Calling add on an id that is already used will update the type of the node or edge at that position
    // This only works if the replaced type is compatible with all the connected nodes and edges

    // We are also able to add multiple edges between the same nodes
    let dublicate_edge_id = gv0.add_edge(a_id, b_id, json!({"id": 3, "type": "AB"}))?;
    gv0.remove_edge(dublicate_edge_id)?;

    // Loops are also allowed as long as the are part of the schema
    let dublicate_edge_id = gv0.add_edge(c_id, c_id, json!({"id": 3, "type": "CC"}))?;
    gv0.remove_edge(dublicate_edge_id)?;

    // if we remove a node all its surrounding edges will be removed aswell
    let a = gv0.remove_node(a_id)?;
    assert_eq!(gv0.has_edge(ab_id), false);
    assert_eq!(gv0.has_edge(ca_id), false);
    gv0.add_node(a)?;
    gv0.add_edge(a_id, b_id, json!({"id": 0, "type": "AB"}))?;
    gv0.add_edge(c_id, a_id, json!({"id": 2, "type": "CA"}))?;

    // Traversal of the graph is done using the get_outgoing, get_incoming and get_incoming_and_outgoing functions
    let a_outgoing: Vec<_> = gv0.get_outgoing(a_id)?.collect();
    let b_incoming: Vec<_> = gv0.get_incoming(b_id)?.collect();

    assert_eq!(a_outgoing.len(), 1);
    assert_eq!(b_incoming.len(), 1);

    let a_outgoing_edge = &a_outgoing[0];
    let b_incoming_edge = &b_incoming[0];

    assert_eq!(a_outgoing_edge.get_source(), a_id);
    assert_eq!(a_outgoing_edge.get_target(), b_id);

    assert_eq!(b_incoming_edge.get_source(), a_id);
    assert_eq!(b_incoming_edge.get_target(), b_id);

    // When traversing in both directions at the same time it can be difficult to keep track of which direction the given edge is going
    // So to make this easer the get_inner and get_outer method can be used
    let b_both: Vec<_> = gv0.get_incoming_and_outgoing(b_id)?.collect();

    assert_eq!(b_both.len(), 2);

    let edge0 = &b_both[0];
    let edge1 = &b_both[1];

    // Both edges startet from the same node
    assert_eq!(edge0.get_inner(), b_id);
    assert_eq!(edge1.get_inner(), b_id);

    // But get_outer will always take you away from the starting node
    assert_ne!(edge0.get_outer(), edge1.get_outer());

    // Using these short hands make traversal code work independant of direction
    // Here is an example of finding the longest path from a node in both directions
    fn longest_distance(
        weight_id: WeightId,
        g: &TypedGraph<WeightId, WeightId, JsonSchema>,
    ) -> Option<usize> {
        // Return None if the node does not exist
        g.get_node_safe(weight_id)?;

        let mut visited: HashMap<_, usize> = HashMap::new();
        let mut front = vec![(weight_id, 0)];
        while let Some((front_id, distance)) = front.pop() {
            if visited.contains_key(&front_id) {
                continue;
            }

            visited.insert(front_id, distance);

            // here we can focus on writing the implementation instead of having to bother with directions
            for edge in g.get_incoming_and_outgoing(front_id).unwrap() {
                front.push((edge.get_outer(), distance + 1));
            }
        }

        visited.values().max().copied()
    }
    println!(
        "Longest distance from {} is {:?}",
        b_id,
        longest_distance(b_id, &gv0)
    );

    // For deep searches of a specific depth the searches can be chained
    // Here we walk along the path A -> B -> C -> A
    // Do note that this will not return any branches that is shorter than the requested length
    let outer: Vec<_> = gv0
        .get_outgoing(a_id)?
        .filter_map(|id| gv0.get_outgoing(id.get_outer()).ok())
        .flatten()
        .filter_map(|id| gv0.get_outgoing(id.get_outer()).ok())
        .flatten()
        .filter_map(|id| gv0.get_outgoing(id.get_outer()).ok())
        .flatten()
        .collect();

    assert_eq!(outer.len(), 1);
    let outer_node = outer[0].get_id();
    assert_eq!(outer_node, a_id);

    // An alternative way of doing the same thing is using a GraphWalker

    /// Create a function to tell how to move forward
    /// Along with the next node it also returns the type of the edge that has been traversed
    fn move_forward<'a>(
        n: &'a Weight,
        gv0: &'a TypedGraph<u64, u64, JsonSchema>,
    ) -> SchemaResult<impl Iterator<Item = (String, &'a Weight)>, WeightId, WeightId, JsonSchema>
    {
        Ok(gv0.get_outgoing(n.get_id())?.map(|e| {
            (
                e.get_weight().get_type(),
                gv0.get_node(e.get_outer()).unwrap(),
            )
        }))
    }

    // Now the walker can use the function to traverse the graph
    let outer: Vec<_> = gv0
        .get_node(a_id)?
        .to_walker(&gv0)?
        .progress(move_forward)
        .progress(move_forward)
        .progress(move_forward)
        .many()?;

    assert_eq!(outer.len(), 1);
    let outer_node = outer[0].get_id();
    assert_eq!(outer_node, a_id);

    // The main benefit of using the walker is that once the move_forward has been made the syntax becomes esier
    // and it allows for a state to be keept for each of the branches

    /// Create a function that add the edge type of the visited edge to the state of the branch
    fn update_state(mut state: Vec<String>, addition: String) -> Vec<String> {
        state.push(addition);
        state
    }

    let outer: Vec<_> = gv0
        .get_node(a_id)?
        .to_walker(&gv0)?
        // First we set the initial state
        .set_state(Vec::<String>::new())
        // When moving forward we then update the state
        .progress_with_state(move_forward, update_state)
        .progress_with_state(move_forward, update_state)
        .progress_with_state(move_forward, update_state)
        .many_with_state()?;

    assert_eq!(outer.len(), 1);
    let walker_target = &outer[0];
    assert_eq!(walker_target.val.get_id(), a_id);

    // Finally we can see the resulting state of the branch
    assert_eq!(walker_target.state, vec!["AB", "BC", "CA"]);

    // Now we can try and migrate the graph to a new schema without the B node
    let schemav1 = JsonSchema {
        version: "V0".to_string(),
        nodes: vec!["A".to_string(), "C".to_string()],
        edges: vec![
            ("CA".to_string(), "C".to_string(), "A".to_string()),
            ("CC".to_string(), "C".to_string(), "C".to_string()),
        ],
    };

    let gv1 = gv0.migrate(schemav1, &DefaultMigrationHandler)?;
    // We then see that the B nodes are gone aswell as their edges
    assert_eq!(gv1.has_node(b_id), false);
    assert_eq!(gv1.has_edge(ab_id), false);
    assert_eq!(gv1.has_edge(bc_id), false);
    // So all we are left with is the A and C nodes
    assert_eq!(gv1.has_node(a_id), true);
    assert_eq!(gv1.has_node(c_id), true);
    assert_eq!(gv1.has_edge(ca_id), true);

    Ok(())
}

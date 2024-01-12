///! In this example we show a simple template for creating a staticly typed schemas
///! 
///! Everything is handled at compile time meaning the schema does not store any data
///! This makes it very safe to use as we are guaranteed to know which types of edges and nodes will be in the graph
///!
///! This example creates make_node_structand make_edge_struct which autogenerating NodeType, EdgeType, Node, Edge and a type for every kind of node and edge
///! 
///! The Node and Edge are then used by the schema to define a set of rules for the graph
///!
///! The schema creates the graph
///! ```
///!        AC
///!  ---------------
///!  v             |
///!  A ---> B ---> C
///!     AB    BC
///! ```

use std::fmt::Display;

use typed_graph::{Key, NodeExt, Id, Typed, EdgeExt, SchemaExt, DisAllowedEdge, TypedGraph, SchemaResult, DowncastMut, Downcast, TypedError, ToGraphWalker};

/// Create NodeType, Node and a type for each NodeType
macro_rules! make_node_struct {
    ($($name:ident $({$($field_name:ident : $field_type:ty),*})?),*) => {
        // Create a type for referencing nodes
        #[derive(PartialEq, Clone, Copy, Debug)]
        pub enum NodeType {
            $(
                $name
            ),*
        }

        impl Display for NodeType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        
        // Create the individual nodes
        $(
            #[derive(Debug, Clone)]
            pub struct $name<K: Key> {
                id: K,
                $(
                    $(
                        pub $field_name: $field_type
                    ),*
                )?
            }

            impl<K: Key> $name<K> {
                fn new(
                    id: K,
                    $(
                        $(
                            $field_name: $field_type
                        ),*
                    )?
                ) -> Self {
                    Self {
                        id,
                        $(
                            $(
                                $field_name
                            ),*
                        )?
                    }
                }
            }

            impl<K: Key> Id<K> for $name<K> {
                fn get_id(&self) -> K {
                    self.id
                }
            
                fn set_id(&mut self, new_id: K) {
                    self.id = new_id
                }
            }
            
            impl<K: Key> Typed for $name<K> {
                type Type = NodeType;
            
                fn get_type(&self) -> Self::Type {
                    NodeType::$name
                }
            }
            
            impl<K: Key> PartialEq<NodeType> for $name<K> {
                fn eq(&self, other: &NodeType) -> bool {
                    other == &NodeType::$name
                }
            }

            impl<K: Key> From<$name<K>> for Node<K> {
                fn from(a: $name<K>) -> Self {
                    Node::$name(a)
                }
            }

            impl<NK, EK, S> DowncastMut<NK, EK, $name<EK>, S> for Node<EK>
            where
                NK: Key,
                EK: Key,
                S: SchemaExt<NK, EK, N = Node<EK>>
            {
                fn downcast_mut(&mut self) -> SchemaResult<&mut $name<EK>, NK, EK, S> {
                    match self {
                        Node::$name(e) => Ok(e),
                        e => Err(TypedError::DownCastFailed(
                            stringify!($name).to_string(),
                            e.get_type().to_string())
                        )
                    }
                }
            }

            impl<NK, EK, S> Downcast<NK, EK, $name<EK>, S> for Node<EK>
            where
                NK: Key,
                EK: Key,
                S: SchemaExt<NK, EK, N = Node<EK>>
            {
                fn downcast(&self) -> SchemaResult<&$name<EK>, NK, EK, S> {
                    match self {
                        Node::$name(e) => Ok(e),
                        e => Err(TypedError::DownCastFailed(
                            stringify!($name).to_string(),
                            e.get_type().to_string())
                        )
                    }
                }
            }
            
            impl<K: Key> NodeExt<K> for $name<K> {}
        )*

        // Create a contianer for all the nodes
        #[derive(Debug, Clone)]
        pub enum Node<K: Key> {
            $(
                $name($name<K>)
            ),*
        }

        impl<K: Key> Id<K> for Node<K> {
            fn get_id(&self) -> K {
                match self {
                    $(
                        Node::$name(v) => v.get_id()
                    ),*
                }
            }

            fn set_id(&mut self, new_id: K) {
                match self {
                    $(
                        Node::$name(v) => v.set_id(new_id)
                    ),*
                }
            }
        }
        
        impl<K: Key> Typed for Node<K> {
            type Type = NodeType;
        
            fn get_type(&self) -> Self::Type {
                match self {
                    $(
                        Node::$name(v) => v.get_type()
                    ),*
                }
            }
        }
        
        impl<K: Key> PartialEq<NodeType> for Node<K> {
            fn eq(&self, other: &NodeType) -> bool {
                match self {
                    $(
                        Node::$name(v) => v.eq(other)
                    ),*
                }
            }
        }

        impl<K: Key> NodeExt<K> for Node<K> {}
    };
}

/// Create EdgeType, Edge and a type for each EdgeType
macro_rules! make_edge_struct {
    ($($name:ident $({$($field_name:ident : $field_type:ty),*})?),*) => {
        // Create a type for referencing edges
        #[derive(PartialEq, Clone, Copy, Debug)]
        pub enum EdgeType {
            $(
                $name
            ),*
        }

        impl Display for EdgeType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        
        // Create the individual edges
        $(
            #[derive(Debug, Clone)]
            pub struct $name<K: Key> {
                id: K,
                $(
                    $(
                        pub $field_name: $field_type
                    ),*
                )?
            }

            impl<K: Key> $name<K> {
                fn new(
                    id: K,
                    $(
                        $(
                            $field_name: $field_type
                        ),*
                    )?
                ) -> Self {
                    Self {
                        id,
                        $(
                            $(
                                $field_name
                            ),*
                        )?
                    }
                }
            }

            impl<K: Key> Id<K> for $name<K> {
                fn get_id(&self) -> K {
                    self.id
                }

                fn set_id(&mut self, new_id: K) {
                    self.id = new_id;
                }
            }

            impl<K: Key> Typed for $name<K> {
                type Type = EdgeType;

                fn get_type(&self) -> Self::Type {
                    EdgeType::$name
                }
            }

            impl<K: Key> PartialEq<EdgeType> for $name<K> {
                fn eq(&self, other: &EdgeType) -> bool {
                    other == &EdgeType::$name
                }
            }

            impl<K: Key> From<$name<K>> for Edge<K> {
                fn from(a: $name<K>) -> Self {
                    Edge::$name(a)
                }
            }

            impl<NK, EK, S> DowncastMut<NK, EK, $name<EK>, S> for Edge<EK>
            where
                NK: Key,
                EK: Key,
                S: SchemaExt<NK, EK, E = Edge<EK>>
            {
                fn downcast_mut(&mut self) -> SchemaResult<&mut $name<EK>, NK, EK, S> {
                    match self {
                        Edge::$name(e) => Ok(e),
                        e => Err(TypedError::DownCastFailed(
                            stringify!($name).to_string(), 
                            e.get_type().to_string())
                        )
                    }
                }
            }

            impl<NK, EK, S> Downcast<NK, EK, $name<EK>, S> for Edge<EK>
            where
                NK: Key,
                EK: Key,
                S: SchemaExt<NK, EK, E = Edge<EK>>
            {
                fn downcast(&self) -> SchemaResult<&$name<EK>, NK, EK, S> {
                    match self {
                        Edge::$name(e) => Ok(e),
                        e => Err(TypedError::DownCastFailed(
                            stringify!($name).to_string(),
                            e.get_type().to_string())
                        )
                    }
                }
            }

            impl<K: Key> EdgeExt<K> for $name<K> {}
        )*

        // Create a container for all the edges
        #[derive(Debug, Clone)]
        pub enum Edge<K: Key> {
            $(
                $name($name<K>)
            ),*
        }

        impl<K: Key> Id<K> for Edge<K> {
            fn get_id(&self) -> K {
                match self {
                    $(
                        Edge::$name(v) => v.get_id()
                    ),*
                }
            }

            fn set_id(&mut self, new_id: K) {
                match self {
                    $(
                        Edge::$name(v) => v.set_id(new_id)
                    ),*
                }
            }
        }
        
        impl<K: Key> Typed for Edge<K> {
            type Type = EdgeType;
        
            fn get_type(&self) -> Self::Type {
                match self {
                    $(
                        Edge::$name(v) => v.get_type()
                    ),*
                }
            }
        }
        
        impl<K: Key> PartialEq<EdgeType> for Edge<K> {
            fn eq(&self, other: &EdgeType) -> bool {
                match self {
                    $(
                        Edge::$name(v) => v.eq(other)
                    ),*
                }
            }
        }

        impl<K: Key> EdgeExt<K> for Edge<K> {}
    };
}

make_node_struct!{
    A {
        name: String
    },
    B {
        name: String
    },
    C {
        name: String
    }
}

make_edge_struct!{
    AB {
        distance: usize
    },
    BC {
        distance: usize
    },
    CA {
        distance: usize
    }
}

#[derive(Default)]
pub struct Schema {}

impl<NK: Key, EK: Key> SchemaExt<NK, EK> for Schema {
    type N = Node<NK>;
    type E = Edge<EK>;

    fn name(&self) -> String {
        "Schema".to_string()
    }

    fn allow_node(
            &self, 
            _node_ty: NodeType
        ) -> Result<(), typed_graph::DisAllowedNode> {
        Ok(())
    }

    fn allow_edge(
            &self, 
            _new_edge_count: usize,
            edge_ty: EdgeType, 
            source: NodeType, 
            target: NodeType,
        ) -> Result<(), typed_graph::DisAllowedEdge> {
        match (source, target, edge_ty) {
            (NodeType::A, NodeType::B, EdgeType::AB)
            | (NodeType::B, NodeType::C, EdgeType::BC)
            | (NodeType::C, NodeType::A, EdgeType::CA) => Ok(()),
            _ => Err(DisAllowedEdge::InvalidType)
        }
    }
}

type Graph<NK, EK> = TypedGraph<NK, EK, Schema>;

fn main() -> SchemaResult<(), usize, usize, Schema> {
    // Using the generics, we can create graphs of different types
    let _g0: Graph<u32, u32> = Graph::default();
    let _g1: Graph<i32, i32> = Graph::default();
    // We can even use different keys for nodes and edges
    let _g2: Graph<i64, u32> = Graph::default();

    // For this example we will be working with double usize
    let mut g: Graph<usize, usize> = Graph::default();

    let a_id = g.add_node(A::new(0, "Stop A".to_string()))?;
    let b_id = g.add_node(B::new(1, "Stop B".to_string()))?;
    let c_id = g.add_node(C::new(2, "Stop C".to_string()))?;

    let ab_id = g.add_edge(a_id, b_id, AB::new(0, 10))?;
    let bc_id = g.add_edge(b_id, c_id, BC::new(1, 5))?;
    let ca_id = g.add_edge(c_id, a_id, CA::new(2, 1))?;

    // We cannot create an instance of AB between C -> A since the schema only allows for AB edges to be between A -> B
    let e = g.add_edge(c_id, a_id, AB::new(0, 3));
    assert!(e.is_err());

    // If we want to retrieve data from the graph
    // We can get the generic node
    let node = g.get_node(a_id)?;
    
    // And then make requests on that
    println!("Node id = {} type = {}", node.get_id(), node.get_type());

    // However if we want the specific type we have to do a match on it
    // But since we have implemented Downcast we can also use the shorthand
    let a: &A<_> = g.get_node_downcast(a_id)?;
    let b: &B<_> = g.get_node_downcast(b_id)?;
    let c: &C<_> = g.get_node_downcast(c_id)?;

    println!("A name = {}", a.name);
    println!("B name = {}", b.name);
    println!("C name = {}", c.name);

    // This will fail if we try to cast it to the wrong type
    let e = g.get_node_downcast::<B<_>>(a_id);
    assert!(e.is_err());

    // But it makes it significantly easier to retrieve a specific type
    let ab: &AB<_> = g.get_edge_downcast(ab_id)?;
    let bc: &BC<_> = g.get_edge_downcast(bc_id)?;
    let ca: &CA<_> = g.get_edge_downcast(ca_id)?;

    println!("AB distance = {}", ab.distance);
    println!("BC distance = {}", bc.distance);
    println!("CA distance = {}", ca.distance);

    // We can now traverse the graph to calculate the total distance between all the nodes
    
    /// Function to retrieve the next node in the chain
    /// The generic allows us to specify the type of the edge we will encounter
    /// Since we have defined the schema we also know which type of node will be encountered
    /// 
    /// For larer projects, these might be defined per node/edge
    fn get_connected_node<'a, E>(node: &'a Node<usize>, g: &'a TypedGraph<usize, usize, Schema>) -> SchemaResult<impl Iterator<Item = (&'a E, &'a Node<usize>)>, usize, usize, Schema> 
    where
        <Schema as SchemaExt<usize, usize>>::E: Downcast<usize, usize, E, Schema>
    {
        Ok(g
            .get_outgoing(node.get_id())?
            .map(|e| (
                e.get_weight_downcast::<E>().unwrap(), 
                g.get_node(e.get_target()).unwrap()
            )))
    }

    // traverse the graph using a GraphWalker
    let distance = node
        .to_walker(&g)?
        .set_state(0)
        .progress_with_state(
            get_connected_node::<AB<_>>, 
            // Boiler plate code for incrementing the distance
            |mut acc, nc| {acc += nc.distance; acc }
        )
        .progress_with_state(
            get_connected_node::<AB<_>>, 
            |mut acc, nc| {acc += nc.distance; acc }
        )
        .progress_with_state(
            get_connected_node::<AB<_>>, 
            |mut acc, nc| {acc += nc.distance; acc }
        )
        .one_with_state()?
        .unwrap()
        .state;

    println!("ABCA distance = {}", distance);

    Ok(())
}
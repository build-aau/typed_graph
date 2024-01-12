from typed_graph import TypedGraph, NodeExt, EdgeExt, SchemaExt, StrEnum, TypeStatus, RustModel
from typing import ClassVar, Any

class NodeType(StrEnum):
    """
    Store a list of all possible types of nodes

    Each type is backed by a unique string
    """
    A = 'A'
    B = 'B'
    C = 'C'

class EdgeType(StrEnum):
    """
    Store a list of all possible types of edges

    Each type is backed by a unique string
    """
    AB = 'AB'
    BC = 'BC'
    CA = 'CA'

class Node(NodeExt[int, NodeType]):
    """
    Node is an abstract class shared by all node types

    It implements get_id and set_id which is pretty similar for all types of nodes
    """
    def get_id(self) -> int:
        return self.id
    
    def set_id(self, id: int) -> None:
        self.id = id

class A(Node):
    """
    Implement a specific type of node

    This has two fields id and name
    """
    id: int
    name: str

    def get_type(self) -> NodeType:
        return NodeType.A
    
class B(Node):
    id: int
    name: str

    def get_type(self) -> NodeType:
        return NodeType.B
    
class C(Node):
    id: int
    name: str

    def get_type(self) -> NodeType:
        return NodeType.C
    
class Edge(EdgeExt[int, EdgeType]):
    """
    Edge is an abstract class shared by all edge types

    It implements get_id and set_id which is pretty similar for all types of edges
    """
    def get_id(self) -> int:
        return self.id
    
    def set_id(self, id: int) -> None:
        self.id = id

class AB(Edge):
    """
    Implement a spefic edge

    The edge has two fields id and distance
    """
    id: int
    distance: int

    def get_type(self) -> EdgeType:
        return EdgeType.AB
    
class BC(Edge):
    id: int
    distance: int

    def get_type(self) -> EdgeType:
        return EdgeType.BC
    
class CA(Edge):
    id: int
    distance: int

    def get_type(self) -> EdgeType:
        return EdgeType.CA
    
class Schema(SchemaExt[Node, Edge, int, int, NodeType, EdgeType]):
    """
    Now we define the schema

    Since all types are predefined, the schema does not need to store any data

    Instead the schema just relies on static data to enforce the schema
    """
    allowed_endpoint: ClassVar[Any] = [
        (EdgeType.AB, NodeType.A, NodeType.B),
        (EdgeType.BC, NodeType.B, NodeType.C),
        (EdgeType.CA, NodeType.C, NodeType.A),
    ]

    def name(self) -> str:
        return 'Schema'
    
    def allow_node(self, node_type: NodeType) -> TypeStatus | bool:
        """
        Check that the node type is actual an instance and that it is one of the existing varients
        """
        # If we returned a TypeStatus the error would be nicer, but this still works
        return isinstance(node_type, NodeType) and node_type in NodeType.__members__.keys()
    
    def allow_edge(self, quantity: int, edge_type: EdgeType, source_type: NodeType, target_type: NodeType) -> TypeStatus | bool:
        """
        Check that all the types are actual instances and the the endpoint is one of the allowed ones
        """
        source_allowed = isinstance(source_type, NodeType) and source_type in NodeType.__members__.keys()
        target_allowed = isinstance(target_type, NodeType) and target_type in NodeType.__members__.keys()
        edge_allowed = isinstance(edge_type, EdgeType) and edge_type in EdgeType.__members__.keys()
        endpoint_allowed = (edge_type, source_type, target_type) in Schema.allowed_endpoint

        # If we returned a TypeStatus the error would be nicer, but this still works
        return source_allowed and target_allowed and edge_allowed and endpoint_allowed
    
Graph = TypedGraph[Node, Edge, int, int, NodeType, EdgeType, Schema]

if __name__ == '__main__':
    s = Schema()
    g = Graph(s)

    a_id = g.add_node(A(id=0, name="Stop A"))
    b_id = g.add_node(B(id=1, name="Stop B"))
    c_id = g.add_node(C(id=2, name="Stop C"))

    ab_id = g.add_edge(a_id, b_id, AB(id=0, distance=10))
    bc_id = g.add_edge(b_id, c_id, BC(id=1, distance=5))
    ca_id = g.add_edge(c_id, a_id, CA(id=2, distance=1))

    # We cannot create an instance of AB between C -> A since the schema only allows for AB edges to be between A -> B
    try: 
        g.add_edge(c_id, a_id, AB(0, 3))
    except Exception as e:
        print(e)

    # If we want to retrieve data from the graph
    # We can treat the node as the generic one
    node = g.get_node(a_id)
    
    # And make requests on that
    node_id = node.get_id()
    node_type = node.get_type()
    print(f"Node id = {node_id} type = {node_type}")

    # However we can also just guess the type
    a: A = g.get_node(a_id)
    b: B = g.get_node(b_id)
    c: C = g.get_node(c_id)

    print('All nodes')
    print(f"{type(a)} name = {a.name}")
    print(f"{type(b)} name = {b.name}")
    print(f"{type(c)} name = {c.name}")

    # And ofcause the same also applies to the edges
    ab: AB = g.get_edge(ab_id)
    bc: BC = g.get_edge(bc_id)
    ca: CA = g.get_edge(ca_id)

    print('All distances')
    print(f"{type(ab)} distance = {ab.distance}")
    print(f"{type(bc)} distance = {bc.distance}")
    print(f"{type(ca)} distance = {ca.distance}")

    # We can also serialize the graph
    g_json = g.model_dump_json()
    
    print('Graph as json')
    print(g_json)

    # And then deserialize the graph 
    gg = Graph.model_validate_json(g_json)

    # All while maintaining the type information
    a = gg.get_node(a_id)
    b = gg.get_node(b_id)
    c = gg.get_node(c_id)

    print('Types after reloading')
    print(type(a), type(b), type(c))

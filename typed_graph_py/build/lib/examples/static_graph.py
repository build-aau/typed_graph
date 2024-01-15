from typed_graph import TypedGraph, NodeExt, EdgeExt, SchemaExt, StrEnum, TypeStatus, RustModel
from typing import ClassVar, Any

class StaticNodeType(StrEnum):
    """
    Store a list of all possible types of nodes

    Each type is backed by a unique string
    """
    A = 'A'
    B = 'B'
    C = 'C'

class StaticEdgeType(StrEnum):
    """
    Store a list of all possible types of edges

    Each type is backed by a unique string
    """
    AB = 'AB'
    BC = 'BC'
    CA = 'CA'

class StaticNode(NodeExt[int, StaticNodeType]):
    """
    Node is an abstract class shared by all node types

    It implements get_id and set_id which is pretty similar for all types of nodes
    """
    def get_id(self) -> int:
        return self.id
    
    def set_id(self, id: int) -> None:
        self.id = id

class A(StaticNode):
    """
    Implement a specific type of node

    This has two fields id and name
    """
    id: int
    name: str

    def get_type(self) -> StaticNodeType:
        return StaticNodeType.A
    
class B(StaticNode):
    id: int
    name: str

    def get_type(self) -> StaticNodeType:
        return StaticNodeType.B
    
class C(StaticNode):
    id: int
    name: str

    def get_type(self) -> StaticNodeType:
        return StaticNodeType.C
    
class StaticEdge(EdgeExt[int, StaticEdgeType]):
    """
    Edge is an abstract class shared by all edge types

    It implements get_id and set_id which is pretty similar for all types of edges
    """
    def get_id(self) -> int:
        return self.id
    
    def set_id(self, id: int) -> None:
        self.id = id

class AB(StaticEdge):
    """
    Implement a spefic edge

    The edge has two fields id and distance
    """
    id: int
    distance: int

    def get_type(self) -> StaticEdgeType:
        return StaticEdgeType.AB
    
class BC(StaticEdge):
    id: int
    distance: int

    def get_type(self) -> StaticEdgeType:
        return StaticEdgeType.BC
    
class CA(StaticEdge):
    id: int
    distance: int

    def get_type(self) -> StaticEdgeType:
        return StaticEdgeType.CA
    
class StaticSchema(SchemaExt[StaticNode, StaticEdge, int, int, StaticNodeType, StaticEdgeType]):
    """
    Now we define the schema

    Since all types are predefined, the schema does not need to store any data

    Instead the schema just relies on static data to enforce the schema
    """
    allowed_endpoint: ClassVar[Any] = [
        (StaticEdgeType.AB, StaticNodeType.A, StaticNodeType.B),
        (StaticEdgeType.BC, StaticNodeType.B, StaticNodeType.C),
        (StaticEdgeType.CA, StaticNodeType.C, StaticNodeType.A),
    ]

    def name(self) -> str:
        return 'Schema'
    
    def allow_node(self, node_type: StaticNodeType) -> TypeStatus | bool:
        """
        Check that the node type is actual an instance and that it is one of the existing varients
        """
        # If we returned a TypeStatus the error would be nicer, but this still works
        return isinstance(node_type, StaticNodeType) and node_type in StaticNodeType.__members__.keys()
    
    def allow_edge(self, quantity: int, edge_type: StaticEdgeType, source_type: StaticNodeType, target_type: StaticNodeType) -> TypeStatus | bool:
        """
        Check that all the types are actual instances and the the endpoint is one of the allowed ones
        """
        source_allowed = isinstance(source_type, StaticNodeType) and source_type in StaticNodeType.__members__.keys()
        target_allowed = isinstance(target_type, StaticNodeType) and target_type in StaticNodeType.__members__.keys()
        edge_allowed = isinstance(edge_type, StaticEdgeType) and edge_type in StaticEdgeType.__members__.keys()
        endpoint_allowed = (edge_type, source_type, target_type) in StaticSchema.allowed_endpoint

        # If we returned a TypeStatus the error would be nicer, but this still works
        return source_allowed and target_allowed and edge_allowed and endpoint_allowed
    
StaticGraph = TypedGraph[StaticNode, StaticEdge, int, int, StaticNodeType, StaticEdgeType, StaticSchema]

if __name__ == '__main__':
    s = StaticSchema()
    g = StaticGraph(s)

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
    gg = StaticGraph.model_validate_json(g_json)

    # All while maintaining the type information
    a = gg.get_node(a_id)
    b = gg.get_node(b_id)
    c = gg.get_node(c_id)

    print('Types after reloading')
    print(type(a), type(b), type(c))

from typed_graph import TypedGraph, NodeExt, EdgeExt, SchemaExt, TypeStatus
from typing import List, Tuple, Dict, Any, ClassVar
from pydantic import RootModel

class JsonWeight(RootModel, NodeExt[int, str], EdgeExt[int, str]):   
    """
    Define a weight in the json graph

    The weight is just a light wrapper around a dictionary that can be easily be exported to json
    """
    tagging: ClassVar[bool] = False

    root: Dict[str, Any]

    def __init__(self, **content: Any):
        """
        Add the content of the dictionary to self

        This also creates attributes on the class
        """
        super().__init__(content)
        for k in content.keys():
            self._register_attribute(k)
            
    def _register_attribute(self, k):
        """
        Register the key as accessible in the class

        The field is stored as a property that is backed by the json container
        """
        def setter(self, value):
            self.root[k] = value
        
        def getter(self):
            return self.root[k]
        
        def deleter(self):
            del self.root[k]
    
        setattr(self.__class__, k, property(getter, setter, deleter))

    def __setitem__(self, key, value):
        """
        Set an item in the json container
        """
        self.root[key] = value

        # If the item is new, then a field is also added to the class
        if key not in self.root:
            self._register_attribute(key)

    def __getitem__(self, key):
        return self.root[key]
    
    def get_id(self) -> int:
        return self.id
    
    def set_id(self, id: int) -> None:
        self.id = id
    
    def get_type(self) -> str:
        return self.type

class JsonSchema(SchemaExt[JsonWeight, JsonWeight, int, int, str, str]):
    version: str
    nodes: List[str]
    edges: List[Tuple[str, str, str]]

    def name(self) -> str:
        return self.name
    
    def allow_node(self, node_type: str) -> TypeStatus:
        if node_type in self.nodes:
            return TypeStatus.Ok
        else:
            return TypeStatus.InvalidType
    
    def allow_edge(self, _quantity: int, edge_type: str, source_type: str, target_type: str) -> TypeStatus:
        if (edge_type, source_type, target_type) in self.edges:
            return TypeStatus.Ok
        else:
            return TypeStatus.InvalidType

JsonGraph = TypedGraph[JsonWeight, JsonWeight, int, int, str, str, JsonSchema]

if __name__ == '__main__':
    s = JsonSchema(
        version = 'V0',
        edges = [
            ('AB', 'A', 'B'),
            ('BC', 'B', 'C'),
            ('CA', 'C', 'A'),
            ('CC', 'C', 'C'),
        ],
        nodes = [
            'A', 'B', 'C'
        ]   
    )


    g = JsonGraph(s)
    # Populate the graph
    a_id = g.add_node(JsonWeight(**{'id': 0, 'type': 'A'}))
    b_id = g.add_node(JsonWeight(**{'id': 1, 'type': 'B'}))
    c_id = g.add_node(JsonWeight(**{'id': 2, 'type': 'C'}))
    ab_id = g.add_edge(a_id, b_id, JsonWeight(**{'id': 0, 'type': 'AB'}))
    bc_id = g.add_edge(b_id, c_id, JsonWeight(**{'id': 1, 'type': 'BC'}))
    ca_id = g.add_edge(c_id, a_id, JsonWeight(**{'id': 2, 'type': 'CA'}))
    cc_id = g.add_edge(c_id, c_id, JsonWeight(**{'id': 3, 'type': 'CC'}))


    # trying to add a type that is not part of the schema will result in  an exception
    try:
        g.add_node(JsonWeight(**{'id': 4, 'type': 'D'}))
    except Exception as e:
        print(e)

    # The same thing happens when trying to add an edge with a type that is not allowed
    try:
        g.add_edge(c_id, a_id, JsonWeight(**{'id': 4, 'type': 'DD'}))
    except Exception as e:
        print(e)



    # Calling add on an id that is already used will update the type of the node or edge at that position
    # This only works if the replaced type is compatible with all the connected nodes and edges



    # We are also able to add multiple edges between the same nodes
    dublicate_edge_id = g.add_edge(a_id, b_id, JsonWeight(**{"id": 3, "type": "AB"}))
    g.remove_edge(dublicate_edge_id)

    # Loops are also allowed as long as the are part of the schema
    dublicate_edge_id = g.add_edge(c_id, c_id, JsonWeight(**{"id": 3, "type": "CC"}))
    g.remove_edge(dublicate_edge_id)


    # if we remove a node all its surrounding edges will be removed aswell
    a = g.remove_node(a_id)

    print('Is removed node there:', g.has_node(a_id), g.has_edge(ab_id), g.has_edge(ca_id))
    g.add_node(a)
    g.add_edge(a_id, b_id, JsonWeight(**{"id": 0, "type": "AB"}))
    g.add_edge(c_id, a_id, JsonWeight(**{"id": 2, "type": "CA"}))


    # Traversal of the graph is done using the get_outgoing, get_incoming and get_incoming_and_outgoing functions
    a_outgoing = list(g.get_outgoing(a_id))
    b_incoming = list(g.get_incoming(b_id))

    print('A outgoing:', len(a_outgoing), ' B incoming:', len(b_incoming))

    a_outgoing_edge = a_outgoing[0]
    b_incoming_edge = b_incoming[0]

    print('A outgoing source =', a_outgoing_edge.source, ' target =', a_outgoing_edge.target)
    print('A incoming source =', b_incoming_edge.source, ' target =', b_incoming_edge.target)


    # When traversing in both directions at the same time it can be difficult to keep track of which direction the given edge is going
    # So to make this easer the get_inner and get_outer method can be used
    b_both = list(g.get_incoming_and_outgoing(b_id))
    print('B outgoing incoming:', len(b_both))
    
    edge0 = b_both[0]
    edge1 = b_both[1]

    # Both edges startet from the same node
    print('B outgoing incoming 0.inner = ', edge0.get_inner(), '1.inner = ', edge1.get_inner())

    # But get_outer will always take you away from the starting node
    print('B outgoing incoming 0.outer = ', edge0.get_outer(), '1.outer = ', edge1.get_outer())


    # Using these short hands make traversal code work independant of direction
    # Here is an example of finding the longest path from a node in both directions
    def longest_distance(weight_id: int, g: JsonGraph) -> int | None:
        # Return None if the node does not exist
        node = g.get_node_safe(weight_id)
        if not node:
            return None

        visited = dict()
        front = [(weight_id, 0)]

        while front:
            front_id, distance = front.pop()

            if front_id in visited:
                continue
    
            visited[front_id] = distance
    
            # here we can focus on writing the implementation instead of having to bother with directions
            for edge in g.get_incoming_and_outgoing(front_id):
                front.append((edge.get_outer(), distance + 1))
    
        return max(visited.values())
    
    distance = longest_distance(b_id, g)
    print(f"Longest distance from {b_id} is {distance}")

    g_json = g.model_dump_json()
    print('Graph as json')
    print(g_json)

    ng = JsonGraph.model_validate_json(g_json)
    print('Graph after load')
    ng_json = ng.model_dump_json()
    print(ng_json)
    

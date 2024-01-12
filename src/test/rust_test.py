from typed_graph import *
import json
from typing import List, Dict, Optional, Tuple, ClassVar, Callable, Any
from pydantic import TypeAdapter, PrivateAttr, model_serializer, model_validator, RootModel

class Action(RustModel):
    pass

class AddNode(Action):
    id: int
    ty: int

class AddEdge(Action):
    id: int
    ty: int
    source: int
    target: int

class RemoveNode(Action):
    id: int

class RemoveEdge(Action):
    id: int

class Weight(RootModel[Tuple[int, int]], NodeExt[int, str], EdgeExt[int, str]):
    tagging: ClassVar[bool] = False

    _id: int = PrivateAttr
    _type: int = PrivateAttr

    def __init__(self, t: Tuple[int, int]):
        super().__init__(t)
        self._id = t[0]
        self._type = t[1]

    def get_id(self) -> int:
        return self._id
    
    def set_id(self, id: int) -> None:
        self._id = id
        self[0] = id

    def get_type(self) -> str:
        return self._type

class Schema(SchemaExt[Weight, Weight, int, int, str, str]):
    tagging: ClassVar[bool] = False

    node_whitelist: Optional[List[int]] = None
    node_blacklist: Optional[List[int]] = None
    edge_whitelist: Optional[List[int]] = None
    edge_blacklist: Optional[List[int]] = None
    edge_endpoint_whitelist: Optional[List[Tuple[int, int, int]]] = None
    edge_endpoint_blacklist: Optional[List[Tuple[int, int, int]]] = None
    edge_endpoint_max_quantity: Optional[Dict[Tuple[int, int, int], int]] = None

    def name(self) -> str:
        return 'TestSchema'
    
    def allow_edge(self, quantity: int, edge_type: str, source_type: str, target_type: str) -> TypeStatus:
        is_whitelist = not self.edge_whitelist or edge_type in self.edge_whitelist
        is_blacklist = not self.edge_blacklist or not edge_type in self.edge_blacklist
        is_endpoint_whitelist = not self.edge_endpoint_whitelist or (edge_type, source_type, target_type) in self.edge_endpoint_whitelist
        is_endpoint_blacklist = not self.edge_endpoint_blacklist or not (edge_type, source_type, target_type) in self.edge_endpoint_blacklist
        is_allowed_type = is_whitelist and is_blacklist and is_endpoint_whitelist and is_endpoint_blacklist

        if not is_allowed_type:
            return TypeStatus.InvalidType

        return TypeStatus.Ok
    
    def allow_node(self, node_type: str) -> TypeStatus:
        is_whitelist = not self.node_whitelist or node_type in self.node_whitelist
        is_blacklist = not self.node_blacklist or not node_type in self.node_blacklist
        is_allowed = is_whitelist and is_blacklist

        if not is_allowed:
            return TypeStatus.InvalidType
        
        return TypeStatus.Ok

Graph = TypedGraph[Weight, Weight, int, int, int, int, Schema]

def run(schema_json: str, json_actions: str) -> str:
    """
    args: list of Actions to perform as a json string
    """

    g = Graph(schema_json)
    
    actions = TypeAdapter(List[Action]).validate_json(json_actions)

    for i, action in enumerate(actions):
        action_name = action.__class__.__name__
        if action_name == 'AddNode':
            g.add_node(Weight((action.id, action.ty)))
        elif action_name == 'AddEdge':
            g.add_edge(action.source, action.target, Weight((action.id, action.ty)))
        elif action_name == 'RemoveNode':
            g.remove_node(action.id)
        elif action_name == 'RemoveEdge':
            g.remove_edge(action.id)
        else:
            # Do not that type allocation relies on __subclasses__ which may be out of sync
            raise Exception(f'Failed deserialize Action {i}. Action came with name {repr(action_name)} which did not match any actions')

    return g.model_dump_json()

if __name__ == '__main__':
    run(
'{"node_whitelist":[0,1,2,3,4,5],"node_blacklist":null,"edge_whitelist":null,"edge_blacklist":null,"edge_endpoint_whitelist":[[0,1,0],[2,4,5],[0,5,5],[7,1,3],[3,3,5],[2,2,1],[2,2,0],[0,1,3],[2,4,0],[5,4,1],[3,1,2],[1,1,4],[1,3,1],[5,1,1],[6,3,2],[6,1,4],[7,1,1],[0,5,4],[1,5,2],[0,0,4],[6,2,1],[6,4,1],[7,4,5],[7,0,4],[7,2,4],[1,3,2],[6,1,2],[2,5,4],[3,5,2],[1,4,1],[2,0,4],[0,2,4],[2,0,5],[2,4,3],[2,5,3],[3,0,4],[3,0,5],[5,2,5],[3,3,3],[6,4,4],[6,2,5],[0,5,3],[6,2,4],[0,4,4],[2,3,1],[0,1,4],[2,1,5],[3,2,3],[1,5,0],[5,5,4],[2,4,4],[4,3,2],[6,2,0],[6,1,3],[2,2,5],[1,1,0],[2,3,4],[5,2,3],[5,5,2],[3,1,0],[6,5,4],[1,3,3],[1,1,1],[5,2,1],[6,4,0],[6,0,2],[6,5,3],[2,2,4],[6,4,5],[2,5,2],[5,4,3],[6,3,4],[7,4,4],[0,0,3],[7,5,1],[5,5,1],[4,2,1],[7,0,2],[6,2,3],[7,3,0],[6,0,3],[7,3,4],[7,4,0],[0,1,2],[3,5,5],[2,2,2],[5,0,5],[3,2,5],[7,0,0],[7,5,0],[7,0,3],[7,0,5],[7,1,2],[7,2,3],[7,2,0],[7,4,1],[7,1,0],[1,4,3],[3,0,2],[3,5,3],[6,3,1],[7,5,5],[6,0,4],[2,3,0],[2,3,5],[1,2,3]],"edge_endpoint_blacklist":null,"edge_endpoint_max_quantity":null}',
'[{"AddNode":{"id":0,"ty":0}},{"AddNode":{"id":1,"ty":3}},{"AddEdge":{"id":0,"ty":7,"source":1,"target":0}},{"AddEdge":{"id":1,"ty":7,"source":0,"target":1}},{"AddNode":{"id":2,"ty":2}},{"RemoveNode":{"id":1}},{"AddEdge":{"id":0,"ty":3,"source":0,"target":2}},{"RemoveEdge":{"id":0}},{"RemoveNode":{"id":0}},{"AddEdge":{"id":0,"ty":2,"source":2,"target":2}},{"AddEdge":{"id":1,"ty":2,"source":2,"target":2}},{"RemoveEdge":{"id":0}}]'
    )
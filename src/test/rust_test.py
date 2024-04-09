from typed_graph import *
import json
from typing import List, Dict, Optional, Tuple, ClassVar, Callable, Any
from pydantic import TypeAdapter, PrivateAttr, model_serializer, model_validator, RootModel

class Action(NestedEnum):
    AddNode: {
        'id': int,
        'ty': int
    }
    AddEdge: {
        'id': int,
        'ty': int,
        'source': int,
        'target': int,
    }
    RemoveNode: {
        'id': int
    }
    RemoveEdge: {
        'id': int
    }

Graph = GenericGraph[int, int, int, int]

def run(schema_json: str, json_actions: str) -> str:
    """
    args: list of Actions to perform as a json string
    """

    g = Graph(GenericSchema[int, int, int, int].model_validate_json(schema_json))

    actions = TypeAdapter(List[Action]).validate_json(json_actions)

    for i, action in enumerate(actions):
        if isinstance(action, Action.AddNode):
            g.add_node(GenericWeight((action.id, action.ty)))
        elif isinstance(action, Action.AddEdge):
            g.add_edge(action.source, action.target, GenericWeight((action.id, action.ty)))
        elif isinstance(action, Action.RemoveNode):
            g.remove_node(action.id)
        elif isinstance(action, Action.RemoveEdge):
            g.remove_edge(action.id)
        else:
            # Do not that type allocation relies on __subclasses__ which may be out of sync
            raise Exception(f'Failed deserialize Action {i}. Action came with name {repr(action)} which did not match any actions')

    return g.model_dump_json()

if __name__ == '__main__':
    run(
'{"node_whitelist":[0,1,2,3,4,5,6,7,8],"node_blacklist":null,"edge_whitelist":null,"edge_blacklist":null,"endpoint_whitelist":[[4,6,1],[8,4,0],[1,2,1],[4,2,1],[8,4,1],[4,1,2],[4,6,5],[1,3,2],[3,3,2],[8,6,1],[0,2,1],[8,8,0],[2,6,2],[3,1,0],[0,1,2],[1,3,3],[0,0,2],[2,1,1],[2,6,5],[7,1,2],[1,8,3],[1,4,0],[7,0,5],[7,1,1],[6,5,1],[4,7,3],[0,7,3],[8,6,3],[6,7,5],[7,4,0],[1,4,3],[3,6,0],[4,3,2],[7,3,2],[7,2,3],[5,2,3],[2,1,4],[2,5,5],[0,1,0],[7,8,5],[5,6,2],[8,5,2],[8,0,2],[5,5,3],[7,7,3],[3,3,5],[0,0,1],[3,7,2],[5,3,2],[1,0,0],[0,6,2],[0,3,2],[7,1,3],[7,5,0],[0,3,5],[3,4,0],[6,7,3],[4,4,5],[6,6,3],[1,0,1],[3,6,1],[4,3,3],[1,7,0],[3,5,0],[2,2,0],[3,5,4],[4,6,0],[1,3,5],[0,4,2],[0,8,3],[2,2,1],[3,8,2],[7,8,0],[5,8,1],[1,3,1],[1,4,2],[6,1,5],[1,5,5],[5,0,5],[4,7,0],[4,8,0],[8,6,2],[6,0,0],[6,2,2],[4,2,3],[1,6,1],[7,5,5],[5,1,0],[4,1,0],[2,8,0],[4,5,1],[2,2,5],[5,7,1],[0,4,0],[6,7,0],[1,2,3],[7,7,5],[3,0,2],[8,8,3],[1,1,3],[6,4,0],[5,2,0],[1,6,0],[6,3,1],[1,1,2],[0,0,3],[2,5,0],[7,1,0],[8,3,1],[4,2,0],[3,1,3],[6,1,2],[3,2,0],[2,3,5],[8,6,5],[1,7,1],[2,5,2],[4,4,2],[4,4,1],[2,4,3],[1,8,5],[4,0,1],[8,1,3],[3,7,3],[1,8,4],[1,8,0],[7,1,5],[6,6,0],[8,5,1],[2,2,3],[4,6,2],[8,0,3],[7,7,0],[3,7,5],[5,8,3],[8,8,5],[1,5,3],[4,1,3],[3,7,1],[0,0,0],[6,7,2],[3,6,3],[3,5,5],[6,5,5],[3,0,5],[6,5,2],[5,2,2],[8,5,0],[7,6,3],[1,2,5],[4,3,1],[2,1,0],[1,8,2],[3,4,2],[7,0,0],[3,5,2],[8,2,3],[8,8,2],[7,2,5],[7,2,0],[1,8,1],[6,4,3],[3,2,3],[6,1,0],[4,1,1],[1,7,3],[3,8,5],[7,8,2],[6,8,2],[4,5,3],[3,3,3],[0,6,5],[7,6,2],[8,7,5],[4,8,3],[5,8,5]],"endpoint_blacklist":null,"endpoint_outgoing_max_quantity":null,"endpoint_incoming_max_quantity":null}',
'[{"AddNode":{"id":0,"ty":4}},{"RemoveNode":{"id":0}},{"AddNode":{"id":1,"ty":6}},{"AddNode":{"id":2,"ty":7}},{"AddEdge":{"id":0,"ty":0,"source":2,"target":2}},{"RemoveEdge":{"id":0}},{"AddEdge":{"id":1,"ty":3,"source":1,"target":2}},{"AddEdge":{"id":2,"ty":2,"source":1,"target":2}},{"RemoveEdge":{"id":2}},{"AddEdge":{"id":3,"ty":0,"source":1,"target":2}},{"RemoveNode":{"id":2}},{"AddEdge":{"id":4,"ty":0,"source":1,"target":1}},{"AddEdge":{"id":5,"ty":3,"source":1,"target":1}},{"AddEdge":{"id":6,"ty":3,"source":1,"target":1}},{"AddEdge":{"id":7,"ty":3,"source":1,"target":1}},{"AddEdge":{"id":8,"ty":3,"source":1,"target":1}},{"RemoveEdge":{"id":7}},{"AddNode":{"id":3,"ty":8}},{"RemoveEdge":{"id":4}},{"RemoveNode":{"id":3}},{"AddNode":{"id":4,"ty":3}}]'
    )
from typed_graph.typed_traits import TypeStatus

class TypedError(Exception):
    def __init__(self, *args: object) -> None:
        super().__init__(*args)

class MissingNodeId(TypedError):
    def __init__(self, node_id) -> None:
        super().__init__(f"Failed to find node at {node_id}")

class MissingEdgeId(TypedError):
    def __init__(self, edge_id) -> None:
        super().__init__(f"Failed to find edge at {edge_id}")

class InvalidNodeType(TypedError):
    def __init__(self, node_type, node_status: TypeStatus) -> None:
        super().__init__(f"Invalid node type {node_type} due to {node_status}")

class InvalidEdgeType(TypedError):
    def __init__(self, edge_type, source_type, target_type, edge_status: TypeStatus) -> None:
        super().__init__(f"Invalid edge type {edge_type} from {source_type} to {target_type} due to {edge_status}")

class MissingJsonField(TypedError):
    def __init__(self, required_field, ctx) -> None:
        super().__init__(f"Missing required field {required_field} in json when parsing {ctx}")

class RecievedNoneValue(TypedError):
    def __init__(self, v, ctx) -> None:
        v_ty = type(v)
        super().__init__(f"Recieved None from {v_ty} when trying to get {ctx}")
        
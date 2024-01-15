from typed_graph import TypedGraph, NodeExt, EdgeExt, GraphData, RustRootModel, SchemaExt, TypeStatus
from typing import Generic, TypeVar, Tuple, Optional, List, Dict

K = TypeVar('K')
T = TypeVar('T')

class GenericWeight(RustRootModel[Tuple[K, T]], NodeExt[K, T], EdgeExt[K, T], Generic[K, T]):
    """
    GenericWeight[K, T]
    """
    root: Tuple[K, T]

    def get_id(self) -> K:
        return self.root[0]

    def set_id(self, id: K):
        self.root[0] = id

    def get_type(self) -> T:
        return self.root[1]
    
    def __getitem__(self, idx: int) -> K | T:
        return self.root[idx]
    
    def __setitem__(self, idx, v):
        self.root[idx] = v

NK = TypeVar('NK')
EK = TypeVar('EK')
NT = TypeVar('NT')
ET = TypeVar('ET')

class GenericSchema(SchemaExt[GenericWeight[NK, NT], GenericWeight[EK, ET], NK, EK, NT, ET], Generic[NK, EK, NT, ET]):
    """
    GenericSchema[NK, EK, NT, ET]
    """
    node_whitelist: Optional[List[NT]] = None
    node_blacklist: Optional[List[NT]] = None
    edge_whitelist: Optional[List[ET]] = None
    edge_blacklist: Optional[List[ET]] = None

    endpoint_whitelist: Optional[List[Tuple[NT, NT, ET]]] = None
    endpoint_blacklist: Optional[List[Tuple[NT, NT, ET]]] = None
    endpoint_max_quantity: Optional[Dict[Tuple[NT, NT, ET], int]] = None

    def name(self) -> str:
        return 'GenericSchema'
    
    def allow_node(self, node_type: NT) -> TypeStatus:
        is_whitelist = self.node_whitelist is None or node_type in self.node_whitelist
        is_blacklist = self.node_blacklist is None or node_type not in self.node_blacklist
        is_allowed = is_whitelist and is_blacklist

        if not is_allowed:
            return TypeStatus.InvalidType
        else:
            return TypeStatus.Ok
        
    def allow_edge(self, quantity: int, edge_type: ET, source_type: NT, target_type: NT) -> TypeStatus | bool:
        is_whitelist = self.edge_whitelist is None or edge_type in self.edge_whitelist
        is_blacklist = self.edge_blacklist is None or edge_type not in self.edge_blacklist

        endpoint = (source_type, target_type, edge_type)
        is_endpoint_whitelist = self.endpoint_whitelist is None or endpoint in self.endpoint_whitelist
        is_endpoint_blacklist = self.endpoint_blacklist is None or endpoint not in self.endpoint_blacklist

        is_allowed = is_whitelist and is_blacklist and is_endpoint_whitelist and is_endpoint_blacklist

        if not is_allowed:
            return TypeStatus.InvalidType
        
        is_quantity_allowed = self.endpoint_max_quantity is None or endpoint not in self.endpoint_max_quantity or quantity <= self.endpoint_max_quantity.get(endpoint)
        if not is_quantity_allowed:
            return TypeStatus.ToMany
        
        return TypeStatus.Ok

class GenericGraph(TypedGraph[GenericWeight[NK, NT], GenericWeight[EK, ET], NK, EK, NT, ET, GenericSchema[NK, EK, NT, ET]], Generic[NK, EK, NT, ET]):
    """
    GenericGraph[NK, EK, NT, ET]
    """
    pass
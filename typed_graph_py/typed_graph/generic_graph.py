from typed_graph import TypedGraph, NodeExt, EdgeExt, GraphData, RustRootModel, SchemaExt, TypeStatus
from typing import Generic, TypeVar, Tuple, Optional, List, Dict, ClassVar
from pydantic import RootModel

K = TypeVar('K')
T = TypeVar('T')

class GenericWeight(RootModel[Tuple[K, T]], NodeExt[K, T], EdgeExt[K, T], Generic[K, T]):
    """
    GenericWeight[K, T]
    """
    root: Tuple[K, T]
    tagging: ClassVar[bool] = False

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
    tagging: ClassVar[bool] = False

    node_whitelist: Optional[List[NT]] = None
    node_blacklist: Optional[List[NT]] = None
    edge_whitelist: Optional[List[ET]] = None
    edge_blacklist: Optional[List[ET]] = None
    edge_endpoint_whitelist: Optional[List[Tuple[NT, NT, ET]]] = None
    edge_endpoint_blacklist: Optional[List[Tuple[NT, NT, ET]]] = None
    edge_endpoint_outgoing_max_quantity: Optional[Dict[Tuple[NT, ET], int]] = None
    edge_endpoint_incoming_max_quantity: Optional[Dict[Tuple[NT, ET], int]] = None

    def name(self) -> str:
        return 'TestSchema'
    
    def allow_edge(self, outgoing_quanitty: int, incoming_quantity: int, edge_type: ET, source_type: NT, target_type: NT) -> TypeStatus:
        is_whitelist = not self.edge_whitelist or edge_type in self.edge_whitelist
        is_blacklist = not self.edge_blacklist or not edge_type in self.edge_blacklist
        is_endpoint_whitelist = not self.edge_endpoint_whitelist or (edge_type, source_type, target_type) in self.edge_endpoint_whitelist
        is_endpoint_blacklist = not self.edge_endpoint_blacklist or not (edge_type, source_type, target_type) in self.edge_endpoint_blacklist
        is_allowed_type = is_whitelist and is_blacklist and is_endpoint_whitelist and is_endpoint_blacklist

        if not is_allowed_type:
            return TypeStatus.InvalidType
        
        max_outgoing = False
        if self.edge_endpoint_outgoing_max_quantity:
            max_outgoing = self.edge_endpoint_outgoing_max_quantity.get((source_type, edge_type), outgoing_quanitty + 1)

        max_incoming = False
        if self.edge_endpoint_incoming_max_quantity:
            max_incoming = self.edge_endpoint_incoming_max_quantity.get((source_type, edge_type), incoming_quantity + 1)

        if not outgoing_quanitty > max_outgoing:
            return TypeStatus.ToManyOutgoing
        
        if not incoming_quantity > max_incoming:
            return TypeStatus.ToManyIncoming

        return TypeStatus.Ok
    
    def allow_node(self, node_type: str) -> TypeStatus:
        is_whitelist = not self.node_whitelist or node_type in self.node_whitelist
        is_blacklist = not self.node_blacklist or not node_type in self.node_blacklist
        is_allowed = is_whitelist and is_blacklist

        if not is_allowed:
            return TypeStatus.InvalidType
        
        return TypeStatus.Ok

GenericGraph = TypedGraph[GenericWeight[NK, NT], GenericWeight[EK, ET], NK, EK, NT, ET, GenericSchema[NK, EK, NT, ET]]
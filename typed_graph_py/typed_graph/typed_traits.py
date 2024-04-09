from abc import abstractmethod
from typing import TypeVar, Generic, ClassVar
from typed_graph.dependency_traits import RustModel, IntEnum

K = TypeVar('K')
T = TypeVar('T')
class GraphData(Generic[K, T]):
    """
    GraphData[K, T]

    Protocol for a type storing an id and type
    """
    abstract: ClassVar[bool] = True

    @abstractmethod
    def get_id(self) -> K:
        raise NotImplementedError

    @abstractmethod
    def set_id(self, id: K) -> None:
        raise NotImplementedError
    
    @abstractmethod
    def get_type(self) -> T:
        raise NotImplementedError
    
NK = TypeVar('NK')
NT = TypeVar('NT')

class NodeExt(RustModel, GraphData[K, T]):
    """
    NodeExt[NK, NT]

    Mark a node class
    
    The node must implement Id and Typed

    It will be serialized using serde. When inheriting form EdgeExt normal serde syntax can be used to define types on fields
    """
    pass

EK = TypeVar('EK')
ET = TypeVar('ET')

class EdgeExt(RustModel, GraphData[K, T]):
    """
    NodeExt[EK, ET]

    Mark an edge class
    
    The edge must implement Id and Typed

    It will be serialized using serde. When inheriting form EdgeExt normal serde syntax can be used to define types on fields
    """
    pass

class TypeStatus(IntEnum):
    """Indicator for whether or not the given type is allowed in the graf"""
    InvalidType = 0
    Ok = 1
    ToManyOutgoing = 2
    ToManyIncoming = 3

    @staticmethod
    def is_allowed(status: 'TypeStatus') -> bool:
        return status == TypeStatus.Ok
    
    def __str__(self) -> str:
        if self == TypeStatus.InvalidType:
            return 'InvalidType'
        elif self == TypeStatus.Ok:
            return 'Ok'
        elif self == TypeStatus.ToManyOutgoing:
            return 'ToManyOutgoing'
        elif self == TypeStatus.ToManyIncoming:
            return 'ToManyIncoming'
        return 'Unknown TypeStatus'

N = TypeVar('N')
E = TypeVar('E')

class SchemaExt(RustModel, Generic[N, E, NK, EK, NT, ET]):
    """
    SchemaExt[N, E, NK, EK, NT, ET]

    Mark a schema class
    
    It will be serialized using serde. When inheriting form EdgeExt normal serde syntax can be used to define types on fields
    """
    abstract: ClassVar[bool] = True

    @abstractmethod
    def name(self) -> str:
        pass

    @abstractmethod
    def allow_node(self, node_type: NT) -> TypeStatus:
        pass

    @abstractmethod
    def allow_edge(self, outgoing_quantity: int, incoming_quantity: int, edge_type: ET, source_type: NT, target_type: NT) -> TypeStatus:
        pass

S = TypeVar('S')
from typed_graph.typed_traits import *
from typed_graph.typed_error import *
from typed_graph.dependency_traits import RustModel
from typing import Dict, Generic, List, Iterator, Callable, get_args, Any
from itertools import chain
from pydantic import model_validator, model_serializer, BaseModel, Field, validate_call, TypeAdapter
from enum import Enum
import json

class EdgeCls(BaseModel, Generic[E, NK]):
    """
    Schema for serializing and deserializing edges in the graph
    """
    weight: E
    source: NK
    target: NK

class TypedGraphCls(BaseModel, Generic[N, E, NK, S]):
    """
    Schema for serializing and deserializing the graph
    """
    graph_schema: S = Field(alias="schema")
    nodes:  List[N]
    edges:  List[EdgeCls[E, NK]]

class Direction(Enum):
    """
    Describes the direction of an edge

    Forward is normal source/target
    Backwards everything is inverted source/target turns into target/source
    """
    Forward = 0,
    Backward = 1

class NodeMetadata(RustModel, Generic[N, EK]):
    """
    Contains metadata about the node

    Each node also stores their adjecent edges
    """
    weight: N
    incoming_edges: List[EK] = []
    outgoing_edges: List[EK] = []

class EdgeMetadata(RustModel, Generic[E, NK]):
    """
    Contains metadata about the edge

    Each edge also stores their source and target node
    """
    weight: E
    source: NK
    target: NK

class EdgeRef(Generic[E, NK]):
    def __init__(self, weight: E, source: NK, target: NK, dir: Direction) -> None:
        self.weight = weight
        self.source = source
        self.target = target
        self.dir = dir

    def get_outer(self) -> NK:
        """
        Get the id of the node this edge was not retrieved from 
        """
        if self.dir == Direction.Forward:
            return self.target
        else:
            return self.source
        
    def get_inner(self) -> NK:
        """
        Get the id of the node this edge was retrieved from
        """
        if self.dir == Direction.Forward:
            return self.source
        else:
            return self.target


class TypedGraph(BaseModel, Generic[N, E, NK, EK, NT, ET, S]):
    """
    Typedraph[N, E, NK, EK, NT, ET, S]

    Generic graph with customizable types
    """
    graph_schema: S = Field(alias="schema")
    nodes: Dict[NK, NodeMetadata[N, EK]] = dict()
    edges: Dict[EK, EdgeMetadata[E, NK]] = dict()

    def __init__(self, schema: S):
        if isinstance(schema, str):
            super().__init__(schema=json.loads(schema))
        else:    
            super().__init__(schema=schema)
    
    def get_schema(self) -> S:
        """Get the schema of the graph"""
        return self.graph_schema
    
    def node_count(self) -> int:
        """Return the total number of nodes"""
        return len(self.nodes)
    
    def edge_count(self) -> int:
        """Return the total number of edges"""
        return len(self.edges)
    
    def _get_node(self, node_id: EK) -> NodeMetadata[E, NK]:
        """
        Internal function for retrieving the metadata of a node

        This should not be used externally as changing the state of these nodes
        Can break the graph
        """
        node = self.nodes.get(node_id)

        if node is None:
            raise MissingNodeId(node_id)
        
        return node
    
    def _get_edge(self, edge_id: EK) -> EdgeMetadata[E, NK]:
        """
        Internal function for retrieving the metadata of an edge

        This should not be used externally as changing the state will break the graph
        """
        edge = self.edges.get(edge_id)

        if edge is None:
            raise MissingEdgeId(edge_id)
        
        return edge

    def get_node_safe(self, node_id: NK) -> N | None:
        """Retrieve a node and if it is not there return None instead"""
        node = self.nodes.get(node_id)

        if node is None:
            return None
        
        return node.weight
    
    def get_node(self, node_id: NK) -> N:
        """Retrieve a node and if it is not fails with MissingNodeId"""
        node = self.get_node_safe(node_id)

        if node is None:
            raise MissingNodeId(node_id)
        
        return node

    def get_edge_safe(self, edge_id: EK) -> E | None:
        """Retrieve a edge and if it is not there return None instead"""
        edge = self.edges.get(edge_id)

        if edge is None:
            return None
        
        return edge.weight
    
    def get_edge(self, edge_id: EK) -> E:
        """Retrieve a edge and if it is not fails with MissingEdgeId"""
        edge = self.get_edge_safe(edge_id)

        if edge is None:
            raise MissingEdgeId(edge_id)
        
        return edge
    
    def get_edge_full(self, edge_id: EK) -> EdgeRef[E, NK]:
        """Get a node and its source and target id"""
        edge = self.edges.get(edge_id)

        if edge is None:
            raise MissingEdgeId(edge_id)
        
        return EdgeRef(edge.weight, edge.source, edge.target, Direction.Forward)
    
    def get_edge_full_safe(self, edge_id: EK) -> EdgeRef[E, NK] | None:
        """Get a node and its source and target id"""
        edge = self.edges.get(edge_id)

        if edge is None:
            return None
        
        return EdgeRef(edge.weight, edge.source, edge.target, Direction.Forward)
    
    def has_node(self, node_id: NK) -> bool:
        """Check if the given node exists"""
        return node_id in self.nodes
    
    def has_edge(self, edge_id: EK) -> bool:
        """Check if the given edge exists"""
        return edge_id in self.edges
    
    def add_node(self, node: N) -> NK:
        """
        Add a node and returns the id of the new node

        This will fail if the given node type is not allowed

        If a node already exist then it is replaced
        """
        node_id = node.get_id()
        
        if node_id is None:
            raise RecievedNoneValue(node,  'id')
        
        current_node = self.nodes.get(node_id)

        # Check the type of the node against the schema
        node_type = node.get_type()
        if node_type is None:
            raise RecievedNoneValue(node,  'type')
        
        node_status = self.graph_schema.allow_node(node_type)

        is_false = isinstance(node_status, bool) and not node_status
        is_not_allowed = isinstance(node_status, TypeStatus) and not TypeStatus.is_allowed(node_status)
        if is_false or is_not_allowed:
            raise InvalidNodeType(node_type, node_status)

        # Check if the node needs to be replaced
        if current_node is not None:
            current_node_type = current_node.weight.get_type()
            if current_node_type is None:
                raise RecievedNoneValue(current_node.weight,  'type')

            # Attempt to replace the node
            # This requires for all edges to be compatible with the new node type
            if current_node_type != node_type:
                edge_ids = chain(current_node.outgoing_edges, current_node.incoming_edges)

                for edge_id in edge_ids:
                    edge = self._get_edge(edge_id)

                    source_node = node if edge.source == node_id else self.get_node(edge.source)
                    target_node = node if edge.target == node_id else self.get_node(edge.target)
                    weight_type = edge.weight.get_type()
                    source_type = source_node.get_type()
                    target_type = target_node.get_type()

                    if weight_type is None:
                        raise RecievedNoneValue(edge.weight,  'type')            
                    if source_type is None:
                        raise RecievedNoneValue(source_node,  'type')
                    if target_type is None:
                        raise RecievedNoneValue(target_node,  'type')

                    # Find the number of other edges of the same type going between the same places
                    quantity = 0
                    outgoing = self.get_outgoing(edge.source)
                    for out_edge in outgoing:
                        out_weight_type = out_edge.weight.get_type()
                        
                        if out_weight_type is None:
                            raise RecievedNoneValue(out_edge.weight,  'type')
                        
                        if out_weight_type != weight_type:
                            continue

                        out_target = self.get_node(out_edge.target)
                        out_target_type = out_target.get_type()
                        
                        if out_target_type is None:
                            raise RecievedNoneValue(out_target,  'type')
                        
                        if out_target_type != target_node.get_type():
                            continue
                        
                        quantity += 1

                    edge_status = self.graph_schema.allow_edge(
                        quantity + 1,
                        weight_type,
                        source_type,
                        target_type
                    )

                    is_false = isinstance(edge_status, bool) and not edge_status
                    is_not_allowed = isinstance(edge_status, TypeStatus) and not TypeStatus.is_allowed(edge_status)

                    if is_false or is_not_allowed:
                        raise InvalidEdgeType(weight_type, source_type, target_type, edge_status)
                
                # Replace the node
                self.nodes[node_id].weight = node
            else:
                # Just perfrom the replacement
                self.nodes[node_id].weight = node
        else:
            # insert the node as normal
            self.nodes[node_id] = NodeMetadata(weight = node)

        return node_id
    
    def add_edge(self, source: NK, target: NK, edge: E) -> EK:
        """
        Add an edge and return the id of the new edge

        This will fail if the given edge type is not allowed

        If the edge already exist then it is replaced
        """
        edge_id = edge.get_id()
        if edge_id is None:
            raise RecievedNoneValue(edge,  'id')

        source_node = self.get_node(source)
        target_node = self.get_node(target)

        edge_type = edge.get_type()
        source_type = source_node.get_type()
        target_type = target_node.get_type()

        if edge_type is None:
            raise RecievedNoneValue(edge,  'type')
        if source_type is None:
            raise RecievedNoneValue(source_node,  'type')
        if target_type is None:
            raise RecievedNoneValue(target_node,  'type')

        # Retrive the number of other edges going from and to the same node
        quantity = 0
        edges = self.get_outgoing(source)
        for out_edge in edges:
            out_weight_type = out_edge.weight.get_type()
            
            if out_weight_type is None:
                raise RecievedNoneValue(out_edge.weight,  'type')
            
            if out_weight_type != edge_type:
                continue

            out_target = self.get_node(out_edge.target)
            out_target_type = out_target.get_type()
            
            if out_target_type is None:
                raise RecievedNoneValue(out_target,  'type')
            
            if out_target_type != target_type:
                continue

            quantity += 1

        # Check if the new edge type is allowed
        edge_status = self.graph_schema.allow_edge(
            quantity + 1,
            edge_type,
            source_type,
            target_type
        )

        is_false = isinstance(edge_status, bool) and not edge_status
        is_not_allowed = isinstance(edge_status, TypeStatus) and not TypeStatus.is_allowed(edge_status)

        if is_false or is_not_allowed:
            raise InvalidEdgeType(edge_type, source_type, target_type, edge_status)
        
        # Check if the edge needs to be replaced
        current_edge = self.edges.get(edge_id)
        if current_edge is not None:
            # If the node already exist then we just update the weight and endpoints
            self.edges[edge_id] = EdgeMetadata(weight = edge, source = source, target = target)

            if current_edge.source != source:
                self.nodes[current_edge.source].outgoing_edges.remove(edge_id)
                self.nodes[source].outgoing_edges.append(edge_id)

            if current_edge.target != target:
                self.nodes[current_edge.target].incoming_edges.remove(edge_id)
                self.nodes[target].incoming_edges.append(edge_id)
        else:
            # Create the node as normal
            self.edges[edge_id] = EdgeMetadata(weight = edge, source = source, target = target)
            self.nodes[source].outgoing_edges.append(edge_id)
            self.nodes[target].incoming_edges.append(edge_id)

        return edge_id

    def remove_node(self, node_id: NK) -> N:
        """
        Remove a node if it exists

        This will also remove all its incoming and outgoing edges
        """
        node = self._get_node(node_id)
        del self.nodes[node_id]

        edges = chain(node.outgoing_edges, node.incoming_edges)
        for edge_id in edges:
            if self.has_edge(edge_id):
                self.remove_edge(edge_id)

        return node.weight
    
    def remove_edge(self, edge_id: EK) -> E:
        """
        Remove an edge if it exists
        """
        edge = self._get_edge(edge_id)
        del self.edges[edge_id]

        if self.has_node(edge.source):
            source_node = self._get_node(edge.source)
            source_node.outgoing_edges.remove(edge_id)

        if self.has_node(edge.target):
            target_node = self._get_node(edge.target)
            target_node.incoming_edges.remove(edge_id)
        
        return edge.weight

    def get_outgoing(self, node_id: NK) -> Iterator[EdgeRef[E, NK]]:
        """
        Get an iterator over all outgoing edges from the node
        """
        node = self._get_node(node_id)
        for edge_id in node.outgoing_edges:
            edge = self._get_edge(edge_id)
            yield EdgeRef(edge.weight, edge.source, edge.target, Direction.Forward)

    def get_incoming(self, node_id: NK) -> Iterator[EdgeRef[E, NK]]:
        """
        Get an iterator over all incoming edges from the node
        """
        node = self._get_node(node_id)
        for edge_id in node.incoming_edges:
            edge = self._get_edge(edge_id)
            yield EdgeRef(edge.weight, edge.source, edge.target, Direction.Backward)

    def get_incoming_and_outgoing(self, node_id: NK) -> Iterator[EdgeRef[E, NK]]:
        """
        Get an iterator over all incoming and outgoing edges from the node
        """
        return chain(self.get_outgoing(node_id), self.get_incoming(node_id))
    
    def get_incoming_filter(self, node_id: NK, f: Callable[[E], bool]) -> Iterator[EdgeRef[E, NK]]:
        """
        Filter the incoming edges based on edge type
        """
        return filter(lambda e: f(e.weight), self.get_incoming(node_id))
    
    def get_outgoing_filter(self, node_id: NK, f: Callable[[E], bool]) -> Iterator[EdgeRef[E, NK]]:
        """
        Filter the outgoing edges based on edge type
        """
        return filter(lambda e: f(e.weight), self.get_outgoing(node_id))
    
    def get_incoming_and_outgoing_filter(self, node_id: NK, f: Callable[[E], bool]) -> Iterator[EdgeRef[E, NK]]:
        """
        Filter incoming and outgoing edges based on edge type
        """
        return filter(lambda e: f(e.weight), self.get_incoming_and_outgoing(node_id))

    def get_nodes(self) -> Iterator[N]:
        """
        Iterate over all nodes
        """
        for node in self.nodes.values():
            yield node.weight

    def get_edges(self) -> Iterator[E]:
        """
        Iterate over all edges
        """
        for edge in self.edges.values():
            yield edge.weight

    def get_node_ids(self) -> Iterator[NK]:
        """
        Iterate over all node ids
        """
        return self.nodes.keys()
    
    def get_edge_ids(self) -> Iterator[EK]:
        """
        Iterate over all edge ids
        """
        return self.edges.keys()

    @model_serializer(mode='plain')
    def _serialize(self, *args, **kwarg) -> dict[str, Any]:
        """
        Parse the graph to json
        """

        nodes = list(self.get_nodes())
        edges = list()
        for node_id in self.get_node_ids():
            for edge in self.get_outgoing(node_id):
                edges.append(EdgeCls(
                    weight = edge.weight,
                    source = edge.source,
                    target = edge.target,
                ))
                
        kwarg['by_alias'] = True

        return TypedGraphCls[N, E, NK, S](
            schema = self.graph_schema,
            nodes = nodes,
            edges = edges
        ).model_dump(*args, **kwarg)
    
    # We use staticmethod instead of classmethods
    # because classmethod for some reason does not include the actual type of the generics
    # if a solution is found to this. It would be great, however for now this is what we have
    @model_validator(mode='wrap')
    def _deserialize(
        cls, 
        d: dict[str, Any], 
        default: Callable[[dict[str, Any]], 'TypedGraph[S]'],
        *args,
        **kwarg
    ) -> 'TypedGraph[S]':
        """
        Parse the graph from json

        the cls argument specifies the type of the graph being passed
        """

        if len(d) == 1:
            return default(d)

        # Check that required fields are present
        if 'schema' not in d:
            raise MissingJsonField('schema', 'TypedGraph')
        if 'nodes' not in d or not isinstance(d['nodes'], list):
            raise MissingJsonField('nodes', 'TypedGraph')
        if 'edges' not in d or not isinstance(d['edges'], list):
            raise MissingJsonField('edges', 'TypedGraph')

        # Retrieve the generics types
        # this is done by going to each of the fields and figuring out which types they have
        # This implementation is to put it lightly stupid, however BaseModel eats the generics from Generic so they are nowhere to be seen
        # this solution is based on https://github.com/pydantic/pydantic/issues/3559

        S = cls.model_fields['graph_schema'].annotation

        nodes = get_args(cls.model_fields['nodes'].annotation)
        edges = get_args(cls.model_fields['edges'].annotation)

        NK = nodes[0]

        N = nodes[1].model_fields['weight'].annotation
        E = edges[1].model_fields['weight'].annotation
        
        # Parse the schema
        schema = TypeAdapter(S).validate_python(d['schema'])

        # Create the graph
        g = cls(schema = schema)

        # Add all the nodes
        for node in d['nodes']:
            g.add_node(TypeAdapter(N).validate_python(node))

        # Add all the edges between the nodes
        for edge in d['edges']:
            if 'source' not in edge:
                raise MissingJsonField('source', 'TypedGraph edge')
            if 'target' not in edge:
                raise MissingJsonField('target', 'TypedGraph edge')
            if 'weight' not in edge:
                raise MissingJsonField('weight', 'TypedGraph edge')

            g.add_edge(
                TypeAdapter(NK).validate_python(edge['source']),
                TypeAdapter(NK).validate_python(edge['target']),
                TypeAdapter(E).validate_python(edge['weight']),
            )

        return g
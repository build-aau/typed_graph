use std::fmt::Debug;
use thiserror::Error;

use crate::{DisAllowedEdge, DisAllowedNode, EdgeKey, NodeKey, SchemaExt, Typed};

pub type TypedResult<T, NK, EK, NT, ET> = Result<T, TypedError<NK, EK, NT, ET>>;
pub type GenericTypedError<NK, EK> = TypedError<NK, EK, String, String>;
pub type GenericTypedResult<T, NK, EK> = Result<T, GenericTypedError<NK, EK>>;

pub type SchemaError<NK, EK, S> = TypedError<
    NK,
    EK,
    <<S as SchemaExt<NK, EK>>::N as Typed>::Type,
    <<S as SchemaExt<NK, EK>>::E as Typed>::Type,
>;

/// Helper type for errors.
pub type SchemaResult<T, NK, EK, S> = Result<T, SchemaError<NK, EK, S>>;

/// Combined error enum.
#[derive(Error, Debug)]
pub enum TypedError<NK, EK, NT, ET> {
    #[error("Used a node key ({0:?}) which has already been removed")]
    NodeKeyRemoved(NK),

    #[error("Used an edge key ({0:?}) which has already been removed")]
    EdgeKeyRemoved(EK),

    #[error("Node id collision ({0:?})")]
    NodeIdCollision(NK),

    #[error("Edge id collision ({0:?})")]
    EdgeIdCollision(EK),

    #[error("Node id missing ({0:?})")]
    NodeIdMissing(NK),

    #[error("Edge id missing ({0:?})")]
    EdgeIdMissing(EK),

    #[error("Failed to get node ({0:?})")]
    MissingNode(NK),

    #[error("Failed to get edge ({0:?})")]
    MissingEdge(EK),

    #[error("Invalid edge type {0} from {1} to {2} due to {3:?}")]
    InvalidEdgeType(ET, NT, NT, DisAllowedEdge),

    #[error("Invalid node type {0} due to {1:?}")]
    InvalidNodeType(NT, DisAllowedNode),

    #[error("The graph has entered an invalid state")]
    InvalidInternalState,

    #[error("Failed to convert {0} into {1}")]
    DownCastFailed(String, String),

    #[error("Node id was changed from {0} to {1} which was not expected")]
    InconsistentNodeIds(NK, NK),

    #[error("Edge id was changed from {0} to {1} which was not expected")]
    InconsistentEdgeIds(EK, EK),

    #[error("Failed to get node using internal key {0:?}")]
    MissingNodeKey(NodeKey),

    #[error("Failed to get edge using internal key {0:?}")]
    MissingEdgeKey(EdgeKey),

    #[error("Failed to move {0:?} to {1:?} since they do not have the same source")]
    InvalidEdgeMove(EK, EK),

    #[cfg(test)]
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

impl<NK, EK, NT, ET> TypedError<NK, EK, NT, ET> {
    pub fn map<NK1, EK1, NT1, ET1, NKF, EKF, NTF, ETF>(
        self,
        nk_map: NKF,
        ek_map: EKF,
        nt_map: NTF,
        et_map: ETF,
    ) -> TypedError<NK1, EK1, NT1, ET1>
    where
        NKF: Fn(NK) -> NK1,
        EKF: Fn(EK) -> EK1,
        NTF: Fn(NT) -> NT1,
        ETF: Fn(ET) -> ET1,
    {
        match self {
            TypedError::NodeKeyRemoved(a) => TypedError::NodeKeyRemoved(nk_map(a)),
            TypedError::EdgeKeyRemoved(a) => TypedError::EdgeKeyRemoved(ek_map(a)),
            TypedError::NodeIdCollision(a) => TypedError::NodeIdCollision(nk_map(a)),
            TypedError::EdgeIdCollision(a) => TypedError::EdgeIdCollision(ek_map(a)),
            TypedError::NodeIdMissing(a) => TypedError::NodeIdMissing(nk_map(a)),
            TypedError::EdgeIdMissing(a) => TypedError::EdgeIdMissing(ek_map(a)),
            TypedError::MissingNode(a) => TypedError::MissingNode(nk_map(a)),
            TypedError::MissingEdge(a) => TypedError::MissingEdge(ek_map(a)),
            TypedError::InvalidEdgeType(a, b, c, e) => {
                TypedError::InvalidEdgeType(et_map(a), nt_map(b), nt_map(c), e)
            }
            TypedError::InvalidNodeType(a, e) => TypedError::InvalidNodeType(nt_map(a), e),
            TypedError::InvalidInternalState => TypedError::InvalidInternalState,
            TypedError::DownCastFailed(a, b) => TypedError::DownCastFailed(a, b),
            TypedError::InconsistentNodeIds(a, b) => {
                TypedError::InconsistentNodeIds(nk_map(a), nk_map(b))
            }
            TypedError::InconsistentEdgeIds(a, b) => {
                TypedError::InconsistentEdgeIds(ek_map(a), ek_map(b))
            }
            TypedError::InvalidEdgeMove(a, b) => TypedError::InvalidEdgeMove(ek_map(a), ek_map(b)),
            TypedError::MissingNodeKey(a) => TypedError::MissingNodeKey(a),
            TypedError::MissingEdgeKey(a) => TypedError::MissingEdgeKey(a),
            #[cfg(test)]
            TypedError::SerdeJsonError(a) => TypedError::SerdeJsonError(a),
        }
    }
}

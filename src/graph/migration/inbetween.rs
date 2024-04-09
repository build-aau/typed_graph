use std::marker::PhantomData;

use crate::*;

/// A Schema that is inbetween two other schemas.
/// This allows a user to make changes to the data before it is fully converted into the other schema
#[derive(Default)]
pub struct InBetween<NK, EK, Old, New> {
    nk: PhantomData<NK>,
    ek: PhantomData<EK>,
    old: Old,
    new: New,
}

impl<NK, EK, Old, New> InBetween<NK, EK, Old, New> {
    pub fn new(old: Old, new: New) -> InBetween<NK, EK, Old, New> {
        InBetween {
            nk: PhantomData,
            ek: PhantomData,
            old,
            new,
        }
    }
}

impl<NK, EK, OldVersion, NewVersion> SchemaExt<NK, EK> for InBetween<NK, EK, OldVersion, NewVersion>
where
    NK: Key,
    EK: Key,
    OldVersion: SchemaExt<NK, EK> + MigrateSchema<NK, EK, NewVersion>,
    NewVersion: SchemaExt<NK, EK>,
{
    type N = EitherVersion<<OldVersion as SchemaExt<NK, EK>>::N, <NewVersion as SchemaExt<NK, EK>>::N>;
    type E = EitherVersion<<OldVersion as SchemaExt<NK, EK>>::E, <NewVersion as SchemaExt<NK, EK>>::E>;

    fn name(&self) -> String {
        self.old.name() + " or " + &self.new.name()
    }

    fn allow_node(&self, node_ty: <Self::N as Typed>::Type) -> Result<(), DisAllowedNode> {
        match node_ty {
            EitherVersion::Old(node_ty) => self.old.allow_node(node_ty),
            EitherVersion::New(node_ty) => self.new.allow_node(node_ty),
        }
    }

    fn allow_edge(
        &self,
        outgoing_edge_count: usize,
        incoming_edge_count: usize,
        edge_ty: <Self::E as Typed>::Type,
        source: <Self::N as Typed>::Type,
        target: <Self::N as Typed>::Type,
    ) -> Result<(), DisAllowedEdge> {
        match (edge_ty, source, target) {
            // The edge is within the old graph
            (EitherVersion::Old(edge_ty), EitherVersion::Old(source), EitherVersion::Old(target)) => {
                self.old.allow_edge(outgoing_edge_count, incoming_edge_count, edge_ty, source, target)
            }
            // The edge is within the new graph
            (EitherVersion::New(edge_ty), EitherVersion::New(source), EitherVersion::New(target)) => {
                self.new.allow_edge(outgoing_edge_count, incoming_edge_count, edge_ty, source, target)
            }

            // The edge is somewhere inbetween the two graphs
            (edge_ty, source, target) => {
                // Only allow the edge if everything can be converted into the new graph
                let updated_content = (
                    self.update_edge_type(&self.new, edge_ty),
                    self.update_node_type(&self.new, source),
                    self.update_node_type(&self.new, target),
                );
                if let (Some(edge_ty), Some(source), Some(target)) = updated_content {
                    self.new.allow_edge(outgoing_edge_count, incoming_edge_count, edge_ty, source, target)
                } else {
                    Err(DisAllowedEdge::InvalidType)
                }
            }
        }
    }
}

impl<NK, EK, OldVersion, NewVersion> MigrateSchema<NK, EK, NewVersion>
    for InBetween<NK, EK, OldVersion, NewVersion>
where
    NK: Key,
    EK: Key,
    OldVersion: SchemaExt<NK, EK> + MigrateSchema<NK, EK, NewVersion>,
    NewVersion: SchemaExt<NK, EK>,
{
    fn update_edge(
        &self,
        new_schema: &NewVersion,
        edge: Self::E,
    ) -> Option<<NewVersion as SchemaExt<NK, EK>>::E> {
        match edge {
            EitherVersion::New(e) => Some(e),
            EitherVersion::Old(e) => self.old.update_edge(new_schema, e),
        }
    }

    fn update_node(
        &self,
        new_schema: &NewVersion,
        node: Self::N,
    ) -> Option<<NewVersion as SchemaExt<NK, EK>>::N> {
        match node {
            EitherVersion::New(n) => Some(n),
            EitherVersion::Old(n) => self.old.update_node(new_schema, n),
        }
    }

    fn update_edge_type(
        &self,
        new_schema: &NewVersion,
        edge_type: <Self::E as Typed>::Type,
    ) -> Option<<<NewVersion as SchemaExt<NK, EK>>::E as Typed>::Type> {
        match edge_type {
            EitherVersion::New(ty) => Some(ty),
            EitherVersion::Old(ty) => self.old.update_edge_type(new_schema, ty),
        }
    }

    fn update_node_type(
        &self,
        new_schema: &NewVersion,
        node_type: <Self::N as Typed>::Type,
    ) -> Option<<<NewVersion as SchemaExt<NK, EK>>::N as Typed>::Type> {
        match node_type {
            EitherVersion::New(ty) => Some(ty),
            EitherVersion::Old(ty) => self.old.update_node_type(new_schema, ty),
        }
    }
}

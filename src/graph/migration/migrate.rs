use std::convert::identity;

use crate::*;

pub type MigrationGraph<NK, EK, Old, New> = TypedGraph<NK, EK, InBetween<NK, EK, Old, New>>;
pub type MigrationResult<T, NK, EK> = GenericTypedResult<T, NK, EK>;

pub trait Migrationhandler<NK, EK, OldVersion, NewVersion> 
where
    NK: Key,
    EK: Key,
    OldVersion: SchemaExt<NK, EK>,
    NewVersion: SchemaExt<NK, EK>,
    OldVersion: MigrateSchema<NK, EK, NewVersion>
{
    fn update_data(&self, g: &mut MigrationGraph<NK, EK, OldVersion, NewVersion>) -> SchemaResult<(), NK, EK, InBetween<NK, EK, OldVersion, NewVersion>>;
}

pub trait MigrateSchema<NK, EK, NewVersion> 
where
    NK: Key,
    EK: Key,
    NewVersion: SchemaExt<NK, EK>,
    Self: SchemaExt<NK, EK>,
{
    /// Update a node from its old type to the new one
    /// 
    /// Returning None indicates that there exists no equivalent in the new schema
    fn update_node(&self, new_schema: &NewVersion, node: Self::N) -> Option<NewVersion::N>;
    /// Update an edge from its old type to the new one
    /// 
    /// Returning None indicates that there exists no equivalent in the new schema
    fn update_edge(&self, new_schema: &NewVersion, edge: Self::E) -> Option<NewVersion::E>;
    /// Update a node type from its old version to its new one
    /// 
    /// Returning None indicates that there exists no equivalent in the new schema
    fn update_node_type(&self, new_schema: &NewVersion, node_type: <Self::N as Typed>::Type) -> Option<<NewVersion::N as Typed>::Type>;
    /// Update an edge type from its old version to its new one
    /// 
    /// Returning None indicates that there exists no equivalent in the new schema
    fn update_edge_type(&self, new_schema: &NewVersion, edge_type: <Self::E as Typed>::Type) -> Option<<NewVersion::E as Typed>::Type>;
}

pub trait Migration<NK, EK, NewVersion>: SchemaExt<NK, EK>
where
    NK: Key,
    EK: Key,
    NewVersion: SchemaExt<NK, EK> + Clone,
    Self: MigrateSchema<NK, EK, NewVersion> + Clone
{

    type Handler: Migrationhandler<NK, EK, Self, NewVersion>;

    /// mirgate the data store in one schema to another
    /// 
    /// Most of the time the default implementation is used as it uses an InBetween representation of the shemas to ensure type safety all throughout the migration process
    fn migrate(
        g: TypedGraph<NK, EK, Self>,
        handler: &Self::Handler,
        new_schema: NewVersion
    ) -> GenericTypedResult<TypedGraph<NK, EK, NewVersion>, NK, EK> {
        // Setup migration enviroment
        let old_schema = g.get_schema().clone();
        let old_name = old_schema.name();
        let new_name = new_schema.name();

        let to_generic_error = |e: SchemaError<NK, EK, InBetween<NK, EK, Self, NewVersion>>|
            e.map(
                identity, 
                identity, 
                |nt| match nt {
                    Either::Old(nt) => format!("{}::{}", old_name, nt),
                    Either::New(nt) => format!("{}::{}", new_name, nt),
                }, 
                |et| match et {
                    Either::Old(et) => format!("{}::{}", old_name, et),
                    Either::New(et) => format!("{}::{}", new_name, et),
                }, 
            );
        
        let mut migration_g: MigrationGraph<NK, EK, Self, NewVersion> = g.update_schema(
            InBetween::new(old_schema, new_schema.clone()), 
            |_, _, n| Some(Either::Old(n)), 
            |_, _, e| Some(Either::Old(e)), 
        ).map_err(to_generic_error)?;


        handler.update_data(&mut migration_g).map_err(to_generic_error)?;

        // Finalize migration
        let new_g = migration_g.update_schema(
            new_schema, 
            |current_schema, new_schema, node| current_schema.update_node(&new_schema, node),
            |current_schema, new_schema, edge| current_schema.update_edge(&new_schema, edge),
        )
        // filter_map returns an error for the new schema
        // So we have to convert it into an error for the joined schema
        .map_err(|e| e
            .map(identity, identity, Either::New, Either::New)
        )
        // And then we can use the same formatter as for the other results
        .map_err(to_generic_error)?;

        Ok(new_g)
    }
}

pub trait DirectMigration<NK, EK, NewVersion>: SchemaExt<NK, EK> + Sized
where
    NK: Key,
    EK: Key,
    NewVersion: SchemaExt<NK, EK>
{
    fn migrate(
        g: TypedGraph<NK, EK, Self>
    ) -> GenericTypedResult<TypedGraph<NK, EK, NewVersion>, NK, EK>;
}
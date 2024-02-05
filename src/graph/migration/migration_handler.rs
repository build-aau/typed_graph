use crate::*;

/// The default migration handler does not alter any of the data in the graph
pub struct DefaultMigrationHandler;

impl<NK, EK, OldVersion, NewVersion> Migrationhandler<NK, EK, OldVersion, NewVersion>
    for DefaultMigrationHandler
where
    NK: Key,
    EK: Key,
    OldVersion: SchemaExt<NK, EK> + Clone,
    NewVersion: SchemaExt<NK, EK> + Clone,

    // Everything should be convertable to the new schema
    OldVersion: MigrateSchema<NK, EK, NewVersion>,
{
    fn update_data(
        &self,
        _g: &mut MigrationGraph<NK, EK, OldVersion, NewVersion>,
    ) -> SchemaResult<(), NK, EK, InBetween<NK, EK, OldVersion, NewVersion>> {
        Ok(())
    }
}

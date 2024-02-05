use crate::{Direction, Downcast, Key, SchemaExt, SchemaResult};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct EdgeRef<'a, NK, EK, S>
where
    NK: Key,
    EK: Key,
    S: SchemaExt<NK, EK>,
{
    pub(crate) weight: &'a S::E,
    pub(crate) source: NK,
    pub(crate) target: NK,
    pub(crate) direction: Direction,
}

impl<'a, NK, EK, S> EdgeRef<'a, NK, EK, S>
where
    NK: Key,
    EK: Key,
    S: SchemaExt<NK, EK>,
{
    pub fn get_weight(&self) -> &'a S::E {
        self.weight
    }

    pub fn get_weight_downcast<E>(&self) -> SchemaResult<&'a E, NK, EK, S>
    where
        S::E: Downcast<'a, NK, EK, &'a E, S>,
    {
        self.weight.downcast()
    }

    pub fn get_source(&self) -> NK {
        self.source
    }

    pub fn get_target(&self) -> NK {
        self.target
    }

    pub fn get_outer(&self) -> NK {
        match self.direction {
            Direction::Incoming => self.get_source(),
            Direction::Outgoing => self.get_target(),
        }
    }

    pub fn get_inner(&self) -> NK {
        match self.direction {
            Direction::Incoming => self.get_target(),
            Direction::Outgoing => self.get_source(),
        }
    }

    pub fn get_direction(&self) -> Direction {
        self.direction
    }
}

impl<'a, NK, EK, S> Deref for EdgeRef<'a, NK, EK, S>
where
    NK: Key,
    EK: Key,
    S: SchemaExt<NK, EK>,
{
    type Target = S::E;
    fn deref(&self) -> &Self::Target {
        &self.weight
    }
}

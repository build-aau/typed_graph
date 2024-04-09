use std::fmt::{Debug, Display};

use crate::*;

#[derive(Debug, Clone)]
pub enum EitherVersion<Old, New> {
    Old(Old),
    New(New),
}

impl<Old, New> From<EitherVersion<Old, New>> for Option<New>
where
    Option<New>: From<Old>,
{
    fn from(value: EitherVersion<Old, New>) -> Self {
        match value {
            EitherVersion::Old(o) => o.into(),
            EitherVersion::New(n) => Some(n),
        }
    }
}

impl<NK: Key, Old: NodeExt<NK>, New: NodeExt<NK>> NodeExt<NK> for EitherVersion<Old, New> {}
impl<EK: Key, Old: EdgeExt<EK>, New: EdgeExt<EK>> EdgeExt<EK> for EitherVersion<Old, New> {}

impl<K: Key, Old: Id<K>, New: Id<K>> Id<K> for EitherVersion<Old, New> {
    fn get_id(&self) -> K {
        match self {
            EitherVersion::New(v) => v.get_id(),
            EitherVersion::Old(v) => v.get_id(),
        }
    }

    fn set_id(&mut self, new_id: K) {
        match self {
            EitherVersion::New(v) => v.set_id(new_id),
            EitherVersion::Old(v) => v.set_id(new_id),
        }
    }
}

impl<Old: Copy, New: Copy> Copy for EitherVersion<Old, New> {}

impl<Old: Typed, New: Typed> Typed for EitherVersion<Old, New> {
    type Type = EitherVersion<<Old as Typed>::Type, <New as Typed>::Type>;
    fn get_type(&self) -> Self::Type {
        match self {
            EitherVersion::New(new) => EitherVersion::New(new.get_type()),
            EitherVersion::Old(old) => EitherVersion::Old(old.get_type()),
        }
    }
}

impl<Old: Display, New: Display> Display for EitherVersion<Old, New> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EitherVersion::New(n) => n.fmt(f),
            EitherVersion::Old(n) => n.fmt(f),
        }
    }
}

impl<Old, New, OldType, NewType> PartialEq<EitherVersion<OldType, NewType>> for EitherVersion<Old, New>
where
    Old: PartialEq<OldType>,
    New: PartialEq<NewType>,
{
    fn eq(&self, other: &EitherVersion<OldType, NewType>) -> bool {
        match (self, other) {
            (EitherVersion::Old(old), EitherVersion::Old(ty)) => old == ty,
            (EitherVersion::New(new), EitherVersion::New(ty)) => new == ty,
            _ => false,
        }
    }
}

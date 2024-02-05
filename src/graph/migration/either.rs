use std::fmt::{Debug, Display};

use crate::*;

#[derive(Debug, Clone)]
pub enum Either<Old, New> {
    Old(Old),
    New(New),
}

impl<Old, New> From<Either<Old, New>> for Option<New>
where
    Option<New>: From<Old>,
{
    fn from(value: Either<Old, New>) -> Self {
        match value {
            Either::Old(o) => o.into(),
            Either::New(n) => Some(n),
        }
    }
}

impl<NK: Key, Old: NodeExt<NK>, New: NodeExt<NK>> NodeExt<NK> for Either<Old, New> {}
impl<EK: Key, Old: EdgeExt<EK>, New: EdgeExt<EK>> EdgeExt<EK> for Either<Old, New> {}

impl<K: Key, Old: Id<K>, New: Id<K>> Id<K> for Either<Old, New> {
    fn get_id(&self) -> K {
        match self {
            Either::New(v) => v.get_id(),
            Either::Old(v) => v.get_id(),
        }
    }

    fn set_id(&mut self, new_id: K) {
        match self {
            Either::New(v) => v.set_id(new_id),
            Either::Old(v) => v.set_id(new_id),
        }
    }
}

impl<Old: Copy, New: Copy> Copy for Either<Old, New> {}

impl<Old: Typed, New: Typed> Typed for Either<Old, New> {
    type Type = Either<<Old as Typed>::Type, <New as Typed>::Type>;
    fn get_type(&self) -> Self::Type {
        match self {
            Either::New(new) => Either::New(new.get_type()),
            Either::Old(old) => Either::Old(old.get_type()),
        }
    }
}

impl<Old: Display, New: Display> Display for Either<Old, New> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Either::New(n) => n.fmt(f),
            Either::Old(n) => n.fmt(f),
        }
    }
}

impl<Old, New, OldType, NewType> PartialEq<Either<OldType, NewType>> for Either<Old, New>
where
    Old: PartialEq<OldType>,
    New: PartialEq<NewType>,
{
    fn eq(&self, other: &Either<OldType, NewType>) -> bool {
        match (self, other) {
            (Either::Old(old), Either::Old(ty)) => old == ty,
            (Either::New(new), Either::New(ty)) => new == ty,
            _ => false,
        }
    }
}

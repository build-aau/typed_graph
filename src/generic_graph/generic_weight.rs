use std::ops::{DerefMut, Deref};
use serde::{Serialize, Deserialize};
use crate::{Id, Typed, Key};
use super::GenericTypeIdentifier;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct GenericWeight<K: Key, T: GenericTypeIdentifier>((K, T));

impl<K: Key, T: GenericTypeIdentifier> Deref for GenericWeight<K, T> {
    type Target = (K, T);
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: Key, T: GenericTypeIdentifier> DerefMut for GenericWeight<K, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K: Key, T: GenericTypeIdentifier> From<(K, T)> for GenericWeight<K, T> {
    fn from(value: (K, T)) -> Self {
        GenericWeight(value)
    }
}

impl<K: Key, T: GenericTypeIdentifier> Id<K> for GenericWeight<K, T> {
    fn get_id(&self) -> K {
        self.0.0
    }

    fn set_id(&mut self, new_id: K) {
        self.0.0 = new_id;
    }
}

impl<K: Key, T: GenericTypeIdentifier> PartialEq<T> for GenericWeight<K, T> {
    fn eq(&self, other: &T) -> bool {
        &self.0.1 == other
    }
}

impl<K: Key, T: GenericTypeIdentifier> Typed for GenericWeight<K, T> {
    type Type = T;

    fn get_type(&self) -> Self::Type {
        self.0.1.clone()
    }
}
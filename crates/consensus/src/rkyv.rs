//! rkyv utilities for consensus types.

#[cfg(not(feature = "std"))]
use once_cell::race::OnceBox as OnceLock;
#[cfg(feature = "std")]
use std::sync::OnceLock;

use rkyv::{
    rancor::Fallible,
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Serialize,
};

/// Rkyv wrapper to archive an `OnceLock<T>` as an `Option<T>`.
///
/// During archiving, we extract the value from the OnceLock. During deserialization,
/// we reconstruct the OnceLock with the value pre-populated if present.
#[derive(Debug)]
pub struct ArchiveOnceLockAsOption;

impl<T> ArchiveWith<OnceLock<T>> for ArchiveOnceLockAsOption
where
    T: Archive + Clone,
{
    type Archived = <Option<T> as Archive>::Archived;
    type Resolver = <Option<T> as Archive>::Resolver;

    fn resolve_with(
        field: &OnceLock<T>,
        resolver: Self::Resolver,
        out: rkyv::Place<Self::Archived>,
    ) {
        // Get the value from OnceLock, cloning if present
        let value = field.get().cloned();
        Archive::resolve(&value, resolver, out);
    }
}

impl<T, S> SerializeWith<OnceLock<T>, S> for ArchiveOnceLockAsOption
where
    T: Serialize<S> + Clone,
    S: Fallible + rkyv::ser::Writer + ?Sized,
{
    fn serialize_with(field: &OnceLock<T>, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        // Get the value from OnceLock
        let value = field.get().cloned();
        Serialize::serialize(&value, serializer)
    }
}

impl<T, D> DeserializeWith<rkyv::Archived<Option<T>>, OnceLock<T>, D> for ArchiveOnceLockAsOption
where
    T: Archive,
    T::Archived: rkyv::Deserialize<T, D>,
    D: Fallible + ?Sized,
{
    fn deserialize_with(
        field: &rkyv::Archived<Option<T>>,
        deserializer: &mut D,
    ) -> Result<OnceLock<T>, D::Error> {
        use rkyv::Deserialize;
        let value: Option<T> = field.deserialize(deserializer)?;
        let lock = OnceLock::new();
        if let Some(v) = value {
            lock.get_or_init(|| v);
        }
        Ok(lock)
    }
}

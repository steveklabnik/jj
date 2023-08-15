//! This file stores the [`WorkingCopyStore`] interface and it's associated
//! [`CachedWorkingCopy`] trait.
//!
//! These must be implemented for Virtual Filesystems such as [EdenFS]
//! to allow cheaper working copy materializations, these traits are used for the `jj run`
//! implementation.
//!
//!
//! [EdenFS]: www.github.com/facebook/sapling/main/blob/eden/fs

use std::{any::Any, path::PathBuf};

use crate::commit::Commit;

/// A `CachedWorkingCopy` is a working copy which is managed by the `WorkingCopyStore`.
pub trait CachedWorkingCopy: Send + Sync {
    /// Does the working copy exist.
    fn exists(&self) -> bool;

    /// The output path for the this `WorkingCopy`.
    /// May look something like `.jj/run/default/{id}/output`
    fn output_path(&self) -> PathBuf;
}

/// A `WorkingCopyStore` manages the working copies on disk for `jj run`.
/// It's an ideal extension point for an virtual filesystem, as they ease the creation of
/// working copies.
pub trait WorkingCopyStore: Send + Sync {
    /// Return `self` as `Any` to allow trait upcasting.
    fn as_any(&self) -> &dyn Any;

    /// The name of the backend, determines how it actually interacts with files.
    fn name(&self) -> &'static str;

    /// Get existing or create `Stores` for `revisions`.
    fn get_or_create_working_copies<'a>(
        &mut self,
        revisions: Vec<Commit>,
    ) -> Vec<Box<dyn CachedWorkingCopy>>;

    /// Are any `Stores` available.
    fn has_stores(&self) -> bool;
}

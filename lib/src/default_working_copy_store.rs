// Copyright 2023 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This file contains the default implementation of the `WorkingCopyStore` for both the Git and
//! Native Backend.
//!
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use itertools::Itertools;

use crate::commit::Commit;
use crate::local_working_copy::TreeState;
use crate::store::Store;
use crate::working_copy_store::{CachedWorkingCopy, WorkingCopyStore};

/// A thin wrapper over a `TreeState` for now.
#[derive(Debug)]
struct StoredWorkingCopy {
    /// Current state of the associated [`WorkingCopy`].
    state: TreeState,
    /// The output path for tools, which do not specify a location.
    /// Like C(++) Compilers, scripts and more.
    /// TODO: Is this necessary?
    output_path: PathBuf,
    /// Path to the associated working copy.
    working_copy_path: PathBuf,
    /// Path to the associated tree state.
    state_path: PathBuf,
}

impl StoredWorkingCopy {
    /// Set up a `StoredWorkingCopy`. It's assumed that all paths exist on disk.
    fn create(
        store: Arc<Store>,
        output_path: PathBuf,
        working_copy_path: PathBuf,
        state_path: PathBuf,
    ) -> Self {
        // Load the tree for our commit.
        let state = TreeState::load(store, working_copy_path, state_path).unwrap();
        Self {
            state,
            output_path,
            working_copy_path,
            state_path,
        }
    }
}

/// The default [`WorkingCopyStore`] for both the Git and native backend.
#[derive(Debug, Default)]
pub struct DefaultWorkingCopyStore {
    /// Where the working copies are stored, in this case `.jj/run/default/`
    stored_paths: PathBuf,
    /// All managed working copies
    stored_working_copies: Vec<StoredWorkingCopy>,
}

/// Creates the required directories for a StoredWorkingCopy.
/// Returns a tuple of (`output_dir`, `working_copy` and `state`).
fn create_working_copy_paths(path: PathBuf) -> Result<(PathBuf, PathBuf, PathBuf), std::io::Error> {
    let output = path.join("output");
    let working_copy = path.join("working_copy");
    let state = path.join("state");
    std::fs::create_dir(output)?;
    std::fs::create_dir(working_copy)?;
    std::fs::create_dir(state)?;
    Ok((output, working_copy, state))
}

impl DefaultWorkingCopyStore {
    fn name() -> &'static str {
        "default"
    }

    fn init(dot_dir: &Path) -> Self {
        let stored_paths = dot_dir.join("run");
        // If the toplevel dir doesn't exist, create it.
        if !stored_paths.exists() {
            std::fs::create_dir(stored_paths).expect("shouldn't fail");
        }

        Self {
            stored_paths,
            ..Default::default()
        }
    }

    fn create_working_copies(
        &mut self,
        revisions: Vec<Commit>,
    ) -> Result<Vec<Box<dyn CachedWorkingCopy>>, std::io::Error> {
        let store = revisions
            .first()
            .expect("revisions shouldn't be empty")
            .store();
        // Use the tree id for a unique directory.
        for rev in revisions {
            let tree_id = rev.tree_id().to_wc_name();
            let path: PathBuf = self.stored_paths.join(tree_id);
            // Create a dir under `.jj/run/`.
            std::fs::create_dir(path)?;
            // And the additional directories.
            let (output, working_copy_path, state) = create_working_copy_paths(path)?;
            let cached_wc =
                StoredWorkingCopy::create(store.clone(), output, working_copy_path, state);
            self.stored_working_copies.push(cached_wc);
        }
        Ok(self.stored_working_copies.clone())
    }
}

impl WorkingCopyStore for DefaultWorkingCopyStore {
    fn as_any(&self) -> dyn std::any::Any {
        Box::new(&self)
    }

    fn name(&self) -> &'static str {
        Self::name()
    }

    fn get_or_create_working_copies(
        &mut self,
        revisions: Vec<Commit>,
    ) -> Vec<Box<dyn CachedWorkingCopy>> {
        let new_ids = revisions
            .into_iter()
            .map(|rev| rev.tree_id().to_wc_name())
            .collect_vec();

        // check if we're the initial invocation.
        let needs_new = if !self.stored_working_copies.is_empty() {
            let mut res;
            for wc in &self.stored_working_copies {
                if !new_ids.contains(&wc.working_copy_path.to_str().unwrap().to_owned()) {
                    res &= true;
                }
            }
            false
        } else {
            true
        };

        let result = if !needs_new {
            self.stored_working_copies.to_vec()
        } else {
            self.create_working_copies(revisions).ok().unwrap()
        };

        result
    }

    fn has_stores(&self) -> bool {
        !self.stored_working_copies.is_empty()
    }
}

impl CachedWorkingCopy for StoredWorkingCopy {
    fn exists(&self) -> bool {
        self.working_copy_path.exists() && self.state_path.exists()
    }

    fn output_path(&self) -> PathBuf {
        self.output_path
    }
}

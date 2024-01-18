// Copyright 2024 The Jujutsu Authors
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

use std::fmt::Debug;

use clap::Subcommand;

use crate::cli_util::{CommandError, CommandHelper};
use crate::commands::gerrit;
use crate::ui::Ui;

/// Interact with Gerrit Code Review.
#[derive(Subcommand, Clone, Debug)]
pub enum GerritCommand {
    /// Send in revisions to Gerrit for code review; this is the primary way to
    /// create changes or update changes for code review.
    ///
    /// This command modifies each commit in the revset to include a `Change-Id`
    /// footer in its commit message if one does not already exist. This ID is
    /// NOT compatible with jj IDs, and is a Gerrit-specific ID.
    ///
    /// Note: this command takes a revset as an argument, and will send in all
    /// revisions in the revset to Gerrit appropriately, so you may post trees
    /// or ranges of commits to Gerrit for review.
    Send(gerrit::send::SendArgs),
}

pub fn cmd_gerrit(
    ui: &mut Ui,
    command: &CommandHelper,
    subcommand: &GerritCommand,
) -> Result<(), CommandError> {
    match subcommand {
        GerritCommand::Send(review) => gerrit::send::cmd_send(ui, command, review),
    }
}

mod send;

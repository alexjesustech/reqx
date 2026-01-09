// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Shell completion generation

use super::{Cli, Shell};
use clap::CommandFactory;
use clap_complete::{generate, shells};
use std::io;

pub fn execute(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    match shell {
        Shell::Bash => generate(shells::Bash, &mut cmd, name, &mut io::stdout()),
        Shell::Zsh => generate(shells::Zsh, &mut cmd, name, &mut io::stdout()),
        Shell::Fish => generate(shells::Fish, &mut cmd, name, &mut io::stdout()),
        Shell::PowerShell => generate(shells::PowerShell, &mut cmd, name, &mut io::stdout()),
    }
}

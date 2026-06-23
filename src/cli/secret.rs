// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `reqx secret` — manage the encrypted secret store.

use crate::cli::SecretAction;
use crate::secret;
use anyhow::Result;
use std::io::Read;

pub fn execute(action: SecretAction) -> Result<()> {
    match action {
        SecretAction::Set { name, env, value } => {
            let value = match value {
                Some(v) => v,
                None => {
                    let mut buf = String::new();
                    std::io::stdin().read_to_string(&mut buf)?;
                    buf.trim_end_matches(['\n', '\r']).to_string()
                }
            };
            secret::set(&env, &name, &value)?;
            println!("secret '{name}' saved to the '{env}' store");
        }
        SecretAction::List { env } => {
            for n in secret::names(&env)? {
                println!("{n}");
            }
        }
        SecretAction::Rm { name, env } => {
            if secret::remove(&env, &name)? {
                println!("removed secret '{name}' from the '{env}' store");
            } else {
                println!("no secret '{name}' in the '{env}' store");
            }
        }
    }
    Ok(())
}

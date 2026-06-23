// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Encrypted, git-friendly secret store.
//!
//! Secrets live in `.reqx/secrets/<env>.enc` as an `age` passphrase-encrypted
//! blob — committable, but useless without the passphrase, which is read from
//! the `REQX_SECRET_KEY` environment variable and never written to disk.
//! Values are referenced from `.reqx` files as `{{secret.NAME}}`.

use age::secrecy::SecretString;
use anyhow::{anyhow, bail, Context, Result};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const ENV_KEY: &str = "REQX_SECRET_KEY";

pub type Secrets = BTreeMap<String, String>;

fn passphrase() -> Result<SecretString> {
    let p = std::env::var(ENV_KEY)
        .map_err(|_| anyhow!("set {ENV_KEY} to the passphrase that encrypts the secret store"))?;
    if p.is_empty() {
        bail!("{ENV_KEY} is set but empty");
    }
    Ok(SecretString::from(p))
}

fn store_path(env: &str) -> PathBuf {
    Path::new(".reqx/secrets").join(format!("{env}.enc"))
}

/// Encrypt `plain` with a passphrase (age scrypt recipient).
fn encrypt(plain: &str, pass: &SecretString) -> Result<Vec<u8>> {
    let recipient = age::scrypt::Recipient::new(pass.clone());
    let encryptor = age::Encryptor::with_recipients(std::iter::once(&recipient as _))
        .context("failed to build encryptor")?;
    let mut out = Vec::new();
    let mut writer = encryptor.wrap_output(&mut out)?;
    writer.write_all(plain.as_bytes())?;
    writer.finish()?;
    Ok(out)
}

/// Decrypt an age passphrase blob.
fn decrypt(data: &[u8], pass: &SecretString) -> Result<String> {
    let identity = age::scrypt::Identity::new(pass.clone());
    let decryptor = age::Decryptor::new(data).context("invalid secret store")?;
    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as _))
        .context("failed to decrypt secret store (wrong passphrase?)")?;
    let mut plain = String::new();
    reader.read_to_string(&mut plain)?;
    Ok(plain)
}

/// Load and decrypt the secret store for `env` (empty if none exists).
pub fn load(env: &str) -> Result<Secrets> {
    let path = store_path(env);
    if !path.exists() {
        return Ok(Secrets::new());
    }
    let encrypted = std::fs::read(&path)
        .with_context(|| format!("failed to read secret store {}", path.display()))?;
    let pass = passphrase()?;
    let plain = decrypt(&encrypted, &pass)?;
    Ok(toml::from_str(&plain).unwrap_or_default())
}

/// Encrypt and write the secret store for `env`.
pub fn save(env: &str, secrets: &Secrets) -> Result<()> {
    let pass = passphrase()?;
    let plain = toml::to_string(secrets).context("failed to serialise secrets")?;
    let encrypted = encrypt(&plain, &pass)?;
    let path = store_path(env);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, encrypted)
        .with_context(|| format!("failed to write secret store {}", path.display()))?;
    Ok(())
}

/// Set (insert/replace) a secret.
pub fn set(env: &str, name: &str, value: &str) -> Result<()> {
    let mut s = load(env)?;
    s.insert(name.to_string(), value.to_string());
    save(env, &s)
}

/// Remove a secret; returns whether it existed.
pub fn remove(env: &str, name: &str) -> Result<bool> {
    let mut s = load(env)?;
    let existed = s.remove(name).is_some();
    if existed {
        save(env, &s)?;
    }
    Ok(existed)
}

/// Sorted secret names (never the values).
pub fn names(env: &str) -> Result<Vec<String>> {
    Ok(load(env)?.into_keys().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trips() {
        let pass = SecretString::from("correct horse battery staple".to_string());
        let blob = encrypt("hello = \"world\"", &pass).unwrap();
        assert_ne!(
            blob, b"hello = \"world\"",
            "ciphertext must differ from plaintext"
        );
        let back = decrypt(&blob, &pass).unwrap();
        assert_eq!(back, "hello = \"world\"");
    }

    #[test]
    fn wrong_passphrase_fails() {
        let pass = SecretString::from("right".to_string());
        let blob = encrypt("x = \"1\"", &pass).unwrap();
        let wrong = SecretString::from("wrong".to_string());
        assert!(decrypt(&blob, &wrong).is_err());
    }
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Process exit codes — the CI contract documented in the README.
//!
//! `reqx run` maps outcomes to these so pipelines can branch on them:
//! `0` all passed · `1` assertion failed · `2` execution error ·
//! `3` parse error · `4` config error.

pub mod exit {
    /// Every request passed its assertions.
    pub const OK: i32 = 0;
    /// At least one assertion failed (the request executed fine).
    pub const ASSERTION_FAILED: i32 = 1;
    /// A request could not be executed (network/HTTP/interpolation error).
    pub const EXECUTION_ERROR: i32 = 2;
    /// A `.reqx` file could not be parsed.
    pub const PARSE_ERROR: i32 = 3;
    /// Configuration or environment could not be loaded.
    pub const CONFIG_ERROR: i32 = 4;
}

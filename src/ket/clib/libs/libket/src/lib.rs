// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

//#![warn(missing_docs)]
#![doc(html_favicon_url = "https://quantumket.org/_static/favicon.ico")]

//! # Libket Quantum Programming Library
//!
//! The Libket library provides a set of tools for quantum programming in Rust.
//! It serves as the runtime library for the Python-embedded quantum programming language Ket.
//!
//! **Note:** For more information about the Ket programming language,
//! please visit <https://quantumket.org>.
//!
//! ## Usage
//!
//! To use this library, add the following line to your `Cargo.toml` file:
//!
//! ```text
//! [dependencies]
//! libket = "0.4.0"
//! ```
//!
//! Additionally, you may need to include the following dependencies for quantum code
//! serialization/deserialization and the KBW quantum computer simulator:
//!
//! ```text
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! kbw = "0.2.0"
//! ```

pub mod c_api;
pub mod error;
pub mod execution;
pub mod ir;
pub mod objects;
pub mod process;
pub mod qasmv2;
pub mod zx;

pub use execution::*;
pub use ir::*;
pub use objects::*;
pub use process::*;

// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

//! C API Wrapper
//!
//! This module provides a wrapper C for the Rust functions.
//!
//! ## Error Handling
//!
//! The `ket_error_message` function allows retrieving error messages associated with error codes.
//! Given an error code, it returns the corresponding error message string.
//!
//! # Safety
//!
//! Care should be taken when using C functions and data structures.

use env_logger::Builder;
use log::LevelFilter;

use crate::error::KetError;

pub mod error;
pub mod objects;
pub mod process;

/// Sets the log level for Libket.
#[no_mangle]
pub extern "C" fn ket_set_log_level(level: u32) -> i32 {
    let level = match level {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => LevelFilter::max(),
    };

    Builder::new().filter_level(level).init();

    KetError::Success.error_code()
}

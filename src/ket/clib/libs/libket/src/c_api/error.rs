// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

//! Error handler for the C API.

use log::trace;

use crate::error::{KetError, Result};

/// Retrieves the error message associated with an error code.
///
/// This function takes an error code as input and returns a pointer to
/// the error message string. The `size` parameter is a mutable reference
/// that will be updated with the length of the error message string.
///
/// # Arguments
///
/// * `error_code` - \[in\] The error code for which to retrieve the error message.
/// * `buffer` - \[out\] A mutable pointer to store the error message string.
/// * `buffer_size` - \[in\] The size of the buffer provided.
/// * `write_size` - \[out\] A mutable reference to store the size of the error message.
///
/// # Safety
///
/// This function is marked as unsafe because it deals with raw pointers.
#[no_mangle]
pub unsafe extern "C" fn ket_error_message(
    error_code: i32,
    buffer: *mut u8,
    buffer_size: usize,
    write_size: &mut usize,
) -> i32 {
    let msg = unsafe { KetError::from_error_code(error_code) }.to_string();

    trace!(
        "ket_error_message( error_code={}, buffer_size={}, msg={} )",
        error_code,
        buffer_size,
        msg,
    );

    let msg = msg.as_bytes();
    *write_size = msg.len();

    if buffer_size >= *write_size {
        let buffer = unsafe { std::slice::from_raw_parts_mut(buffer, buffer_size) };
        buffer[..*write_size].copy_from_slice(msg);
        0
    } else {
        1
    }
}

/// Wraps the error handling logic for FFI functions.
///
/// This function is used to handle errors returned from FFI functions.
/// It takes a `Result` as input, where the `Ok` variant indicates success and
/// the `Err` variant represents an error. It returns the corresponding error
/// code based on the success or error case.
///
/// # Arguments
///
/// * `error` - The `Result` representing the success or failure of an operation.
///
/// # Returns
///
/// The error code. If the operation succeeded, it returns the error code for
/// success (`0`). If an error occurred, it returns the corresponding error code.
pub(super) fn wrapper(error: Result<()>) -> i32 {
    match error {
        Ok(_) => KetError::Success.error_code(),
        Err(error) => error.error_code(),
    }
}

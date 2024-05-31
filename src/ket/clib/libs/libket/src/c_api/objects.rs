// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

//! C API for the objects module.

use log::trace;

use crate::{error::KetError, Process};

/// Retrieves the status of a qubit in the `Process` instance.
///
/// # Arguments
///
/// * `process` -  \[in\] A reference to the `Process` instance.
/// * `qubit` -  \[in\] The index of the qubit to query.
/// * `allocated` -  \[out\] A mutable pointer to a `bool` indicating if the qubit is allocated.
/// * `measured` -  \[out\] A mutable pointer to a `bool` indicating if the qubit is measured.
///
/// # Returns
///
/// An integer representing the error code. `0` indicates success.
#[no_mangle]
pub extern "C" fn ket_process_get_qubit_status(
    process: &Process,
    qubit: usize,
    allocated: &mut bool,
    measured: &mut bool,
) -> i32 {
    let qubit_status = process.get_qubit_status(qubit);
    *allocated = qubit_status.allocated;
    *measured = qubit_status.measured;

    trace!("{:?}", qubit_status);

    KetError::Success.error_code()
}

/// Retrieves the measurement result from the `Process` instance.
///
/// # Arguments
///
/// * `process` -  \[in\] A reference to the `Process` instance.
/// * `index` -  \[in\] The index of the measurement to query.
/// * `available` -  \[out\] A mutable pointer to a `bool` indicating if the result is available.
/// * `result` -  \[out\] A mutable pointer to a `u64` storing the measurement result.
///
/// # Returns
///
/// An integer representing the error code. `0` indicates success.
#[no_mangle]
pub extern "C" fn ket_process_get_measurement(
    process: &Process,
    index: usize,
    available: &mut bool,
    result: &mut u64,
) -> i32 {
    let measurement = process.get_measurement(index);
    if let Some(measurement) = measurement.result {
        *result = measurement;
        *available = true;
    } else {
        *available = false;
    }

    trace!("{:?}", measurement);

    KetError::Success.error_code()
}

/// Retrieves the expected value from the `Process` instance.
///
/// # Arguments
///
/// * `process` -  \[in\] A reference to the `Process` instance.
/// * `index` -  \[in\] The index of the expected value to query.
/// * `available` -  \[out\] A mutable pointer to a `bool` indicating if the result is available.
/// * `result` -  \[out\] A mutable pointer to a `f64` storing the expected value.
///
/// # Returns
///
/// An integer representing the error code. `0` indicates success.
#[no_mangle]
pub extern "C" fn ket_process_get_exp_value(
    process: &Process,
    index: usize,
    available: &mut bool,
    result: &mut f64,
) -> i32 {
    let exp_value = process.get_exp_value(index);
    if let Some(exp_value) = exp_value.result {
        *result = exp_value;
        *available = true;
    } else {
        *available = false;
    }

    KetError::Success.error_code()
}

/// Retrieves the sample data from the `Process` instance.
///
/// # Arguments
///
/// * `process` -  \[in\] A reference to the `Process` instance.
/// * `index` -  \[in\] The index of the sample to query.
/// * `available` -  \[out\] A mutable pointer to a `bool` indicating if the result is available.
/// * `result` -  \[out\] A mutable pointer to the array of `u64` storing the sample data.
/// * `count` -  \[out\] A mutable pointer to the array of `u64` storing the sample counts.
/// * `size` -  \[out\] A mutable pointer to the size of the sample data arrays.
///
/// # Returns
///
/// An integer representing the error code. `0` indicates success.
#[no_mangle]
pub extern "C" fn ket_process_get_sample(
    process: &Process,
    index: usize,
    available: &mut bool,
    result: &mut *const u64,
    count: &mut *const u64,
    size: &mut usize,
) -> i32 {
    let sample = process.get_sample(index);
    if let Some(sample) = sample.result.as_ref() {
        *result = sample.0.as_ptr();
        *count = sample.1.as_ptr();
        *size = sample.0.len();
        *available = true;
    } else {
        *available = false;
    }

    KetError::Success.error_code()
}

/// Retrieves the size of the dump data from the `Process` instance.
///
/// # Arguments
///
/// * `process` -  \[in\] A reference to the `Process` instance.
/// * `index` -  \[in\] The index of the dump to query.
/// * `available` -  \[out\] A mutable pointer to a `bool` indicating if the result is available.
/// * `size` -  \[out\] A mutable pointer to the size of the basis states in the dump.
///
/// # Returns
///
/// An integer representing the error code. `0` indicates success.
#[no_mangle]
pub extern "C" fn ket_process_get_dump_size(
    process: &Process,
    index: usize,
    available: &mut bool,
    size: &mut usize,
) -> i32 {
    let dump = process.get_dump(index);
    if let Some(dump) = dump.result.as_ref() {
        *size = dump.basis_states.len();
        *available = true;
    } else {
        *available = false;
    }

    KetError::Success.error_code()
}

/// Retrieves the dump data from the `Process` instance.
///
/// # Arguments
///
/// * `process` -  \[in\] A reference to the `Process` instance.
/// * `index` -  \[in\] The index of the dump to query.
/// * `iterator` -  \[in\] The iterator for accessing individual basis states in the dump.
/// * `basis_state` -  \[out\] A mutable pointer to the array of `u64` storing the basis state.
/// * `basis_state_size` -  \[out\] A mutable pointer to the size of the basis state array.
/// * `amplitude_real` -  \[out\] A mutable pointer to the real part of the amplitude.
/// * `amplitude_imag` -  \[out\] A mutable pointer to the imaginary part of the amplitude.
///
/// # Returns
///
/// An integer representing the error code. `0` indicates success.
///
/// # Safety
///
/// This function is marked as unsafe due to the use of raw pointers.
#[no_mangle]
pub unsafe extern "C" fn ket_process_get_dump(
    process: &Process,
    index: usize,
    iterator: usize,
    basis_state: &mut *const u64,
    basis_state_size: &mut usize,
    amplitude_real: &mut f64,
    amplitude_imag: &mut f64,
) -> i32 {
    let dump = process.get_dump(index).result.as_ref().unwrap();
    let state = dump.basis_states[iterator].as_ptr();
    let size = dump.basis_states[iterator].len();
    *basis_state = state;
    *basis_state_size = size;
    *amplitude_real = dump.amplitudes_real[iterator];
    *amplitude_imag = dump.amplitudes_imag[iterator];

    KetError::Success.error_code()
}

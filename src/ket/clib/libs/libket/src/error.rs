// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

//! This module defines the error types used in the quantum programming library.

use std::result;
/// Enumeration of possible errors in the quantum processing library.
#[derive(thiserror::Error, Debug, Clone, Copy)]
#[repr(i32)]
#[allow(missing_docs)]
pub enum KetError {
    #[error("The operation completed successfully.")]
    Success,

    #[error("Cannot set a qubit as a control twice.")]
    ControlTwice,

    #[error("Requested data is not available.")]
    DataNotAvailable,

    #[error("Cannot operate with a deallocated qubit.")]
    DeallocatedQubit,

    #[error("The provided qubit index is out of bounds.")]
    QubitIndexOutOfBounds,

    #[error("The number of qubits exceeds the allowed limit.")]
    NumberOfQubitsExceeded,

    #[error("No inverse scope to end.")]
    NoAdj,

    #[error("No control scope to end.")]
    NoCtrl,

    #[error("Cannot apply a non-gate instruction within a controlled or inverse scope.")]
    NonGateInstructionInAdj,

    #[error("A qubit cannot be both targeted and controlled at the same time.")]
    TargetInControl,

    #[error("Cannot append statements to a terminated process.")]
    ProcessReadyToExecute,

    #[error("Result does not contain the expected number of values.")]
    UnexpectedResultData,

    #[error("Cannot dump qubits (feature disabled).")]
    DumpNotAllowed,

    #[error("Cannot calculate the expected value (feature disabled).")]
    ExpValueNotAllowed,

    #[error("Cannot sampling qubits (feature disabled).")]
    SampleNotAllowed,

    #[error("Cannot measure qubit (feature disabled).")]
    MeasureNotAllowed,

    #[error("The provided buffer is too small.")]
    SmallBufferSize,

    #[error("The generated QASMv2 is not syntactically correct")]
    InvalidQASM,

    #[error("An undefined error occurred.")]
    UndefinedError,

    #[error("Provided gate not supported")]
    GateNotSupported,

    #[error("Process is not configured")]
    UnconfiguredProcess,

    #[error("Classic bits are not implemented")]
    BitsNotSupported,

    #[error("Gate to be exported is not supported")]
    UnsuportedGateExport,

    #[error(
        "Ket can only export to qelib gate set, as it internally represents U gates as RZ RY RZ"
    )]
    PureQASMGateExportError,
}

/// Alias for a `Result` type using `KetError` as the error variant.
pub type Result<T> = result::Result<T, KetError>;

impl KetError {
    /// Returns the numeric error code associated with the error variant.
    pub fn error_code(&self) -> i32 {
        *self as i32
    }

    /// Converts an error code to a `KetError` variant.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it assumes that the error code is valid.
    pub unsafe fn from_error_code(error_code: i32) -> KetError {
        unsafe { std::mem::transmute(error_code) }
    }
}

#[cfg(test)]
mod tests {
    use super::KetError;

    #[test]
    fn success_is_zero() {
        assert!(KetError::Success.error_code() == 0)
    }

    #[test]
    fn print_error_code() {
        let mut error_code = 0;
        loop {
            let error = unsafe { KetError::from_error_code(error_code) };
            let error_str = format!("{:#?}", error);
            let error_str = error_str
                .split_inclusive(char::is_uppercase)
                .map(|part| {
                    let size = part.len();
                    let lest = part.chars().last().unwrap();
                    if size > 1 && char::is_uppercase(lest) {
                        format!("{}_{}", &part[..size - 1], lest)
                    } else {
                        String::from(part)
                    }
                })
                .collect::<Vec<String>>()
                .concat()
                .to_uppercase();
            println!("#define KET_{} {}", error_str, error_code);

            if let KetError::UndefinedError = error {
                break;
            } else {
                error_code += 1;
            }
        }
    }
}

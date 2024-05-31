// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides structures that hold the status of qubits and measurements.

use crate::ir::{DumpData, PauliHamiltonian};

/// Represents the status of a qubit.
#[derive(Debug, Clone)]
pub struct QubitStatus {
    /// Indicates whether the qubit is allocated.
    pub allocated: bool,
    /// Indicates whether the qubit is measured.
    pub measured: bool,
}

impl Default for QubitStatus {
    fn default() -> Self {
        Self {
            allocated: true,
            measured: false,
        }
    }
}

/// Represents the result of a quantum measurement.
#[derive(Debug, Clone)]
pub struct Measurement {
    /// Measured qubits.
    pub qubits: Vec<usize>,
    /// Measurement result.
    pub result: Option<u64>,
}

/// Represents the result of an expected value calculation.
#[derive(Debug, Clone)]
pub struct ExpValue {
    /// Hamiltonian used for he expected value calculation.
    pub hamiltonian: PauliHamiltonian,
    /// Expected value.
    pub result: Option<f64>,
}

/// Represents the result of a quantum sampling operation.
#[derive(Debug, Clone)]
pub struct Sample {
    /// Measured qubits.
    pub qubits: Vec<usize>,
    /// Number of shots used for the sample.
    pub shots: u64,
    /// Sampled result.
    pub result: Option<(Vec<u64>, Vec<u64>)>,
}

/// Represents the result of a quantum state dump.
#[derive(Debug, Clone)]
pub struct Dump {
    /// Qubits used for the dump.
    pub qubits: Vec<usize>,
    /// State dump result.
    pub result: Option<DumpData>,
}

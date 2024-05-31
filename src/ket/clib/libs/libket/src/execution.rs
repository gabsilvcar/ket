// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides traits and structures for configuring quantum execution.

use crate::{
    ir::{DumpData, PauliHamiltonian, QuantumGate},
    Instruction, ResultData,
};

/// A trait defining the interface for live quantum execution.
pub trait LiveExecution {
    /// Allocates a qubit.
    fn alloc(&mut self, target: usize);

    /// Frees a previously allocated qubit.
    fn free(&mut self, target: usize);

    /// Applies a quantum gate.
    fn gate(&mut self, gate: &QuantumGate, target: usize, control: &[usize]);

    /// Measures the specified qubits.
    fn measure(&mut self, qubits: &[usize]) -> u64;

    /// Calculates the expected value for a given Hamiltonian.
    fn exp_value(&mut self, hamiltonian: &PauliHamiltonian) -> f64;

    /// Performs qubit sampling.
    fn sample(&mut self, qubits: &[usize], shots: u64) -> (Vec<u64>, Vec<u64>);

    /// Dumps the state of the specified qubits.
    fn dump(&mut self, qubits: &[usize]) -> DumpData;
}

/// Enum representing the status of a quantum execution.
pub enum ExecutionStatus {
    /// Execution is newly created.
    New,

    /// Execution is ready to start.
    Ready,

    /// Execution is currently running.
    Running,

    /// Execution has completed successfully.
    Completed,

    /// Execution encountered an error.
    Error,
}

/// A trait defining the interface for batch quantum execution.
pub trait BatchExecution {
    /// Submits a set of quantum instructions for execution.
    fn submit_execution(&mut self, instructions: &[Instruction]);

    /// Retrieves the result of the quantum execution.
    fn get_result(&mut self) -> ResultData;

    /// Retrieves the current status of the quantum execution.
    fn get_status(&self) -> ExecutionStatus;
}

/// Configuration struct for controlling quantum execution behavior.
pub struct Configuration {
    /// Flag indicating whether measurement operations are allowed.
    pub allow_measure: bool,

    /// Flag indicating whether sampling operations are allowed.
    pub allow_sample: bool,

    /// Flag indicating whether expected value calculations are allowed.
    pub allow_exp_value: bool,

    /// Flag indicating whether dumping quantum state is allowed.
    pub allow_dump: bool,

    /// Flag indicating whether the process remains valid after measurement.
    pub valid_after_measure: bool,

    /// Flag indicating whether execution continues after sampling.
    pub continue_after_sample: bool,

    /// Flag indicating whether execution continues after expected value calculations.
    pub continue_after_exp_value: bool,

    /// Flag indicating whether execution continues after quantum state dumps.
    pub continue_after_dump: bool,

    /// Flag indicating whether quantum gates should be decomposed.
    pub decompose: bool,

    /// Option for live quantum execution, specifying a processor implementing LiveExecution.
    pub live_quantum_execution: Option<Box<dyn LiveExecution>>,

    /// Option for batch quantum execution, specifying a processor implementing BatchExecution.
    pub batch_execution: Option<Box<dyn BatchExecution>>,

    /// Maximum number of qubits in the quantum process.
    pub num_qubits: usize,

    /// Optional timeout for quantum execution.
    pub execution_timeout: Option<f64>,

    pub optimize: bool
}

impl Configuration {
    /// Creates a new Configuration instance with the specified number of qubits.
    ///
    /// This is used for testing purpose only. The quantum executor must provide
    /// the `Configuration` instance.
    pub fn new(num_qubits: usize) -> Self {
        Self {
            allow_measure: true,
            allow_sample: true,
            allow_exp_value: true,
            allow_dump: true,
            valid_after_measure: true,
            continue_after_sample: true,
            continue_after_exp_value: true,
            continue_after_dump: true,
            decompose: false,
            live_quantum_execution: None,
            batch_execution: None,
            num_qubits,
            execution_timeout: None,
            optimize: true,
        }
    }
}

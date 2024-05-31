// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0
//! This module contains the IR (Intermediate Representation) of the quantum circuit.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Structure representing the data dumped from a quantum state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpData {
    /// Basis states of the quantum state.
    pub basis_states: Vec<Vec<u64>>,

    /// Real part of the amplitudes.
    pub amplitudes_real: Vec<f64>,

    /// Imaginary part of the amplitudes.
    pub amplitudes_imag: Vec<f64>,
}

/// Enum representing different angle representations for quantum gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Angle {
    /// Scalar angle.
    Scalar(f64),
    /// Pi fraction angle.
    PiFraction {
        /// Top part of the fraction.
        top: i32,
        /// Bottom part of the fraction.
        bottom: u32,
    },
}

impl Angle {
    /// Returns the inverse of the angle.
    pub fn inverse(&self) -> Angle {
        match self {
            Angle::Scalar(angle) => Angle::Scalar(-angle),
            Angle::PiFraction { top, bottom } => Angle::PiFraction {
                top: -top,
                bottom: *bottom,
            },
        }
    }

    /// Returns an angle representing π.
    pub fn pi() -> Angle {
        Angle::PiFraction { top: 1, bottom: 1 }
    }

    /// Return the angle as a scalar value.
    pub fn scalar(&self) -> f64 {
        match self {
            Angle::Scalar(angle) => *angle,
            Angle::PiFraction { top, bottom } => {
                let top = *top as f64;
                let bottom = *bottom as f64;
                top / bottom
            }
        }
    }
}

impl std::fmt::Display for Angle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Angle::Scalar(angle) => write!(f, "{}", angle),
            Angle::PiFraction { top, bottom } => {
                if *top == 1 {
                    write!(f, "pi/{}", bottom)
                } else {
                    write!(f, "({} * pi)/{}", top, bottom)
                }
            }
        }
    }
}

/// Enum representing various quantum gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantumGate {
    /// Pauli X gate.
    PauliX,
    /// Pauli Y gate.
    PauliY,
    /// Pauli Z gate.
    PauliZ,
    /// X-axis rotation gate.
    RotationX(Angle),
    /// Y-axis rotation gate.
    RotationY(Angle),
    /// Z-axis rotation gate.
    RotationZ(Angle),
    /// Phase shift gate.
    Phase(Angle),
    /// Hadamard gate.
    Hadamard,
}

impl QuantumGate {
    /// Returns the inverse of the quantum gate.
    pub fn inverse(&self) -> QuantumGate {
        match self {
            QuantumGate::RotationX(angle) => QuantumGate::RotationX(angle.inverse()),
            QuantumGate::RotationY(angle) => QuantumGate::RotationY(angle.inverse()),
            QuantumGate::RotationZ(angle) => QuantumGate::RotationZ(angle.inverse()),
            QuantumGate::Phase(angle) => QuantumGate::Phase(angle.inverse()),
            QuantumGate::Hadamard => QuantumGate::Hadamard,
            QuantumGate::PauliX => QuantumGate::PauliX,
            QuantumGate::PauliY => QuantumGate::PauliY,
            QuantumGate::PauliZ => QuantumGate::PauliZ,
        }
    }
}

/// Enum representing Pauli operators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pauli {
    /// Pauli X operator.
    PauliX,
    /// Pauli Y operator.
    PauliY,
    /// Pauli Z operator.
    PauliZ,
}

/// Structure representing a term in a Pauli product.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauliTerm {
    /// Pauli operator.
    pub pauli: Pauli,

    /// Qubit index.
    pub qubit: usize,
}

/// Type representing a product of Pauli terms.
pub type PauliProduct = Vec<PauliTerm>;

/// Structure representing a Hamiltonian in terms of Pauli products.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PauliHamiltonian {
    /// List of Pauli products.
    pub products: Vec<PauliProduct>,

    /// Coefficients associated with each Pauli product.
    pub coefficients: Vec<f64>,
}

/// Enum representing different quantum instructions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    /// Alloc instruction, allocating a qubit.
    Alloc {
        /// The target index of the allocated qubit.
        target: usize,
    },

    /// Free instruction, freeing a previously allocated qubit.
    Free {
        /// The target index of the qubit to be freed.
        target: usize,
    },

    /// Gate instruction, applying a quantum gate to a target qubit with optional control qubits.
    Gate {
        /// The quantum gate to be applied.
        gate: QuantumGate,

        /// The target index of the qubit on which the gate is applied.
        target: usize,

        /// The list of control qubits influencing the gate operation.
        control: Vec<usize>,
    },

    /// Measure instruction, measuring specified qubits and recording the result.
    Measure {
        /// The list of qubits to be measured.
        qubits: Vec<usize>,

        /// The index to store the measurement result.
        output: usize,
    },

    /// ExpValue instruction, calculating the expected value of a Hamiltonian.
    ExpValue {
        /// The Hamiltonian for which the expected value is calculated.
        hamiltonian: PauliHamiltonian,

        /// The index to store the calculated expected value.
        output: usize,
    },

    /// Sample instruction, sampling specified qubits a certain number of times.
    Sample {
        /// The list of qubits to be sampled.
        qubits: Vec<usize>,

        /// The number of shots for the sampling process.
        shots: u64,

        /// The index to store the sampling results.
        output: usize,
    },

    /// Dump instruction, capturing the state of specified qubits.
    Dump {
        /// The list of qubits to be dumped.
        qubits: Vec<usize>,

        /// The index to store the dump data.
        output: usize,
    },
}

/// Enum representing the status of a quantum process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
    /// The process is in the building state, indicating that quantum operations can be added.
    Building,

    /// The process is in the live state, indicating that quantum operations are executed iteratively.
    Live,

    /// The process is in the ready state, indicating that it is prepared for execution.
    Ready,

    /// The process is in the running state, indicating that quantum operations are being executed.
    Running,

    /// The process is in the terminated state, indicating that the execution has completed.
    Terminated,
}

/// Structure representing metadata associated with a quantum process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Maximum number of qubits allocated simultaneously.
    pub qubit_simultaneous: usize,

    /// Timeout duration for the quantum execution.
    pub timeout: Option<u64>,

    /// Status of the quantum process.
    pub status: ProcessStatus,

    /// Total execution time of the quantum process.
    pub execution_time: Option<f64>,

    /// Map of gate counts at each depth.
    pub gate_count: HashMap<usize, usize>,

    /// Depth of the quantum circuit.
    pub depth: usize,
}

impl Metadata {
    /// Creates a new Metadata instance.
    pub(crate) fn new(live: bool) -> Metadata {
        Metadata {
            qubit_simultaneous: 0,
            timeout: None,
            status: if live {
                ProcessStatus::Live
            } else {
                ProcessStatus::Building
            },
            execution_time: None,
            gate_count: HashMap::new(),
            depth: 0,
        }
    }
}

/// Structure representing the result data of a quantum process execution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResultData {
    /// Measurement results.
    pub measurements: Vec<u64>,

    /// Expected values.
    pub exp_values: Vec<f64>,

    /// Sampled results.
    pub samples: Vec<(Vec<u64>, Vec<u64>)>,

    /// Dumped quantum state data.
    pub dumps: Vec<DumpData>,

    /// Total execution time of the quantum process.
    pub execution_time: Option<f64>,
}

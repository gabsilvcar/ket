// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

use ket::error::KetError;
use ket::{Configuration, Pauli, PauliHamiltonian, PauliTerm, Process, QuantumGate};

fn main() -> Result<(), KetError> {
    // Configuration instance must be provided by the quantum execution.
    // See the KBW documentation for examples.
    let configuration = Configuration::new(2);

    // Create a new process with the provided configurations.
    // The configuration will specify the maximum number of qubits the quantum
    // execution can handle, the execution mode, and more.
    let mut process = Process::new(configuration);

    // Allocate qubits and return their references for later usage.
    let qubit_a = process.allocate_qubit()?;
    let qubit_b = process.allocate_qubit()?;

    // Apply a Hadamard gate to the first qubit.
    process.apply_gate(QuantumGate::Hadamard, qubit_a)?;

    // Push the first qubit to the control stack, apply a Pauli X gate to the second qubit,
    // and pop the qubit from the control stack.
    process.ctrl_push(&[qubit_a])?;
    process.apply_gate(QuantumGate::PauliX, qubit_b)?;
    process.ctrl_pop()?;

    // Measure the qubits and return the results references.
    let _m_a = process.measure(&[qubit_a])?;
    let _m_b = process.measure(&[qubit_b])?;

    let _exp = process.exp_values(PauliHamiltonian {
        coefficients: vec![1.0],
        products: vec![vec![
            PauliTerm {
                pauli: Pauli::PauliX,
                qubit: qubit_a,
            },
            PauliTerm {
                pauli: Pauli::PauliX,
                qubit: qubit_b,
            },
        ]],
    })?;

    Ok(())
}

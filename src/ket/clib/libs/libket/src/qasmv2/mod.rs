// SPDX-FileCopyrightText: 2024 Gabriel da Silva Cardoso <cardoso.gabriel@grad.ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

pub mod exporter;
pub mod importer;
pub mod instruction_set;
pub mod verifier;

#[cfg(test)]
mod tests {
    use crate::error::KetError;
    use crate::qasmv2::instruction_set::InstructionSet;
    use crate::qasmv2::verifier::verify_qasmv2;
    use crate::*;

    #[test]
    fn test_qasm_bell_state() -> Result<(), KetError> {
        let configuration = Configuration::new(2);

        let mut process = Process::new(configuration);

        let qubit_a = process.allocate_qubit()?;
        let qubit_b = process.allocate_qubit()?;

        process.apply_gate(QuantumGate::Hadamard, qubit_a)?;

        process.ctrl_push(&[qubit_a])?;
        process.apply_gate(QuantumGate::PauliX, qubit_b)?;
        process.ctrl_pop()?;

        let _m_a = process.measure(&[qubit_a])?;
        let _m_b = process.measure(&[qubit_b])?;
        let _m_a = process.dump(&[qubit_a])?;


        let qasm = process.to_qasmv2(true, InstructionSet::QELIB)?;
        println!("{}", &qasm);
        if verify_qasmv2(qasm) {
            Ok(())
        } else {
            Err(KetError::InvalidQASM)
        }
    }

    fn test_import_export(
        qasm: &str,
        expected: &str,
        import_set: InstructionSet,
        export_set: InstructionSet,
    ) -> Result<(), KetError> {
        fn clean_qasm(qasm: &str) -> String {
            qasm.replace("// Generated from libket", "")
                .replace(|c: char| c.is_whitespace(), "")
        }

        let qubits = 10;
        let mut process = Process::new(Configuration::new(qubits));
        for _ in 0..qubits {
            process.allocate_qubit()?;
        }

        process.from_qasmv2(qasm, import_set, false).unwrap();
        let qasm_synthesized = process.to_qasmv2(false, export_set)?;
        if clean_qasm(expected) == clean_qasm(&*qasm_synthesized) {
            Ok(())
        } else {
            println!("Original qasm: \n{}", qasm);
            println!("Synthesized qasm: \n{}", qasm_synthesized);

            Err(KetError::InvalidQASM)
        }
    }

    /// Tests decompostion of QELIB gates to the default instruction set U (represented as
    /// RZ RY RZ in the libket) and CNOT gates.
    #[test]
    fn test_qasm_controlled() -> Result<(), KetError> {
        let qasm = r#"OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[10];
        h q[0];
        cx q[0], q[1];
        "#;

        let expected = r#"// Generated from libket
        OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[10];
        h q[0];
        cx q[0], q[1];

        "#;
        test_import_export(qasm, expected, InstructionSet::QELIB, InstructionSet::QELIB)
    }

    /// Tests all explicitly supported single qubit gates
    #[test]
    fn test_gate_transform() -> Result<(), KetError> {
        let qasm = r#"OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[10];
        h q[0];
        x q[0];
        y q[0];
        z q[0];
        t q[0];
        s q[0];
        sdg q[0];
        tdg q[0];
        "#;
        test_import_export(qasm, qasm, InstructionSet::QELIB, InstructionSet::QELIB)
    }

    /// Multicontrolled gates are imported as many U and CX gates, making it hard to identify
    /// Toffoli gates for optimization. To remedy this, we use an edited version of the qelib that
    /// does not include ccx in the instruction set so that it's possible to declare it as opaque
    /// later
    #[test]
    fn test_multicontrolled_gates() -> Result<(), KetError> {
        let qasm = r#"OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[10];
        ccx q[0], q[1], q[2];
        "#;
        test_import_export(qasm, qasm, InstructionSet::QELIB, InstructionSet::QELIB)
    }

    /// RX gate conversion is a special case and will be converted to RZ and RY gates because
    /// all qelib ports are first converted to U gates. It's not viable doing the same workaround
    /// of ccx gates to rx gates, because it's declaration is in a vast amount of ports
    #[test]
    fn test_rx_gate() -> Result<(), KetError> {
        let qasm = r#"OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[10];
        rx (pi/2) q[0];
        "#;
        let expected = r#"OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[10];
        s q[0];
        ry (pi/2) q[0];
        sdg q[0];
        "#;
        test_import_export(qasm, expected, InstructionSet::QELIB, InstructionSet::QELIB)
    }

    #[test]
    fn test_bell() -> Result<(), KetError> {
        let qasm = r#"OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[10];
        h q[0];
        h q[1];
        h q[0];
        cz q[1], q[0];
        h q[1];
        "#;
        let expected = r#"OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[10];
        h q[0];
        h q[1];
        h q[0];
        h q[0];
        cx q[1], q[0];
        h q[0];
        h q[1];
        "#;
        test_import_export(qasm, expected, InstructionSet::QELIB, InstructionSet::QELIB)
    }
}

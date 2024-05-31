// SPDX-FileCopyrightText: 2024 Gabriel da Silva Cardoso <cardoso.gabriel@grad.ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

pub mod optimize;
// mod utils;

#[cfg(test)]
mod tests {
    use crate::error::KetError;
    use crate::qasmv2::instruction_set::InstructionSet::QELIB;
    use crate::{Configuration, Process, QuantumGate};

    #[test]
    fn test_tof_10() -> Result<(), KetError> {
        let qasm = r#"OPENQASM 2.0;
        include "qelib1.inc";
        qreg qubits[19];
        h qubits[10];
        h qubits[10];
        ccx qubits[0],qubits[1],qubits[10];
        h qubits[10];
        h qubits[10];
        h qubits[11];
        h qubits[11];
        ccx qubits[2],qubits[10],qubits[11];
        h qubits[11];
        h qubits[11];
        h qubits[12];
        h qubits[12];
        ccx qubits[3],qubits[11],qubits[12];
        h qubits[12];
        h qubits[12];
        h qubits[13];
        h qubits[13];
        ccx qubits[4],qubits[12],qubits[13];
        h qubits[13];
        h qubits[13];
        h qubits[14];
        h qubits[14];
        ccx qubits[5],qubits[13],qubits[14];
        h qubits[14];
        h qubits[14];
        h qubits[15];
        h qubits[15];
        ccx qubits[6],qubits[14],qubits[15];
        h qubits[15];
        h qubits[15];
        h qubits[16];
        h qubits[16];
        ccx qubits[7],qubits[15],qubits[16];
        h qubits[16];
        h qubits[16];
        h qubits[17];
        h qubits[17];
        ccx qubits[8],qubits[16],qubits[17];
        h qubits[17];
        h qubits[17];
        h qubits[18];
        h qubits[18];
        ccx qubits[9],qubits[17],qubits[18];
        h qubits[18];
        h qubits[18];
        h qubits[17];
        h qubits[17];
        ccx qubits[8],qubits[16],qubits[17];
        h qubits[17];
        h qubits[17];
        h qubits[16];
        h qubits[16];
        ccx qubits[7],qubits[15],qubits[16];
        h qubits[16];
        h qubits[16];
        h qubits[15];
        h qubits[15];
        ccx qubits[6],qubits[14],qubits[15];
        h qubits[15];
        h qubits[15];
        h qubits[14];
        h qubits[14];
        ccx qubits[5],qubits[13],qubits[14];
        h qubits[14];
        h qubits[14];
        h qubits[13];
        h qubits[13];
        ccx qubits[4],qubits[12],qubits[13];
        h qubits[13];
        h qubits[13];
        h qubits[12];
        h qubits[12];
        ccx qubits[3],qubits[11],qubits[12];
        h qubits[12];
        h qubits[12];
        h qubits[11];
        h qubits[11];
        ccx qubits[2],qubits[10],qubits[11];
        h qubits[11];
        h qubits[11];
        h qubits[10];
        h qubits[10];
        ccx qubits[0],qubits[1],qubits[10];
        h qubits[10];
        h qubits[10];
        "#;
        let qubits = 19;
        let mut process = Process::new(Configuration::new(qubits));
        process.from_qasmv2(qasm, QELIB, false)?;
        process.optimize().unwrap();
        println!("{}", process.to_qasmv2(false, QELIB).unwrap());
        Ok(())
    }

    #[test]
    fn test_dump() -> Result<(), KetError> {
        let configuration = Configuration::new(2);

        let mut process = Process::new(configuration);

        let qubit_a = process.allocate_qubit()?;
        let qubit_b = process.allocate_qubit()?;

        process.apply_gate(QuantumGate::Hadamard, qubit_a)?;

        process.ctrl_push(&[qubit_a])?;
        process.apply_gate(QuantumGate::PauliX, qubit_b)?;
        process.ctrl_pop()?;

        let _m_a = process.dump(&[qubit_a, qubit_b])?;

        process.optimize().unwrap();
        println!("{}", process.to_qasmv2(false, QELIB).unwrap());

        Ok(())
    }

    #[test]
    fn test_measure() -> Result<(), KetError> {
        let qasm = r#"// // Generated from libket
        OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[2];
        creg c[2];
        h q[0];
        measure q[0] -> c[0];
        x q[0];
        x q[1];
        measure q[0] -> c[0];
        measure q[1] -> c[1];
        "#;
        let qubits = 2;
        let mut process = Process::new(Configuration::new(qubits));
        process.from_qasmv2(qasm, QELIB, false)?;

        process.optimize().unwrap();
        // println!("{}", process.to_qasmv2(false, QELIB).unwrap());

        Ok(())
    }
}

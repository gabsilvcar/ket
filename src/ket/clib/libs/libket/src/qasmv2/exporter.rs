// SPDX-FileCopyrightText: 2024 Gabriel da Silva Cardoso <cardoso.gabriel@grad.ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;
use std::fmt::format;

use regex::Regex;

use crate::error::KetError;
use crate::qasmv2::instruction_set::InstructionSet;
use crate::{Instruction, Process, QuantumGate};

pub fn to_qasmv2(
    process: &Process,
    measurements: bool,
    instruction_set: InstructionSet,
) -> Result<String, KetError> {
    if instruction_set == InstructionSet::DEFAULT {
        return Err(KetError::PureQASMGateExportError);
    }

    let header = "// Generated from libket\n\
                            OPENQASM 2.0;\n\
                            include \"qelib1.inc\";\n";

    let num_qubits = process.metadata.qubit_simultaneous;

    let mut alloc = format!("qreg q[{}];\n", num_qubits);
    let mut bits = 0;
    let mut qubit_stack: VecDeque<usize> = (0..num_qubits).collect();
    let mut qubit_map: VecDeque<usize> = VecDeque::from(vec![0; num_qubits]);

    let mut gates = String::new();
    let mut ket_instr = String::new();

    let mut has_dump = false;
    let mut has_measure = false;

    for instruction in &process.instructions {
        match instruction {
            Instruction::Alloc { target } => {
                qubit_map[*target] = qubit_stack.pop_front().unwrap();
            }
            Instruction::Free { target } => {
                qubit_stack.push_front(qubit_map[*target]);
            }
            Instruction::Gate {
                gate,
                target,
                control,
            } => {
                gates += &match control.len() {
                    0 => match gate {
                        QuantumGate::PauliX => format!("x q[{}];\n", qubit_map[*target]),
                        QuantumGate::PauliY => format!("y q[{}];\n", qubit_map[*target]),
                        QuantumGate::PauliZ => format!("z q[{}];\n", qubit_map[*target]),
                        QuantumGate::RotationX(angle) => {
                            format!("rx ({}) q[{}];\n", angle, qubit_map[*target])
                        }
                        QuantumGate::RotationY(angle) => {
                            format!("ry ({}) q[{}];\n", angle, qubit_map[*target])
                        }
                        QuantumGate::RotationZ(angle) => {
                            format!("rz ({}) q[{}];\n", angle, qubit_map[*target])
                        }
                        QuantumGate::Phase(angle) => {
                            format!("u1 ({}) q[{}];\n", angle, qubit_map[*target])
                        }
                        QuantumGate::Hadamard => format!("h q[{}];\n", qubit_map[*target]),
                    },
                    1 => match gate {
                        QuantumGate::PauliX => {
                            format!(
                                "cx q[{}], q[{}];\n",
                                qubit_map[control[0]], qubit_map[*target]
                            )
                        }
                        QuantumGate::PauliZ => {
                            format!(
                                "cz q[{}], q[{}];\n",
                                qubit_map[control[0]], qubit_map[*target]
                            )
                        }
                        _ => {
                            return Err(KetError::UnsuportedGateExport);
                        }
                    },
                    2 => match gate {
                        QuantumGate::PauliX => {
                            format!(
                                "ccx q[{}], q[{}], q[{}];\n",
                                qubit_map[control[0]], qubit_map[control[1]], qubit_map[*target]
                            )
                        }
                        _ => {
                            return Err(KetError::UnsuportedGateExport);
                        }
                    },
                    _ => return Err(KetError::UnsuportedGateExport),
                };
            }
            Instruction::Measure { qubits, .. } => {
                for qubit in qubits {
                    gates += &format!("measure q[{}] -> c[{}];\n", qubit_map[*qubit], bits);
                }
                bits += 1;
            }
            Instruction::ExpValue { .. } => {println!("expvalue"); return Err(KetError::UnsuportedGateExport); },
            Instruction::Sample { .. } => return {println!("sample"); Err(KetError::UnsuportedGateExport)},
            Instruction::Dump { qubits, .. } => {
                if !has_dump {
                    ket_instr += &format!("opaque dump(classic) {};\n", "a");
                    has_dump = true;
                }
                for qubit in qubits {
                    gates += &format!("dump({}) q[{}];\n", bits, qubit_map[*qubit]);
                }
                bits += 1;
            },
        }
    }
    let mut measure_qubits = String::new();
    if measurements {
        measure_qubits = (0..num_qubits)
            .map(|q| format!("measure q[{}] -> c[{}];", q, q))
            .collect::<Vec<String>>()
            .join("\n");
    }
    if instruction_set == InstructionSet::QELIB {
        gates = qelib_single_gate_replacer(&mut gates);
    }

    if bits > 0 {
        alloc += &format!("creg c[{}];\n", bits);
    }

    Ok(header.to_owned() + &alloc + &ket_instr +&gates + &measure_qubits + "\n")
}

pub fn qelib_single_gate_replacer(qasm: &mut str) -> String {
    let t = make_gate_regex(r"rz (pi/4) REG;\s");
    let phase = make_gate_regex(r"rz (pi/2) REG;\s");
    let sdg = make_gate_regex(r"rz ((-1 * pi)/2) REG;\s");
    let tdg = make_gate_regex(r"rz ((-1 * pi)/4) REG;\s");

    let instruction_map: Vec<Vec<&str>> = vec![
        vec!["t {};\n", &t],
        vec!["s {};\n", &phase],
        vec!["sdg {};\n", &sdg],
        vec!["tdg {};\n", &tdg],
    ];

    instruction_map_replacer(instruction_map, qasm)
}

fn instruction_map_replacer(instruction_map: Vec<Vec<&str>>, qasm: &str) -> String {
    let mut result = String::from(qasm);
    let qasm_clone = qasm.to_owned();

    for gate in instruction_map {
        let re = Regex::new(gate[1]).unwrap();
        for captures in re.captures_iter(&qasm_clone) {
            let mut registers: Vec<String> = Vec::new();

            for capture in captures.iter().skip(1).flatten() {
                registers.push(String::from(capture.as_str()));
            }
            result = result.replace(
                captures.get(0).unwrap().as_str(),
                &gate[0].replace("{}", &registers[0]),
            );
        }
    }
    result
}

fn make_gate_regex(qasm_expression: &str) -> String {
    qasm_expression
        .replace('(', r#"\("#)
        .replace(')', r#"\)"#)
        .replace('*', r#"\*"#)
        .replace('/', r#"\/"#)
        .replace("REG", "(.*?)")
}

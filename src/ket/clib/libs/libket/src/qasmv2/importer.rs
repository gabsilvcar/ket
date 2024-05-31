// SPDX-FileCopyrightText: 2024 Gabriel da Silva Cardoso <cardoso.gabriel@grad.ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

use crate::error::{KetError, Result};
use crate::qasmv2::instruction_set::{InstructionSet, EDITED_QELIB, OPAQUE_QELIB_GATES};
use crate::{Angle, Process, QuantumGate};
use openqasm as oq;
use openqasm::parser::FilePolicy;
use openqasm::{GateWriter, GenericError, ProgramVisitor, Symbol, Value};

pub fn from_qasmv2(
    process: &mut Process,
    qasm: &str,
    instruction_set: InstructionSet,
    ignore_qreg: bool
) -> Result<()> {
    let mut writer = QasmInterpreter {
        process,
        instruction_set,
        ignore_qreg,
    };
    let mut cache = oq::SourceCache::new();
    let mut parser = oq::Parser::new(&mut cache).with_file_policy(FilePolicy::Ignore);

    parser.parse_source::<String>(qasm.parse().unwrap(), None);

    match instruction_set {
        InstructionSet::DEFAULT => {}
        InstructionSet::QELIB => {
            parser.parse_source::<String>(EDITED_QELIB.parse().unwrap(), None);
            parser.parse_source::<String>(OPAQUE_QELIB_GATES.parse().unwrap(), None);
        }
    }

    let program = parser.done().to_errors().unwrap();
    program.type_check().to_errors().unwrap();

    let mut l = oq::translate::Linearize::new(&mut writer, usize::MAX);
    l.visit_program(&program).to_errors().unwrap();
    Ok(())
}

pub struct QasmInterpreter<'a> {
    pub process: &'a mut Process,
    pub instruction_set: InstructionSet,
    pub ignore_qreg: bool
}

impl QasmInterpreter<'_> {
    fn get_process(&mut self) -> Result<&mut Process> {
        Ok(self.process)
    }
}

impl GateWriter for &mut QasmInterpreter<'_> {
    type Error = KetError;

    fn initialize(&mut self, qubits: &[Symbol], bits: &[Symbol]) -> Result<()> {
        if self.ignore_qreg {
            return Ok(());
        }

        for i in 0..qubits.len()
        {
            self.get_process()?.allocate_qubit()?;
        }

        // if !bits.is_empty() {
        //     return Err(KetError::BitsNotSupported);
        // }
        Ok(())
    }

    fn write_cx(&mut self, copy: usize, xor: usize) -> Result<()> {
        self.get_process()?.ctrl_push(&[copy])?;
        self.get_process()?.apply_gate(QuantumGate::PauliX, xor)?;
        self.get_process()?.ctrl_pop().unwrap();
        Ok(())
    }

    fn write_u(&mut self, theta: Value, phi: Value, lambda: Value, reg: usize) -> Result<()> {
        let mut gate_angles = String::new();

        // will only verify the angles if QELIB mode is enabled
        if self.instruction_set == InstructionSet::QELIB {
            gate_angles = format!("{}, {}, {}", theta, phi, lambda);
        }

        match gate_angles.as_str() {
            // Hadamard Gate U(π/2, 0, π/1)
            "π/2, 0, π/1" => {
                self.get_process()?
                    .apply_gate(QuantumGate::Hadamard, reg)
                    .unwrap();
            }
            // X Gate U(π/1, 0, π/1)
            "π/1, 0, π/1" => {
                self.get_process()?
                    .apply_gate(QuantumGate::PauliX, reg)
                    .unwrap();
            }
            // Y Gate U(π/1, π/2, π/2)
            "π/1, π/2, π/2" => {
                self.get_process()?
                    .apply_gate(QuantumGate::PauliY, reg)
                    .unwrap();
            }
            // Z Gate U(0, 0, π/1)
            "0, 0, π/1" => {
                self.get_process()?
                    .apply_gate(QuantumGate::PauliZ, reg)
                    .unwrap();
            }
            // Rz(theta)Ry(phi)Rz(lambda) =  U(phi, theta, lambda)
            _ => {
                if *lambda.b.numer() != 0 {
                    self.get_process()?
                        .apply_gate(
                            QuantumGate::RotationZ(Angle::PiFraction {
                                top: *lambda.b.numer() as i32,
                                bottom: *lambda.b.denom() as u32,
                            }),
                            reg,
                        )
                        .unwrap();
                }

                if *theta.b.numer() != 0 {
                    self.get_process()?
                        .apply_gate(
                            QuantumGate::RotationY(Angle::PiFraction {
                                top: *theta.b.numer() as i32,
                                bottom: *theta.b.denom() as u32,
                            }),
                            reg,
                        )
                        .unwrap();
                }

                if *phi.b.numer() != 0 {
                    self.get_process()?
                        .apply_gate(
                            QuantumGate::RotationZ(Angle::PiFraction {
                                top: *phi.b.numer() as i32,
                                bottom: *phi.b.denom() as u32,
                            }),
                            reg,
                        )
                        .unwrap();
                }
            }
        }

        Ok(())
    }

    fn write_opaque(&mut self, name: &Symbol, _params: &[Value], regs: &[usize]) -> Result<()> {
        match name.to_string().as_str() {
            "ccx" => {
                self.get_process()?.ctrl_push(&regs[0..2])?;
                self.get_process()?
                    .apply_gate(QuantumGate::PauliX, regs[2])?;
                self.get_process()?.ctrl_pop().unwrap();

                Ok(())
            }
            "dump" => {
                self.get_process()?.dump(&[regs[0]]);
                Ok(())
            }

            _ => {
                println!("undefined gate: {}", name.to_string().as_str());
                Err(KetError::GateNotSupported)
            }
        }
    }

    fn write_barrier(&mut self, _: &[usize]) -> Result<()> {
        // TODO stop optimizations (once implemented)
        Err(KetError::GateNotSupported)
    }

    fn write_measure(&mut self, from: usize, _to: usize) -> Result<()> {
        self.get_process()?.measure(&[from]);
        Ok(())
    }

    fn write_reset(&mut self, _reg: usize) -> Result<()> {
        Err(KetError::GateNotSupported)
    }

    fn start_conditional(&mut self, _reg: usize, _count: usize, _value: u64) -> Result<()> {
        Err(KetError::GateNotSupported)
    }

    fn end_conditional(&mut self) -> Result<()> {
        Err(KetError::GateNotSupported)
    }
}

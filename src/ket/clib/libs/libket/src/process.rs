// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

//! This module contains the `Process` struct, which encapsulates the necessary information for
//! handling qubit allocations and creating quantum circuits.

use log::info;

use crate::{
    error::{KetError, Result},
    ir::{Instruction, Metadata, PauliHamiltonian, ProcessStatus, QuantumGate, ResultData},
    objects::{Dump, ExpValue, Measurement, QubitStatus, Sample},
    Angle, Configuration,
};

use crate::qasmv2::exporter::to_qasmv2;
use crate::qasmv2::importer::from_qasmv2;
use crate::qasmv2::instruction_set::InstructionSet;
use crate::zx::optimize::optimize;

/// Quantum Process for managing qubit allocation and circuit creation.
///
/// This struct encapsulates the necessary information for handling qubit allocations and
/// creating quantum circuits. It provides functions to apply quantum gates, measure
/// qubits, and execute quantum code.
///
/// # Examples
///
/// ```rust
/// # use ket::error::KetError;
/// use ket::{Configuration, Process, QuantumGate};
///
/// # fn main() -> Result<(), KetError> {
/// // Configuration instance must be provided by the quantum execution.
/// // See the KBW documentation for examples.
/// let configuration = Configuration::new(2);
///
/// // Create a new process with the provided configurations.
/// // The configuration will specify the maximum number of qubits the quantum
/// // execution can handle, the execution mode, and more.
/// let mut process = Process::new(configuration);
///
/// // Allocate qubits and return their references for later usage.
/// let qubit_a = process.allocate_qubit()?;
/// let qubit_b = process.allocate_qubit()?;
///
/// // Apply a Hadamard gate to the first qubit.
/// process.apply_gate(QuantumGate::Hadamard, qubit_a)?;
///
/// // Push the first qubit to the control stack, apply a Pauli X gate to the second qubit,
/// // and pop the qubit from the control stack.
/// process.ctrl_push(&[qubit_a])?;
/// process.apply_gate(QuantumGate::PauliX, qubit_b)?;
/// process.ctrl_pop()?;
///
/// // Measure the qubits and return the results references.
/// let m_a = process.measure(&[qubit_a])?;
/// let m_b = process.measure(&[qubit_b])?;
///
/// # Ok(())
/// # }
/// ```
pub struct Process {
    /// Metadata generated in the process lifetime
    pub(crate) metadata: Metadata,

    /// Configuration provided by the quantum executor (QPU or Simulator)
    pub config: Configuration,

    /// List of quantum instructions (the quantum circuit)
    pub(crate) instructions: Vec<Instruction>,

    /// Qubit stack for managing nested control scopes.
    pub(crate) ctrl_stack: Vec<Vec<usize>>,

    /// Control qubit list generated from the control stack
    pub(crate) ctrl_list: Vec<usize>,
    pub(crate) ctrl_list_is_up_to_date: bool,

    /// Instructions stack fo handling nested inverse scopes
    pub(crate) adj_stack: Vec<Vec<Instruction>>,

    /// List of measurement results
    pub(crate) measurements: Vec<Measurement>,

    /// List of expected values
    pub(crate) exp_values: Vec<ExpValue>,

    /// List of samples result
    pub(crate) samples: Vec<Sample>,

    /// List of quantum state dump results
    pub(crate) dumps: Vec<Dump>,

    /// Number of qubits allocated
    pub qubit_allocated: usize,
    pub qubits: Vec<QubitStatus>,
}

impl Process {
    /// Creates a new `Process` with the given configurations
    pub fn new(config: Configuration) -> Self {
        Self {
            metadata: Metadata::new(config.live_quantum_execution.is_some()),
            config,
            instructions: Default::default(),
            ctrl_stack: Default::default(),
            ctrl_list: Default::default(),
            ctrl_list_is_up_to_date: Default::default(),
            adj_stack: Default::default(),
            measurements: Default::default(),
            exp_values: Default::default(),
            samples: Default::default(),
            dumps: Default::default(),
            qubit_allocated: Default::default(),
            qubits: Default::default(),
        }
    }

    /// Returns a list of control qubits
    ///
    /// Update the control qubits list if necessary and return it.
    fn get_control_qubits(&mut self) -> &[usize] {
        if !self.ctrl_list_is_up_to_date {
            self.ctrl_list_is_up_to_date = true;
            self.ctrl_list = Vec::new();
            for inner_ctrl in self.ctrl_stack.iter() {
                self.ctrl_list.extend(inner_ctrl.iter());
            }
        }
        &self.ctrl_list
    }

    /// Return an error if the given qubit index is in the control qubit list
    fn assert_target_not_in_control(&mut self, target: usize) -> Result<()> {
        if self.get_control_qubits().contains(&target) {
            Err(KetError::TargetInControl)
        } else {
            Ok(())
        }
    }

    /// Return an error if the given qubit index has the allocated status `false`
    fn assert_qubit_allocated(&self, qubit: usize) -> Result<()> {
        let qubit = self.qubits.get(qubit);
        match qubit {
            Some(qubit) => {
                if !qubit.allocated {
                    Err(KetError::DeallocatedQubit)
                } else {
                    Ok(())
                }
            }
            None => Err(KetError::QubitIndexOutOfBounds),
        }
    }

    /// Return and error if the process is read for execute
    fn assert_not_ready_for_execution(&self) -> Result<()> {
        match self.metadata.status {
            ProcessStatus::Building | ProcessStatus::Live => Ok(()),
            _ => Err(KetError::ProcessReadyToExecute),
        }
    }

    /// Returns an error if there are no inverse scopes opened
    fn assert_not_adj(&self) -> Result<()> {
        if self.adj_stack.is_empty() {
            Ok(())
        } else {
            Err(KetError::NonGateInstructionInAdj)
        }
    }

    /// Allocate a qubit and return its index.
    ///
    /// The qubit index resulting from the allocation is used in quantum gate application,
    /// measurement, quantum state dump, and expected value calculations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ket::error::KetError;
    /// # use ket::{Configuration, Process};
    /// #
    /// # fn main() -> Result<(), KetError> {
    /// # let mut process = Process::new(Configuration::new(2));
    /// let qubit_index = process.allocate_qubit()?;
    /// // Now you can use the allocated qubit_index in quantum operations.
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the process is in an inverse scope, if it is ready for
    /// execution, or if the number of allocated qubits exceeds the configured limit.
    pub fn allocate_qubit(&mut self) -> Result<usize> {
        self.assert_not_adj()?;
        self.assert_not_ready_for_execution()?;
        if self.qubit_allocated >= self.config.num_qubits {
            return Err(KetError::NumberOfQubitsExceeded);
        }

        let index = self.qubits.len();
        self.qubits.push(Default::default());

        self.qubit_allocated += 1;

        if self.qubit_allocated > self.metadata.qubit_simultaneous {
            self.metadata.qubit_simultaneous = self.qubit_allocated;
        }

        self.instructions.push(Instruction::Alloc { target: index });

        if let Some(processor) = self.config.live_quantum_execution.as_mut() {
            processor.alloc(index);
        }

        Ok(index)
    }

    /// Frees a previously allocated qubit
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ket::error::KetError;
    /// # use ket::{Configuration, Process};
    /// #
    /// # fn main() -> Result<(), KetError> {
    /// # let mut process = Process::new(Configuration::new(2));
    /// let qubit_index = process.allocate_qubit()?;
    /// // Perform quantum operations...
    /// process.free_qubit(qubit_index)?;
    /// // Qubit has been freed and can no longer be used in quantum operations.
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the process is in an inverse scope, if it is ready for
    /// execution, or if the specified qubit has not been allocated.
    pub fn free_qubit(&mut self, qubit: usize) -> Result<()> {
        self.assert_not_adj()?;
        self.assert_not_ready_for_execution()?;
        self.assert_qubit_allocated(qubit)?;

        self.instructions.push(Instruction::Free { target: qubit });

        self.qubits[qubit].allocated = false;

        if let Some(processor) = self.config.live_quantum_execution.as_mut() {
            processor.free(qubit);
        }

        Ok(())
    }

    /// Applies a quantum gate to a target qubit
    ///
    /// This function considers the control qubit list to apply the quantum gate
    /// in the target qubit.
    ///
    /// If the process has an opened inverse scope, the gate will only be applied
    /// when the scope is closed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ket::error::KetError;
    /// # use ket::{Configuration, Process, QuantumGate};
    /// #
    /// # fn main() -> Result<(), KetError> {
    /// # let mut process = Process::new(Configuration::new(2));
    /// let qubit_index = process.allocate_qubit()?;
    /// process.apply_gate(QuantumGate::Hadamard, qubit_index)?;
    /// // Perform additional quantum operations...
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the process is ready for execution, if the specified
    /// qubit has not been allocated, or if the target qubit is part of the control
    ///  qubits.
    pub fn apply_gate(&mut self, gate: QuantumGate, target: usize) -> Result<()> {
        self.assert_not_ready_for_execution()?;
        self.assert_qubit_allocated(target)?;
        self.assert_target_not_in_control(target)?;
        let control = self.get_control_qubits().to_vec();

        if self.config.decompose {
            todo!();
        }

        let add_adj_gate = self.adj_stack.len() % 2 == 1;

        let gate = if add_adj_gate { gate.inverse() } else { gate };

        self.metadata.depth += 1;
        self.metadata
            .gate_count
            .entry(control.len() + 1)
            .and_modify(|count| *count += 1)
            .or_insert(1);

        if !self.adj_stack.is_empty() {
            self.adj_stack.last_mut().unwrap().push(Instruction::Gate {
                gate,
                target,
                control,
            });
        } else {
            if let Some(processor) = self.config.live_quantum_execution.as_mut() {
                info!(
                    "live execution: gate={:?}, target={}, control={:?}",
                    gate, target, control
                );

                processor.gate(&gate, target, &control)
            }

            self.instructions.push(Instruction::Gate {
                gate,
                target,
                control,
            });
        }

        Ok(())
    }

    /// Applies a global phase
    ///
    /// This function considers the control qubit list to apply the global phase.
    /// If the process has an opened controlled scope, a controlled-phase gate
    /// is applied on the control qubits. Otherwise, the global-phase is ignored.
    pub fn apply_global_phase(&mut self, phase: Angle) -> Result<()> {
        self.assert_not_ready_for_execution()?;

        let control = self.get_control_qubits().to_vec();

        if control.is_empty() {
            return Ok(());
        }

        self.metadata.depth += 1;
        self.metadata
            .gate_count
            .entry(control.len())
            .and_modify(|count| *count += 1)
            .or_insert(1);

        let phase_gate = Instruction::Gate {
            gate: QuantumGate::Phase(phase.clone()),
            target: control[0],
            control: control[1..].to_vec(),
        };

        if !self.adj_stack.is_empty() {
            self.adj_stack.last_mut().unwrap().push(phase_gate);
        } else {
            if let Some(processor) = self.config.live_quantum_execution.as_mut() {
                processor.gate(&QuantumGate::Phase(phase), control[0], &control[1..])
            }

            self.instructions.push(phase_gate);
        }

        Ok(())
    }

    /// Measures the specified qubits
    ///
    /// This function performs measurements on the specified qubits.
    /// It updates the internal state of the process, records measurement instructions, and
    /// returns the index of the measurement result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ket::error::KetError;
    /// # use ket::{Configuration, Process};
    /// #
    /// # fn main() -> Result<(), KetError> {
    /// # let mut process = Process::new(Configuration::new(2));
    /// let qubit_index = process.allocate_qubit()?;
    /// let measurement_index = process.measure(&[qubit_index])?;
    /// // Perform additional quantum operations or measurements...
    /// # Ok(())
    /// # }
    /// ```
    ///     
    /// # Errors
    ///
    /// Returns an error if the process is in an adjacent scope, if the process
    /// is ready for execution, or if measurements are not allowed based on the
    /// process configuration.
    pub fn measure(&mut self, qubits: &[usize]) -> Result<usize> {
        self.assert_not_adj()?;
        self.assert_not_ready_for_execution()?;
        if !self.config.allow_measure {
            return Err(KetError::MeasureNotAllowed);
        }

        for qubit in qubits {
            self.assert_qubit_allocated(*qubit)?;
            self.qubits[*qubit].measured = true;
        }

        if !self.config.valid_after_measure {
            for qubit in qubits {
                self.qubits[*qubit].allocated = false;
            }
        }

        let measure_index = self.measurements.len();

        let result = self
            .config
            .live_quantum_execution
            .as_mut()
            .map(|processor| processor.measure(qubits));

        self.measurements.push(Measurement {
            qubits: qubits.to_vec(),
            result,
        });

        self.instructions.push(Instruction::Measure {
            qubits: qubits.to_vec(),
            output: measure_index,
        });

        Ok(measure_index)
    }

    /// Calculates the expected values of a Pauli Hamiltonian
    ///
    /// This function calculates the expected values of a Pauli Hamiltonian. It updates
    /// the internal state of the process, records the calculation instructions, and
    /// returns the index of the expected value result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ket::error::KetError;
    /// # use ket::{Configuration, Process};
    /// use ket::{Pauli, PauliHamiltonian, PauliTerm};
    ///
    /// # fn main() -> Result<(), KetError> {
    /// # let configuration = Configuration::new(2);
    /// # let mut process = Process::new(configuration);
    /// let qubit_a = process.allocate_qubit()?;
    /// let qubit_b = process.allocate_qubit()?;
    /// let _exp = process.exp_values(PauliHamiltonian {
    ///     coefficients: vec![1.0],
    ///     products: vec![vec![
    ///         PauliTerm {
    ///             pauli: Pauli::PauliX,
    ///             qubit: qubit_a,
    ///         },
    ///         PauliTerm {
    ///             pauli: Pauli::PauliX,
    ///             qubit: qubit_b,
    ///         },
    ///     ]],
    /// })?;
    /// # Ok(())
    /// # }    
    /// ```
    ///     
    /// # Errors
    ///
    /// Returns an error if the process is in an adjacent scope, if the process
    /// is ready for execution, or if expected value calculations are not allowed based on the
    /// process configuration. Additionally, it verifies whether the qubits involved in the
    /// Hamiltonian are allocated.
    pub fn exp_values(&mut self, hamiltonian: PauliHamiltonian) -> Result<usize> {
        self.assert_not_adj()?;
        self.assert_not_ready_for_execution()?;

        if !self.config.allow_exp_value {
            return Err(KetError::ExpValueNotAllowed);
        }

        for term in hamiltonian.products.iter().flat_map(|terms| terms.iter()) {
            self.assert_qubit_allocated(term.qubit)?;
        }

        let index = self.exp_values.len();

        let result = self
            .config
            .live_quantum_execution
            .as_mut()
            .map(|processor| processor.exp_value(&hamiltonian));

        self.exp_values.push(ExpValue {
            hamiltonian: hamiltonian.clone(),
            result,
        });

        self.instructions.push(Instruction::ExpValue {
            hamiltonian,
            output: index,
        });

        if !self.config.continue_after_exp_value {
            self.prepare_for_execution()?;
        }

        Ok(index)
    }

    /// Performs sampling on specified qubits
    ///
    /// This function performs sampling on the specified qubits with a specified number of shots.
    /// It updates the internal state of the process, records sampling instructions, and
    /// returns the index of the sample result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ket::error::KetError;
    /// # use ket::{Configuration, Process};
    /// #
    /// # fn main() -> Result<(), KetError> {
    /// # let mut process = Process::new(Configuration::new(2));
    /// let qubit_a = process.allocate_qubit()?;
    /// let qubit_b = process.allocate_qubit()?;
    /// let shots = 1000;
    /// let sample_index = process.sample(&[qubit_a, qubit_b], shots)?;
    /// // Perform additional quantum operations or sampling...
    /// # Ok(())
    /// # }
    /// ```
    ///     
    /// # Errors
    ///
    /// Returns an error if the process is in an adjacent scope, if the process
    /// is ready for execution, or if sampling is not allowed based on the
    /// process configuration. Additionally, it verifies whether the qubits involved in the
    /// sampling are allocated.
    pub fn sample(&mut self, qubits: &[usize], shots: u64) -> Result<usize> {
        self.assert_not_adj()?;
        self.assert_not_ready_for_execution()?;

        if !self.config.allow_sample {
            return Err(KetError::SampleNotAllowed);
        }

        for qubit in qubits {
            self.assert_qubit_allocated(*qubit)?;
        }

        let index = self.samples.len();

        let result = self
            .config
            .live_quantum_execution
            .as_mut()
            .map(|processor| processor.sample(qubits, shots));

        self.samples.push(Sample {
            qubits: qubits.to_vec(),
            shots,
            result,
        });

        self.instructions.push(Instruction::Sample {
            qubits: qubits.to_vec(),
            shots,
            output: index,
        });

        if !self.config.continue_after_exp_value {
            self.prepare_for_execution()?;
        }

        Ok(index)
    }

    /// Dumps the state of specified qubits
    ///
    /// This function dumps the state of the specified qubits. It updates the internal state
    /// of the process, records dump instructions, and returns the index of the dump result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ket::error::KetError;
    /// # use ket::{Configuration, Process};
    /// #
    /// # fn main() -> Result<(), KetError> {
    /// # let mut process = Process::new(Configuration::new(2));
    /// let qubit_a = process.allocate_qubit()?;
    /// let qubit_b = process.allocate_qubit()?;
    /// let dump_index = process.dump(&[qubit_a, qubit_b])?;
    /// // Perform additional quantum operations or dumps...
    /// # Ok(())
    /// # }
    /// ```
    ///     
    /// # Errors
    ///
    /// Returns an error if the process is in an adjacent scope, if the process
    /// is ready for execution, or if dumping is not allowed based on the
    /// process configuration. Additionally, it verifies whether the qubits involved in the
    /// dump operation are allocated.
    pub fn dump(&mut self, qubits: &[usize]) -> Result<usize> {
        self.assert_not_adj()?;
        self.assert_not_ready_for_execution()?;

        if !self.config.allow_dump {
            return Err(KetError::DumpNotAllowed);
        }

        for qubit in qubits {
            self.assert_qubit_allocated(*qubit)?;
        }

        let dump_index = self.dumps.len();

        let result = self
            .config
            .live_quantum_execution
            .as_mut()
            .map(|processor| processor.dump(qubits));

        self.dumps.push(Dump {
            qubits: qubits.to_vec(),
            result,
        });

        self.instructions.push(Instruction::Dump {
            qubits: qubits.to_vec(),
            output: dump_index,
        });

        if !self.config.continue_after_dump {
            self.prepare_for_execution()?;
        }

        Ok(dump_index)
    }

    /// Pushes control qubits onto the control stack
    ///
    /// This function pushes control qubits onto the control stack. It updates the internal state
    /// of the process and ensures that qubits are not controlled more than once.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ket::error::KetError;
    /// # use ket::{Configuration, Process};
    /// #
    /// # fn main() -> Result<(), KetError> {
    /// # let mut process = Process::new(Configuration::new(2));
    /// let qubit_a = process.allocate_qubit()?;
    /// let qubit_b = process.allocate_qubit()?;
    /// process.ctrl_push(&[qubit_a, qubit_b])?;
    /// // Perform additional quantum operations...
    /// # Ok(())
    /// # }
    /// ```
    ///     
    /// # Errors
    ///
    /// Returns an error if the process is ready for execution or if the control qubits
    /// are allocated more than once during a control operation.
    pub fn ctrl_push(&mut self, qubits: &[usize]) -> Result<()> {
        self.assert_not_ready_for_execution()?;
        let qubits = qubits.to_vec();
        for ctrl_list in self.ctrl_stack.iter() {
            for qubit in &qubits {
                self.assert_qubit_allocated(*qubit)?;
                if ctrl_list.contains(qubit) {
                    return Err(KetError::ControlTwice);
                }
            }
        }

        self.ctrl_stack.push(qubits);

        self.ctrl_list_is_up_to_date = false;

        Ok(())
    }

    /// Pops the last added control qubits from the control stack
    ///     
    /// # Errors
    ///
    /// Returns an error if the process is ready for execution or if there are no control
    /// configurations on the control stack to pop.
    pub fn ctrl_pop(&mut self) -> Result<()> {
        self.assert_not_ready_for_execution()?;
        self.ctrl_list_is_up_to_date = false;

        match self.ctrl_stack.pop() {
            Some(_) => Ok(()),
            None => Err(KetError::NoCtrl),
        }
    }

    /// Begins an adjoint block, where gates are inverted upon insertion
    ///
    /// # Errors
    ///
    /// Returns an error if the process is ready for execution.
    pub fn adj_begin(&mut self) -> Result<()> {
        self.assert_not_ready_for_execution()?;

        self.adj_stack.push(Vec::new());
        Ok(())
    }

    /// Ends the adjoint block, reverting to normal gate insertion
    ///
    /// # Errors
    ///
    /// Returns an error if the process is ready for execution or if there is no adjoint block to end.
    pub fn adj_end(&mut self) -> Result<()> {
        self.assert_not_ready_for_execution()?;

        if self.adj_stack.is_empty() {
            return Err(KetError::NoAdj);
        }

        if self.adj_stack.len() == 1 {
            while let Some(instruction) = self.adj_stack.last_mut().unwrap().pop() {
                if let Some(processor) = self.config.live_quantum_execution.as_mut() {
                    match &instruction {
                        Instruction::Gate {
                            gate,
                            target,
                            control,
                        } => {
                            info!(
                                "live execution: gate={:?}, target={}, control={:?}",
                                gate, target, control
                            );
                            processor.gate(gate, *target, control)
                        }
                        _ => panic!(),
                    }
                }
                self.instructions.push(instruction);
            }
            self.adj_stack.pop();
        } else {
            let mut popped = self.adj_stack.pop().unwrap();
            while let Some(instruction) = popped.pop() {
                self.adj_stack.last_mut().unwrap().push(instruction);
            }
        }

        Ok(())
    }

    /// Prepares the process for quantum execution
    pub fn prepare_for_execution(&mut self) -> Result<()> {

        if let ProcessStatus::Building = self.metadata.status {
            if self.config.optimize {
                    self.optimize().unwrap();
            }
            let mut result = None;
            if let Some(processor) = self.config.batch_execution.as_mut() {
                processor.submit_execution(&self.instructions);
                self.metadata.status = ProcessStatus::Running;
                result = Some(processor.get_result());
                self.metadata.status = ProcessStatus::Terminated;
            } else {
                self.metadata.status = ProcessStatus::Ready;
            }

            if let Some(result) = result {
                self.set_result(result)?;
            }
        }
        Ok(())
    }

    /// Returns the status of the specified qubit
    pub fn get_qubit_status(&self, qubit: usize) -> &QubitStatus {
        &self.qubits[qubit]
    }

    /// Returns the measurement result at the specified index
    pub fn get_measurement(&self, index: usize) -> &Measurement {
        &self.measurements[index]
    }

    /// Returns the expected value result at the specified index
    pub fn get_exp_value(&self, index: usize) -> &ExpValue {
        &self.exp_values[index]
    }

    /// Returns the sample result at the specified index
    pub fn get_sample(&self, index: usize) -> &Sample {
        &self.samples[index]
    }

    /// Returns the dump result at the specified index
    pub fn get_dump(&self, index: usize) -> &Dump {
        &self.dumps[index]
    }

    /// Return process metadata
    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Set the quantum execution result
    ///
    /// This function allow to manually set the quantum execution result for the process.
    /// However, this function should only be used for testing purposes. The result must be provided
    /// by the quantum executor set in the configuration.
    pub fn set_result(&mut self, mut results: ResultData) -> Result<()> {
        if self.measurements.len() != results.measurements.len()
            || self.exp_values.len() != results.exp_values.len()
            || self.samples.len() != results.samples.len()
            || self.dumps.len() != results.dumps.len()
        {
            return Err(KetError::UnexpectedResultData);
        }
        results
            .measurements
            .drain(..)
            .zip(self.measurements.iter_mut())
            .for_each(|(result, measurement)| {
                measurement.result = Some(result);
            });

        results
            .exp_values
            .drain(..)
            .zip(self.exp_values.iter_mut())
            .for_each(|(result, exp_value)| {
                exp_value.result = Some(result);
            });

        results
            .samples
            .drain(..)
            .zip(self.samples.iter_mut())
            .for_each(|(result, sample)| {
                assert_eq!(result.0.len(), result.1.len());
                sample.result = Some(result);
            });

        results
            .dumps
            .drain(..)
            .zip(self.dumps.iter_mut())
            .for_each(|(result, dump)| {
                dump.result = Some(result);
            });

        self.metadata.execution_time = results.execution_time;

        self.metadata.status = ProcessStatus::Terminated;
        Ok(())
    }

    /// Return the instructions in JSON
    ///
    /// This functions is used in the C API for get the instructions out of the process.
    pub(crate) fn instructions_json(&self) -> String {
        serde_json::to_string(&self.instructions).unwrap()
    }

    /// Return the metadata in JSON
    ///
    /// This functions is used in the C API for get the metadata out of the process.
    pub(crate) fn metadata_json(&self) -> String {
        serde_json::to_string(&self.metadata).unwrap()
    }

    /// Return the quantum circuit in OpenQASM v2 format
    ///
    /// The `measurements` parameter determines if the generated code will include measurement instructions
    pub fn to_qasmv2(&self, measurements: bool, instruction_set: InstructionSet) -> Result<String> {
        to_qasmv2(self, measurements, instruction_set)
    }

    pub fn from_qasmv2(&mut self, qasm: &str, instruction_set: InstructionSet, ignore_qreg: bool) -> Result<()> {
        from_qasmv2(self, qasm, instruction_set, ignore_qreg)
    }

    pub fn optimize(&mut self) -> Result<()> {
        optimize(self)
    }
}

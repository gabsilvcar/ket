// SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
// SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

use crate::bitwise::*;
use crate::error::{KBWError, Result};
use crate::quantum_execution::QuantumExecution;
use itertools::Itertools;
use log::error;
use num::{complex::Complex64, Zero};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rayon::prelude::*;
use std::f64::consts::FRAC_1_SQRT_2;

pub struct Dense {
    state_0: Vec<Complex64>,
    state_1: Vec<Complex64>,
    state: bool,
}

impl Dense {
    fn get_states(&mut self) -> (&mut [Complex64], &mut [Complex64]) {
        self.state = !self.state;
        if self.state {
            (&mut self.state_1, &mut self.state_0)
        } else {
            (&mut self.state_0, &mut self.state_1)
        }
    }

    fn get_current_state(&self) -> &[Complex64] {
        if self.state {
            &self.state_0
        } else {
            &self.state_1
        }
    }
}

impl QuantumExecution for Dense {
    fn new(num_qubits: usize) -> Result<Self> {
        if num_qubits > 32 {
            error!("dense implementation supports up to 32 qubits");
            return Err(KBWError::UnsupportedNumberOfQubits);
        }

        let num_states = 1 << num_qubits;
        let mut state_0 = Vec::new();
        let mut state_1 = Vec::new();
        state_0.resize(num_states, Complex64::zero());
        state_1.resize(num_states, Complex64::zero());

        state_0[0] = Complex64::new(1.0, 0.0);

        Ok(Dense {
            state: true,
            state_0,
            state_1,
        })
    }

    fn pauli_x(&mut self, target: usize, control: &[usize]) {
        let (current_state, next_state) = self.get_states();

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                *amp = current_state[if ctrl_check(state, control) {
                    bit_flip(state, target)
                } else {
                    state
                }];
            });
    }

    fn pauli_y(&mut self, target: usize, control: &[usize]) {
        let (current_state, next_state) = self.get_states();

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[bit_flip(state, target)]
                        * if is_one_at(state, target) {
                            Complex64::i()
                        } else {
                            -Complex64::i()
                        };
                } else {
                    *amp = current_state[state];
                }
            });
    }

    fn pauli_z(&mut self, target: usize, control: &[usize]) {
        let (current_state, next_state) = self.get_states();

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) && is_one_at(state, target) {
                    *amp = -current_state[state];
                } else {
                    *amp = current_state[state];
                }
            });
    }

    fn hadamard(&mut self, target: usize, control: &[usize]) {
        let (current_state, next_state) = self.get_states();

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[bit_flip(state, target)] * FRAC_1_SQRT_2;
                } else {
                    *amp = Complex64::zero();
                }
            });

        current_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp *= if is_one_at(state, target) {
                        -FRAC_1_SQRT_2
                    } else {
                        FRAC_1_SQRT_2
                    };
                }
            });

        next_state
            .par_iter_mut()
            .zip(current_state.par_iter())
            .for_each(|(next_amp, current_amp)| {
                *next_amp += *current_amp;
            });
    }

    fn phase(&mut self, lambda: f64, target: usize, control: &[usize]) {
        let (current_state, next_state) = self.get_states();

        let phase = Complex64::exp(lambda * Complex64::i());

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) && is_one_at(state, target) {
                    *amp = current_state[state] * phase;
                } else {
                    *amp = current_state[state];
                }
            });
    }

    fn rx(&mut self, theta: f64, target: usize, control: &[usize]) {
        let (current_state, next_state) = self.get_states();

        let cons_theta_2 = Complex64::from(f64::cos(theta / 2.0));
        let sin_theta_2 = -Complex64::i() * f64::sin(theta / 2.0);

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[bit_flip(state, target)] * sin_theta_2;
                } else {
                    *amp = Complex64::zero();
                }
            });

        current_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp *= cons_theta_2;
                }
            });

        next_state
            .par_iter_mut()
            .zip(current_state.par_iter())
            .for_each(|(next_amp, current_amp)| {
                *next_amp += *current_amp;
            });
    }

    fn ry(&mut self, theta: f64, target: usize, control: &[usize]) {
        let (current_state, next_state) = self.get_states();

        let cons_theta_2 = Complex64::from(f64::cos(theta / 2.0));
        let p_sin_theta_2 = Complex64::from(f64::sin(theta / 2.0));
        let m_sin_theta_2 = -p_sin_theta_2;

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[bit_flip(state, target)]
                        * if is_one_at(state, target) {
                            p_sin_theta_2
                        } else {
                            m_sin_theta_2
                        };
                } else {
                    *amp = Complex64::zero();
                }
            });

        current_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp *= cons_theta_2;
                }
            });

        next_state
            .par_iter_mut()
            .zip(current_state.par_iter())
            .for_each(|(next_amp, current_amp)| {
                *next_amp += *current_amp;
            });
    }

    fn rz(&mut self, theta: f64, target: usize, control: &[usize]) {
        let (current_state, next_state) = self.get_states();

        let phase_0 = Complex64::exp(-theta / 2.0 * Complex64::i());
        let phase_1 = Complex64::exp(theta / 2.0 * Complex64::i());

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[state]
                        * if is_one_at(state, target) {
                            phase_1
                        } else {
                            phase_0
                        };
                } else {
                    *amp = current_state[state];
                }
            });
    }

    fn measure<R: Rng>(&mut self, target: usize, rng: &mut R) -> bool {
        let (current_state, next_state) = self.get_states();

        let p1: f64 = current_state
            .par_iter()
            .enumerate()
            .map(|(state, amp)| {
                if is_one_at(state, target) {
                    amp.norm().powi(2)
                } else {
                    0.0
                }
            })
            .sum();

        let p0 = match 1.0 - p1 {
            p0 if p0 >= 0.0 => p0,
            _ => 0.0,
        };

        let result = WeightedIndex::new([p0, p1]).unwrap().sample(rng) == 1;

        let p = 1.0 / f64::sqrt(if result { p1 } else { p0 });

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                *amp = if is_one_at(state, target) == result {
                    current_state[state] * p
                } else {
                    Complex64::zero()
                };
            });

        result
    }

    fn dump(&mut self, qubits: &[usize]) -> ket::DumpData {
        let state = self.get_current_state();
        let (basis_states, amplitudes_real, amplitudes_imag): (Vec<_>, Vec<_>, Vec<_>) = state
            .iter()
            .enumerate()
            .filter(|(_state, amp)| amp.norm() > 1e-15)
            .map(|(state, amp)| {
                let state = qubits
                    .iter()
                    .rev()
                    .enumerate()
                    .map(|(index, qubit)| (is_one_at(state, *qubit) as usize) << index)
                    .reduce(|a, b| a | b)
                    .unwrap_or(0);

                (Vec::from([state as u64]), amp.re, amp.im)
            })
            .multiunzip();

        ket::DumpData {
            basis_states,
            amplitudes_real,
            amplitudes_imag,
        }
    }

    fn debug_state(&self) -> Option<String> {
        let state = self.get_current_state();
        let mut state_str = String::new();
        state_str.push_str("State:\n");
        for (index, amp) in state.iter().enumerate() {
            if amp.norm() < 1e-15 {
                continue;
            }
            state_str.push_str(&format!("{:032b}: {:?}\n", index, amp));
        }
        Some(state_str)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bell_state_live() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let configuration =
            crate::quantum_execution::QubitManager::<crate::sparse::Sparse>::configuration(2, true);
        let mut process = ket::Process::new(configuration);
        let qubit_a = process.allocate_qubit()?;
        let qubit_b = process.allocate_qubit()?;

        process.apply_gate(ket::QuantumGate::Hadamard, qubit_a)?;
        process.ctrl_push(&[qubit_a])?;
        process.apply_gate(ket::QuantumGate::PauliX, qubit_b)?;
        process.ctrl_pop()?;

        let d_ab = process.dump(&[qubit_a, qubit_b])?;
        let m_a = process.measure(&[qubit_a])?;
        let m_b = process.measure(&[qubit_b])?;
        let d_ab2 = process.dump(&[qubit_a, qubit_b])?;

        println!("{:?}", process.get_dump(d_ab));
        println!("{:?}", process.get_measurement(m_a));
        println!("{:?}", process.get_measurement(m_b));
        println!("{:?}", process.get_dump(d_ab2));

        Ok(())
    }

    #[test]
    fn bell_state_batch() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let configuration =
            crate::quantum_execution::QubitManager::<crate::sparse::Sparse>::configuration(
                2, false,
            );
        let mut process = ket::Process::new(configuration);
        let qubit_a = process.allocate_qubit()?;
        let qubit_b = process.allocate_qubit()?;

        process.apply_gate(ket::QuantumGate::Hadamard, qubit_a)?;
        process.ctrl_push(&[qubit_a])?;
        process.apply_gate(ket::QuantumGate::PauliX, qubit_b)?;
        process.ctrl_pop()?;

        let d_ab = process.dump(&[qubit_a, qubit_b])?;
        let m_a = process.measure(&[qubit_a])?;
        let m_b = process.measure(&[qubit_b])?;
        let d_ab2 = process.dump(&[qubit_a, qubit_b])?;

        println!("{:?}", process.get_dump(d_ab));
        println!("{:?}", process.get_measurement(m_a));
        println!("{:?}", process.get_measurement(m_b));
        println!("{:?}", process.get_dump(d_ab2));

        Ok(())
    }
}

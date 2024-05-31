// SPDX-FileCopyrightText: 2024 Gabriel da Silva Cardoso <cardoso.gabriel@grad.ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

use quizx::circuit::Circuit;

pub fn random_pauli_exp(
    qs: usize,
    depth: usize,
    seed: u64,
    min_weight: usize,
    max_weight: usize,
) -> String {
    println!(
        "qubits: {}, depth: {}, min_weight: {}, max_weight: {}, seed: {}",
        qs, depth, min_weight, max_weight, seed
    );
    let c = Circuit::random_pauli_gadget()
        .qubits(qs)
        .depth(depth)
        .seed(seed)
        .min_weight(min_weight)
        .max_weight(max_weight)
        .build();

    println!("{}", c.to_qasm());
    c.to_qasm()
}

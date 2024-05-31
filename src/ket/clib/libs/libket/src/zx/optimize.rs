// SPDX-FileCopyrightText: 2024 Gabriel da Silva Cardoso <cardoso.gabriel@grad.ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

use std::fs::metadata;
use std::mem;
use quizx::circuit::Circuit;
use quizx::extract::ToCircuit;

use crate::error::Result;
use crate::qasmv2::instruction_set::InstructionSet;
use crate::{Configuration, Metadata, Process};
use quizx::hash_graph::Graph;
use regex::Regex;

pub fn optimize(process: &mut Process) -> Result<()> {
    println!("optimizing");
    let qasm = process.to_qasmv2(false, InstructionSet::QELIB).unwrap();

    prepare_process(process);

    let qasm_sections = separate_code_sections(&qasm);

    let mut is_first_iteration = true;

    for section in qasm_sections {
        println!("--{}", &section);
        let measure_regex = Regex::new(r"measure q\[(\d+)\] -> c\[(\d+)\];").unwrap();

        let mut measure_vec: Vec<Vec<usize>> = Vec::new();
        let mut measure_current_bit = 0;

        for cap in measure_regex.captures_iter(&*section) {
            let to_measure = cap[1].parse::<usize>().unwrap();
            let bit = cap[2].parse::<usize>().unwrap();

            if measure_vec.len() > 0 && &bit == &measure_current_bit {
                measure_vec.last_mut().unwrap().push(to_measure);
            } else {
                measure_vec.push(vec![to_measure]);
                measure_current_bit = bit;
            }
        }

        let dump_regex = Regex::new(r"dump\((\d+)\) q\[(\d+)\];").unwrap();
        let mut dump_vec: Vec<Vec<usize>> = Vec::new();
        let mut dump_current_bit = 0;

        for cap in dump_regex.captures_iter(&*section) {
            let to_dump = cap[2].parse::<usize>().unwrap();
            let bit = cap[1].parse::<usize>().unwrap();

            if dump_vec.len() > 0 && &bit == &dump_current_bit {
                dump_vec.last_mut().unwrap().push(to_dump);
            } else {
                dump_vec.push(vec![to_dump]);
                dump_current_bit = bit;
            }
        }



        let clean_qasm = measure_regex.replace_all(&*section, "").to_string();

        let qasm_optimized = zx_optimize(&clean_qasm);

        process.from_qasmv2(&*qasm_optimized, InstructionSet::QELIB, !is_first_iteration);

        if is_first_iteration {
            is_first_iteration = false;
        }

        for measure_val in measure_vec {
            process.measure(&measure_val).unwrap();
        }
        for dump_val in dump_vec {
            process.dump(&dump_val).unwrap();
        }
    }


    // process.from_qasmv2(&qasm_optimized, InstructionSet::QELIB);

    // let dump_regex = Regex::new(r"dump q\[(\d+)\];").unwrap();;
    // let mut dump_vec = Vec::new();
    // for cap in dump_regex.captures_iter(&*qasm) {
    //     dump_vec.push(cap[1].parse::<usize>().unwrap());
    // }
    // process.dump(&dump_vec).unwrap();

    Ok(())
}

fn prepare_process(process: &mut Process) {
    process.instructions.clear();
    process.ctrl_stack.clear();
    process.ctrl_list_is_up_to_date = false;
    process.adj_stack.clear();
    process.qubits.clear();
    process.qubit_allocated = 0;
    process.dumps.clear();
    process.measurements.clear();
}

fn zx_optimize(qasm: &str) -> String {
    let c = Circuit::from_qasm(qasm).unwrap();
    let mut g: Graph = c.clone().to_graph();
    quizx::simplify::clifford_simp(&mut g);
    let c_optimized = g.to_circuit().unwrap();
    c_optimized.to_qasm()
}

fn separate_code_sections(input: &str) -> Vec<String> {
    let mut sections: Vec<String> = Vec::new();
    let mut current_section = Vec::new();
    let mut measure_section = Vec::new();

    let re = Regex::new(r"(creg c\[\d+\];)").unwrap();


    if let Some(mat) = re.find(input) {
        let split_index = mat.start() + mat.as_str().len();
        let mut qasm_regs = &input[..split_index];
        let gates = &input[split_index..];


        for line in gates.lines() {
            if line.trim().starts_with("measure|dump") {
                measure_section.push(line.to_string());
            } else {
                if !measure_section.is_empty() {
                    current_section.append(&mut measure_section);
                    sections.push(format!("{}\n{}", &qasm_regs, current_section.join("\n")));
                    current_section = Vec::new();
                    measure_section = Vec::new();
                }
                current_section.push(line.to_string());
            }
        }

        if !measure_section.is_empty() {
            current_section.append(&mut measure_section);
        }
        if !current_section.join("\n").replace(|c: char| c.is_whitespace(), "").is_empty() {
            sections.push(format!("{}\n{}", &qasm_regs, current_section.join("\n")));
        }

        sections
    } else {
        return vec![input.to_string()]
    }
}
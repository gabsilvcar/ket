import os
import time
import csv

import numpy as np
import pandas as pd

from ket import *
import pyzx as zx


def execute(circuit, file_name):
    times_quizx = []
    times_pyzx = []
    times_pyzx_teleport = []
    times_pyzx_full = []

    original = circuit
    original_basic_gates = original.to_basic_gates()

    # Measure QUIZX optimization 10 times
    for _ in range(10):
        p = Process(execution="batch", optimize=True)
        p.import_qasmv2(circuit.to_qasm())
        t = time.time()
        p.zx_optimize()
        times_quizx.append(time.time() - t)
        quizx_optimized = zx.Circuit.from_qasm(p.get_qasmv2())
    print(quizx_optimized.verify_equality(original))

    # Measure PyZX optimizations 10 times
    for _ in range(10):
        g = original_basic_gates.to_graph()

        t = time.time()
        zx.simplify.clifford_simp(g)
        pyzx_optimized = zx.extract_circuit(g)
        times_pyzx.append(time.time() - t)

        g2 = original_basic_gates.to_graph()
        t = time.time()
        zx.simplify.teleport_reduce(g2)
        times_pyzx_teleport.append(time.time() - t)
        pyzx_teleported_reduce = zx.Circuit.from_graph(g2)

        g3 = original_basic_gates.to_graph()
        t = time.time()
        zx.simplify.full_reduce(g3)
        pyzx_full_reduce = zx.extract_circuit(g3)
        times_pyzx_full.append(time.time() - t)

    # Compute the mean of the collected times
    quizx_optimization_extraction_time = np.mean(times_quizx)
    pyzx_optimization_extraction_time = np.mean(times_pyzx)
    pyzx_teleport_reduction_time = np.mean(times_pyzx_teleport)
    pyzx_full_reduction_time = np.mean(times_pyzx_full)

    pt = original_basic_gates.tcount() / len(original_basic_gates.gates)

    return [file_name,
            pt,
            len(original.gates),
            len(original_basic_gates.gates),
            len(quizx_optimized.gates),
            len(pyzx_optimized.gates),
            len(pyzx_teleported_reduce.gates),
            len(pyzx_full_reduce.gates),

            original.tcount(),
            original_basic_gates.tcount(),
            quizx_optimized.tcount(),
            pyzx_optimized.tcount(),
            pyzx_teleported_reduce.tcount(),
            pyzx_full_reduce.tcount(),

            original.depth(),
            original_basic_gates.depth(),
            quizx_optimized.depth(),
            pyzx_optimized.depth(),
            pyzx_teleported_reduce.depth(),
            pyzx_full_reduce.depth(),

            original.twoqubitcount(),
            original_basic_gates.twoqubitcount(),
            quizx_optimized.twoqubitcount(),
            pyzx_optimized.twoqubitcount(),
            pyzx_teleported_reduce.twoqubitcount(),
            pyzx_full_reduce.twoqubitcount(),

            quizx_optimization_extraction_time,
            pyzx_optimization_extraction_time,
            pyzx_teleport_reduction_time,
            pyzx_full_reduction_time
            ]
    # return pt, gates, quizx_optimization_extraction_time, pyzx_optimization_extraction_time, pyzx_teleport_reduction_time, pyzx_full_reduction_time, tgates, depth, two_qubit


# Path to the directory containing the files
folders = ['circuits/small']
data = []
columns = ['file',
           'p(t)',

           'original-gates',
           'basic-gates',
           'quizx-clifford-simplified-gates',
           'pyzx-clifford-simplified-gates',
           'pyzx-teleport-reduced-gates',
           'pyzx-full-reduction-gates',

           'original-t-gates',
           'basic-t-gates',
           'quizx-clifford-simplified-t-gates',
           'pyzx-clifford-simplified-t-gates',
           'pyzx-teleport-reduced-t-gates',
           'pyzx-full-reduction-t-gates',

           'original-depth',
           'basic-depth',
           'quizx-clifford-simplified-depth',
           'pyzx-clifford-simplified-depth',
           'pyzx-teleport-reduced-depth',
           'pyzx-full-reduction-depth',

           'original-two-qubit',
           'basic-gates-two-qubit',
           'quizx-clifford-simplified-two-qubit',
           'pyzx-clifford-simplified-two-qubit',
           'pyzx-teleport-reduced-two-qubit',
           'pyzx-full-reduction-two-qubit',

           'quizx-clifford-simplified-time',
           "pyzx-clifford-simplified-time",
           "pyzx-teleport-reduced-time",
           "pyzx-full-reduction-time"
           ]
file = "benchmarks-random-clifford+T-2.0.csv"
if os.path.isfile(file):
    df = pd.read_csv(file)
    print("loaded previous data")
else:
    df = pd.DataFrame(columns=columns)
    print("starting from scratch")

i = 0

for i in range(100):
    qubit_amount = 10
    gate_count = 1000
    cliffordTGraph = zx.generate.cliffordT(qubit_amount, gate_count)
    extracted_circuit = zx.extract_circuit(cliffordTGraph.copy(), optimize_czs=False, optimize_cnots=0, up_to_perm=False)

    df = pd.concat([pd.DataFrame([execute(extracted_circuit, i)], columns=df.columns), df])

df.to_csv(file, index=False)

#
# # Save data to CSV
# with open('circuit_data.csv', 'w', newline='') as file:
#     writer = csv.writer(file)
#     headers = ['File Name', 'p(t)', 'Gates Original', 'Gates Original Basic', 'Gates QUIZX Optimized', 'Gates PyZX Optimized', 'Gates PyZX Teleported Reduce', 'Gates PyZX Full Reduce',
#                'QUIZX Optimization Time', 'PyZX Optimization Time', 'T-Gates Original', 'T-Gates Original Basic', 'T-Gates QUIZX Optimized', 'T-Gates PyZX Optimized', 'T-Gates PyZX Teleported Reduce', 'T-Gates PyZX Full Reduce',
#                'Depth Original', 'Depth Original Basic', 'Depth QUIZX Optimized', 'Depth PyZX Optimized', 'Depth PyZX Teleported Reduce', 'Depth PyZX Full Reduce',
#                'Two-Qubit Original', 'Two-Qubit Original Basic', 'Two-Qubit QUIZX Optimized', 'Two-Qubit PyZX Optimized', 'Two-Qubit PyZX Teleported Reduce', 'Two-Qubit PyZX Full Reduce']
#     writer.writerow(headers)
#     for row in data:
#         writer.writerow(row)
import os
import time
import csv

import numpy as np
import pandas as pd

from ket import *
import pyzx as zx

qubit_amount = 10
gate_count = 1000
cliffordTGraph = zx.generate.cliffordT(qubit_amount, gate_count)
extracted_circuit = zx.extract_circuit(cliffordTGraph.copy(), optimize_czs=False, optimize_cnots=0, up_to_perm=False)
p = Process(execution="batch", optimize=True)
p.import_qasmv2(extracted_circuit.to_qasm())
quizx_optimized = zx.Circuit.from_qasm(p.get_qasmv2())
g3 = extracted_circuit.to_basic_gates().to_graph()

zx.simplify.full_reduce(g3)
pyzx_full_reduce = zx.extract_circuit(g3)

print(p.get_qasmv2())
print("=============")
print(pyzx_full_reduce.to_qasm())
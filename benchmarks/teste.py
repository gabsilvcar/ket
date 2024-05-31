from ket import *
import pyzx as zx
qasm = """OPENQASM 2.0;
include "qelib1.inc";
qreg q[384];
h q[1];
"""
p = Process(execution="batch", optimize=True)
p.import_qasmv2(qasm)

# p.zx_optimize()
# print(p.get_qasmv2())

c1 = zx.Circuit.from_qasm(qasm)
c2 = zx.Circuit.from_qasm(p.get_qasmv2())

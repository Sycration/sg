"""Loads a .sgp file, solves it and prints the formula of every dimension, with SymPy.

    pip install '.[sympy]'
    python3 examples/python/derive.py tests/law_of_cosines.sgp
"""

import sys

import sggeo
import sympy

with open(sys.argv[1], encoding="utf-8") as f:
    doc = sggeo.Doc.from_sgp(f.read())
s = doc.solve()
print(f"sggeo {sggeo.version()}: solve {s.ok}, dof {s.dof}")
values = doc.values()
for i in range(doc.dim_count):
    try:
        f = doc.derive_dim(i)
    except sggeo.Error as e:
        print(f"dim {i}: {e}")
        continue
    e = f.sympy()
    print(f"{f.plain}\n  latex: {f.latex}\n  exact {f.exact}, value {f.value}, measured {f.actual}")
    print(f"  SymPy: {e}, simplified {sympy.simplify(e)}")
    if all(s.name in values for s in e.free_symbols):
        print(f"  at the document's values: {e.evalf(subs={s: values[s.name] for s in e.free_symbols})}")
try:
    sggeo.Doc.from_sgp("not sgp")
except sggeo.Error as e:
    print(f"bad input: {e}")

"""Builds the law of cosines construction from scratch (the law_of_cosines test in
tests/gen_example.rs), solves it, prints the derived formula of the angle, solves it for
a side with SymPy and writes the document as a .sgp file.

    pip install '.[sympy]'
    python3 examples/python/law_of_cosines.py law_of_cosines.sgp
"""

import sys

import sggeo
import sympy

if len(sys.argv) != 2:
    sys.exit("usage: python3 law_of_cosines.py OUT.sgp")
out = sys.argv[1]


def must(ok, what):
    if not ok:
        raise RuntimeError(f"{what} rejected")


doc = sggeo.Doc()

# a triangle with one corner on the origin
o = doc.origin
b = doc.add_point(4.0, 0.2)
c = doc.add_point(1.0, 3.0)
ab = doc.add_line(o, b)
bc = doc.add_line(b, c)
ca = doc.add_line(c, o)
must(doc.constrain("horizontal", ab), "horizontal OB")

# side lengths 5, 4, 3, labels at (0.3, 0.3) from each side's midpoint
for side, length in [(ab, "5"), (bc, "4"), (ca, "3")]:
    d = doc.add_dim("length", side, off_x=0.3, off_y=0.3)
    must(doc.dim_driving(d), "length is determined already")
    must(doc.set_dim(d, length), "length value")

# the angle at the origin, measured
ang = doc.add_dim("angle", ab, ca)
must(doc.set_dim_driving(ang, False), "measured angle")

s = doc.solve()
must(s.ok, "solve")
print(f"solved, {s.dof} degrees of freedom left")

f = doc.derive_dim(ang)
print(f"{f.plain}\n  latex: {f.latex}\n  exact {f.exact}, value {f.value}, measured {f.actual}")

# the formula as a SymPy expression in the sides d1, d2, d3, solved for the side opposite the angle
theta, d2 = sympy.Symbol("θ1", real=True), sympy.Symbol("d2", positive=True)
e = f.sympy().subs(sympy.Symbol("d2", real=True), d2)
print(f"  {d2} = {sympy.solve(sympy.Eq(theta, e), d2)[0]}")

with open(out, "w", encoding="utf-8") as fh:
    fh.write(doc.to_sgp())
print(f"wrote {out}")

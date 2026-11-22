"""Tests of the Python module sggeo, which tests/python.rs builds and runs them against.

SGGEO_EXAMPLES names the directory of example constructions (test_examples/); every
SGGEO_EXAMPLE_STRIDE-th of them (default 8) has its formulas checked through SymPy.
"""

import glob
import math
import os
import threading
import unittest

import sggeo
import stub
import sympy


def triangle():
    """The triangle of the README: driving sides 5, 4, 3 and the measured angle at the origin."""
    doc = sggeo.Doc()
    o = doc.origin
    b = doc.add_point(4.0, 0.2)
    c = doc.add_point(1.0, 3.0)
    ob, bc, co = doc.add_line(o, b), doc.add_line(b, c), doc.add_line(c, o)
    assert doc.constrain("horizontal", ob)
    for side, length in [(ob, "5"), (bc, "4"), (co, "3")]:
        assert doc.set_dim(doc.add_dim("length", side), length)
    angle = doc.add_dim("angle", ob, co)
    assert doc.set_dim_driving(angle, False)
    return doc, angle


def value_at(expr, values):
    """Expression `expr` evaluated with its symbols at `values` (by name)."""
    z = complex(expr.evalf(30, subs={s: values[s.name] for s in expr.free_symbols}))
    assert abs(z.imag) < 1e-9 * max(1.0, abs(z.real)), z
    return z.real


class Construction(unittest.TestCase):
    def test_triangle(self):
        doc, angle = triangle()
        solved = doc.solve()
        self.assertTrue(solved.ok and solved)
        self.assertEqual((solved.dof, solved.errors), (0, []))
        self.assertEqual(doc.dim_name(angle), "θ1")
        self.assertEqual(doc.dim_kind(angle), "angle")
        self.assertFalse(doc.dim_driving(angle))
        f = doc.derive_dim(angle)
        self.assertEqual(f.plain, "θ1 = acos((d1^2 + d3^2 - d2^2)/(2d1*d3))")
        self.assertEqual(str(f), f.plain)
        self.assertEqual(f.name, "θ1")
        self.assertTrue(f.exact and f.is_angle)
        self.assertAlmostEqual(f.value, math.acos(0.6))
        self.assertAlmostEqual(f.actual, doc.dim_value(angle))
        self.assertIn("\\arccos", f._repr_latex_())
        self.assertEqual(f.code("python"), "import math\ntheta1 = math.acos((d1**2 + d3**2 - d2**2) / (2 * d1 * d3))")

    def test_reading(self):
        doc, angle = triangle()
        doc.solve()
        self.assertEqual(doc.ent_kind(doc.origin), "point")
        self.assertEqual(doc.ent_kind(doc.x_axis), "line")
        self.assertEqual(doc.point_pos(doc.origin), (0.0, 0.0))
        self.assertIsNone(doc.point_pos(doc.x_axis))
        self.assertIsNone(doc.ent_kind(10_000))
        self.assertTrue(math.isnan(doc.dim_value(10_000)))
        self.assertLess(doc.dim_uncertainty(angle), 1e-11)
        # the lists of the library: each language writes the formula, each kind reads back
        f = doc.derive_dim(angle)
        self.assertIn(("sympy", "SymPy"), sggeo.code_languages())
        for name, _ in sggeo.code_languages():
            self.assertTrue(f.code(name))
        self.assertEqual(doc.dim_kind(angle), "angle")
        self.assertIn("angle", sggeo.dim_kinds())
        self.assertIn("on_curve", sggeo.con_kinds())
        self.assertEqual(sggeo.ent_kinds()[0], doc.ent_kind(doc.origin))
        self.assertTrue(math.isnan(doc.dim_uncertainty(10_000)))
        self.assertEqual(doc.dim_count, 4)
        self.assertEqual(doc.values()["d1"], 5.0)
        a, b, inf = doc.line_pts(doc.dim_ents(0)[0])
        self.assertFalse(inf)
        x, y = doc.point_pos(b)
        self.assertAlmostEqual(math.hypot(x, y), 5.0)
        polys = doc.ent_samples(doc.dim_ents(0)[0], -10, -10, 10, 10, 0.01)
        self.assertEqual(len(polys), 1)
        self.assertTrue(all(len(p) == 2 for p in polys[0]))

    def test_errors(self):
        doc = sggeo.Doc()
        with self.assertRaisesRegex(sggeo.Error, "unknown dimension kind"):
            doc.add_dim("nope", doc.origin)
        self.assertTrue(issubclass(sggeo.Error, ValueError))
        with self.assertRaises(sggeo.Error):
            doc.add_line(doc.origin, 10_000)
        with self.assertRaises(sggeo.Error):
            triangle()[0].derive_dim(10_000)
        with self.assertRaises(sggeo.Error):
            triangle()[0].derive_dim(0).code("cobol")
        self.assertIsNotNone(doc.check_expr("1 +"))

    def test_undo_copy_and_sgp(self):
        doc, angle = triangle()
        doc.solve()
        text = doc.to_sgp()
        back = sggeo.Doc.from_sgp(text)
        back.solve()
        self.assertEqual(back.derive_dim(angle).plain, doc.derive_dim(angle).plain)
        copy = doc.copy()
        self.assertTrue(doc.set_dim(0, "6"))
        doc.solve()
        self.assertEqual(copy.values()["d1"], 5.0)
        self.assertEqual(doc.values()["d1"], 6.0)
        self.assertTrue(doc.undo())
        doc.solve()
        self.assertEqual(doc.values()["d1"], 5.0)
        n = doc.undo_len()
        p = doc.add_point(7, 7)
        q = doc.add_point(8, 9)
        doc.add_line(p, q)
        self.assertTrue(doc.join_undo(n))
        self.assertTrue(doc.undo())
        self.assertEqual(doc.undo_len(), n)

    def test_constructions(self):
        doc = sggeo.Doc()
        a, b, c = doc.add_point(0, 0), doc.add_point(4, 0), doc.add_point(1, 3)
        circle, center = doc.add_circle_through(a, b, c)
        self.assertEqual(doc.ent_kind(circle), "circle")
        self.assertEqual(doc.circle_vals(circle)[0], center)
        line, mid = doc.add_perp_bisector(a, b)
        self.assertEqual((doc.ent_kind(line), doc.ent_kind(mid)), ("line", "point"))
        ellipse = doc.add_conic("ellipse", a, 3, 2)
        self.assertEqual(doc.conic_vals(ellipse)[0], "ellipse")
        self.assertEqual(len(doc.add_foci(ellipse)), 2)
        self.assertTrue(doc.add_constraints([("horizontal", doc.add_line(a, b), None)]))
        self.assertTrue(doc.solve().ok)
        self.assertTrue(doc.delete(ents=[ellipse]))
        self.assertIsNone(doc.ent_kind(ellipse))

    def test_derive_releases_the_gil(self):
        doc, angle = triangle()
        doc.solve()
        out = []
        threads = [threading.Thread(target=lambda: out.append(doc.derive_dim(angle).plain)) for _ in range(4)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        self.assertEqual(len(set(out)), 1)
        # a set flag stops the search at once, with the value as a number
        cancel = sggeo.Cancel()
        cancel.cancel()
        f = doc.derive_dim(angle, extra_ms=10_000, cancel=cancel)
        self.assertFalse(f.exact)
        self.assertAlmostEqual(float(f.sympy()), math.acos(0.6))

    def test_latex_png(self):
        png = sggeo.latex_png("\\sqrt{2}", 16, 96, 0x336699)
        self.assertEqual(png[:8], b"\x89PNG\r\n\x1a\n")
        with self.assertRaises(ValueError):
            sggeo.latex_png("x", rgb=1 << 24)

    def test_version(self):
        self.assertEqual(sggeo.version(), sggeo.__version__)


class Typing(unittest.TestCase):
    def test_stub_is_current(self):
        # sggeo.pyi, which the wheel carries for editors, is what stub.py writes from this module
        path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "sggeo.pyi")
        with open(path, encoding="utf-8") as f:
            self.assertEqual(f.read(), stub.stub(), "sggeo.pyi is out of date: python3 tests/python/stub.py > sggeo.pyi")

    def test_limits(self):
        # at a = b the focus distance holds only where b >= a; the law of cosines has no such limit here
        path = os.path.join(os.environ["SGGEO_EXAMPLES"], "gen_conics", "ellipse_degenerate_circle_focus.sgp")
        with open(path, encoding="utf-8") as f:
            doc = sggeo.Doc.from_sgp(f.read())
        doc.solve()
        self.assertEqual([p for p, _ in doc.derive_dim(3).limits], ["b1^2 - a1^2 ≥ 0"])
        doc, angle = triangle()
        doc.solve()
        self.assertEqual(doc.derive_dim(angle).limits, [])

    def test_svg(self):
        doc, angle = triangle()
        doc.solve()
        svg = doc._repr_svg_()
        self.assertTrue(svg.startswith("<svg ") and svg.endswith("</svg>"))
        self.assertIn("θ1 = 53.1301°", svg)
        self.assertIn('width="200"', doc.to_svg(200, 100))
        with self.assertRaises(sggeo.Error):
            doc.to_svg(5, 5)


class SymPy(unittest.TestCase):
    def test_formula_as_expression(self):
        doc, angle = triangle()
        doc.solve()
        e = doc.derive_dim(angle).sympy()
        d1, d2, d3 = sympy.symbols("d1 d2 d3", real=True)
        self.assertEqual(e.free_symbols, {d1, d2, d3})
        self.assertEqual(sympy.simplify(e - sympy.acos((d1**2 + d3**2 - d2**2) / (2 * d1 * d3))), 0)
        self.assertAlmostEqual(value_at(e, doc.values()), math.acos(0.6))
        # the cosine of the angle is a rational function of the sides
        self.assertEqual(sympy.cos(e), (d1**2 + d3**2 - d2**2) / (2 * d1 * d3))

    def test_code(self):
        doc, angle = triangle()
        doc.solve()
        self.assertEqual(
            doc.derive_dim(angle).code("sympy"),
            'import sympy\nd1, d2, d3 = sympy.symbols("d1 d2 d3", real=True)\n'
            "theta1 = sympy.acos((d1**2 + d3**2 - d2**2) / (2 * d1 * d3))",
        )

    def test_expressions_as_arguments(self):
        doc, angle = triangle()
        d1, d2 = sympy.symbols("d1 d2", real=True)
        # a SymPy expression drives a dimension, and defines a variable
        self.assertTrue(doc.set_dim(1, sympy.Rational(4, 5) * d1))
        v = doc.add_def_var(sympy.sqrt(d1**2 - d2**2) + abs(d2 - d1) + sympy.E, name="h")
        self.assertEqual(doc.var_name(v), "h")
        self.assertEqual(doc.var_expr(v), "sqrt(d1^2 - d2^2) + abs(d1 - d2) + exp(1)")
        self.assertTrue(doc.solve().ok)
        self.assertAlmostEqual(doc.var_value(v), 3 + 1 + math.e)
        f = doc.derive_var(v)
        self.assertAlmostEqual(f.actual, 3 + 1 + math.e)
        self.assertNotIn("h", doc.values())
        self.assertAlmostEqual(value_at(f.sympy(), doc.values()), 3 + 1 + math.e)
        # numbers and text pass as they are; angles typed as numbers are degrees
        self.assertTrue(doc.set_dim(2, 3))
        self.assertTrue(doc.set_dim(2, 3.0))
        self.assertIsNone(doc.check_expr(d1**2 / 3))
        self.assertIsNotNone(doc.check_expr(sympy.Symbol("nowhere")))
        # a curve as an equation or as y = f(x)
        x, y = sympy.symbols("x y")
        self.assertIsNone(doc.check_curve(sympy.Eq(x**2 + y**2, d1**2)))
        curve = doc.add_curve(x**2 / d1)
        self.assertEqual(doc.curve_src(curve), "x^2/d1")
        self.assertTrue(doc.set_curve(curve, sympy.Eq(y, x**3)))

    def test_real_roots(self):
        # an odd root of a negative number is the real root, as sggeo evaluates it
        doc = sggeo.Doc()
        k = doc.add_var("k", -8.0)
        v = doc.add_def_var("cbrt(k) + k^(2/3)")
        doc.solve()
        self.assertAlmostEqual(doc.var_value(v), 2.0)
        f = doc.derive_var(v)
        self.assertAlmostEqual(f.actual, 2.0)
        # the same in the Python code, with the math module
        scope = {"k": -8.0}
        exec(f.code("python"), scope)
        self.assertAlmostEqual(scope["f1"], 2.0)
        self.assertEqual(f.code("sympy").splitlines()[-1], "f1 = sympy.real_root(k, 3)**2 + sympy.real_root(k, 3)")
        self.assertAlmostEqual(value_at(f.sympy(), doc.values()), 2.0)

    def test_example_formulas(self):
        """The formulas of the example constructions, as SymPy expressions, evaluate to their values."""
        root = os.environ.get("SGGEO_EXAMPLES")
        if not root:
            self.skipTest("SGGEO_EXAMPLES is not set")
        stride = int(os.environ.get("SGGEO_EXAMPLE_STRIDE", "8"))
        files = sorted(glob.glob(os.path.join(root, "*", "*.sgp")))[::stride]
        self.assertTrue(files, f"no examples in {root}")
        checked = 0
        for path in files:
            with open(path, encoding="utf-8") as fh:
                doc = sggeo.Doc.from_sgp(fh.read())
            doc.solve()
            values = doc.values()
            for i in range(doc.dim_count):
                if doc.dim_name(i) is None or doc.dim_driving(i):
                    continue
                try:
                    f = doc.derive_dim(i)
                except sggeo.Error:
                    continue
                e = f.sympy()
                # free coordinates of the construction have no value to read back
                if any(s.name not in values for s in e.free_symbols):
                    continue
                with self.subTest(file=os.path.relpath(path, root), dim=f.name):
                    self.assertTrue(math.isclose(value_at(e, values), f.value, rel_tol=1e-7, abs_tol=1e-9), f.code("sympy"))
                    checked += 1
        self.assertGreater(checked, len(files) // 2)


if __name__ == "__main__":
    unittest.main()

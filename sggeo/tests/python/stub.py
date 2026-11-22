"""Writes sggeo.pyi, the type stub of the sggeo module, for editors and type checkers: the types below with
the parameters and docstrings of the built module itself. With the module importable,

    python3 tests/python/stub.py > sggeo.pyi     # in the sggeo directory

test_sggeo checks that sggeo.pyi is what this writes, so a change to the module's functions or their
documentation shows there. A function of the module without a type here, or a type here for a function
the module does not have, is an error.
"""

import ast
import inspect
import sys
import textwrap

import sggeo

# Each class's members as name: signature with types, "@property name: type" for a property and
# "@staticmethod" before a static method's signature. The parameters must be those of the module.
TYPES = {
    "Doc": {
        "__init__": "(self) -> None",
        "from_sgp": "@staticmethod (text: str) -> Doc",
        "to_sgp": "(self) -> str",
        "to_svg": "(self, width: float = 480.0, height: float = 360.0) -> str",
        "_repr_svg_": "(self) -> str",
        "solve": "(self) -> Solved",
        "undo": "(self) -> bool",
        "redo": "(self) -> bool",
        "checkpoint": "(self) -> None",
        "dof": "(self) -> int",
        "undo_len": "(self) -> int",
        "join_undo": "(self, len: int) -> bool",
        "copy": "(self) -> Doc",
        "__copy__": "(self) -> Doc",
        "load_sgp": "(self, text: str) -> None",
        "apply_sgp": "(self, text: str) -> bool",
        "ent_by_name": "(self, name: str) -> int | None",
        "clear": "(self) -> None",
        "generation": "@property int",
        "conflict_count": "@property int",
        "origin": "@property int",
        "x_axis": "@property int",
        "y_axis": "@property int",
        "dim_count": "@property int",
        "var_count": "@property int",
        "ent_count": "@property int",
        "con_count": "@property int",
        "add_point": "(self, x: float, y: float) -> int",
        "add_line": "(self, a: int, b: int, infinite: bool = False, axis: str | None = None) -> int",
        "add_circle": "(self, c: int, r: float) -> int",
        "add_arc": "(self, c: int, a: int, b: int) -> int",
        "add_curve": "(self, src: _ExprLike) -> int",
        "set_curve": "(self, i: int, src: _ExprLike) -> bool",
        "add_conic": "(self, kind: str, c: int, a: float, b: float, t: float = 0.0) -> int",
        "add_point_on": "(self, c: int, x: float, y: float) -> int",
        "add_circle_to": "(self, c: int, p: int) -> int",
        "add_arc_toward": "(self, c: int, a: int, x: float, y: float) -> int",
        "add_midpoint": "(self, a: int, b: int) -> int",
        "add_line_midpoint": "(self, l: int) -> int",
        "add_intersection": "(self, a: int, b: int, x: float, y: float) -> int",
        "add_intersection_near": "(self, a: int, b: int, x: float, y: float) -> int",
        "add_perp_line": "(self, l: int, p: int) -> int",
        "add_parallel_line": "(self, l: int, p: int) -> int",
        "add_perp_bisector": "(self, a: int, b: int) -> tuple[int, int]",
        "add_angle_bisector": "(self, l1: int, l2: int, x: float, y: float) -> tuple[int, int]",
        "add_tangent_line": "(self, p: int, c: int, x: float, y: float) -> tuple[int, int]",
        "add_tangent_line_near": "(self, p: int, c: int, x: float, y: float) -> tuple[int, int]",
        "add_reflection": "(self, p: int, l: int) -> int",
        "add_circle_through": "(self, a: int, b: int, c: int) -> tuple[int, int]",
        "add_arc_through": "(self, a: int, m: int, b: int) -> tuple[int, int]",
        "add_conic_by_foci": "(self, kind: str, f1: int, f2: int, p: int) -> int",
        "add_parabola_by_focus": "(self, f: int, l: int) -> int",
        "add_conic_from": "(self, kind: str, c: int, ax: float, ay: float, sx: float, sy: float) -> int",
        "add_parabola_toward": "(self, c: int, x: float, y: float) -> int",
        "add_foci": "(self, c: int) -> list[int]",
        "add_directrix": "(self, c: int, x: float, y: float) -> int",
        "add_asymptotes": "(self, c: int) -> list[int]",
        "constrain": "(self, kind: str, a: int, b: int | None = None) -> bool",
        "constrain_near": "(self, kind: str, a: int, b: int | None, x: float, y: float) -> bool",
        "can_constrain": "(self, kind: str, a: int, b: int | None = None) -> bool",
        "constrain_symmetric": "(self, a: int, b: int, l: int) -> bool",
        "add_constraints": "(self, cons: Sequence[tuple[str, int, int | None]]) -> bool",
        "add_dim": "(self, kind: str, a: int, b: int | None = None, off_x: float = 0.3, off_y: float = 0.3) -> int",
        "set_dim": "(self, i: int, value: _ExprLike) -> bool",
        "set_dim_driving": "(self, i: int, driving: bool) -> bool",
        "set_dim_numeric": "(self, i: int, numeric: bool) -> bool",
        "set_dim_offset": "(self, i: int, off_x: float, off_y: float) -> None",
        "drag_dim_offset": "(self, i: int, off_x: float, off_y: float) -> None",
        "toggle_supp": "(self, i: int) -> bool",
        "add_var": "(self, name: str | None = None, value: float = 1.0, free: bool = True) -> int",
        "add_def_var": "(self, expr: _ExprLike, name: str | None = None) -> int",
        "set_var": "(self, i: int, value: float) -> bool",
        "set_var_expr": "(self, i: int, expr: _ExprLike) -> bool",
        "check_expr": "(self, expr: _ExprLike) -> str | None",
        "check_curve": "(self, src: _ExprLike) -> str | None",
        "set_var_free": "(self, i: int, free: bool) -> bool",
        "set_var_numeric": "(self, i: int, numeric: bool) -> bool",
        "delete_var": "(self, i: int) -> bool",
        "delete_ent": "(self, e: int) -> bool",
        "delete_con": "(self, c: int) -> bool",
        "delete_dim": "(self, i: int) -> bool",
        "delete": "(self, *, ents: Sequence[int] = ..., cons: Sequence[int] = ..., dims: Sequence[int] = ..., vars: Sequence[int] = ...) -> bool",
        "lock_point": "(self, e: int, on: bool = True) -> bool",
        "lock_points": "(self, pts: Sequence[int], on: bool = True) -> bool",
        "point_locked": "(self, e: int) -> bool",
        "set_construction": "(self, e: int, on: bool = True) -> bool",
        "set_construction_all": "(self, ents: Sequence[int], on: bool = True) -> bool",
        "set_hidden": "(self, e: int, on: bool = True) -> bool",
        "set_infinite": "(self, e: int, on: bool = True) -> bool",
        "toggle_infinite": "(self, lines: Sequence[int]) -> bool",
        "drag_point": "(self, e: int, x: float, y: float) -> bool",
        "drag_points": "(self, pts: Sequence[int], targets: Sequence[tuple[float, float]]) -> bool",
        "drag_circle": "(self, e: int, x: float, y: float) -> bool",
        "drag_conic_handle": "(self, e: int, which: int, x: float, y: float) -> bool",
        "drag_curve": "(self, e: int, x: float, y: float) -> bool",
        "drag_var": "(self, i: int, value: float) -> bool",
        "rename": "(self, old: str, new: str) -> bool",
        "dim_name": "(self, i: int) -> str | None",
        "dim_value": "(self, i: int) -> float",
        "dim_uncertainty": "(self, i: int) -> float",
        "dim_driving": "(self, i: int) -> bool",
        "dim_numeric": "(self, i: int) -> bool",
        "dim_offset": "(self, i: int) -> tuple[float, float] | None",
        "dim_anchor": "(self, i: int) -> tuple[float, float] | None",
        "dim_anchor_of": "(self, kind: str, a: int, b: int | None = None) -> tuple[float, float]",
        "dim_expr": "(self, i: int) -> str | None",
        "dim_set_value": "(self, i: int) -> float",
        "dim_ents": "(self, i: int) -> list[int]",
        "dim_supp": "(self, i: int) -> bool",
        "dim_conflict": "(self, i: int) -> bool",
        "dim_error": "(self, i: int) -> str | None",
        "dim_kind": "(self, i: int) -> str | None",
        "var_name": "(self, i: int) -> str | None",
        "var_value": "(self, i: int) -> float",
        "var_expr": "(self, i: int) -> str | None",
        "var_free": "(self, i: int) -> bool",
        "var_numeric": "(self, i: int) -> bool",
        "ent_kind": "(self, e: int) -> str | None",
        "ent_name": "(self, e: int) -> str | None",
        "point_pos": "(self, e: int) -> tuple[float, float] | None",
        "line_pts": "(self, e: int) -> tuple[int, int, bool] | None",
        "circle_vals": "(self, e: int) -> tuple[int, float] | None",
        "arc_pts": "(self, e: int) -> tuple[int, int, int] | None",
        "conic_vals": "(self, e: int) -> tuple[str, int, float, float, float] | None",
        "curve_src": "(self, e: int) -> str | None",
        "ent_construction": "(self, e: int) -> bool",
        "ent_hidden": "(self, e: int) -> bool",
        "ent_fixed": "(self, e: int) -> bool",
        "ent_conflict": "(self, e: int) -> bool",
        "ent_infinite": "(self, e: int) -> bool",
        "con_kind": "(self, c: int) -> str | None",
        "con_ents": "(self, c: int) -> list[int]",
        "con_conflict": "(self, c: int) -> bool",
        "con_error": "(self, c: int) -> str | None",
        "con_turned": "(self, c: int) -> bool",
        "con_contact": "(self, c: int) -> tuple[float, float, bool] | None",
        "ent_samples": "(self, e: int, x0: float, y0: float, x1: float, y1: float, px: float) -> list[list[tuple[float, float]]]",
        "derive_dim": "(self, i: int, extra_ms: int = 0, cancel: Cancel | None = None) -> Formula",
        "derive_var": "(self, i: int, extra_ms: int = 0, cancel: Cancel | None = None) -> Formula",
        "values": "(self) -> dict[str, float]",
    },
    "Formula": {
        "name": "@property str",
        "plain": "@property str",
        "latex": "@property str",
        "latex_short": "@property str",
        "defs": "@property list[tuple[str, str]]",
        "exact": "@property bool",
        "is_angle": "@property bool",
        "value": "@property float",
        "actual": "@property float",
        "size": "@property int",
        "limits": "@property list[tuple[str, str]]",
        "code": "(self, lang: str) -> str",
        "sympy": "(self) -> Any",
        "_repr_latex_": "(self) -> str",
        "__str__": "(self) -> str",
        "__repr__": "(self) -> str",
    },
    "Solved": {
        "ok": "@property bool",
        "dof": "@property int",
        "errors": "@property list[str]",
        "__bool__": "(self) -> bool",
        "__repr__": "(self) -> str",
    },
    "Cancel": {
        "__init__": "(self) -> None",
        "cancel": "(self) -> None",
    },
}

FUNCTIONS = {
    "latex_png": "(latex: str, size_pt: float = 12.0, dpi: float = 96.0, rgb: int = 0) -> bytes",
    "code_languages": "() -> list[tuple[str, str]]",
    "con_kinds": "() -> list[str]",
    "dim_kinds": "() -> list[str]",
    "ent_kinds": "() -> list[str]",
    "version": "() -> str",
}

# the members of a class that are listed: its own, without Python's machinery, plus the methods above
SPECIAL = {"__init__", "__copy__", "__str__", "__repr__", "__bool__"}


def members(cls):
    names = {n for n in vars(cls) if not (n.startswith("__") and n not in SPECIAL)}
    # a class with a constructor of its own has __init__ in TYPES; the module's classes show it as __new__
    if "__new__" in vars(cls) and cls.__text_signature__ is not None:
        names.add("__init__")
    return names


def params(sig):
    """(name, kind, default) of each parameter of an annotated signature, as `inspect` names them."""
    fn = ast.parse(f"def f{sig}: ...").body[0].args
    out = []
    pos = fn.posonlyargs + fn.args
    defaults = [None] * (len(pos) - len(fn.defaults)) + fn.defaults
    for a, d in zip(pos, defaults):
        out.append((a.arg, "POSITIONAL_OR_KEYWORD", None if d is None else ast.unparse(d)))
    for a, d in zip(fn.kwonlyargs, fn.kw_defaults):
        out.append((a.arg, "KEYWORD_ONLY", None if d is None else ast.unparse(d)))
    return out


def runtime_params(obj, drop_self):
    sig = inspect.signature(obj)
    out = []
    for p in sig.parameters.values():
        kind = "KEYWORD_ONLY" if p.kind is p.KEYWORD_ONLY else "POSITIONAL_OR_KEYWORD"
        default = None if p.default is p.empty else ("..." if p.default is Ellipsis else repr(p.default))
        out.append((p.name, kind, default))
    return out[1:] if drop_self else out


def check(where, sig, obj, drop_self):
    if obj is None:
        return
    mine = params(sig.split(" -> ")[0])
    if drop_self:
        assert mine and mine[0][0] == "self", f"{where}: no self"
        mine = mine[1:]
    theirs = runtime_params(obj, False)
    if theirs and theirs[0][0] in ("self", "$self"):
        theirs = theirs[1:]
    if mine != theirs:
        sys.exit(f"{where}: the module has the parameters {theirs}, the types here {mine}")


def doc(obj, indent):
    text = inspect.getdoc(obj)
    if not text:
        return ""
    body = text.replace("\\", "\\\\").replace('"""', '\\"\\"\\"')
    if "\n" not in body and len(indent) + len(body) + 6 <= 110:
        return f'{indent}"""{body}"""\n'
    return f'{indent}"""\n{textwrap.indent(body, indent)}\n{indent}"""\n'


def stub():
    out = [
        '"""' + inspect.getdoc(sggeo) + '"""',
        "",
        "# Written by tests/python/stub.py from the module; do not edit.",
        "",
        "from collections.abc import Sequence",
        "from typing import Any, final",
        "",
        "# an expression as sggeo reads it: text, a number, or a SymPy expression (a curve: a sympy.Eq)",
        "_ExprLike = str | float | Any",
        "",
        f"__all__ = {sorted(sggeo.__all__)!r}",
        "__version__: str",
        "",
        "class Error(ValueError):",
        doc(sggeo.Error, "    ").rstrip("\n") or "    ...",
    ]
    for cls_name, types in TYPES.items():
        cls = getattr(sggeo, cls_name)
        have, want = members(cls), set(types)
        if have != want:
            sys.exit(f"{cls_name}: no type here for {sorted(have - want)}, no such member for {sorted(want - have)}")
        # (the classes of the module cannot be subclassed)
        out += ["", "@final", f"class {cls_name}:", doc(cls, "    ").rstrip("\n")]
        for name, sig in types.items():
            # (a static method's docstring is that of the function, not of its staticmethod wrapper)
            obj = cls if name == "__init__" else getattr(cls, name) if sig.startswith("@staticmethod ") else vars(cls)[name]
            if sig.startswith("@property "):
                out.append(f"    @property\n    def {name}(self) -> {sig[len('@property '):]}:")
            elif sig.startswith("@staticmethod "):
                sig = sig[len("@staticmethod "):]
                check(f"{cls_name}.{name}", sig, getattr(cls, name), False)
                out.append(f"    @staticmethod\n    def {name}{sig}:")
            else:
                check(f"{cls_name}.{name}", sig, None if name == "__init__" or name in SPECIAL - {"__copy__"} else obj, True)
                out.append(f"    def {name}{sig}:")
            text = "" if name == "__init__" else doc(obj, "        ")
            out.append(text.rstrip("\n") if text else "        ...")
    have = {n for n in vars(sggeo) if callable(getattr(sggeo, n)) and not inspect.isclass(getattr(sggeo, n)) and not n.startswith("_")}
    if have != set(FUNCTIONS):
        sys.exit(f"functions: no type here for {sorted(have - set(FUNCTIONS))}, no such function for {sorted(set(FUNCTIONS) - have)}")
    for name, sig in FUNCTIONS.items():
        check(name, sig, getattr(sggeo, name), False)
        out += ["", f"def {name}{sig}:", doc(getattr(sggeo, name), "    ").rstrip("\n") or "    ..."]
    return "\n".join(out) + "\n"


if __name__ == "__main__":
    sys.stdout.write(stub())

# sggeo

Geometry model, constraint solver and closed-form formula derivation for 2D geometric
constructions.

A construction is a document of entities (points, lines, circles, arcs, conics, curves),
constraints between them, dimensions and variables. sggeo solves the constraints numerically and
derives a formula for a measured dimension or a variable in terms of the driving dimensions and
the other variables, for example `θ1 = acos((d1^2 + d3^2 - d2^2)/(2d1*d3))` for an angle of a
triangle with three driving side lengths.

sggeo is a Rust crate, a C shared library, a WebAssembly module with JavaScript bindings and a
Python module, whose formulas are SymPy expressions.

## Contents

| Platform | Files |
| --- | --- |
| Linux | `libsggeo.so`, `sggeo.h` |
| macOS | `libsggeo.dylib` (install name `@rpath/libsggeo.dylib`), `sggeo.h` |
| Windows | `sggeo.dll`, `libsggeo.dll.a` (import library), `sggeo.h` |
| Windows XP | `sggeo.dll`, `sggeo.lib` (import library), `sggeo.h` |
| Web | `sggeo_bg.wasm`, `sggeo.js` (ES module), `sggeo.d.ts` |

The Windows XP builds are for 32-bit Windows XP (x86, which requires a processor with SSE2) and for
Windows XP x64 and Server 2003 (x64). They use only the C runtime that ships with Windows
(`msvcrt.dll`), so they need no Visual C++ redistributable. `sggeo.h` and `sggeo.lib` also work with
Visual C++ 2008, whose C compiler is C89 only:

```bat
cl /Tc program.c sggeo.lib
```

## C

`sggeo.h` documents every function. Entities, constraints, dimensions and variables are numbered
by index. Functions that add something return its index, or -1 on error. Functions that change
the document return 1 on success, 0 when the change is rejected (the constraints would not solve,
or the change is redundant) and -1 on error. `sg_last_error()` holds the message of the last
failed call on the calling thread. Strings from the library are released with `sg_string_free`,
formulas with `sg_formula_free` and documents with `sg_doc_free`.

```c
#include "sggeo.h"
#include <stdio.h>

int main(void) {
    SgDoc *doc = sg_doc_new();

    /* a triangle with one corner on the origin and one side horizontal */
    size_t o = sg_doc_origin(doc);
    size_t b = sg_doc_add_point(doc, 4.0, 0.2);
    size_t c = sg_doc_add_point(doc, 1.0, 3.0);
    size_t ob = sg_doc_add_line(doc, o, b, false);
    size_t bc = sg_doc_add_line(doc, b, c, false);
    size_t co = sg_doc_add_line(doc, c, o, false);
    sg_doc_constrain(doc, SG_CON_HORIZONTAL, ob, 0);

    /* driving side lengths 5, 4, 3 */
    size_t sides[3] = {ob, bc, co};
    const char *lengths[3] = {"5", "4", "3"};
    for (int k = 0; k < 3; k++) {
        size_t d = sg_doc_add_dim(doc, SG_DIM_LENGTH, sides[k], 0, 0.3, 0.3);
        sg_doc_set_dim(doc, d, lengths[k]);
    }

    /* measured angle at the origin */
    size_t angle = sg_doc_add_dim(doc, SG_DIM_ANGLE, ob, co, 0.3, 0.3);
    sg_doc_set_dim_driving(doc, angle, false);

    sg_doc_solve(doc, NULL);
    SgFormula *f = sg_derive(doc, SG_TARGET_DIM, angle, 0, NULL);
    if (f) {
        printf("%s\n", f->plain); /* θ1 = acos((d1^2 + d3^2 - d2^2)/(2d1*d3)) */
        sg_formula_free(f);
    } else {
        fprintf(stderr, "%s\n", sg_last_error());
    }
    sg_doc_free(doc);
    return 0;
}
```

```sh
cc -I. triangle.c -L. -lsggeo -o triangle
LD_LIBRARY_PATH=. ./triangle
```

The header is plain C99 and also compiles as C++.

## JavaScript

`sggeo.d.ts` documents the `Doc` class. Kinds of constraints, dimensions and conics are strings
(`"horizontal"`, `"length"`, `"ellipse"`, ...). Methods that change the document throw on wrong
arguments and return `false` when the change is rejected; `deriveDim` and `deriveVar` throw when
no formula is found.

```js
import init, { Doc } from "./sggeo.js";

await init(); // loads sggeo_bg.wasm from next to sggeo.js

const doc = new Doc();
const o = doc.origin;
const b = doc.addPoint(4.0, 0.2);
const c = doc.addPoint(1.0, 3.0);
const ob = doc.addLine(o, b);
const bc = doc.addLine(b, c);
const co = doc.addLine(c, o);
doc.constrain("horizontal", ob);

for (const [side, length] of [[ob, "5"], [bc, "4"], [co, "3"]]) {
  doc.setDim(doc.addDim("length", side, undefined, 0.3, 0.3), length);
}

const angle = doc.addDim("angle", ob, co, 0.3, 0.3);
doc.setDimDriving(angle, false);

doc.solve();
console.log(doc.deriveDim(angle).plain); // θ1 = acos((d1^2 + d3^2 - d2^2)/(2d1*d3))
```

In Node.js, pass the module bytes to `init`:

```js
import { readFile } from "node:fs/promises";
await init({ module_or_path: await readFile(new URL("./sggeo_bg.wasm", import.meta.url)) });
```

## Python

The `sggeo` module needs Python 3.9 or later. The release archives for Linux (x64 and arm64, glibc
2.17 or later), macOS (x64 and arm64, 11 or later) and Windows (x64 and arm64) hold it as a wheel
in `lib/`. Elsewhere pip builds it from the repository with [maturin](https://www.maturin.rs/) and
a Rust toolchain (the build needs the vendored crate in `vendor/` beside this directory, so it
cannot be packed as a source distribution). SymPy is needed for `Formula.sympy` and for SymPy
expressions as arguments; the `sympy` extra installs it:

```sh
pip install 'lib/sggeo-<version>-cp39-abi3-<platform>.whl[sympy]'   # from a release archive
pip install 'sggeo[sympy] @ git+https://github.com/Sycration/sg#subdirectory=sggeo'  # from source
pip install '.[sympy]'           # in this directory; or maturin develop -r
```

The module carries type hints (`sggeo.pyi`) for editors and type checkers. In Jupyter a `Doc`
shows as a drawing of the construction (`Doc.to_svg`, also `sg_doc_to_svg` in C and `toSvg` in
JavaScript) and a `Formula` typeset.

`help(sggeo.Doc)` documents the methods, which are those of the JavaScript `Doc` with Python names
(`add_point`, `derive_dim`, ...). Kinds are strings as in JavaScript. Methods that change the
document raise `sggeo.Error` (a `ValueError`) on wrong arguments and return `False` when the change
is rejected; `derive_dim` and `derive_var` raise `sggeo.Error` when no formula is found. Other
Python threads run while a formula is derived, so a derivation on a thread can be stopped from
another with a `sggeo.Cancel`.

```python
import sggeo
import sympy

doc = sggeo.Doc()
o = doc.origin
b = doc.add_point(4.0, 0.2)
c = doc.add_point(1.0, 3.0)
ob, bc, co = doc.add_line(o, b), doc.add_line(b, c), doc.add_line(c, o)
doc.constrain("horizontal", ob)

for side, length in [(ob, 5), (bc, 4), (co, 3)]:
    doc.set_dim(doc.add_dim("length", side), length)

angle = doc.add_dim("angle", ob, co)
doc.set_dim_driving(angle, False)

doc.solve()
f = doc.derive_dim(angle)
print(f.plain)          # θ1 = acos((d1^2 + d3^2 - d2^2)/(2d1*d3))

e = f.sympy()           # acos((d1**2 - d2**2 + d3**2)/(2*d1*d3))
values = doc.values()   # {"d1": 5.0, "d2": 4.0, "d3": 3.0, "θ1": 0.927...}
print(e.evalf(subs={s: values[s.name] for s in e.free_symbols}))  # 0.927295218001612
```

`Formula.sympy()` is the formula as a SymPy expression. The quantities it reads are real symbols
named as sggeo names them (`sympy.Symbol("θ1", real=True)`), angles are in radians, an odd root of
a negative number is its real root (`sympy.real_root`), and a number found numerically is a
`sympy.Float`. `Doc.values()` gives the values of the dimensions and variables by name: the value
a driving dimension is set to and the measured value of a reference dimension.

The other way, the methods that take an expression as text (`set_dim`, `add_def_var`,
`set_var_expr`, `check_expr`, `add_curve`, `set_curve`, `check_curve`) also take a number or a
SymPy expression, written as sggeo reads it; a curve can be a `sympy.Eq`:

```python
d1 = sympy.Symbol("d1", real=True)
x, y = sympy.symbols("x y")
doc.set_dim(1, sympy.Rational(4, 5) * d1)   # d2 = 4/5 d1
doc.add_curve(sympy.Eq(x**2 + y**2, d1**2))
```

A plain number typed for an angle is in degrees, as everywhere in sggeo.

## Rust

Add the crate as a path dependency:

```toml
[dependencies]
sggeo = { path = "path/to/sggeo" }
```

```rust
use sggeo::ops::{self, Document, Target};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string("triangle.sgp")?;
    let mut doc = Document::from_sgp(&text)?;
    ops::solve(&mut doc);
    for i in 0..ops::dim_count(&doc) {
        match ops::derive(&doc, Target::Dim(i), 0, None) {
            Ok(f) => println!("{}", f.plain),
            Err(e) => println!("dimension {i}: {e}"),
        }
    }
    Ok(())
}
```

`sggeo::ops` is the interface the C, JS and Python bindings wrap: each of its functions has a C
function (`sg_doc_*`), a JS method and a Python method, and the `parity` test keeps it so. The other modules are the library
internals; `Document::sketch` hands them to Rust code that needs them.

`cargo doc -p sggeo --open` builds the API documentation.

## Documents

Documents are saved and loaded as `.sgp` text: `sg_doc_to_sgp` / `sg_doc_from_sgp` in C,
`toSgp` / `Doc.fromSgp` in JavaScript and `sggeo::sgp` in Rust. A loaded document is solved
before its values are read. The text states the construction one step per line, in an order it can
be built in, and then the solved positions ([the format](../docs/sgp-format.md)):

```
sgp 2

given
  triangle ABC
    d1: |AB| = 4
    d2: |BC| = 3
    d3: |CA| = 5

construct
  l1 = bisect(AB, AC)
  l2 = bisect(BA, BC)
  I = meet(l1, l2)

find
  r: dist(I, AB)

where
  A (0, 0)
  ...
```

A document reads back as the same construction, with its entities, constraints and dimensions
numbered in the order of the file.

Every change is one undo step. A change made with several calls, such as adding two points and
then a line between them, becomes one step by reading the length of the undo history before the
calls and joining the steps recorded since: `sg_doc_undo_len` / `sg_doc_join_undo` in C,
`undoLen` / `joinUndo` in JavaScript, `ops::undo_len` / `ops::join_undo` in Rust.

## Formulas

A formula is exact where one is found, otherwise some part of it is a number found numerically
(`exact` is false). The derivation has a default time budget; the `extra_ms` argument of
`sg_derive`, `deriveDim` / `derive_dim`, `deriveVar` / `derive_var` and `ops::derive` extends it
(steps of 10 000 ms suit retrying a hard target). A derivation can take seconds: run it on a copy
of the document (`sg_doc_copy`, `copy`, `ops::copy`) on another thread, with a cancel flag
(`sg_cancel_new`, `Cancel`, `ops::new_cancel`) to stop it early. Besides plain text and LaTeX, a formula has a LaTeX
form with repeated subexpressions written `u_1`, `u_2`, … and their definitions. A dimension or
variable marked numeric appears in the formulas of other targets as its current value.

A formula holds around the construction it was derived from, but not always everywhere: the
distance to a focus of an ellipse with a = b has the root √(b² − a²), and when b becomes less
than a it has no value (the foci go onto the other axis). When the construction is at such a limit,
the formula lists the conditions it meets only just (`limits` of the formula, `limits` and
`n_limits` of `SgFormula` in C, as plain text and LaTeX: `b1^2 - a1^2 ≥ 0`): a radicand of a square
root that is 0, an argument of `acos` or `asin` that is ±1, an argument of `ln` that is 0, each
one that a small change of the quantities can break. The apps show them under the formula.

A formula is kept when it gives the measured value: to 1e-7, or, where the measurement itself is
known less precisely, to that. A measured value is usually known to rounding, but at a double
root (the foci of an ellipse with a = b, a tangent at a point of inflection) the solved positions
are known only to about the square root of it, 1e-7. `sg_doc_dim_uncertainty`, `dimUncertainty`,
`dim_uncertainty` and `ops::dim_uncertainty` estimate how precisely a dimension's measured value
is known, for a display to round it to; a formula that is only a number has the digits known.

A formula can also be written as source code in C, Rust, JavaScript, Python, Mathematica, Maxima
or SymPy: `sg_formula_code` with a `SG_LANG_*` constant in C (the string is released with
`sg_string_free`), `Formula.code` with `"c"`, `"rust"`, `"js"`, `"python"`, `"mathematica"`,
`"maxima"` or `"sympy"` in JavaScript and Python, and `ops::formula_code` with an `ops::Lang` in
Rust. The code is one statement per line: the repeated subexpressions assigned to temporaries
`u_1`, `u_2`, …, then the formula assigned to a variable named after the target. The other
quantities are read from variables of their names, made into identifiers of the language (Greek
letters spelled out except in Mathematica). Angles are in radians, and an odd root of a negative
number is its real root, as sggeo evaluates it (`cbrt(-8)` is -2). The SymPy code declares the
quantities the formula reads as real symbols, so the variable named after the target holds a
SymPy expression. The law of cosines example in Python reads

```python
import math
theta1 = math.acos((d1**2 + d3**2 - d2**2) / (2 * d1 * d3))
```

A client lists the languages from sggeo rather than naming them, so a language added to sggeo
reaches its menus: `sg_lang_name` and `sg_lang_label` in C (from 0 until NULL), `codeLanguages()`
in JavaScript, `code_languages()` in Python and `ops::code_languages` in Rust give each one's name
and the name to show (`js`, "JavaScript"). The kinds of constraint, dimension and entity are listed
the same way, in the order of their `SG_CON_*`, `SG_DIM_*` and `SG_ENT_*` numbers:
`sg_con_kind_name`, `conKinds()`, `con_kinds()`, `ops::con_kinds` and their `dim` and `ent`
counterparts.

The LaTeX of a formula can be typeset as a PNG image at a given size, resolution and color, on a
transparent background: `sg_latex_png` in C (the bytes are released with `sg_bytes_free`),
`latexPng` in JavaScript and `ops::latex_png` in Rust. The typesetting is done by
[latex-rust](https://crates.io/crates/latex-rust) with the STIX Two Math font.

## Building from source

Run inside the `sggeo` directory; the outputs land in the workspace's `target/` directory.

```sh
cargo build --release                                   # C library and Rust crate
cbindgen --config cbindgen.toml --output include/sggeo.h  # regenerates the C header
```

The Wasm module requires `rustup target add wasm32-unknown-unknown` and
`cargo install wasm-bindgen-cli` at the `wasm-bindgen` version in `Cargo.lock`:

```sh
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir pkg ../target/wasm32-unknown-unknown/release/sggeo.wasm
```

`wasm-bindgen --target nodejs` produces a CommonJS module for Node.js instead.

The Python module is the crate built with its `python` feature; maturin builds it as a wheel
(`abi3`, one for each platform that serves Python 3.9 and later):

```sh
maturin build -r                                        # wheel in ../target/wheels
```

`../build.sh` builds the wheels of the release archives, cross-compiled like the C library
(`../release.sh --publish --pypi` can also upload them to PyPI; sggeo is not published there yet). `sggeo.pyi` is written by
`tests/python/stub.py` from the built module (the types are listed there, the parameters and
documentation come from the module); the `python` test fails when it is out of date.

The `python` test (`cargo test --test python`) builds the module and runs `tests/python/` against
it, skipping with a notice without Python 3 and SymPy (`SGGEO_REQUIRE_PYTHON` makes that a
failure; `PYTHON` names the interpreter).

The `examples/` directory holds C, JavaScript and Python programs that build a construction or load
a `.sgp` file and print its formulas.

## License

GPL-3.0-or-later. See [LICENSE](../LICENSE).

//! Python interface: the extension module `sggeo`, through PyO3, with the
//! `python` feature. Build and install it with `pip install ./sggeo` (or
//! `maturin develop` in the `sggeo` directory); see pyproject.toml.
//!
//! Entities (points, lines, circles, ...), dimensions and variables are
//! numbered by index; deleting leaves empty slots, so indices stay valid.
//! Methods that change the document raise `sggeo.Error` when the arguments
//! are wrong, and return False when the change is rejected (the constraints
//! would not solve, or it is redundant).
//!
//! Formulas convert to SymPy expressions (`Formula.sympy`), and the methods
//! that take an expression as text also take a SymPy expression.

use crate::ops::{self, ConKind, ConicFeature, DimType, Document, Lang, Target};
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyBytes, PyDict, PyInt};
use std::ffi::CString;

create_exception!(sggeo, Error, PyValueError, "A call that sggeo refused, with the reason as its message.");

fn err(e: String) -> PyErr {
    Error::new_err(e)
}

/// Text of an expression passed in: a string as it is, a number, or a SymPy
/// expression (an `Eq` becomes `lhs = rhs`) written as sggeo reads it.
fn text(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(s) = obj.extract::<String>() {
        return Ok(s);
    }
    if obj.is_instance_of::<PyInt>() {
        return Ok(obj.str()?.to_string());
    }
    if let Ok(x) = obj.extract::<f64>()
        && !obj.hasattr("free_symbols")?
    {
        return Ok(format!("{x:?}"));
    }
    let sympy = obj.py().import("sympy")?;
    let e = sympy.call_method1("sympify", (obj,))?;
    if e.is_instance(&sympy.getattr("Equality")?)? {
        return Ok(format!("{} = {}", text(&e.getattr("lhs")?)?, text(&e.getattr("rhs")?)?));
    }
    // E is Euler's number in SymPy and a name in sggeo
    let dict = PyDict::new(obj.py());
    dict.set_item(sympy.getattr("E")?, sympy.getattr("exp")?.call((1,), Some(&[("evaluate", false)].into_py_dict(obj.py())?))?)?;
    let e = e.call_method1("xreplace", (dict,))?;
    let s: String = sympy.call_method1("sstr", (e,))?.extract()?;
    Ok(sgg_syntax(&s))
}

/// SymPy's text of an expression in sggeo's syntax: `^` for `**` and the
/// function `Abs` as `abs`; the rest reads the same.
fn sgg_syntax(s: &str) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        out.push_str(if word == "Abs" { "abs" } else { word });
        word.clear();
    };
    let mut cs = s.chars().peekable();
    while let Some(c) = cs.next() {
        if c.is_alphanumeric() || c == '_' || c == '.' && !word.is_empty() {
            word.push(c);
            continue;
        }
        flush(&mut word, &mut out);
        if c == '*' && cs.peek() == Some(&'*') {
            cs.next();
            out.push('^');
        } else {
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}


/// A geometry document with its solver state and undo history.
#[pyclass(name = "Doc", module = "sggeo")]
pub struct PyDoc {
    doc: Document,
}

/// A derived formula. `str()` gives `plain`; in Jupyter it shows typeset.
#[pyclass(name = "Formula", module = "sggeo", frozen)]
pub struct Formula {
    /// Name of the target.
    #[pyo3(get)]
    name: String,
    /// `name = formula` as plain text.
    #[pyo3(get)]
    plain: String,
    /// `name = formula` as LaTeX.
    #[pyo3(get)]
    latex: String,
    /// `name = formula` as LaTeX with repeated subexpressions replaced by
    /// `u_{1}`, `u_{2}`, ..., which `defs` defines.
    #[pyo3(get)]
    latex_short: String,
    /// The subexpressions of `latex_short`, in order, as (LaTeX name, LaTeX
    /// expression) pairs; an expression can name earlier subexpressions.
    #[pyo3(get)]
    defs: Vec<(String, String)>,
    /// False when some part had to be solved numerically.
    #[pyo3(get)]
    exact: bool,
    /// True when the target is an angle (in radians).
    #[pyo3(get)]
    is_angle: bool,
    /// Formula evaluated at the current configuration.
    #[pyo3(get)]
    value: f64,
    /// Measured value of the target.
    #[pyo3(get)]
    actual: f64,
    /// Complexity of the formula: its number of operations and terms.
    #[pyo3(get)]
    size: usize,
    /// The conditions for the formula to hold that the construction meets
    /// only just: a radicand of a square root that is 0 here (`b1^2 - a1^2 ≥
    /// 0`), an argument of `acos` or `asin` that is ±1, an argument of `ln`
    /// that is 0. A little way past such a condition the formula has no value
    /// or another branch holds. Usually none. Each as (plain text, LaTeX).
    #[pyo3(get)]
    limits: Vec<(String, String)>,
    src: ops::Formula,
}

#[pymethods]
impl Formula {
    /// The formula as source code in language `lang`: "c", "rust", "js",
    /// "python", "mathematica", "maxima" or "sympy". Statements assign the
    /// repeated subexpressions of the formula to temporaries `u_1`, `u_2`,
    /// ... and then the formula to a variable named after the target, one
    /// per line, reading the other quantities from variables of their names
    /// (names are made into identifiers of the language, Greek letters
    /// spelled out). Angles are in radians. C needs `<math.h>`; the Python
    /// code imports `math` when it uses it. The SymPy code declares the other
    /// quantities as real symbols, so the target holds a SymPy expression.
    fn code(&self, lang: &str) -> PyResult<String> {
        Ok(ops::formula_code(&self.src, Lang::from_name(lang).map_err(err)?))
    }

    /// The formula (the right side) as a SymPy expression. The quantities
    /// it reads are real symbols named as sggeo names them, such as
    /// `Symbol("d1", real=True)` or `Symbol("θ1", real=True)`; angles are in
    /// radians. A number found numerically is a SymPy Float.
    fn sympy<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let code = ops::formula_code(&self.src, Lang::Sympy);
        let target = code.lines().last().and_then(|l| l.split_once(" = ")).map(|(v, _)| v.to_string()).ok_or_else(|| err("empty formula".into()))?;
        let globals = PyDict::new(py);
        py.run(&CString::new(code).map_err(|e| err(e.to_string()))?, Some(&globals), None)?;
        let value = globals.get_item(&target)?.ok_or_else(|| err(format!("the code assigns no {target}")))?;
        py.import("sympy")?.call_method1("sympify", (value,))
    }

    fn __str__(&self) -> String {
        self.plain.clone()
    }

    fn __repr__(&self) -> String {
        format!("<sggeo.Formula {}>", self.plain)
    }

    /// Typeset display in Jupyter.
    fn _repr_latex_(&self) -> String {
        format!("$\\displaystyle {}$", self.latex)
    }
}

/// Result of `Doc.solve`.
#[pyclass(name = "Solved", module = "sggeo", frozen, get_all)]
pub struct Solved {
    /// True when every constraint is satisfied.
    ok: bool,
    /// Remaining degrees of freedom.
    dof: usize,
    /// Messages for constraints or dimensions that could not be turned into equations.
    errors: Vec<String>,
}

#[pymethods]
impl Solved {
    fn __bool__(&self) -> bool {
        self.ok
    }

    fn __repr__(&self) -> String {
        format!("<sggeo.Solved ok={} dof={} errors={:?}>", if self.ok { "True" } else { "False" }, self.dof, self.errors)
    }
}

/// A flag that stops a derivation early, from another thread.
#[pyclass(name = "Cancel", module = "sggeo", frozen)]
pub struct Cancel {
    c: ops::Cancel,
}

#[pymethods]
impl Cancel {
    /// New, unset flag.
    #[new]
    fn new() -> Cancel {
        Cancel { c: ops::new_cancel() }
    }

    /// Sets the flag: a derivation given it stops soon with an error.
    fn cancel(&self) {
        ops::cancel(&self.c);
    }
}

fn conic_name(k: ops::ConicKind) -> &'static str {
    match k {
        ops::ConicKind::Ellipse => "ellipse",
        ops::ConicKind::Hyperbola => "hyperbola",
        ops::ConicKind::Parabola => "parabola",
    }
}

fn conic_kind(s: &str) -> PyResult<ops::ConicKind> {
    match s {
        "ellipse" => Ok(ops::ConicKind::Ellipse),
        "hyperbola" => Ok(ops::ConicKind::Hyperbola),
        "parabola" => Ok(ops::ConicKind::Parabola),
        _ => Err(err(format!("unknown conic kind {s:?}"))),
    }
}

fn pair(p: [f64; 2]) -> (f64, f64) {
    (p[0], p[1])
}

#[pymethods]
impl PyDoc {
    /// New document holding only the origin point and the x and y axes.
    #[new]
    fn new() -> PyDoc {
        PyDoc { doc: Document::new() }
    }

    /// Document decoded from .sgp text. Call `solve` before reading values from it.
    #[staticmethod]
    fn from_sgp(text: &str) -> PyResult<PyDoc> {
        Ok(PyDoc { doc: Document::from_sgp(text).map_err(err)? })
    }

    /// Document encoded as .sgp text.
    fn to_sgp(&self) -> PyResult<String> {
        ops::to_sgp(&self.doc).map_err(err)
    }

    /// The construction drawn as an SVG picture of `width` × `height` pixels
    /// (16 to 4096 each): its curves, its points with their names, and each
    /// dimension as `name = value` at its label (driving ones purple, measured
    /// ones green, angles in degrees), scaled to fit, with the axes in grey. In
    /// Jupyter a `Doc` shows as this picture.
    #[pyo3(signature = (width = 480.0, height = 360.0))]
    fn to_svg(&self, width: f64, height: f64) -> PyResult<String> {
        ops::to_svg(&self.doc, width, height).map_err(err)
    }

    fn _repr_svg_(&self) -> PyResult<String> {
        self.to_svg(480.0, 360.0)
    }

    /// Solves the constraints and stores the solution in the document. The
    /// result is true when every constraint is satisfied.
    fn solve(&mut self) -> Solved {
        let ok = ops::solve(&mut self.doc);
        Solved { ok, dof: ops::dof(&self.doc), errors: ops::errors(&self.doc) }
    }

    /// Undoes the last change; False when there is nothing to undo.
    fn undo(&mut self) -> bool {
        ops::undo(&mut self.doc)
    }

    /// Redoes the last undone change; False when there is nothing to redo.
    fn redo(&mut self) -> bool {
        ops::redo(&mut self.doc)
    }

    /// Records the current document as an undo step. A drag (`drag_point`,
    /// `drag_var`, ...) is not undoable by itself, so a checkpoint before it
    /// makes the whole drag one undo step.
    fn checkpoint(&mut self) {
        ops::checkpoint(&mut self.doc);
    }

    /// Remaining degrees of freedom after the last solve.
    fn dof(&self) -> usize {
        ops::dof(&self.doc)
    }

    /// Number of steps in the undo history; see `join_undo`.
    fn undo_len(&self) -> usize {
        ops::undo_len(&self.doc)
    }

    /// Joins the undo steps recorded since the undo history held `len` steps
    /// (a value of `undo_len` read before them) into one step, so that a
    /// change made with several calls undoes at once. False when fewer than
    /// two steps were recorded since. The history keeps at most 300 steps;
    /// past that, steps recorded since `len` can be left unjoined.
    fn join_undo(&mut self, len: usize) -> bool {
        ops::join_undo(&mut self.doc, len)
    }

    /// A copy of the document in its current state, with the same solve
    /// status and without the undo history.
    fn copy(&self) -> PyDoc {
        PyDoc { doc: ops::copy(&self.doc) }
    }

    fn __copy__(&self) -> PyDoc {
        self.copy()
    }

    /// Replaces the content of the document with the document decoded from
    /// .sgp text, as one undoable change, and solves it.
    fn load_sgp(&mut self, text: &str) -> PyResult<()> {
        ops::load_sgp(&mut self.doc, text).map_err(err)
    }

    /// Applies .sgp statements (lines of a .sgp file, without the header) to
    /// the document as one undoable change; a `where` line placing a point
    /// that is there drags it. False when the change is rejected (the
    /// constraints would not solve or already imply it).
    fn apply_sgp(&mut self, text: &str) -> PyResult<bool> {
        ops::apply_sgp(&mut self.doc, text).map_err(err)
    }

    /// The entity named `name` as a .sgp file names it (a point or curve by
    /// its name, a line by its points run together: `AB`); None when there
    /// is none.
    fn ent_by_name(&self, name: &str) -> Option<usize> {
        ops::ent_by_name(&self.doc, name).ok()
    }

    /// Replaces the content of the document with an empty document (origin
    /// and axes only), as one undoable change.
    fn clear(&mut self) {
        ops::clear(&mut self.doc);
    }

    /// Counter raised by every change of the document, including drags;
    /// equal values mean an unchanged document.
    #[getter]
    fn generation(&self) -> u64 {
        ops::generation(&self.doc)
    }

    /// Number of constraints, dimensions and entities the last solve left
    /// unsatisfied.
    #[getter]
    fn conflict_count(&self) -> usize {
        ops::conflict_count(&self.doc)
    }

    // ------------------------------------------------------------ construction

    /// The origin point.
    #[getter]
    fn origin(&self) -> usize {
        ops::origin(&self.doc)
    }

    /// The x axis (an infinite line through the origin).
    #[getter]
    fn x_axis(&self) -> usize {
        ops::x_axis(&self.doc)
    }

    /// The y axis (an infinite line through the origin).
    #[getter]
    fn y_axis(&self) -> usize {
        ops::y_axis(&self.doc)
    }

    /// Adds a free point at (x, y), its starting position. Returns the entity.
    fn add_point(&mut self, x: f64, y: f64) -> PyResult<usize> {
        ops::add_point(&mut self.doc, x, y).map_err(err)
    }

    /// Adds a line from point a to point b: a segment, or an infinite line
    /// when `infinite`. With `axis` "horizontal" or "vertical", the line is
    /// also constrained along that axis. Returns the entity.
    #[pyo3(signature = (a, b, infinite = false, axis = None))]
    fn add_line(&mut self, a: usize, b: usize, infinite: bool, axis: Option<&str>) -> PyResult<usize> {
        match axis {
            None => ops::add_line(&mut self.doc, a, b, infinite),
            Some("horizontal") => ops::line_between(&mut self.doc, a, b, infinite, Some(true)),
            Some("vertical") => ops::line_between(&mut self.doc, a, b, infinite, Some(false)),
            Some(o) => Err(format!("unknown axis {o}")),
        }
        .map_err(err)
    }

    /// Adds a circle around point c with starting radius r (the radius stays
    /// free until constrained). Returns the entity.
    fn add_circle(&mut self, c: usize, r: f64) -> PyResult<usize> {
        ops::add_circle(&mut self.doc, c, r).map_err(err)
    }

    /// Adds an arc around point c from point a counterclockwise to point b
    /// (|ca| = |cb| is enforced). Returns the entity.
    fn add_arc(&mut self, c: usize, a: usize, b: usize) -> PyResult<usize> {
        ops::add_arc(&mut self.doc, c, a, b).map_err(err)
    }

    /// Adds a curve: `y = f(x)` or `F(x, y) = G(x, y)` in x, y, variables
    /// and dimensions, as text or a SymPy expression or `Eq`. Returns the
    /// entity.
    fn add_curve(&mut self, src: &Bound<'_, PyAny>) -> PyResult<usize> {
        ops::add_curve(&mut self.doc, &text(src)?).map_err(err)
    }

    /// Sets the definition of curve `i`, as `add_curve` takes it. False when
    /// rejected.
    fn set_curve(&mut self, i: usize, src: &Bound<'_, PyAny>) -> PyResult<bool> {
        ops::set_curve(&mut self.doc, i, &text(src)?).map_err(err)
    }

    /// Adds a conic: `kind` is "ellipse", "hyperbola" or "parabola",
    /// centered on point c (the vertex of a parabola), with half-axes a and
    /// b (b is ignored for a parabola, where a is the focal distance) and
    /// axis angle t in radians. Returns the entity.
    #[pyo3(signature = (kind, c, a, b, t = 0.0))]
    fn add_conic(&mut self, kind: &str, c: usize, a: f64, b: f64, t: f64) -> PyResult<usize> {
        ops::add_conic(&mut self.doc, conic_kind(kind)?, c, a, b, t).map_err(err)
    }

    /// Adds a point at (x, y) on curve c, bound to it by a constraint.
    fn add_point_on(&mut self, c: usize, x: f64, y: f64) -> PyResult<usize> {
        ops::add_point_on(&mut self.doc, c, x, y).map_err(err)
    }

    /// Adds the circle around point c through point p, bound to it.
    fn add_circle_to(&mut self, c: usize, p: usize) -> PyResult<usize> {
        ops::circle_to(&mut self.doc, c, p).map_err(err)
    }

    /// Adds the arc around point c from point a, ending at a new point in
    /// the direction of (x, y).
    fn add_arc_toward(&mut self, c: usize, a: usize, x: f64, y: f64) -> PyResult<usize> {
        ops::arc_toward(&mut self.doc, c, a, [x, y]).map_err(err)
    }

    /// Adds the midpoint of points a and b, bound to them. Returns the point.
    fn add_midpoint(&mut self, a: usize, b: usize) -> PyResult<usize> {
        ops::midpoint(&mut self.doc, a, b).map_err(err)
    }

    /// Adds the midpoint of line l, bound to it. Returns the point.
    fn add_line_midpoint(&mut self, l: usize) -> PyResult<usize> {
        ops::line_midpoint(&mut self.doc, l).map_err(err)
    }

    /// Adds a point at a crossing of curves a and b, starting from (x, y).
    fn add_intersection(&mut self, a: usize, b: usize, x: f64, y: f64) -> PyResult<usize> {
        ops::intersection(&mut self.doc, a, b, [x, y]).map_err(err)
    }

    /// Adds a point at the crossing of curves a and b nearest to (x, y).
    fn add_intersection_near(&mut self, a: usize, b: usize, x: f64, y: f64) -> PyResult<usize> {
        ops::intersect_near(&mut self.doc, a, b, [x, y]).map_err(err)
    }

    /// Adds the infinite line through point p perpendicular to line l.
    fn add_perp_line(&mut self, l: usize, p: usize) -> PyResult<usize> {
        ops::dir_line(&mut self.doc, l, p, true).map_err(err)
    }

    /// Adds the infinite line through point p parallel to line l.
    fn add_parallel_line(&mut self, l: usize, p: usize) -> PyResult<usize> {
        ops::dir_line(&mut self.doc, l, p, false).map_err(err)
    }

    /// Adds the perpendicular bisector of points a and b. Returns
    /// (line, midpoint).
    fn add_perp_bisector(&mut self, a: usize, b: usize) -> PyResult<(usize, usize)> {
        ops::perp_bisector(&mut self.doc, a, b).map_err(err)
    }

    /// Adds the bisector of lines l1 and l2 in the sector containing (x, y).
    /// Returns (line, vertex); the vertex is a shared endpoint, or a new
    /// point at the crossing.
    fn add_angle_bisector(&mut self, l1: usize, l2: usize, x: f64, y: f64) -> PyResult<(usize, usize)> {
        ops::angle_bisector(&mut self.doc, l1, l2, [x, y]).map_err(err)
    }

    /// Adds the segment from point p touching curve c at a contact point
    /// starting from (x, y). Returns (segment, contact point).
    fn add_tangent_line(&mut self, p: usize, c: usize, x: f64, y: f64) -> PyResult<(usize, usize)> {
        ops::tangent_line(&mut self.doc, p, c, [x, y]).map_err(err)
    }

    /// Like `add_tangent_line`, with the contact point nearest to (x, y).
    fn add_tangent_line_near(&mut self, p: usize, c: usize, x: f64, y: f64) -> PyResult<(usize, usize)> {
        ops::tangent_line_near(&mut self.doc, p, c, [x, y]).map_err(err)
    }

    /// Adds the mirror image of point p in line l.
    fn add_reflection(&mut self, p: usize, l: usize) -> PyResult<usize> {
        ops::reflection(&mut self.doc, p, l).map_err(err)
    }

    /// Adds the circle through points a, b and c. Returns (circle, center),
    /// the center being a new point.
    fn add_circle_through(&mut self, a: usize, b: usize, c: usize) -> PyResult<(usize, usize)> {
        ops::circle_through(&mut self.doc, a, b, c).map_err(err)
    }

    /// Adds the arc from point a through point m to point b. Returns (arc,
    /// center), the center being a new point.
    fn add_arc_through(&mut self, a: usize, m: usize, b: usize) -> PyResult<(usize, usize)> {
        ops::arc_through(&mut self.doc, a, m, b).map_err(err)
    }

    /// Adds the ellipse or hyperbola (`kind` "ellipse" or "hyperbola") with
    /// foci f1 and f2 through point p.
    fn add_conic_by_foci(&mut self, kind: &str, f1: usize, f2: usize, p: usize) -> PyResult<usize> {
        ops::conic_by_foci(&mut self.doc, conic_kind(kind)?, f1, f2, p).map_err(err)
    }

    /// Adds the parabola with focus f and directrix line l.
    fn add_parabola_by_focus(&mut self, f: usize, l: usize) -> PyResult<usize> {
        ops::parabola_by_focus(&mut self.doc, f, l).map_err(err)
    }

    /// Adds the ellipse or hyperbola centered on point c, its first axis
    /// ending at (ax, ay) and its second half-axis reaching the line through
    /// (sx, sy) parallel to the first axis.
    #[allow(clippy::too_many_arguments)]
    fn add_conic_from(&mut self, kind: &str, c: usize, ax: f64, ay: f64, sx: f64, sy: f64) -> PyResult<usize> {
        ops::conic_from(&mut self.doc, conic_kind(kind)?, c, [ax, ay], [sx, sy]).map_err(err)
    }

    /// Adds the parabola with vertex at point c opening toward (x, y), its
    /// focal distance the distance to (x, y).
    fn add_parabola_toward(&mut self, c: usize, x: f64, y: f64) -> PyResult<usize> {
        ops::parabola_toward(&mut self.doc, c, [x, y]).map_err(err)
    }

    /// Adds the foci of conic c: two points, one for a parabola.
    fn add_foci(&mut self, c: usize) -> PyResult<Vec<usize>> {
        ops::conic_feature(&mut self.doc, ConicFeature::Foci, c, [0.0; 2]).map_err(err)
    }

    /// Adds the directrix of conic c belonging to the focus nearest (x, y).
    fn add_directrix(&mut self, c: usize, x: f64, y: f64) -> PyResult<usize> {
        ops::conic_feature(&mut self.doc, ConicFeature::Directrix, c, [x, y]).map_err(err)?.first().copied().ok_or_else(|| err("no directrix".into()))
    }

    /// Adds both asymptotes of hyperbola c.
    fn add_asymptotes(&mut self, c: usize) -> PyResult<Vec<usize>> {
        ops::conic_feature(&mut self.doc, ConicFeature::Asymptotes, c, [0.0; 2]).map_err(err)
    }

    /// Adds a constraint: `kind` is "coincident", "on_curve", "midpoint",
    /// "horizontal", "vertical", "parallel", "perpendicular", "collinear",
    /// "equal", "concentric" or "tangent", on entities a and b ("horizontal"
    /// and "vertical" take a line, or two points). False when it is
    /// rejected.
    #[pyo3(signature = (kind, a, b = None))]
    fn constrain(&mut self, kind: &str, a: usize, b: Option<usize>) -> PyResult<bool> {
        ops::constrain(&mut self.doc, ConKind::from_name(kind).map_err(err)?, a, b.unwrap_or(usize::MAX)).map_err(err)
    }

    /// Like `constrain`; a tangency between two curves whose contact point
    /// is solved for starts its contact search at the point of the curves
    /// nearest to (x, y), such as where the user clicked.
    fn constrain_near(&mut self, kind: &str, a: usize, b: Option<usize>, x: f64, y: f64) -> PyResult<bool> {
        ops::constrain_near(&mut self.doc, ConKind::from_name(kind).map_err(err)?, a, b.unwrap_or(usize::MAX), [x, y]).map_err(err)
    }

    /// True when `constrain` would accept these arguments; the constraint
    /// might still be rejected for not solving. The document is unchanged.
    #[pyo3(signature = (kind, a, b = None))]
    fn can_constrain(&self, kind: &str, a: usize, b: Option<usize>) -> PyResult<bool> {
        Ok(ops::can_constrain(&self.doc, ConKind::from_name(kind).map_err(err)?, a, b.unwrap_or(usize::MAX)))
    }

    /// Adds the constraints `cons`, each a (kind, a, b) tuple as `constrain`
    /// takes it (b is None where a kind takes one entity), as one undoable
    /// change, rejecting them together when any would not solve or is
    /// redundant. False when rejected.
    fn add_constraints(&mut self, cons: Vec<(String, usize, Option<usize>)>) -> PyResult<bool> {
        let cons = cons.iter().map(|(k, a, b)| Ok((ConKind::from_name(k)?, *a, b.unwrap_or(usize::MAX)))).collect::<Result<Vec<_>, String>>().map_err(err)?;
        ops::add_constraints(&mut self.doc, &cons).map_err(err)
    }

    /// Constrains points a and b to be mirror images of each other in line
    /// `l`. False when rejected.
    fn constrain_symmetric(&mut self, a: usize, b: usize, l: usize) -> PyResult<bool> {
        ops::symmetric(&mut self.doc, a, b, l).map_err(err)
    }

    /// Adds a dimension: `kind` is "distance", "hdist", "vdist",
    /// "point_line", "length", "radius", "diameter", "angle", "conic_a",
    /// "conic_b", "x" or "y", on entities a and b, with its label at
    /// (off_x, off_y) from the dimension's anchor (the midpoint of a length,
    /// the vertex of an angle). It is driving when that keeps the
    /// constraints solvable, otherwise a reference (measured) dimension. For
    /// an angle, the sector holding the label selects the angle or its
    /// supplement. Returns the dimension.
    #[pyo3(signature = (kind, a, b = None, off_x = 0.3, off_y = 0.3))]
    fn add_dim(&mut self, kind: &str, a: usize, b: Option<usize>, off_x: f64, off_y: f64) -> PyResult<usize> {
        ops::add_dim(&mut self.doc, DimType::from_name(kind).map_err(err)?, a, b.unwrap_or(usize::MAX), [off_x, off_y]).map_err(err)
    }

    /// Sets dimension `i` as typed: a number (degrees for angles) or an
    /// expression of variables and dimensions, as text or a SymPy
    /// expression, drives it; empty text makes it a reference dimension.
    /// False when the value cannot be reached.
    fn set_dim(&mut self, i: usize, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        ops::set_dim(&mut self.doc, i, &text(value)?).map_err(err)
    }

    /// Makes dimension `i` driving or reference. False when making it
    /// driving is rejected (the constraints already determine it).
    fn set_dim_driving(&mut self, i: usize, driving: bool) -> PyResult<bool> {
        ops::set_driving(&mut self.doc, i, driving).map_err(err)
    }

    /// Marks dimension `i` numeric or not. Formulas derived for other targets
    /// use a numeric dimension's current value as a plain number. Solving and
    /// the dimension itself are unaffected.
    fn set_dim_numeric(&mut self, i: usize, numeric: bool) -> PyResult<bool> {
        ops::set_dim_numeric(&mut self.doc, i, numeric).map_err(err)
    }

    /// Moves the label of dimension `i` to (off_x, off_y) from the
    /// dimension's anchor (see `dim_anchor`), as one undoable change. For a
    /// reference angle, the sector holding the label selects the angle or
    /// its supplement; a driving angle keeps its choice.
    fn set_dim_offset(&mut self, i: usize, off_x: f64, off_y: f64) -> PyResult<()> {
        ops::set_dim_offset(&mut self.doc, i, [off_x, off_y]).map_err(err)
    }

    /// Moves the label of dimension `i` like `set_dim_offset`, as a drag:
    /// not an undoable change by itself (see `checkpoint`).
    fn drag_dim_offset(&mut self, i: usize, off_x: f64, off_y: f64) -> PyResult<()> {
        ops::drag_dim_offset(&mut self.doc, i, [off_x, off_y]).map_err(err)
    }

    /// Toggles angle dimension `i` between the angle of its lines and its
    /// supplement; a driving dimension with a plain value is set to the
    /// supplement's value. False when rejected.
    fn toggle_supp(&mut self, i: usize) -> PyResult<bool> {
        ops::toggle_supp(&mut self.doc, i).map_err(err)
    }

    /// Adds a value variable. Without a name, the first unused of a, b, c, k,
    /// ... is taken. A free variable (default) is set by `value`, otherwise
    /// it is solved from the constraints, starting at `value`. Returns the
    /// variable.
    #[pyo3(signature = (name = None, value = 1.0, free = true))]
    fn add_var(&mut self, name: Option<&str>, value: f64, free: bool) -> PyResult<usize> {
        ops::add_var(&mut self.doc, name, value, free).map_err(err)
    }

    /// Adds a variable defined by the expression `expr` of variables and
    /// dimensions, as text or a SymPy expression. Without a name, f1, f2, ...
    /// is taken. Returns the variable.
    #[pyo3(signature = (expr, name = None))]
    fn add_def_var(&mut self, expr: &Bound<'_, PyAny>, name: Option<&str>) -> PyResult<usize> {
        ops::add_defvar(&mut self.doc, name, &text(expr)?).map_err(err)
    }

    /// Sets the value of value variable `i`. False when the value cannot be reached.
    fn set_var(&mut self, i: usize, value: f64) -> PyResult<bool> {
        ops::set_var(&mut self.doc, i, value).map_err(err)
    }

    /// Sets variable `i` as typed: a plain number makes it a free value
    /// variable, any other text or SymPy expression an expression of
    /// variables and dimensions. False when rejected.
    fn set_var_expr(&mut self, i: usize, expr: &Bound<'_, PyAny>) -> PyResult<bool> {
        ops::set_var_expr(&mut self.doc, i, &text(expr)?).map_err(err)
    }

    /// Checks an expression as `set_var_expr` and `set_dim` read it: the
    /// reason when it does not parse or names an unknown quantity, None
    /// when it is valid.
    fn check_expr(&self, expr: &Bound<'_, PyAny>) -> PyResult<Option<String>> {
        Ok(ops::check_expr(&self.doc, &text(expr)?).err())
    }

    /// Checks a curve definition as `add_curve` reads it: the reason when it
    /// does not parse, None when it is valid.
    fn check_curve(&self, src: &Bound<'_, PyAny>) -> PyResult<Option<String>> {
        Ok(ops::check_curve(&self.doc, &text(src)?).err())
    }

    /// Makes value variable `i` free (set by its value) or solved from the
    /// constraints. False when rejected.
    fn set_var_free(&mut self, i: usize, free: bool) -> PyResult<bool> {
        ops::set_var_free(&mut self.doc, i, free).map_err(err)
    }

    /// Marks variable `i` numeric or not. Formulas derived for other targets
    /// use a numeric variable's current value as a plain number. Solving and
    /// the variable itself (value, expression, free or solved) are
    /// unaffected.
    fn set_var_numeric(&mut self, i: usize, numeric: bool) -> PyResult<bool> {
        ops::set_var_numeric(&mut self.doc, i, numeric).map_err(err)
    }

    /// Removes variable `i`, leaving an empty slot; expressions naming it
    /// fail to parse until edited. False when rejected.
    fn delete_var(&mut self, i: usize) -> PyResult<bool> {
        ops::delete_var(&mut self.doc, i).map_err(err)
    }

    /// Deletes entity `e` and what depends on it (constraints and dimensions
    /// that reference it, and features built from it). False when the
    /// constraints would no longer solve.
    fn delete_ent(&mut self, e: usize) -> PyResult<bool> {
        ops::delete_ent(&mut self.doc, e).map_err(err)
    }

    /// Deletes constraint `c`. False when the constraints would no longer
    /// solve.
    fn delete_con(&mut self, c: usize) -> PyResult<bool> {
        ops::delete_con(&mut self.doc, c).map_err(err)
    }

    /// Deletes dimension `i`; expressions naming it fail to parse until
    /// edited. False when the constraints would no longer solve.
    fn delete_dim(&mut self, i: usize) -> PyResult<bool> {
        ops::delete_dim(&mut self.doc, i).map_err(err)
    }

    /// Deletes the given entities, constraints, dimensions and variables as
    /// one undoable change, with everything that depends on the entities.
    /// False when the constraints would no longer solve; nothing is deleted.
    #[pyo3(signature = (*, ents = Vec::new(), cons = Vec::new(), dims = Vec::new(), vars = Vec::new()))]
    fn delete(&mut self, ents: Vec<usize>, cons: Vec<usize>, dims: Vec<usize>, vars: Vec<usize>) -> PyResult<bool> {
        ops::delete(&mut self.doc, &ents, &cons, &dims, &vars).map_err(err)
    }

    /// Locks point `e` (its coordinates become fixed) or unlocks it.
    #[pyo3(signature = (e, on = true))]
    fn lock_point(&mut self, e: usize, on: bool) -> PyResult<bool> {
        ops::lock_point(&mut self.doc, e, on).map_err(err)
    }

    /// Locks or unlocks the points `pts` as one undoable change.
    #[pyo3(signature = (pts, on = true))]
    fn lock_points(&mut self, pts: Vec<usize>, on: bool) -> PyResult<bool> {
        ops::lock_points(&mut self.doc, &pts, on).map_err(err)
    }

    /// True when point `e` is locked (both coordinates fixed).
    fn point_locked(&self, e: usize) -> PyResult<bool> {
        ops::point_locked(&self.doc, e).map_err(err)
    }

    /// Sets the construction flag of entity `e` (a dashed helper).
    #[pyo3(signature = (e, on = true))]
    fn set_construction(&mut self, e: usize, on: bool) -> PyResult<bool> {
        ops::set_construction(&mut self.doc, e, on).map_err(err)
    }

    /// Sets the construction flag of the non-point entities `ents` as one
    /// undoable change.
    #[pyo3(signature = (ents, on = true))]
    fn set_construction_all(&mut self, ents: Vec<usize>, on: bool) -> PyResult<bool> {
        ops::set_construction_all(&mut self.doc, &ents, on).map_err(err)
    }

    /// Sets the hidden flag of entity `e`. Hidden entities are not drawn.
    #[pyo3(signature = (e, on = true))]
    fn set_hidden(&mut self, e: usize, on: bool) -> PyResult<bool> {
        ops::set_hidden(&mut self.doc, e, on).map_err(err)
    }

    /// Makes line `e` infinite or a segment.
    #[pyo3(signature = (e, on = true))]
    fn set_infinite(&mut self, e: usize, on: bool) -> PyResult<bool> {
        ops::set_infinite(&mut self.doc, e, on).map_err(err)
    }

    /// Switches each of `lines` between infinite and segment, as one undoable
    /// change.
    fn toggle_infinite(&mut self, lines: Vec<usize>) -> PyResult<bool> {
        ops::toggle_infinite(&mut self.doc, &lines).map_err(err)
    }

    /// Pulls point `e` toward (x, y) by solving the constraints with it
    /// pinned there (a drag, not an undoable change). False when the
    /// configuration cannot reach the target.
    fn drag_point(&mut self, e: usize, x: f64, y: f64) -> PyResult<bool> {
        ops::drag_point(&mut self.doc, e, x, y).map_err(err)
    }

    /// Pulls the radius of circle `e` so its edge passes through (x, y).
    fn drag_circle(&mut self, e: usize, x: f64, y: f64) -> PyResult<bool> {
        ops::drag_circle_edge(&mut self.doc, e, x, y).map_err(err)
    }

    /// Pulls a handle of conic `e` (`which` 0 the first-axis end, 1 the
    /// second-axis end) toward (x, y).
    fn drag_conic_handle(&mut self, e: usize, which: u8, x: f64, y: f64) -> PyResult<bool> {
        ops::drag_conic_handle(&mut self.doc, e, which, x, y).map_err(err)
    }

    /// Pulls each point of `pts` toward its target in `targets`, (x, y)
    /// pairs, together, solving the constraints (such as the ends of a
    /// dragged line). A drag, not an undoable change. False when the points
    /// cannot reach their targets.
    fn drag_points(&mut self, pts: Vec<usize>, targets: Vec<(f64, f64)>) -> PyResult<bool> {
        if targets.len() != pts.len() {
            return Err(err("pts and targets differ in length".into()));
        }
        let targets: Vec<[f64; 2]> = targets.into_iter().map(|(x, y)| [x, y]).collect();
        ops::drag_points(&mut self.doc, &pts, &targets).map_err(err)
    }

    /// Pulls curve `e` so it passes through (x, y), solving the constraints.
    /// A drag, not an undoable change. False when the curve cannot reach the
    /// point.
    fn drag_curve(&mut self, e: usize, x: f64, y: f64) -> PyResult<bool> {
        ops::drag_curve(&mut self.doc, e, x, y).map_err(err)
    }

    /// Sets value variable `i` to `value` and solves, as a drag: not an
    /// undoable change by itself (see `checkpoint`). False when the
    /// constraints do not solve at that value; the variable keeps its value.
    fn drag_var(&mut self, i: usize, value: f64) -> PyResult<bool> {
        ops::drag_var(&mut self.doc, i, value).map_err(err)
    }

    /// Gives the point, dimension or variable named `old` the name `new`,
    /// rewriting expressions that use it. False when `new` is not an
    /// identifier or is taken.
    fn rename(&mut self, old: &str, new: &str) -> PyResult<bool> {
        ops::rename(&mut self.doc, old, new).map_err(err)
    }

    // ------------------------------------------------------------ reading

    /// Number of dimension slots. Removed dimensions leave empty slots.
    #[getter]
    fn dim_count(&self) -> usize {
        ops::dim_count(&self.doc)
    }

    /// Name of dimension `i`; None for an empty slot or a bad index.
    fn dim_name(&self, i: usize) -> Option<String> {
        ops::dim_name(&self.doc, i).ok()
    }

    /// Current measured value of dimension `i` (radians for angles); NaN
    /// for an empty slot or a bad index. A value that is 0 up to the
    /// rounding of the solved coordinates is exactly 0.
    fn dim_value(&self, i: usize) -> f64 {
        ops::dim_value(&self.doc, i).unwrap_or(f64::NAN)
    }

    /// How precisely the measured value of dimension `i` (`dim_value`) is
    /// known: about how far it can be from the exact value of the
    /// construction. Near 1e-13 of the value where the construction is
    /// regular; at a double root (the foci of an ellipse with a = b) about the
    /// square root of that. A display can round the value to it. NaN for an
    /// empty slot or a bad index.
    fn dim_uncertainty(&self, i: usize) -> f64 {
        ops::dim_uncertainty(&self.doc, i).unwrap_or(f64::NAN)
    }

    /// True when dimension `i` is driving, False when it is a reference dimension.
    fn dim_driving(&self, i: usize) -> PyResult<bool> {
        ops::dim_driving(&self.doc, i).map_err(err)
    }

    /// True when dimension `i` is numeric (substituted as a number in formulas).
    fn dim_numeric(&self, i: usize) -> PyResult<bool> {
        ops::dim_numeric(&self.doc, i).map_err(err)
    }

    /// Offset of the label of dimension `i` from the dimension's anchor as
    /// (x, y); None for an empty slot or a bad index.
    fn dim_offset(&self, i: usize) -> Option<(f64, f64)> {
        ops::dim_offset(&self.doc, i).ok().map(pair)
    }

    /// World coordinates of the anchor of dimension `i` (the midpoint of a
    /// length, the vertex of an angle, the centre of a circle or conic) as
    /// (x, y); None for an empty slot or a bad index. The label sits at
    /// `dim_offset` from here.
    fn dim_anchor(&self, i: usize) -> Option<(f64, f64)> {
        ops::dim_anchor(&self.doc, i).ok().map(pair)
    }

    /// World coordinates of the anchor a dimension of kind `kind` on entities
    /// a and b would have, as (x, y), for placing its label before adding it.
    #[pyo3(signature = (kind, a, b = None))]
    fn dim_anchor_of(&self, kind: &str, a: usize, b: Option<usize>) -> PyResult<(f64, f64)> {
        ops::dim_anchor_of(&self.doc, DimType::from_name(kind).map_err(err)?, a, b.unwrap_or(usize::MAX)).map(pair).map_err(err)
    }

    /// Expression driving dimension `i`: empty when a plain value drives it
    /// or it is a reference dimension; None for a bad index.
    fn dim_expr(&self, i: usize) -> Option<String> {
        ops::dim_expr(&self.doc, i).ok()
    }

    /// The value dimension `i` is set to (radians for angles): for a driving
    /// dimension the value it drives toward, which differs from `dim_value`
    /// while the constraints conflict; for a reference dimension its measured
    /// value. NaN for a bad index.
    fn dim_set_value(&self, i: usize) -> f64 {
        ops::dim_set_value(&self.doc, i).unwrap_or(f64::NAN)
    }

    /// The entities dimension `i` measures, in the order `add_dim` takes them.
    fn dim_ents(&self, i: usize) -> PyResult<Vec<usize>> {
        ops::dim_ents(&self.doc, i).map_err(err)
    }

    /// True when angle dimension `i` measures the supplementary angle of its
    /// lines.
    fn dim_supp(&self, i: usize) -> PyResult<bool> {
        ops::dim_supp(&self.doc, i).map_err(err)
    }

    /// True when the last solve left dimension `i` unsatisfied.
    fn dim_conflict(&self, i: usize) -> PyResult<bool> {
        ops::dim_conflict(&self.doc, i).map_err(err)
    }

    /// Why dimension `i` could not be turned into an equation; None when it
    /// could.
    fn dim_error(&self, i: usize) -> PyResult<Option<String>> {
        ops::dim_error(&self.doc, i).map_err(err)
    }

    /// Kind name of dimension `i` ("distance", "radius", "angle", ...); None
    /// for an empty slot or a bad index.
    fn dim_kind(&self, i: usize) -> Option<&'static str> {
        ops::dim_kind(&self.doc, i).ok().map(|k| k.name())
    }

    /// Number of variable slots. Removed variables leave empty slots.
    #[getter]
    fn var_count(&self) -> usize {
        ops::var_count(&self.doc)
    }

    /// Name of variable `i`; None for an empty slot or a bad index.
    fn var_name(&self, i: usize) -> Option<String> {
        ops::var_name(&self.doc, i).ok()
    }

    /// Value of variable `i` (the value of its expression for a defined
    /// variable); NaN for an empty slot, a bad index or an expression that
    /// does not parse.
    fn var_value(&self, i: usize) -> f64 {
        ops::var_value(&self.doc, i).unwrap_or(f64::NAN)
    }

    /// Expression of variable `i` (empty for a value variable); None for an
    /// empty slot or a bad index.
    fn var_expr(&self, i: usize) -> Option<String> {
        ops::var_expr(&self.doc, i).ok()
    }

    /// True when variable `i` is free, False when it is solved or defined by
    /// an expression.
    fn var_free(&self, i: usize) -> PyResult<bool> {
        ops::var_free(&self.doc, i).map_err(err)
    }

    /// True when variable `i` is numeric (substituted as a number in formulas).
    fn var_numeric(&self, i: usize) -> PyResult<bool> {
        ops::var_numeric(&self.doc, i).map_err(err)
    }

    // ------------------------------------------------------------ entities

    /// Number of entity slots. Deleted entities leave empty slots.
    #[getter]
    fn ent_count(&self) -> usize {
        ops::ent_count(&self.doc)
    }

    /// Kind of entity `e`: "point", "line", "circle", "arc", "conic" or
    /// "curve"; None for an empty slot or a bad index.
    fn ent_kind(&self, e: usize) -> Option<&'static str> {
        ops::ent_kind(&self.doc, e).ok().map(|k| k.name())
    }

    /// Name of entity `e` (points are named, curves when a .sgp file names
    /// them); None for an empty slot, a bad index or an unnamed entity.
    fn ent_name(&self, e: usize) -> Option<String> {
        ops::ent_name(&self.doc, e).ok()
    }

    /// World coordinates of point `e` as (x, y); None for an empty slot or a
    /// non-point.
    fn point_pos(&self, e: usize) -> Option<(f64, f64)> {
        ops::point_pos(&self.doc, e).ok().map(pair)
    }

    /// Endpoints of line `e` as (a, b, infinite); None for a non-line.
    fn line_pts(&self, e: usize) -> Option<(usize, usize, bool)> {
        ops::line_pts(&self.doc, e).ok()
    }

    /// Center point and radius of circle or arc `e` as (center, radius);
    /// None for a non-circle.
    fn circle_vals(&self, e: usize) -> Option<(usize, f64)> {
        ops::circle_vals(&self.doc, e).ok()
    }

    /// Center and endpoints of arc `e` as (c, a, b); None for a non-arc.
    fn arc_pts(&self, e: usize) -> Option<(usize, usize, usize)> {
        ops::arc_pts(&self.doc, e).ok()
    }

    /// Kind, center and axis values of conic `e` as
    /// (kind, center, a, b, angle); None for a non-conic.
    fn conic_vals(&self, e: usize) -> Option<(&'static str, usize, f64, f64, f64)> {
        let (k, c, a, b, t) = ops::conic_vals(&self.doc, e).ok()?;
        Some((conic_name(k), c, a, b, t))
    }

    /// Source text of function curve `e`; None for a non-curve.
    fn curve_src(&self, e: usize) -> Option<String> {
        ops::curve_src(&self.doc, e).ok()
    }

    /// True when entity `e` is a construction (dashed helper).
    fn ent_construction(&self, e: usize) -> PyResult<bool> {
        ops::ent_construction(&self.doc, e).map_err(err)
    }

    /// True when entity `e` is hidden.
    fn ent_hidden(&self, e: usize) -> PyResult<bool> {
        ops::ent_hidden(&self.doc, e).map_err(err)
    }

    /// True when the constraints determine entity `e` completely: nothing
    /// about its position or shape is left free.
    fn ent_fixed(&self, e: usize) -> PyResult<bool> {
        ops::ent_fixed(&self.doc, e).map_err(err)
    }

    /// True when the last solve left entity `e` in conflict.
    fn ent_conflict(&self, e: usize) -> PyResult<bool> {
        ops::ent_conflict(&self.doc, e).map_err(err)
    }

    /// True when line `e` is infinite.
    fn ent_infinite(&self, e: usize) -> PyResult<bool> {
        ops::ent_infinite(&self.doc, e).map_err(err)
    }

    /// Number of constraint slots. Deleted constraints leave empty slots.
    #[getter]
    fn con_count(&self) -> usize {
        ops::con_count(&self.doc)
    }

    /// Kind name of constraint `c` (e.g. "coincident", "parallel",
    /// "symmetric", ...); None for an empty slot or a bad index.
    fn con_kind(&self, c: usize) -> Option<&'static str> {
        ops::con_kind(&self.doc, c).ok().map(|k| k.name())
    }

    /// The entities of constraint `c`, in the order they appear in it.
    fn con_ents(&self, c: usize) -> PyResult<Vec<usize>> {
        ops::con_ents(&self.doc, c).map_err(err)
    }

    /// True when the last solve left constraint `c` unsatisfied.
    fn con_conflict(&self, c: usize) -> PyResult<bool> {
        ops::con_conflict(&self.doc, c).map_err(err)
    }

    /// Why constraint `c` could not be turned into equations; None when it
    /// could.
    fn con_error(&self, c: usize) -> PyResult<Option<String>> {
        ops::con_error(&self.doc, c).map_err(err)
    }

    /// True for an "offset" constraint whose offset is turned by 90° (the
    /// point of a perpendicular line).
    fn con_turned(&self, c: usize) -> PyResult<bool> {
        ops::con_turned(&self.doc, c).map_err(err)
    }

    /// Contact point of tangency constraint `c` as (x, y, solved), where
    /// `solved` is True for a point the solver finds that no entity marks;
    /// None for other constraints.
    fn con_contact(&self, c: usize) -> PyResult<Option<(f64, f64, bool)>> {
        Ok(ops::con_contact(&self.doc, c).map_err(err)?.map(|t| (t.at[0], t.at[1], t.solved)))
    }

    /// World-coordinate polylines drawing entity `e` inside the window from
    /// (x0, y0) to (x1, y1) with pixels of size `px`, as a list of
    /// polylines, each a list of (x, y) pairs: one point for a point, the
    /// endpoints of a segment, a segment crossing the window for an infinite
    /// line, the outline of a circle, arc or conic (two polylines for a
    /// hyperbola), and the parts of a function curve inside the window.
    #[allow(clippy::too_many_arguments)]
    fn ent_samples(&self, e: usize, x0: f64, y0: f64, x1: f64, y1: f64, px: f64) -> PyResult<Vec<Vec<(f64, f64)>>> {
        let polys = ops::ent_samples(&self.doc, e, [x0, y0], [x1, y1], px).map_err(err)?;
        Ok(polys.into_iter().map(|pl| pl.into_iter().map(pair).collect()).collect())
    }

    // ------------------------------------------------------------ formulas

    /// Closed-form formula of dimension `i`. `extra_ms` extends the search
    /// beyond the default budget; `cancel` stops it early. Other Python
    /// threads run meanwhile. Raises `sggeo.Error` when no formula is found.
    #[pyo3(signature = (i, extra_ms = 0, cancel = None))]
    fn derive_dim(&self, py: Python<'_>, i: usize, extra_ms: u64, cancel: Option<PyRef<'_, Cancel>>) -> PyResult<Formula> {
        self.derive(py, Target::Dim(i), extra_ms, cancel)
    }

    /// Closed-form formula of variable `i`, as `derive_dim` derives it.
    #[pyo3(signature = (i, extra_ms = 0, cancel = None))]
    fn derive_var(&self, py: Python<'_>, i: usize, extra_ms: u64, cancel: Option<PyRef<'_, Cancel>>) -> PyResult<Formula> {
        self.derive(py, Target::Var(i), extra_ms, cancel)
    }

    /// The dimensions and value variables by name, with their values: the
    /// value a driving dimension is set to (radians for angles), the measured
    /// value of a reference dimension, and the value of a variable (a
    /// variable defined by an expression is never a symbol of a formula,
    /// which reads its expression). The symbols of `Formula.sympy` take
    /// these values:
    /// `expr.subs({s: values[s.name] for s in expr.free_symbols})`.
    fn values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let out = PyDict::new(py);
        for i in 0..ops::dim_count(&self.doc) {
            let (Ok(name), Ok(driving)) = (ops::dim_name(&self.doc, i), ops::dim_driving(&self.doc, i)) else { continue };
            let v = if driving { ops::dim_set_value(&self.doc, i) } else { ops::dim_value(&self.doc, i) };
            out.set_item(name, v.unwrap_or(f64::NAN))?;
        }
        for i in 0..ops::var_count(&self.doc) {
            if let (Ok(name), Ok(""), Ok(v)) = (ops::var_name(&self.doc, i), ops::var_expr(&self.doc, i).as_deref(), ops::var_value(&self.doc, i)) {
                out.set_item(name, v)?;
            }
        }
        Ok(out)
    }
}

impl PyDoc {
    fn derive(&self, py: Python<'_>, t: Target, extra_ms: u64, cancel: Option<PyRef<'_, Cancel>>) -> PyResult<Formula> {
        // a copy derives while other threads run, as the document stays with Python
        let doc = ops::copy(&self.doc);
        let cancel = cancel.map(|c| c.c.clone());
        let src = py.detach(move || ops::derive(&doc, t, extra_ms, cancel.as_ref())).map_err(err)?;
        let f = src.clone();
        Ok(Formula {
            src,
            name: f.name,
            plain: f.plain,
            latex: f.latex,
            latex_short: f.latex_short,
            defs: f.defs,
            exact: f.exact,
            is_angle: f.is_angle,
            value: f.value,
            actual: f.actual,
            size: f.size,
            limits: f.limits,
        })
    }
}

/// LaTeX text, such as the `latex` or `latex_short` of a `Formula`, typeset
/// in display style as the bytes of a PNG image: glyphs of color `rgb`
/// (0xRRGGBB) on a transparent background, at `size_pt` points and `dpi`
/// dots per inch (a point is 1/72 inch, so 12 pt at 96 dpi puts 16 pixels to
/// the em).
#[pyfunction]
#[pyo3(signature = (latex, size_pt = 12.0, dpi = 96.0, rgb = 0))]
fn latex_png<'py>(py: Python<'py>, latex: &str, size_pt: f64, dpi: f64, rgb: u32) -> PyResult<Bound<'py, PyBytes>> {
    if rgb > 0xFF_FFFF {
        return Err(PyValueError::new_err("rgb is a color 0xRRGGBB"));
    }
    let [_, r, g, b] = rgb.to_be_bytes();
    let png = ops::latex_png(latex, size_pt, dpi, [r, g, b]).map_err(err)?;
    Ok(PyBytes::new(py, &png))
}

/// The languages of `Formula.code`, in the order of their numbers in C: each
/// as `(name, label)`, the name `code` takes and the name to show (`("js",
/// "JavaScript")`).
#[pyfunction]
fn code_languages() -> Vec<(&'static str, &'static str)> {
    ops::code_languages()
}

/// The kinds of constraint by name (as `constrain` and `con_kind` have
/// them), in the order of their numbers in C.
#[pyfunction]
fn con_kinds() -> Vec<&'static str> {
    ops::con_kinds()
}

/// The kinds of dimension by name (as `add_dim` and `dim_kind` have them),
/// in the order of their numbers in C.
#[pyfunction]
fn dim_kinds() -> Vec<&'static str> {
    ops::dim_kinds()
}

/// The kinds of entity by name (as `ent_kind` has them), in the order of
/// their numbers in C.
#[pyfunction]
fn ent_kinds() -> Vec<&'static str> {
    ops::ent_kinds()
}

/// Version of the library.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// sggeo: geometry with constraints that gives closed-form formulas for what
/// you measure. `Doc` holds a construction; `Doc.derive_dim` and
/// `Doc.derive_var` derive the formula of a measured quantity, which
/// `Formula.sympy` turns into a SymPy expression.
#[pymodule]
fn sggeo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDoc>()?;
    m.add_class::<Formula>()?;
    m.add_class::<Solved>()?;
    m.add_class::<Cancel>()?;
    m.add("Error", m.py().get_type::<Error>())?;
    m.add_function(wrap_pyfunction!(latex_png, m)?)?;
    m.add_function(wrap_pyfunction!(code_languages, m)?)?;
    m.add_function(wrap_pyfunction!(con_kinds, m)?)?;
    m.add_function(wrap_pyfunction!(dim_kinds, m)?)?;
    m.add_function(wrap_pyfunction!(ent_kinds, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

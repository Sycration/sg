"""sggeo: geometry with constraints that gives closed-form formulas for what
you measure. `Doc` holds a construction; `Doc.derive_dim` and
`Doc.derive_var` derive the formula of a measured quantity, which
`Formula.sympy` turns into a SymPy expression."""

# Written by tests/python/stub.py from the module; do not edit.

from collections.abc import Sequence
from typing import Any, final

# an expression as sggeo reads it: text, a number, or a SymPy expression (a curve: a sympy.Eq)
_ExprLike = str | float | Any

__all__ = ['Cancel', 'Doc', 'Error', 'Formula', 'Solved', '__version__', 'code_languages', 'con_kinds', 'dim_kinds', 'ent_kinds', 'latex_png', 'version']
__version__: str

class Error(ValueError):
    """A call that sggeo refused, with the reason as its message."""

@final
class Doc:
    """A geometry document with its solver state and undo history."""
    def __init__(self) -> None:
        ...
    @staticmethod
    def from_sgp(text: str) -> Doc:
        """Document decoded from .sgp text. Call `solve` before reading values from it."""
    def to_sgp(self) -> str:
        """Document encoded as .sgp text."""
    def to_svg(self, width: float = 480.0, height: float = 360.0) -> str:
        """
        The construction drawn as an SVG picture of `width` × `height` pixels
        (16 to 4096 each): its curves, its points with their names, and each
        dimension as `name = value` at its label (driving ones purple, measured
        ones green, angles in degrees), scaled to fit, with the axes in grey. In
        Jupyter a `Doc` shows as this picture.
        """
    def _repr_svg_(self) -> str:
        ...
    def solve(self) -> Solved:
        """
        Solves the constraints and stores the solution in the document. The
        result is true when every constraint is satisfied.
        """
    def undo(self) -> bool:
        """Undoes the last change; False when there is nothing to undo."""
    def redo(self) -> bool:
        """Redoes the last undone change; False when there is nothing to redo."""
    def checkpoint(self) -> None:
        """
        Records the current document as an undo step. A drag (`drag_point`,
        `drag_var`, ...) is not undoable by itself, so a checkpoint before it
        makes the whole drag one undo step.
        """
    def dof(self) -> int:
        """Remaining degrees of freedom after the last solve."""
    def undo_len(self) -> int:
        """Number of steps in the undo history; see `join_undo`."""
    def join_undo(self, len: int) -> bool:
        """
        Joins the undo steps recorded since the undo history held `len` steps
        (a value of `undo_len` read before them) into one step, so that a
        change made with several calls undoes at once. False when fewer than
        two steps were recorded since. The history keeps at most 300 steps;
        past that, steps recorded since `len` can be left unjoined.
        """
    def copy(self) -> Doc:
        """
        A copy of the document in its current state, with the same solve
        status and without the undo history.
        """
    def __copy__(self) -> Doc:
        ...
    def load_sgp(self, text: str) -> None:
        """
        Replaces the content of the document with the document decoded from
        .sgp text, as one undoable change, and solves it.
        """
    def apply_sgp(self, text: str) -> bool:
        """
        Applies .sgp statements (lines of a .sgp file, without the header) to
        the document as one undoable change; a `where` line placing a point
        that is there drags it. False when the change is rejected (the
        constraints would not solve or already imply it).
        """
    def ent_by_name(self, name: str) -> int | None:
        """
        The entity named `name` as a .sgp file names it (a point or curve by
        its name, a line by its points run together: `AB`); None when there
        is none.
        """
    def clear(self) -> None:
        """
        Replaces the content of the document with an empty document (origin
        and axes only), as one undoable change.
        """
    @property
    def generation(self) -> int:
        """
        Counter raised by every change of the document, including drags;
        equal values mean an unchanged document.
        """
    @property
    def conflict_count(self) -> int:
        """
        Number of constraints, dimensions and entities the last solve left
        unsatisfied.
        """
    @property
    def origin(self) -> int:
        """The origin point."""
    @property
    def x_axis(self) -> int:
        """The x axis (an infinite line through the origin)."""
    @property
    def y_axis(self) -> int:
        """The y axis (an infinite line through the origin)."""
    @property
    def dim_count(self) -> int:
        """Number of dimension slots. Removed dimensions leave empty slots."""
    @property
    def var_count(self) -> int:
        """Number of variable slots. Removed variables leave empty slots."""
    @property
    def ent_count(self) -> int:
        """Number of entity slots. Deleted entities leave empty slots."""
    @property
    def con_count(self) -> int:
        """Number of constraint slots. Deleted constraints leave empty slots."""
    def add_point(self, x: float, y: float) -> int:
        """Adds a free point at (x, y), its starting position. Returns the entity."""
    def add_line(self, a: int, b: int, infinite: bool = False, axis: str | None = None) -> int:
        """
        Adds a line from point a to point b: a segment, or an infinite line
        when `infinite`. With `axis` "horizontal" or "vertical", the line is
        also constrained along that axis. Returns the entity.
        """
    def add_circle(self, c: int, r: float) -> int:
        """
        Adds a circle around point c with starting radius r (the radius stays
        free until constrained). Returns the entity.
        """
    def add_arc(self, c: int, a: int, b: int) -> int:
        """
        Adds an arc around point c from point a counterclockwise to point b
        (|ca| = |cb| is enforced). Returns the entity.
        """
    def add_curve(self, src: _ExprLike) -> int:
        """
        Adds a curve: `y = f(x)` or `F(x, y) = G(x, y)` in x, y, variables
        and dimensions, as text or a SymPy expression or `Eq`. Returns the
        entity.
        """
    def set_curve(self, i: int, src: _ExprLike) -> bool:
        """
        Sets the definition of curve `i`, as `add_curve` takes it. False when
        rejected.
        """
    def add_conic(self, kind: str, c: int, a: float, b: float, t: float = 0.0) -> int:
        """
        Adds a conic: `kind` is "ellipse", "hyperbola" or "parabola",
        centered on point c (the vertex of a parabola), with half-axes a and
        b (b is ignored for a parabola, where a is the focal distance) and
        axis angle t in radians. Returns the entity.
        """
    def add_point_on(self, c: int, x: float, y: float) -> int:
        """Adds a point at (x, y) on curve c, bound to it by a constraint."""
    def add_circle_to(self, c: int, p: int) -> int:
        """Adds the circle around point c through point p, bound to it."""
    def add_arc_toward(self, c: int, a: int, x: float, y: float) -> int:
        """
        Adds the arc around point c from point a, ending at a new point in
        the direction of (x, y).
        """
    def add_midpoint(self, a: int, b: int) -> int:
        """Adds the midpoint of points a and b, bound to them. Returns the point."""
    def add_line_midpoint(self, l: int) -> int:
        """Adds the midpoint of line l, bound to it. Returns the point."""
    def add_intersection(self, a: int, b: int, x: float, y: float) -> int:
        """Adds a point at a crossing of curves a and b, starting from (x, y)."""
    def add_intersection_near(self, a: int, b: int, x: float, y: float) -> int:
        """Adds a point at the crossing of curves a and b nearest to (x, y)."""
    def add_perp_line(self, l: int, p: int) -> int:
        """Adds the infinite line through point p perpendicular to line l."""
    def add_parallel_line(self, l: int, p: int) -> int:
        """Adds the infinite line through point p parallel to line l."""
    def add_perp_bisector(self, a: int, b: int) -> tuple[int, int]:
        """
        Adds the perpendicular bisector of points a and b. Returns
        (line, midpoint).
        """
    def add_angle_bisector(self, l1: int, l2: int, x: float, y: float) -> tuple[int, int]:
        """
        Adds the bisector of lines l1 and l2 in the sector containing (x, y).
        Returns (line, vertex); the vertex is a shared endpoint, or a new
        point at the crossing.
        """
    def add_tangent_line(self, p: int, c: int, x: float, y: float) -> tuple[int, int]:
        """
        Adds the segment from point p touching curve c at a contact point
        starting from (x, y). Returns (segment, contact point).
        """
    def add_tangent_line_near(self, p: int, c: int, x: float, y: float) -> tuple[int, int]:
        """Like `add_tangent_line`, with the contact point nearest to (x, y)."""
    def add_reflection(self, p: int, l: int) -> int:
        """Adds the mirror image of point p in line l."""
    def add_circle_through(self, a: int, b: int, c: int) -> tuple[int, int]:
        """
        Adds the circle through points a, b and c. Returns (circle, center),
        the center being a new point.
        """
    def add_arc_through(self, a: int, m: int, b: int) -> tuple[int, int]:
        """
        Adds the arc from point a through point m to point b. Returns (arc,
        center), the center being a new point.
        """
    def add_conic_by_foci(self, kind: str, f1: int, f2: int, p: int) -> int:
        """
        Adds the ellipse or hyperbola (`kind` "ellipse" or "hyperbola") with
        foci f1 and f2 through point p.
        """
    def add_parabola_by_focus(self, f: int, l: int) -> int:
        """Adds the parabola with focus f and directrix line l."""
    def add_conic_from(self, kind: str, c: int, ax: float, ay: float, sx: float, sy: float) -> int:
        """
        Adds the ellipse or hyperbola centered on point c, its first axis
        ending at (ax, ay) and its second half-axis reaching the line through
        (sx, sy) parallel to the first axis.
        """
    def add_parabola_toward(self, c: int, x: float, y: float) -> int:
        """
        Adds the parabola with vertex at point c opening toward (x, y), its
        focal distance the distance to (x, y).
        """
    def add_foci(self, c: int) -> list[int]:
        """Adds the foci of conic c: two points, one for a parabola."""
    def add_directrix(self, c: int, x: float, y: float) -> int:
        """Adds the directrix of conic c belonging to the focus nearest (x, y)."""
    def add_asymptotes(self, c: int) -> list[int]:
        """Adds both asymptotes of hyperbola c."""
    def constrain(self, kind: str, a: int, b: int | None = None) -> bool:
        """
        Adds a constraint: `kind` is "coincident", "on_curve", "midpoint",
        "horizontal", "vertical", "parallel", "perpendicular", "collinear",
        "equal", "concentric" or "tangent", on entities a and b ("horizontal"
        and "vertical" take a line, or two points). False when it is
        rejected.
        """
    def constrain_near(self, kind: str, a: int, b: int | None, x: float, y: float) -> bool:
        """
        Like `constrain`; a tangency between two curves whose contact point
        is solved for starts its contact search at the point of the curves
        nearest to (x, y), such as where the user clicked.
        """
    def can_constrain(self, kind: str, a: int, b: int | None = None) -> bool:
        """
        True when `constrain` would accept these arguments; the constraint
        might still be rejected for not solving. The document is unchanged.
        """
    def constrain_symmetric(self, a: int, b: int, l: int) -> bool:
        """
        Constrains points a and b to be mirror images of each other in line
        `l`. False when rejected.
        """
    def add_constraints(self, cons: Sequence[tuple[str, int, int | None]]) -> bool:
        """
        Adds the constraints `cons`, each a (kind, a, b) tuple as `constrain`
        takes it (b is None where a kind takes one entity), as one undoable
        change, rejecting them together when any would not solve or is
        redundant. False when rejected.
        """
    def add_dim(self, kind: str, a: int, b: int | None = None, off_x: float = 0.3, off_y: float = 0.3) -> int:
        """
        Adds a dimension: `kind` is "distance", "hdist", "vdist",
        "point_line", "length", "radius", "diameter", "angle", "conic_a",
        "conic_b", "x" or "y", on entities a and b, with its label at
        (off_x, off_y) from the dimension's anchor (the midpoint of a length,
        the vertex of an angle). It is driving when that keeps the
        constraints solvable, otherwise a reference (measured) dimension. For
        an angle, the sector holding the label selects the angle or its
        supplement. Returns the dimension.
        """
    def set_dim(self, i: int, value: _ExprLike) -> bool:
        """
        Sets dimension `i` as typed: a number (degrees for angles) or an
        expression of variables and dimensions, as text or a SymPy
        expression, drives it; empty text makes it a reference dimension.
        False when the value cannot be reached.
        """
    def set_dim_driving(self, i: int, driving: bool) -> bool:
        """
        Makes dimension `i` driving or reference. False when making it
        driving is rejected (the constraints already determine it).
        """
    def set_dim_numeric(self, i: int, numeric: bool) -> bool:
        """
        Marks dimension `i` numeric or not. Formulas derived for other targets
        use a numeric dimension's current value as a plain number. Solving and
        the dimension itself are unaffected.
        """
    def set_dim_offset(self, i: int, off_x: float, off_y: float) -> None:
        """
        Moves the label of dimension `i` to (off_x, off_y) from the
        dimension's anchor (see `dim_anchor`), as one undoable change. For a
        reference angle, the sector holding the label selects the angle or
        its supplement; a driving angle keeps its choice.
        """
    def drag_dim_offset(self, i: int, off_x: float, off_y: float) -> None:
        """
        Moves the label of dimension `i` like `set_dim_offset`, as a drag:
        not an undoable change by itself (see `checkpoint`).
        """
    def toggle_supp(self, i: int) -> bool:
        """
        Toggles angle dimension `i` between the angle of its lines and its
        supplement; a driving dimension with a plain value is set to the
        supplement's value. False when rejected.
        """
    def add_var(self, name: str | None = None, value: float = 1.0, free: bool = True) -> int:
        """
        Adds a value variable. Without a name, the first unused of a, b, c, k,
        ... is taken. A free variable (default) is set by `value`, otherwise
        it is solved from the constraints, starting at `value`. Returns the
        variable.
        """
    def add_def_var(self, expr: _ExprLike, name: str | None = None) -> int:
        """
        Adds a variable defined by the expression `expr` of variables and
        dimensions, as text or a SymPy expression. Without a name, f1, f2, ...
        is taken. Returns the variable.
        """
    def set_var(self, i: int, value: float) -> bool:
        """Sets the value of value variable `i`. False when the value cannot be reached."""
    def set_var_expr(self, i: int, expr: _ExprLike) -> bool:
        """
        Sets variable `i` as typed: a plain number makes it a free value
        variable, any other text or SymPy expression an expression of
        variables and dimensions. False when rejected.
        """
    def check_expr(self, expr: _ExprLike) -> str | None:
        """
        Checks an expression as `set_var_expr` and `set_dim` read it: the
        reason when it does not parse or names an unknown quantity, None
        when it is valid.
        """
    def check_curve(self, src: _ExprLike) -> str | None:
        """
        Checks a curve definition as `add_curve` reads it: the reason when it
        does not parse, None when it is valid.
        """
    def set_var_free(self, i: int, free: bool) -> bool:
        """
        Makes value variable `i` free (set by its value) or solved from the
        constraints. False when rejected.
        """
    def set_var_numeric(self, i: int, numeric: bool) -> bool:
        """
        Marks variable `i` numeric or not. Formulas derived for other targets
        use a numeric variable's current value as a plain number. Solving and
        the variable itself (value, expression, free or solved) are
        unaffected.
        """
    def delete_var(self, i: int) -> bool:
        """
        Removes variable `i`, leaving an empty slot; expressions naming it
        fail to parse until edited. False when rejected.
        """
    def delete_ent(self, e: int) -> bool:
        """
        Deletes entity `e` and what depends on it (constraints and dimensions
        that reference it, and features built from it). False when the
        constraints would no longer solve.
        """
    def delete_con(self, c: int) -> bool:
        """
        Deletes constraint `c`. False when the constraints would no longer
        solve.
        """
    def delete_dim(self, i: int) -> bool:
        """
        Deletes dimension `i`; expressions naming it fail to parse until
        edited. False when the constraints would no longer solve.
        """
    def delete(self, *, ents: Sequence[int] = ..., cons: Sequence[int] = ..., dims: Sequence[int] = ..., vars: Sequence[int] = ...) -> bool:
        """
        Deletes the given entities, constraints, dimensions and variables as
        one undoable change, with everything that depends on the entities.
        False when the constraints would no longer solve; nothing is deleted.
        """
    def lock_point(self, e: int, on: bool = True) -> bool:
        """Locks point `e` (its coordinates become fixed) or unlocks it."""
    def lock_points(self, pts: Sequence[int], on: bool = True) -> bool:
        """Locks or unlocks the points `pts` as one undoable change."""
    def point_locked(self, e: int) -> bool:
        """True when point `e` is locked (both coordinates fixed)."""
    def set_construction(self, e: int, on: bool = True) -> bool:
        """Sets the construction flag of entity `e` (a dashed helper)."""
    def set_construction_all(self, ents: Sequence[int], on: bool = True) -> bool:
        """
        Sets the construction flag of the non-point entities `ents` as one
        undoable change.
        """
    def set_hidden(self, e: int, on: bool = True) -> bool:
        """Sets the hidden flag of entity `e`. Hidden entities are not drawn."""
    def set_infinite(self, e: int, on: bool = True) -> bool:
        """Makes line `e` infinite or a segment."""
    def toggle_infinite(self, lines: Sequence[int]) -> bool:
        """
        Switches each of `lines` between infinite and segment, as one undoable
        change.
        """
    def drag_point(self, e: int, x: float, y: float) -> bool:
        """
        Pulls point `e` toward (x, y) by solving the constraints with it
        pinned there (a drag, not an undoable change). False when the
        configuration cannot reach the target.
        """
    def drag_points(self, pts: Sequence[int], targets: Sequence[tuple[float, float]]) -> bool:
        """
        Pulls each point of `pts` toward its target in `targets`, (x, y)
        pairs, together, solving the constraints (such as the ends of a
        dragged line). A drag, not an undoable change. False when the points
        cannot reach their targets.
        """
    def drag_circle(self, e: int, x: float, y: float) -> bool:
        """Pulls the radius of circle `e` so its edge passes through (x, y)."""
    def drag_conic_handle(self, e: int, which: int, x: float, y: float) -> bool:
        """
        Pulls a handle of conic `e` (`which` 0 the first-axis end, 1 the
        second-axis end) toward (x, y).
        """
    def drag_curve(self, e: int, x: float, y: float) -> bool:
        """
        Pulls curve `e` so it passes through (x, y), solving the constraints.
        A drag, not an undoable change. False when the curve cannot reach the
        point.
        """
    def drag_var(self, i: int, value: float) -> bool:
        """
        Sets value variable `i` to `value` and solves, as a drag: not an
        undoable change by itself (see `checkpoint`). False when the
        constraints do not solve at that value; the variable keeps its value.
        """
    def rename(self, old: str, new: str) -> bool:
        """
        Gives the point, dimension or variable named `old` the name `new`,
        rewriting expressions that use it. False when `new` is not an
        identifier or is taken.
        """
    def dim_name(self, i: int) -> str | None:
        """Name of dimension `i`; None for an empty slot or a bad index."""
    def dim_value(self, i: int) -> float:
        """
        Current measured value of dimension `i` (radians for angles); NaN
        for an empty slot or a bad index. A value that is 0 up to the
        rounding of the solved coordinates is exactly 0.
        """
    def dim_uncertainty(self, i: int) -> float:
        """
        How precisely the measured value of dimension `i` (`dim_value`) is
        known: about how far it can be from the exact value of the
        construction. Near 1e-13 of the value where the construction is
        regular; at a double root (the foci of an ellipse with a = b) about the
        square root of that. A display can round the value to it. NaN for an
        empty slot or a bad index.
        """
    def dim_driving(self, i: int) -> bool:
        """True when dimension `i` is driving, False when it is a reference dimension."""
    def dim_numeric(self, i: int) -> bool:
        """True when dimension `i` is numeric (substituted as a number in formulas)."""
    def dim_offset(self, i: int) -> tuple[float, float] | None:
        """
        Offset of the label of dimension `i` from the dimension's anchor as
        (x, y); None for an empty slot or a bad index.
        """
    def dim_anchor(self, i: int) -> tuple[float, float] | None:
        """
        World coordinates of the anchor of dimension `i` (the midpoint of a
        length, the vertex of an angle, the centre of a circle or conic) as
        (x, y); None for an empty slot or a bad index. The label sits at
        `dim_offset` from here.
        """
    def dim_anchor_of(self, kind: str, a: int, b: int | None = None) -> tuple[float, float]:
        """
        World coordinates of the anchor a dimension of kind `kind` on entities
        a and b would have, as (x, y), for placing its label before adding it.
        """
    def dim_expr(self, i: int) -> str | None:
        """
        Expression driving dimension `i`: empty when a plain value drives it
        or it is a reference dimension; None for a bad index.
        """
    def dim_set_value(self, i: int) -> float:
        """
        The value dimension `i` is set to (radians for angles): for a driving
        dimension the value it drives toward, which differs from `dim_value`
        while the constraints conflict; for a reference dimension its measured
        value. NaN for a bad index.
        """
    def dim_ents(self, i: int) -> list[int]:
        """The entities dimension `i` measures, in the order `add_dim` takes them."""
    def dim_supp(self, i: int) -> bool:
        """
        True when angle dimension `i` measures the supplementary angle of its
        lines.
        """
    def dim_conflict(self, i: int) -> bool:
        """True when the last solve left dimension `i` unsatisfied."""
    def dim_error(self, i: int) -> str | None:
        """
        Why dimension `i` could not be turned into an equation; None when it
        could.
        """
    def dim_kind(self, i: int) -> str | None:
        """
        Kind name of dimension `i` ("distance", "radius", "angle", ...); None
        for an empty slot or a bad index.
        """
    def var_name(self, i: int) -> str | None:
        """Name of variable `i`; None for an empty slot or a bad index."""
    def var_value(self, i: int) -> float:
        """
        Value of variable `i` (the value of its expression for a defined
        variable); NaN for an empty slot, a bad index or an expression that
        does not parse.
        """
    def var_expr(self, i: int) -> str | None:
        """
        Expression of variable `i` (empty for a value variable); None for an
        empty slot or a bad index.
        """
    def var_free(self, i: int) -> bool:
        """
        True when variable `i` is free, False when it is solved or defined by
        an expression.
        """
    def var_numeric(self, i: int) -> bool:
        """True when variable `i` is numeric (substituted as a number in formulas)."""
    def ent_kind(self, e: int) -> str | None:
        """
        Kind of entity `e`: "point", "line", "circle", "arc", "conic" or
        "curve"; None for an empty slot or a bad index.
        """
    def ent_name(self, e: int) -> str | None:
        """
        Name of entity `e` (points are named, curves when a .sgp file names
        them); None for an empty slot, a bad index or an unnamed entity.
        """
    def point_pos(self, e: int) -> tuple[float, float] | None:
        """
        World coordinates of point `e` as (x, y); None for an empty slot or a
        non-point.
        """
    def line_pts(self, e: int) -> tuple[int, int, bool] | None:
        """Endpoints of line `e` as (a, b, infinite); None for a non-line."""
    def circle_vals(self, e: int) -> tuple[int, float] | None:
        """
        Center point and radius of circle or arc `e` as (center, radius);
        None for a non-circle.
        """
    def arc_pts(self, e: int) -> tuple[int, int, int] | None:
        """Center and endpoints of arc `e` as (c, a, b); None for a non-arc."""
    def conic_vals(self, e: int) -> tuple[str, int, float, float, float] | None:
        """
        Kind, center and axis values of conic `e` as
        (kind, center, a, b, angle); None for a non-conic.
        """
    def curve_src(self, e: int) -> str | None:
        """Source text of function curve `e`; None for a non-curve."""
    def ent_construction(self, e: int) -> bool:
        """True when entity `e` is a construction (dashed helper)."""
    def ent_hidden(self, e: int) -> bool:
        """True when entity `e` is hidden."""
    def ent_fixed(self, e: int) -> bool:
        """
        True when the constraints determine entity `e` completely: nothing
        about its position or shape is left free.
        """
    def ent_conflict(self, e: int) -> bool:
        """True when the last solve left entity `e` in conflict."""
    def ent_infinite(self, e: int) -> bool:
        """True when line `e` is infinite."""
    def con_kind(self, c: int) -> str | None:
        """
        Kind name of constraint `c` (e.g. "coincident", "parallel",
        "symmetric", ...); None for an empty slot or a bad index.
        """
    def con_ents(self, c: int) -> list[int]:
        """The entities of constraint `c`, in the order they appear in it."""
    def con_conflict(self, c: int) -> bool:
        """True when the last solve left constraint `c` unsatisfied."""
    def con_error(self, c: int) -> str | None:
        """
        Why constraint `c` could not be turned into equations; None when it
        could.
        """
    def con_turned(self, c: int) -> bool:
        """
        True for an "offset" constraint whose offset is turned by 90° (the
        point of a perpendicular line).
        """
    def con_contact(self, c: int) -> tuple[float, float, bool] | None:
        """
        Contact point of tangency constraint `c` as (x, y, solved), where
        `solved` is True for a point the solver finds that no entity marks;
        None for other constraints.
        """
    def ent_samples(self, e: int, x0: float, y0: float, x1: float, y1: float, px: float) -> list[list[tuple[float, float]]]:
        """
        World-coordinate polylines drawing entity `e` inside the window from
        (x0, y0) to (x1, y1) with pixels of size `px`, as a list of
        polylines, each a list of (x, y) pairs: one point for a point, the
        endpoints of a segment, a segment crossing the window for an infinite
        line, the outline of a circle, arc or conic (two polylines for a
        hyperbola), and the parts of a function curve inside the window.
        """
    def derive_dim(self, i: int, extra_ms: int = 0, cancel: Cancel | None = None) -> Formula:
        """
        Closed-form formula of dimension `i`. `extra_ms` extends the search
        beyond the default budget; `cancel` stops it early. Other Python
        threads run meanwhile. Raises `sggeo.Error` when no formula is found.
        """
    def derive_var(self, i: int, extra_ms: int = 0, cancel: Cancel | None = None) -> Formula:
        """Closed-form formula of variable `i`, as `derive_dim` derives it."""
    def values(self) -> dict[str, float]:
        """
        The dimensions and value variables by name, with their values: the
        value a driving dimension is set to (radians for angles), the measured
        value of a reference dimension, and the value of a variable (a
        variable defined by an expression is never a symbol of a formula,
        which reads its expression). The symbols of `Formula.sympy` take
        these values:
        `expr.subs({s: values[s.name] for s in expr.free_symbols})`.
        """

@final
class Formula:
    """A derived formula. `str()` gives `plain`; in Jupyter it shows typeset."""
    @property
    def name(self) -> str:
        """Name of the target."""
    @property
    def plain(self) -> str:
        """`name = formula` as plain text."""
    @property
    def latex(self) -> str:
        """`name = formula` as LaTeX."""
    @property
    def latex_short(self) -> str:
        """
        `name = formula` as LaTeX with repeated subexpressions replaced by
        `u_{1}`, `u_{2}`, ..., which `defs` defines.
        """
    @property
    def defs(self) -> list[tuple[str, str]]:
        """
        The subexpressions of `latex_short`, in order, as (LaTeX name, LaTeX
        expression) pairs; an expression can name earlier subexpressions.
        """
    @property
    def exact(self) -> bool:
        """False when some part had to be solved numerically."""
    @property
    def is_angle(self) -> bool:
        """True when the target is an angle (in radians)."""
    @property
    def value(self) -> float:
        """Formula evaluated at the current configuration."""
    @property
    def actual(self) -> float:
        """Measured value of the target."""
    @property
    def size(self) -> int:
        """Complexity of the formula: its number of operations and terms."""
    @property
    def limits(self) -> list[tuple[str, str]]:
        """
        The conditions for the formula to hold that the construction meets
        only just: a radicand of a square root that is 0 here (`b1^2 - a1^2 ≥
        0`), an argument of `acos` or `asin` that is ±1, an argument of `ln`
        that is 0. A little way past such a condition the formula has no value
        or another branch holds. Usually none. Each as (plain text, LaTeX).
        """
    def code(self, lang: str) -> str:
        """
        The formula as source code in language `lang`: "c", "rust", "js",
        "python", "mathematica", "maxima" or "sympy". Statements assign the
        repeated subexpressions of the formula to temporaries `u_1`, `u_2`,
        ... and then the formula to a variable named after the target, one
        per line, reading the other quantities from variables of their names
        (names are made into identifiers of the language, Greek letters
        spelled out). Angles are in radians. C needs `<math.h>`; the Python
        code imports `math` when it uses it. The SymPy code declares the other
        quantities as real symbols, so the target holds a SymPy expression.
        """
    def sympy(self) -> Any:
        """
        The formula (the right side) as a SymPy expression. The quantities
        it reads are real symbols named as sggeo names them, such as
        `Symbol("d1", real=True)` or `Symbol("θ1", real=True)`; angles are in
        radians. A number found numerically is a SymPy Float.
        """
    def _repr_latex_(self) -> str:
        """Typeset display in Jupyter."""
    def __str__(self) -> str:
        """Return str(self)."""
    def __repr__(self) -> str:
        """Return repr(self)."""

@final
class Solved:
    """Result of `Doc.solve`."""
    @property
    def ok(self) -> bool:
        """True when every constraint is satisfied."""
    @property
    def dof(self) -> int:
        """Remaining degrees of freedom."""
    @property
    def errors(self) -> list[str]:
        """Messages for constraints or dimensions that could not be turned into equations."""
    def __bool__(self) -> bool:
        """True if self else False"""
    def __repr__(self) -> str:
        """Return repr(self)."""

@final
class Cancel:
    """A flag that stops a derivation early, from another thread."""
    def __init__(self) -> None:
        ...
    def cancel(self) -> None:
        """Sets the flag: a derivation given it stops soon with an error."""

def latex_png(latex: str, size_pt: float = 12.0, dpi: float = 96.0, rgb: int = 0) -> bytes:
    """
    LaTeX text, such as the `latex` or `latex_short` of a `Formula`, typeset
    in display style as the bytes of a PNG image: glyphs of color `rgb`
    (0xRRGGBB) on a transparent background, at `size_pt` points and `dpi`
    dots per inch (a point is 1/72 inch, so 12 pt at 96 dpi puts 16 pixels to
    the em).
    """

def code_languages() -> list[tuple[str, str]]:
    """
    The languages of `Formula.code`, in the order of their numbers in C: each
    as `(name, label)`, the name `code` takes and the name to show (`("js",
    "JavaScript")`).
    """

def con_kinds() -> list[str]:
    """
    The kinds of constraint by name (as `constrain` and `con_kind` have
    them), in the order of their numbers in C.
    """

def dim_kinds() -> list[str]:
    """
    The kinds of dimension by name (as `add_dim` and `dim_kind` have them),
    in the order of their numbers in C.
    """

def ent_kinds() -> list[str]:
    """
    The kinds of entity by name (as `ent_kind` has them), in the order of
    their numbers in C.
    """

def version() -> str:
    """Version of the library."""

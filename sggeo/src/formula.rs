//! Formula derivation for a dimension or variable of a document, with the
//! result formatted as plain text and LaTeX. Shared by the Rust, C and JS
//! interfaces.

use crate::derive;
use crate::expr::bx::{Bx, latex, plain, to_bx};
use crate::expr::{Expr, sym};
use crate::model::{Doc, Var};
use std::borrow::Cow;
use std::collections::HashMap;

/// First symbol number of the subexpression names `u_1`, `u_2`, … that
/// common subexpression elimination introduces.
pub const CSE_BASE: u32 = 0xF000_0000;

/// Display name of symbol `s`: the parameter name from `names`, `u_k` for a
/// subexpression, or `p<s>`.
pub fn sym_name(names: &HashMap<usize, String>, s: u32) -> String {
    if s >= CSE_BASE {
        return format!("u_{}", s - CSE_BASE + 1);
    }
    names.get(&(s as usize)).cloned().unwrap_or_else(|| format!("p{s}"))
}

/// Quantity whose formula is derived.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    /// Index into `Doc::dims`.
    Dim(usize),
    /// Index into `Doc::vars`.
    Var(usize),
}

/// Name, expression and angle flag of target `t`.
pub fn target_expr(doc: &Doc, t: Target) -> Result<(String, Expr, bool), String> {
    match t {
        Target::Dim(i) => {
            let d = doc.dims.get(i).and_then(|d| d.as_ref()).ok_or("gone")?;
            let e = if d.driving { doc.dim_value_expr(i, 0)? } else { doc.measure(i) };
            Ok((d.name.clone(), e, d.kind.is_angle()))
        }
        Target::Var(i) => {
            let v = doc.vars.get(i).and_then(|v| v.as_ref()).ok_or("gone")?;
            let e = if v.expr.trim().is_empty() { sym(v.param as u32) } else { doc.parse_expr(&v.expr, 0)? };
            Ok((v.name.clone(), e, false))
        }
    }
}

/// The document the formula of target `t` is derived from. Every numeric
/// variable and dimension other than `t` becomes a plain value fixed at its
/// current value, so derivation substitutes it as a number: a defined
/// variable or a dimension given by an expression loses the expression, a
/// solved variable is fixed, and a reference dimension is replaced, for
/// name lookups, by a numeric variable of the same name. `doc` itself when
/// nothing is numeric.
pub fn numeric_doc(doc: &Doc, t: Target) -> Cow<'_, Doc> {
    let var_on = |i: usize, v: &Var| v.numeric && t != Target::Var(i);
    let vars: Vec<(usize, f64)> = doc
        .vars
        .iter()
        .enumerate()
        .filter_map(|(i, v)| Some((i, v.as_ref().filter(|v| var_on(i, v))?)))
        .filter_map(|(i, v)| {
            let val = if v.expr.trim().is_empty() { doc.params[v.param].v } else { doc.eval_now(&doc.parse_expr(&v.expr, 0).ok()?) };
            Some((i, val))
        })
        .collect();
    let dims: Vec<(usize, f64)> = doc
        .dims
        .iter()
        .enumerate()
        .filter_map(|(i, d)| Some((i, d.as_ref().filter(|d| d.numeric && t != Target::Dim(i))?)))
        .filter_map(|(i, d)| {
            let val = if d.driving { doc.eval_now(&doc.dim_value_expr(i, 0).ok()?) } else { doc.measure_value(i) };
            Some((i, val))
        })
        .collect();
    if vars.is_empty() && dims.is_empty() {
        return Cow::Borrowed(doc);
    }
    let mut d = doc.clone();
    for (i, val) in vars {
        let v = d.vars[i].as_mut().unwrap();
        v.expr.clear();
        let p = v.param;
        d.params[p].v = val;
        d.params[p].locked = true;
    }
    for (i, val) in dims {
        let x = d.dims[i].as_mut().unwrap();
        if x.driving {
            x.expr.clear();
            let p = x.param;
            d.params[p].v = val;
        } else {
            let name = std::mem::take(&mut x.name);
            let p = d.add_param(val, true);
            d.vars.push(Some(Var { name, expr: String::new(), param: p, numeric: true }));
        }
    }
    Cow::Owned(d)
}

/// A derived formula.
#[derive(Clone, Debug)]
pub struct Formula {
    /// Name of the target.
    pub name: String,
    /// `name = formula` as plain text.
    pub plain: String,
    /// `name = formula` as LaTeX.
    pub latex: String,
    /// False when some part had to be solved numerically.
    pub exact: bool,
    /// True when the target is an angle (in radians).
    pub is_angle: bool,
    /// Formula evaluated at the current configuration.
    pub value: f64,
    /// Measured value of the target.
    pub actual: f64,
    /// The formula itself.
    pub expr: Expr,
}

/// Derives the formula of target `t`, searching `extra_ms` longer than the
/// default budget (steps of 10 000 ms suit retrying a hard target).
pub fn derive_formula(doc: &Doc, t: Target, extra_ms: u64) -> Result<Formula, String> {
    let doc = numeric_doc(doc, t);
    let (name, e, is_angle) = target_expr(&doc, t)?;
    let d = derive::derive_harder(&doc, &e, extra_ms)?;
    // a measured dimension's own measurement (an angle more exactly than its expression gives it)
    let actual = match t {
        Target::Dim(i) if doc.dims[i].as_ref().is_some_and(|d| !d.driving) => doc.measure_value(i),
        _ => d.actual,
    };
    let names = doc.param_names();
    let full = to_bx(&d.expr, &|s| sym_name(&names, s));
    Ok(Formula {
        plain: format!("{} = {}", name, plain(&full)),
        latex: format!("{} = {}", latex(&Bx::Sym(name.clone())), latex(&full)),
        name,
        exact: d.exact,
        is_angle,
        value: d.value,
        actual,
        expr: d.expr,
    })
}

/// The conditions for formula `e` to hold that the current configuration of
/// `doc` meets only just, as plain text and LaTeX (`b1^2 - a1^2 ≥ 0`): the
/// radicand of an even root that is 0 here, the argument of `acos` or `asin`
/// that is ±1, the argument of `ln` that is 0. Where such a condition fails,
/// a little way off, the formula has no value or another branch holds, as
/// the foci of an ellipse with a = b go onto the other axis when b passes a.
/// A condition that is 0 here but holds all around (a square) is not one.
pub fn limits(doc: &Doc, e: &Expr) -> Vec<(String, String)> {
    use crate::expr::{Fun, Node, add, eval, neg, one, sym_set};
    // each condition as an expression that must not be negative, with how it reads
    fn walk(e: &Expr, out: &mut Vec<(Expr, Expr, &'static str, &'static str)>) {
        match e.node() {
            Node::Pow(b, x) => {
                if x.as_rat().is_some_and(|r| r.d % 2 == 0) {
                    out.push((b.clone(), b.clone(), "≥ 0", "\\geq 0"));
                }
                walk(b, out);
                walk(x, out);
            }
            Node::Fun(f, u) => {
                match f {
                    Fun::Acos | Fun::Asin => {
                        out.push((add(vec![one(), neg(u.clone())]), u.clone(), "≤ 1", "\\leq 1"));
                        out.push((add(vec![u.clone(), one()]), u.clone(), "≥ -1", "\\geq -1"));
                    }
                    Fun::Ln => out.push((u.clone(), u.clone(), "> 0", "> 0")),
                    _ => {}
                }
                walk(u, out);
            }
            Node::Add(v) | Node::Mul(v) => v.iter().for_each(|x| walk(x, out)),
            Node::Atan2(a, b) => {
                walk(a, out);
                walk(b, out);
            }
            Node::Num(_) | Node::Pi | Node::Sym(_) => {}
        }
    }
    // the size of the parts of `e`, which its rounding is relative to
    fn magnitude(e: &Expr, at: &dyn Fn(u32) -> f64) -> f64 {
        match e.node() {
            Node::Add(v) => v.iter().map(|x| magnitude(x, at)).sum(),
            Node::Mul(v) => v.iter().map(|x| magnitude(x, at)).product(),
            _ => eval(e, at).abs(),
        }
    }
    let mut conds = Vec::new();
    walk(e, &mut conds);
    let vals: Vec<f64> = doc.params.iter().map(|p| p.v).collect();
    let here = |s: u32| vals.get(s as usize).copied().unwrap_or(f64::NAN);
    let syms: Vec<u32> = {
        let mut v: Vec<u32> = sym_set(e).into_iter().collect();
        v.sort();
        v
    };
    let size = vals.iter().map(|x| x.abs()).filter(|x| x.is_finite()).fold(1.0, f64::max);
    // the quantities moved by 1% each, up or down by a hash of the quantity and the trial (splitmix64)
    let moved: Vec<Vec<f64>> = (0..16u64)
        .map(|k| {
            let mut v = vals.clone();
            for &s in &syms {
                let mut h = (s as u64) << 8 ^ k;
                h = (h ^ h >> 30).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                h = (h ^ h >> 27).wrapping_mul(0x94d0_49bb_1331_11eb);
                if let Some(x) = v.get_mut(s as usize) {
                    let step = if *x == 0.0 { 1e-2 * size } else { 1e-2 * x.abs() };
                    *x += if (h ^ h >> 31) & 1 == 0 { step } else { -step };
                }
            }
            v
        })
        .collect();
    let names = doc.param_names();
    let nm = |s: u32| sym_name(&names, s);
    let mut out: Vec<(String, String)> = Vec::new();
    for (g, shown, rel_plain, rel_latex) in conds {
        let (v, m) = (eval(&g, &here), magnitude(&g, &here));
        if !(v.is_finite() && v.abs() <= 1e-9 * m.max(f64::MIN_POSITIVE)) {
            continue;
        }
        let fails = moved.iter().any(|w| {
            let at = |s: u32| w.get(s as usize).copied().unwrap_or(f64::NAN);
            eval(&g, &at) < -1e-9 * magnitude(&g, &at)
        });
        let bx = to_bx(&shown, &nm);
        let item = (format!("{} {rel_plain}", plain(&bx)), format!("{} {rel_latex}", latex(&bx)));
        if fails && !out.contains(&item) {
            out.push(item);
        }
    }
    out
}

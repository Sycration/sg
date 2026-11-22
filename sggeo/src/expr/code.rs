//! Conversion of expressions into source code for programming languages and
//! computer algebra systems.

use super::bx::{Names, sorted_terms};
use super::*;

/// Language a formula is written in by [`program`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    /// C99 with `<math.h>`.
    C,
    /// Rust, on `f64`.
    Rust,
    /// JavaScript.
    Js,
    /// Python 3 with the `math` module.
    Python,
    /// Wolfram Mathematica.
    Mathematica,
    /// Maxima.
    Maxima,
    /// Python 3 with SymPy: a symbolic expression in symbols for the other
    /// quantities.
    Sympy,
}

impl Lang {
    pub const ALL: [Lang; 7] = [Lang::C, Lang::Rust, Lang::Js, Lang::Python, Lang::Mathematica, Lang::Maxima, Lang::Sympy];

    pub fn name(self) -> &'static str {
        match self {
            Lang::C => "c",
            Lang::Rust => "rust",
            Lang::Js => "js",
            Lang::Python => "python",
            Lang::Mathematica => "mathematica",
            Lang::Maxima => "maxima",
            Lang::Sympy => "sympy",
        }
    }

    /// The language's name to show: "JavaScript" for `js`.
    pub fn label(self) -> &'static str {
        match self {
            Lang::C => "C",
            Lang::Rust => "Rust",
            Lang::Js => "JavaScript",
            Lang::Python => "Python",
            Lang::Mathematica => "Mathematica",
            Lang::Maxima => "Maxima",
            Lang::Sympy => "SymPy",
        }
    }

    pub fn from_name(s: &str) -> Result<Lang, String> {
        Lang::ALL.into_iter().find(|l| l.name() == s).ok_or_else(|| format!("unknown language {s:?}"))
    }

    pub fn from_index(i: u32) -> Result<Lang, String> {
        Lang::ALL.get(i as usize).copied().ok_or_else(|| format!("unknown language {i}"))
    }

    /// Integer literals are not floating point.
    fn typed(self) -> bool {
        matches!(self, Lang::C | Lang::Rust)
    }

    /// Words a name may not take, for keywords and for the names the
    /// emitted code relies on.
    fn reserved(self) -> &'static [&'static str] {
        match self {
            Lang::C => &[
                "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float", "for",
                "goto", "if", "inline", "int", "long", "register", "restrict", "return", "short", "signed", "sizeof", "static", "struct",
                "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "sin", "cos", "tan", "asin", "acos", "atan",
                "atan2", "sinh", "cosh", "tanh", "exp", "log", "fabs", "sqrt", "pow",
            ],
            Lang::Rust => &[
                "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let",
                "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
                "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final",
                "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen", "std",
            ],
            Lang::Js => &[
                "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete", "do", "else", "enum", "export",
                "extends", "false", "finally", "for", "function", "if", "implements", "import", "in", "instanceof", "interface", "let",
                "new", "null", "package", "private", "protected", "public", "return", "static", "super", "switch", "this", "throw",
                "true", "try", "typeof", "var", "void", "while", "with", "yield", "await", "arguments", "eval", "undefined", "NaN",
                "Infinity", "Math",
            ],
            Lang::Python => &[
                "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif",
                "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or",
                "pass", "raise", "return", "try", "while", "with", "yield", "math", "abs",
            ],
            Lang::Sympy => &[
                "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif",
                "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or",
                "pass", "raise", "return", "try", "while", "with", "yield", "sympy",
            ],
            Lang::Mathematica => &[
                "C", "D", "E", "I", "K", "N", "O", "Pi", "Degree", "Infinity", "Indeterminate", "True", "False", "Null", "Gamma", "Beta",
                "Zeta", "Re", "Im", "Arg", "Sign", "Abs", "Sqrt", "Exp", "Log", "Sin", "Cos", "Tan", "Max", "Min", "Mod", "Power",
                "Plus", "Times", "Det", "Dot", "Cross", "Norm", "Area", "Length", "Line", "Point", "Circle", "Disk", "Angle",
            ],
            Lang::Maxima => &[
                "and", "or", "not", "if", "then", "else", "elseif", "do", "for", "while", "thru", "step", "unless", "from", "in", "next",
                "true", "false", "inf", "minf", "infinity", "und", "ind",
            ],
        }
    }
}

const P_ADD: u8 = 1;
const P_MUL: u8 = 2;
const P_NEG: u8 = 3;
const P_POW: u8 = 4;
const P_ATOM: u8 = 5;

fn paren(s: String, need: bool) -> String {
    if need { format!("({s})") } else { s }
}

fn greek(c: char) -> Option<&'static str> {
    Some(match c {
        'α' => "alpha", 'β' => "beta", 'γ' => "gamma", 'δ' => "delta", 'ε' | 'ϵ' => "epsilon", 'ζ' => "zeta", 'η' => "eta",
        'θ' | 'ϑ' => "theta", 'ι' => "iota", 'κ' => "kappa", 'λ' => "lambda", 'μ' => "mu", 'ν' => "nu", 'ξ' => "xi",
        'π' => "pi", 'ρ' => "rho", 'σ' => "sigma", 'τ' => "tau", 'υ' => "upsilon", 'φ' | 'ϕ' => "phi", 'χ' => "chi",
        'ψ' => "psi", 'ω' => "omega", 'Γ' => "Gamma", 'Δ' => "Delta", 'Θ' => "Theta", 'Λ' => "Lambda", 'Ξ' => "Xi",
        'Π' => "Pi", 'Σ' => "Sigma", 'Φ' => "Phi", 'Ψ' => "Psi", 'Ω' => "Omega",
        _ => return None,
    })
}

/// An identifier for display name `name`: Greek letters spelled out (kept
/// in Mathematica, which has no `_` in names either), other characters
/// outside ASCII letters and digits dropped to `_`, and a reserved word
/// suffixed.
fn ident(name: &str, lang: Lang) -> String {
    let wl = lang == Lang::Mathematica;
    let mut s = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || wl && c.is_alphabetic() && c != 'π' {
            s.push(c);
        } else if let Some(g) = greek(c) {
            s.push_str(g);
        } else if !wl {
            s.push('_');
        }
    }
    if s.chars().all(|c| c == '_') || s.starts_with(|c: char| c.is_ascii_digit()) {
        s.insert(0, 'v');
    }
    if lang.reserved().contains(&s.as_str()) {
        s.push(if wl { '$' } else { '_' });
    }
    s
}

struct Emit<'a> {
    lang: Lang,
    names: &'a HashMap<u32, String>,
}

impl Emit<'_> {
    fn int(&self, n: i128) -> String {
        if self.lang.typed() { format!("{n}.0") } else { n.to_string() }
    }

    fn float(&self, f: f64) -> String {
        let s = format!("{f:?}");
        match self.lang {
            Lang::Mathematica => s.replace('e', "*^"),
            Lang::C | Lang::Rust => s,
            // the shortest form, without a needless `.0`
            _ => s.strip_suffix(".0").map(str::to_string).unwrap_or(s),
        }
    }

    fn num(&self, n: Num) -> (String, u8) {
        match n {
            Num::R(r) if r.d == 1 => (self.int(r.n), P_ATOM),
            // `2 / 3` is a float in Python
            Num::R(r) if self.lang == Lang::Sympy => (format!("sympy.Rational({}, {})", r.n, r.d), P_ATOM),
            Num::R(r) => (format!("{} / {}", self.int(r.n), self.int(r.d)), P_MUL),
            Num::F(f) => (self.float(f), P_ATOM),
        }
    }

    fn one(&self) -> String {
        self.int(1)
    }

    fn expr(&self, e: &Expr) -> (String, u8) {
        if e.is_negative_form() {
            let (s, l) = self.expr(&neg(e.clone()));
            return (format!("-{}", self.neg_operand(s, l)), P_NEG);
        }
        match e.node() {
            Node::Num(n) => self.num(*n),
            Node::Pi => (
                match self.lang {
                    Lang::C => "3.14159265358979323846",
                    Lang::Rust => "std::f64::consts::PI",
                    Lang::Js => "Math.PI",
                    Lang::Python => "math.pi",
                    Lang::Mathematica => "Pi",
                    Lang::Maxima => "%pi",
                    Lang::Sympy => "sympy.pi",
                }
                .into(),
                P_ATOM,
            ),
            Node::Sym(s) => (self.names[s].clone(), P_ATOM),
            Node::Add(ts) => {
                let mut out = String::new();
                for (i, t) in sorted_terms(ts).iter().enumerate() {
                    let minus = t.is_negative_form();
                    let (s, l) = self.expr(&if minus { neg(t.clone()) } else { t.clone() });
                    match (i, minus) {
                        (0, true) => out += &format!("-{}", self.neg_operand(s, l)),
                        (0, false) => out += &s,
                        (_, true) => out += &format!(" - {}", paren(s, l <= P_ADD)),
                        (_, false) => out += &format!(" + {}", paren(s, l <= P_ADD)),
                    }
                }
                (out, P_ADD)
            }
            Node::Mul(fs) => self.product(fs),
            Node::Pow(b, x) => self.power(b, x),
            Node::Fun(f, a) => {
                let name = match (self.lang, f) {
                    (Lang::C, Fun::Abs) => "fabs",
                    (Lang::C | Lang::Js | Lang::Python | Lang::Maxima, Fun::Ln) => "log",
                    (Lang::Python, Fun::Abs) => "abs",
                    (Lang::Sympy, Fun::Abs) => "Abs",
                    (Lang::Sympy, Fun::Ln) => "log",
                    (Lang::Mathematica, _) => match f {
                        Fun::Sin => "Sin",
                        Fun::Cos => "Cos",
                        Fun::Tan => "Tan",
                        Fun::Asin => "ArcSin",
                        Fun::Acos => "ArcCos",
                        Fun::Atan => "ArcTan",
                        Fun::Sinh => "Sinh",
                        Fun::Cosh => "Cosh",
                        Fun::Tanh => "Tanh",
                        Fun::Exp => "Exp",
                        Fun::Ln => "Log",
                        Fun::Abs => "Abs",
                    },
                    _ => f.name(),
                };
                let lib = !(self.lang == Lang::Python && *f == Fun::Abs);
                (self.call(name, a, &[], lib), P_ATOM)
            }
            Node::Atan2(y, x) => {
                let s = match self.lang {
                    Lang::Mathematica => self.call("ArcTan", x, &[y], true),
                    _ => self.call("atan2", y, &[x], true),
                };
                (s, P_ATOM)
            }
        }
    }

    /// The operand of a unary minus, which JS does not allow to be a power.
    fn neg_operand(&self, s: String, l: u8) -> String {
        paren(s, l <= P_NEG || self.lang == Lang::Js && l == P_POW)
    }

    /// Call of function `name` from the language's math library (`lib`) on
    /// `a` and `rest`: a method of `a` in Rust.
    fn call(&self, name: &str, a: &Expr, rest: &[&Expr], lib: bool) -> String {
        let args = |v: &[&Expr]| v.iter().map(|e| self.expr(e).0).collect::<Vec<_>>().join(", ");
        let all: Vec<&Expr> = std::iter::once(a).chain(rest.iter().copied()).collect();
        match self.lang {
            Lang::Rust => self.method(a, name, &args(rest)),
            Lang::Js if lib => format!("Math.{name}({})", args(&all)),
            Lang::Python if lib => format!("math.{name}({})", args(&all)),
            Lang::Sympy => format!("sympy.{name}({})", args(&all)),
            Lang::Mathematica => format!("{name}[{}]", args(&all)),
            _ => format!("{name}({})", args(&all)),
        }
    }

    /// Rust call of `f64` method `name` on `a` with arguments `rest`. A
    /// constant has no type to call a method on, so it is passed to the
    /// method as a function: `f64::sqrt(2.0)`.
    fn method(&self, a: &Expr, name: &str, rest: &str) -> String {
        let (s, l) = self.expr(a);
        if sym_set(a).is_empty() {
            let sep = if rest.is_empty() { "" } else { ", " };
            format!("f64::{name}({s}{sep}{rest})")
        } else {
            format!("{}.{name}({rest})", paren(s, l < P_ATOM))
        }
    }

    fn product(&self, fs: &[Expr]) -> (String, u8) {
        let mut top: Vec<(String, u8)> = Vec::new();
        let mut bot: Vec<(String, u8)> = Vec::new();
        for f in fs {
            match f.as_num() {
                Some(Num::R(r)) => {
                    if r.n != 1 {
                        top.push((self.int(r.n), P_ATOM));
                    }
                    if r.d != 1 {
                        bot.push((self.int(r.d), P_ATOM));
                    }
                }
                Some(Num::F(x)) => top.push((self.float(x), P_ATOM)),
                None => match f.node() {
                    Node::Pow(b, x) if x.is_negative_form() => bot.push(self.power(b, &neg(x.clone()))),
                    _ => top.push(self.expr(f)),
                },
            }
        }
        let join = |v: Vec<(String, u8)>| -> (String, u8) {
            if v.len() == 1 {
                return v.into_iter().next().unwrap();
            }
            (v.into_iter().map(|(s, l)| paren(s, l <= P_NEG)).collect::<Vec<_>>().join(" * "), P_MUL)
        };
        // a quotient of plain numbers would be a float in Python
        let plain = |v: &[(String, u8)]| v.iter().all(|(s, _)| s.parse::<i128>().is_ok());
        if self.lang == Lang::Sympy && !bot.is_empty() && plain(&top) && plain(&bot) {
            let t = top.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>().join(" * ");
            let b = bot.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>().join(" * ");
            let t = if t.is_empty() { "1".to_string() } else { t };
            return (format!("sympy.Rational({t}, {b})"), P_ATOM);
        }
        let (t, tl) = if top.is_empty() { (self.one(), P_ATOM) } else { join(top) };
        if bot.is_empty() {
            return (t, tl);
        }
        let (b, bl) = join(bot);
        (format!("{} / {}", paren(t, tl <= P_NEG), paren(b, bl <= P_MUL)), P_MUL)
    }

    fn power(&self, b: &Expr, x: &Expr) -> (String, u8) {
        if x.is_negative_form() {
            let (s, l) = self.power(b, &neg(x.clone()));
            return (format!("{} / {}", self.one(), paren(s, l <= P_MUL)), P_MUL);
        }
        let r = x.as_rat();
        if r.is_some_and(|r| r.n == 1 && r.d == 1) {
            return self.expr(b);
        }
        if r.is_some_and(|r| r.n == 1 && r.d == 2) {
            let name = if self.lang == Lang::Mathematica { "Sqrt" } else { "sqrt" };
            return (self.call(name, b, &[], true), P_ATOM);
        }
        // an odd root of a negative number is the real root, as sggeo evaluates it; Maxima takes it so
        if let Some(r) = r.filter(|r| r.d % 2 == 1 && r.d != 1 && self.lang != Lang::Maxima) {
            let root = self.real_root(b, r.d);
            return if r.n == 1 { root } else { self.int_power(root, r.n) };
        }
        let int = r.filter(|r| r.d == 1).map(|r| r.n);
        let infix = |op: &str| {
            let (bs, bl) = self.expr(b);
            let (xs, xl) = self.expr(x);
            (format!("{}{op}{}", paren(bs, bl <= P_POW), paren(xs, xl < P_ATOM)), P_POW)
        };
        match self.lang {
            Lang::C => (format!("pow({}, {})", self.expr(b).0, self.expr(x).0), P_ATOM),
            Lang::Rust => match int {
                Some(k) => (self.method(b, "powi", &k.to_string()), P_ATOM),
                None => (self.method(b, "powf", &self.expr(x).0), P_ATOM),
            },
            Lang::Js => infix(" ** "),
            // `**` takes a negative base to a complex number for a fractional exponent
            Lang::Python if int.is_none() => (self.call("pow", b, &[x], true), P_ATOM),
            Lang::Python | Lang::Sympy => infix("**"),
            Lang::Mathematica | Lang::Maxima => infix("^"),
        }
    }

    /// The real `d`-th root of `b`, for odd `d`: negative for a negative `b`.
    fn real_root(&self, b: &Expr, d: i128) -> (String, u8) {
        let (bs, bl) = self.expr(b);
        let inv = format!("{} / {}", self.int(1), self.int(d));
        match self.lang {
            Lang::C if d == 3 => (format!("cbrt({bs})"), P_ATOM),
            Lang::C => (format!("copysign(pow(fabs({bs}), {inv}), {bs})"), P_ATOM),
            Lang::Rust if d == 3 => (self.method(b, "cbrt", ""), P_ATOM),
            Lang::Rust => (format!("{}.powf({inv}).copysign({bs})", self.method(b, "abs", "")), P_ATOM),
            Lang::Js if d == 3 => (format!("Math.cbrt({bs})"), P_ATOM),
            Lang::Js => (format!("Math.sign({bs}) * Math.abs({bs}) ** ({inv})"), P_MUL),
            // math.cbrt is only in Python 3.11 and later
            Lang::Python => (format!("math.copysign(abs({bs}) ** ({inv}), {bs})"), P_ATOM),
            Lang::Mathematica => (format!("Surd[{bs}, {d}]"), P_ATOM),
            Lang::Sympy => (format!("sympy.real_root({bs}, {d})"), P_ATOM),
            Lang::Maxima => (format!("{}^({inv})", paren(bs, bl <= P_POW)), P_POW),
        }
    }

    /// `(s, l)` to the integer power `n`.
    fn int_power(&self, (s, l): (String, u8), n: i128) -> (String, u8) {
        match self.lang {
            Lang::C => (format!("pow({s}, {})", self.int(n)), P_ATOM),
            Lang::Rust => (format!("{}.powi({n})", paren(s, l < P_ATOM)), P_ATOM),
            Lang::Js => (format!("{} ** {n}", paren(s, l <= P_POW)), P_POW),
            Lang::Python | Lang::Sympy => (format!("{}**{n}", paren(s, l <= P_POW)), P_POW),
            Lang::Mathematica | Lang::Maxima => (format!("{}^{n}", paren(s, l <= P_POW)), P_POW),
        }
    }
}

/// Source code in `lang` that assigns the subexpressions `defs` in order
/// and then `main` to a variable named after `lhs`, one statement per line.
/// Symbols are named after `names`, made into identifiers of the language.
pub fn program(lang: Lang, lhs: &str, main: &Expr, defs: &[(u32, Expr)], names: Names) -> String {
    let mut syms: Vec<u32> = sym_set(main).into_iter().collect();
    for (s, d) in defs {
        syms.push(*s);
        syms.extend(sym_set(d));
    }
    syms.sort();
    syms.dedup();
    let mut used: HashSet<String> = HashSet::new();
    let mut unique = |name: &str| {
        let base = ident(name, lang);
        let mut s = base.clone();
        let mut k = 2;
        while used.contains(&s) {
            s = if lang == Lang::Mathematica { format!("{base}{k}") } else { format!("{base}_{k}") };
            k += 1;
        }
        used.insert(s.clone());
        s
    };
    let target = unique(lhs);
    let map: HashMap<u32, String> = syms.iter().map(|&s| (s, unique(&names(s)))).collect();
    let em = Emit { lang, names: &map };
    let rows: Vec<(String, String)> =
        defs.iter().map(|(s, d)| (map[s].clone(), em.expr(d).0)).chain([(target, em.expr(main).0)]).collect();
    let n = rows.len();
    let mut out: Vec<String> = rows
        .into_iter()
        .enumerate()
        .map(|(i, (v, e))| match lang {
            Lang::C => format!("double {v} = {e};"),
            Lang::Rust => format!("let {v}: f64 = {e};"),
            Lang::Js => format!("const {v} = {e};"),
            Lang::Python => format!("{v} = {e}"),
            // the last value shows as the result
            Lang::Mathematica if i + 1 == n => format!("{v} = {e}"),
            Lang::Mathematica => format!("{v} = {e};"),
            Lang::Maxima => format!("{v} : {e};"),
            Lang::Sympy => format!("{v} = {e}"),
        })
        .collect();
    if lang == Lang::Python && out.iter().any(|l| l.contains("math.")) {
        out.insert(0, "import math".into());
    }
    if lang == Lang::Sympy {
        // the quantities the formula reads are real symbols of their names
        let given: Vec<u32> = syms.into_iter().filter(|s| !defs.iter().any(|(d, _)| d == s)).collect();
        let vars = given.iter().map(|s| map[s].as_str()).collect::<Vec<_>>().join(", ");
        let quoted = |s: String| s.replace('\\', "\\\\").replace('"', "\\\"");
        match given[..] {
            [] => {}
            [s] => out.insert(0, format!("{vars} = sympy.Symbol(\"{}\", real=True)", quoted(names(s)))),
            _ => {
                let all = given.iter().map(|&s| quoted(names(s))).collect::<Vec<_>>().join(" ");
                out.insert(0, format!("{vars} = sympy.symbols(\"{all}\", real=True)"));
            }
        }
        out.insert(0, "import sympy".into());
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(s: u32) -> String {
        ["a", "b", "θ", "x_A", "in", "E"].get(s as usize).map(|s| s.to_string()).unwrap_or(format!("u_{s}"))
    }

    fn code(lang: Lang, e: &Expr) -> String {
        program(lang, "d", e, &[], &names)
    }

    #[test]
    fn law_of_cosines() {
        // sqrt(a^2 + b^2 - 2 a b cos θ)
        let e = pow(
            add(vec![
                pow(sym(0), int(2)),
                pow(sym(1), int(2)),
                mul(vec![int(-2), sym(0), sym(1), fun(Fun::Cos, sym(2))]),
            ]),
            num(Num::R(Rat::new(1, 2).unwrap())),
        );
        assert_eq!(code(Lang::C, &e), "double d = sqrt(pow(a, 2.0) + pow(b, 2.0) - 2.0 * a * b * cos(theta));");
        assert_eq!(code(Lang::Rust, &e), "let d: f64 = (a.powi(2) + b.powi(2) - 2.0 * a * b * theta.cos()).sqrt();");
        assert_eq!(code(Lang::Js, &e), "const d = Math.sqrt(a ** 2 + b ** 2 - 2 * a * b * Math.cos(theta));");
        assert_eq!(code(Lang::Python, &e), "import math\nd = math.sqrt(a**2 + b**2 - 2 * a * b * math.cos(theta))");
        assert_eq!(code(Lang::Mathematica, &e), "d = Sqrt[a^2 + b^2 - 2 * a * b * Cos[θ]]");
        assert_eq!(code(Lang::Maxima, &e), "d : sqrt(a^2 + b^2 - 2 * a * b * cos(theta));");
        assert_eq!(
            code(Lang::Sympy, &e),
            "import sympy\na, b, theta = sympy.symbols(\"a b θ\", real=True)\nd = sympy.sqrt(a**2 + b**2 - 2 * a * b * sympy.cos(theta))"
        );
    }

    #[test]
    fn quotients_and_signs() {
        // -a^2 / (3 b) and atan2(x_A, in)
        let e = mul(vec![num(Num::R(Rat::new(-1, 3).unwrap())), pow(sym(0), int(2)), pow(sym(1), int(-1))]);
        assert_eq!(code(Lang::C, &e), "double d = -(pow(a, 2.0) / (3.0 * b));");
        assert_eq!(code(Lang::Js, &e), "const d = -(a ** 2 / (3 * b));");
        assert_eq!(code(Lang::Maxima, &e), "d : -(a^2 / (3 * b));");
        let e = atan2(sym(3), sym(4));
        assert_eq!(code(Lang::Python, &e), "import math\nd = math.atan2(x_A, in_)");
        assert_eq!(code(Lang::Mathematica, &e), "d = ArcTan[in, xA]");
        assert_eq!(code(Lang::Rust, &e), "let d: f64 = x_A.atan2(in_);");
        // -(a^2) in JS needs the parentheses
        assert_eq!(code(Lang::Js, &neg(pow(sym(0), int(2)))), "const d = -(a ** 2);");
        assert_eq!(code(Lang::Python, &neg(pow(sym(0), int(2)))), "d = -a**2");
        assert_eq!(code(Lang::Sympy, &atan2(sym(3), sym(4))), "import sympy\nx_A, in_ = sympy.symbols(\"x_A in\", real=True)\nd = sympy.atan2(x_A, in_)");
    }

    #[test]
    fn odd_roots_are_real() {
        // a^(1/5) + b^(2/3) - (a + b)^(3/5), real for negative a and b, and b^(3/2), which is not
        let r = |n, d| num(Num::R(Rat::new(n, d).unwrap()));
        let e = add(vec![pow(sym(0), r(1, 5)), pow(sym(1), r(2, 3)), neg(pow(add(vec![sym(0), sym(1)]), r(3, 5))), pow(sym(1), r(3, 2))]);
        let want = [
            (
                Lang::C,
                "double d = pow(b, 3.0 / 2.0) + pow(cbrt(b), 2.0) + copysign(pow(fabs(a), 1.0 / 5.0), a) \
                 - pow(copysign(pow(fabs(a + b), 1.0 / 5.0), a + b), 3.0);",
            ),
            (
                Lang::Rust,
                "let d: f64 = b.powf(3.0 / 2.0) + b.cbrt().powi(2) + a.abs().powf(1.0 / 5.0).copysign(a) \
                 - (a + b).abs().powf(1.0 / 5.0).copysign(a + b).powi(3);",
            ),
            (
                Lang::Js,
                "const d = b ** (3 / 2) + Math.cbrt(b) ** 2 + Math.sign(a) * Math.abs(a) ** (1 / 5) \
                 - (Math.sign(a + b) * Math.abs(a + b) ** (1 / 5)) ** 3;",
            ),
            (
                Lang::Python,
                "import math\nd = math.pow(b, 3 / 2) + math.copysign(abs(b) ** (1 / 3), b)**2 + math.copysign(abs(a) ** (1 / 5), a) \
                 - math.copysign(abs(a + b) ** (1 / 5), a + b)**3",
            ),
            (Lang::Mathematica, "d = b^(3 / 2) + Surd[b, 3]^2 + Surd[a, 5] - Surd[a + b, 5]^3"),
            (Lang::Maxima, "d : b^(3 / 2) + b^(2 / 3) + a^(1 / 5) - (a + b)^(3 / 5);"),
        ];
        for (lang, want) in want {
            assert_eq!(code(lang, &e), want);
        }
    }

    #[test]
    fn sympy_numbers() {
        let r = |n, d| num(Num::R(Rat::new(n, d).unwrap()));
        // fractions are exact; a quotient of integers would be a float in Python
        let e = mul(vec![r(-1, 3), pow(sym(0), int(2)), pow(sym(1), int(-1))]);
        assert_eq!(code(Lang::Sympy, &e), "import sympy\na, b = sympy.symbols(\"a b\", real=True)\nd = -(a**2 / (3 * b))");
        assert_eq!(code(Lang::Sympy, &fun(Fun::Acos, r(-5, 6))), "import sympy\nd = sympy.acos(-sympy.Rational(5, 6))");
        assert_eq!(code(Lang::Sympy, &mul(vec![r(2, 3), pi()])), "import sympy\nd = (2 * sympy.pi) / 3");
        // odd roots are real roots, other powers stay powers
        let e = add(vec![pow(sym(0), r(2, 3)), pow(sym(1), r(3, 2)), pow(sym(2), r(1, 5)), fun(Fun::Ln, fun(Fun::Abs, sym(5)))]);
        assert_eq!(
            code(Lang::Sympy, &e),
            "import sympy\na, b, theta, E = sympy.symbols(\"a b θ E\", real=True)\nd = b**sympy.Rational(3, 2) + sympy.log(sympy.Abs(E)) + sympy.real_root(a, 3)**2 + sympy.real_root(theta, 5)"
        );
        // one symbol, and temporaries that are not symbols
        let defs = [(6, add(vec![sym(0), int(1)]))];
        assert_eq!(
            program(Lang::Sympy, "d", &mul(vec![sym(6), sym(6)]), &defs, &names),
            "import sympy\na = sympy.Symbol(\"a\", real=True)\nu_6 = a + 1\nd = u_6**2"
        );
    }

    #[test]
    fn literals_and_names() {
        // sqrt(2) * E^(2/3) * pi
        let e = mul(vec![pow(int(2), num(Num::R(Rat::new(1, 2).unwrap()))), pow(sym(5), num(Num::R(Rat::new(2, 3).unwrap()))), pi()]);
        assert_eq!(code(Lang::Rust, &e), "let d: f64 = f64::sqrt(2.0) * std::f64::consts::PI * E.cbrt().powi(2);");
        assert_eq!(code(Lang::Python, &e), "import math\nd = math.sqrt(2) * math.pi * math.copysign(abs(E) ** (1 / 3), E)**2");
        assert_eq!(code(Lang::Mathematica, &e), "d = Sqrt[2] * Pi * Surd[E$, 3]^2");
        assert_eq!(code(Lang::Maxima, &e), "d : sqrt(2) * %pi * E^(2 / 3);");
        let e = mul(vec![num(Num::F(1.5e-7)), sym(0)]);
        assert_eq!(code(Lang::Mathematica, &e), "d = 1.5*^-7 * a");
        assert_eq!(code(Lang::Js, &e), "const d = 1.5e-7 * a;");
        // a constant has no type to call a method on in Rust
        let e = fun(Fun::Acos, num(Num::R(Rat::new(-5, 6).unwrap())));
        assert_eq!(code(Lang::Rust, &e), "let d: f64 = f64::acos(-(5.0 / 6.0));");
        // a name already taken is numbered
        let tail = program(Lang::C, "a", &sym(0), &[], &names);
        assert_eq!(tail, "double a = a_2;");
    }
}

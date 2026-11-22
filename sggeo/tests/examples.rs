//! The example constructions the generated tests save in `test_examples/`:
//! they load and solve, their formulas match their measurements, and the
//! formulas known to be short stay short.

mod testkit;

use sggeo::derive;
use sggeo::ops;
use sggeo::expr::bx::{plain, to_bx};
use sggeo::formula::{Target, sym_name, target_expr};
use sggeo::sketch::Sketch;
use testkit::EXAMPLES_DIR;

/// The example file at `path`, opened and solved.
fn open(path: &std::path::Path) -> Sketch {
    let text = std::fs::read_to_string(path).unwrap();
    let doc = sggeo::sgp::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut sk = Sketch::from_doc(doc);
    sk.solve_full();
    sk
}

/// Every example file written by earlier test runs opens and solves without
/// errors.
#[test]
fn example_files_load() {
    let Ok(groups) = std::fs::read_dir(EXAMPLES_DIR) else { return };
    for group in groups.flatten() {
        let Ok(files) = std::fs::read_dir(group.path()) else { continue };
        for f in files.flatten() {
            let path = f.path();
            if path.extension().is_none_or(|e| e != "sgp") {
                continue;
            }
            let sk = open(&path);
            assert!(sk.errors.is_empty(), "{}: errors {:?}", path.display(), sk.errors);
        }
    }
}

/// Formula lengths. Each entry: an example file written by the generated
/// tests, and the most characters any of its reference dimensions' formulas
/// may have.
#[test]
fn known_short_formulas_stay_short() {
    let cases: &[(&str, usize)] = &[
        ("gen_angles/polar_walk_large_turning", 260),
        ("gen_angles/quadrilateral_angle_sum", 30),
        ("gen_angles/regular_pentagon_turning_angles", 130),
        ("gen_circle_tangency/wedge_chain_of_three", 90),
        ("gen_circle_tangency/descartes_inner_circle", 180),
        ("gen_constructions/napoleon_triangle", 150),
        ("gen_constructions/two_reflections_rotation_distance", 30),
        ("gen_constructions/bisector_reflection_on_side", 20),
        ("gen_invariants/ptolemy_diagonals", 80),
        ("gen_linkages/regular_pentagon_closing_side", 40),
        ("gen_linkages/hinged_triangles_median", 40),
        ("gen_triangle_centers/incenter_distance_to_sides_floating", 90),
        ("gen_variables/pentagon_golden_ratio", 120),
        // a polynomial part divided out of a quotient (`divide_out`): OH² = 9R² − (a² + b² + c²)
        ("gen_triangle_centers/euler_line_og_gh", 130),
        ("gen_constructions/nine_point_center", 130),
        ("gen_constructions/orthocenter_distance", 130),
        ("gen_functions/quadratic_fit_solved_coefficients", 240),
        // nested roots, short in the normal form of their roots (`nf`)
        ("gen_constructions/simson_feet_distance", 330),
        ("gen_constructions/fermat_point", 420),
        ("gen_constructions/pedal_triangle_sides", 380),
        ("gen_constructions/varignon_other_diagonal", 300),
        ("gen_conics/confocal_orthogonal", 300),
        ("gen_linkages/four_bar_floating_distances", 370),
        // joints on two circles around computed joints (`solve_circle_circle`)
        ("gen_linkages/four_bar_coupler_triangle_point", 950),
        ("gen_linkages/peaucellier_inversor_rotating", 2600),
    ];
    derive::set_time_scale(1.0);
    for (file, limit) in cases {
        let path = std::path::Path::new(EXAMPLES_DIR).join(format!("{file}.sgp"));
        if !path.exists() {
            eprintln!("skipped (no example file yet): {file}");
            continue;
        }
        let sk = open(&path);
        let names = sk.doc.param_names();
        for i in 0..sk.doc.dims.len() {
            let Some(d) = sk.doc.dims[i].as_ref() else { continue };
            if d.driving {
                continue;
            }
            let (name, target, _) = target_expr(&sk.doc, Target::Dim(i)).expect("bad target");
            let der = derive::derive(&sk.doc, &target).unwrap();
            let text = plain(&to_bx(&der.expr, &|s| sym_name(&names, s)));
            assert!(der.exact, "{file}: {name} is not exact");
            assert!(text.len() <= *limit, "{file}: {name} has {} characters (limit {limit}): {text}", text.len());
            // the same derivation again gives the same formula
            let again = derive::derive(&sk.doc, &target).unwrap();
            assert_eq!(text, plain(&to_bx(&again.expr, &|s| sym_name(&names, s))), "{file}: {name} differs between two derivations");
        }
    }
}

/// Derives every reference dimension of the construction file named by
/// SGGEO_DERIVE_FILE, for debugging a formula that was not found
/// (SGGEO_DERIVE_LEVEL: extra search in steps of 10 s, default 0).
#[test]
#[ignore]
fn derive_file() {
    let Ok(path) = std::env::var("SGGEO_DERIVE_FILE") else { return };
    let sk = open(std::path::Path::new(&path));
    println!("dof {}, conflicts {:?}, errors {:?}", sk.dof, sk.conflicts, sk.errors);
    let level: u64 = std::env::var("SGGEO_DERIVE_LEVEL").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    for i in 0..sk.doc.dims.len() {
        let Some(d) = sk.doc.dims[i].as_ref() else { continue };
        if d.driving {
            continue;
        }
        let (name, target, _) = target_expr(&sk.doc, Target::Dim(i)).expect("bad target");
        let t0 = std::time::Instant::now();
        match derive::derive_harder(&sk.doc, &target, level * 10_000) {
            Ok(der) => println!(
                "{name} = {}   [exact {}, value {}, measured {}, {:?}]",
                plain(&to_bx(&der.expr, &|s| format!("s{s}"))),
                der.exact,
                der.value,
                der.actual,
                t0.elapsed()
            ),
            Err(e) => println!("{name}: error {e}"),
        }
    }
}

/// The ellipse of this file (as an earlier solver wrote it) has a = b = 3
/// only to within rounding, which put its foci 7.6e-7 from the center, so the
/// distances to them measured 3 ± 7e-7 and no exact formula matched them.
const ROUNDED_DEGENERATE_ELLIPSE: &str = "sgp 2\n\ngiven\n  e1 = ellipse(O)\n    a1: a(e1) = 3\n    b1: b(e1) = 3\n  point B\n\nconstruct\n  A = focus(e1)\n    O == mid(A, B)\n  point C on e1\n    x_C: x(C) = 1\n\nfind\n  d1: dist(C, A)\n  d2: dist(C, B)\n\nwhere\n  e1 a 2.9999999999999534 b 3.0000000000000466 angle 0° [fixed angle]\n  B (-3.2653365231889804e-41, 7.582899409302634e-7)\n  A (3.265336523188393e-41, -7.582899409302567e-7)\n  C (1, 2.8284271247462285)\n";

/// A file saved a rounding away from a double root opens exactly on it.
#[test]
fn degenerate_ellipse_saved_off_by_rounding() {
    let text = ROUNDED_DEGENERATE_ELLIPSE;
    let mut sk = Sketch::from_doc(sggeo::sgp::from_str(text).unwrap());
    sk.solve_full();
    let names = sk.doc.param_names();
    for i in [3, 4] {
        assert!((sk.doc.measure_value(i) - 3.0).abs() < 1e-14, "measured {}", sk.doc.measure_value(i));
        let (name, target, _) = target_expr(&sk.doc, Target::Dim(i)).expect("bad target");
        let der = derive::derive(&sk.doc, &target).unwrap();
        let text = plain(&to_bx(&der.expr, &|s| sym_name(&names, s)));
        assert!(der.exact, "{name} = {text} is not exact");
        assert!((der.value - 3.0).abs() < 1e-14, "{name} = {text} gives {}", der.value);
    }
}

/// Every example file, as (group/name, path), in order.
fn example_files() -> Vec<(String, std::path::PathBuf)> {
    let mut files = Vec::new();
    let Ok(groups) = std::fs::read_dir(EXAMPLES_DIR) else { return files };
    for group in groups.flatten() {
        for f in std::fs::read_dir(group.path()).into_iter().flatten().flatten() {
            let path = f.path();
            if path.extension().is_some_and(|e| e == "sgp") {
                let name = format!("{}/{}", group.file_name().to_string_lossy(), path.file_stem().unwrap().to_string_lossy());
                files.push((name, path));
            }
        }
    }
    files.sort();
    files
}

/// Derived without solving first, the same file still has its exact formulas:
/// they miss the measurements by 7e-7, within how precisely the measurements
/// are known there (`ops::dim_uncertainty`, about √ε at the double root).
#[test]
fn degenerate_ellipse_derived_as_saved() {
    let doc = ops::Document::from_sgp(ROUNDED_DEGENERATE_ELLIPSE).unwrap();
    for i in [3, 4] {
        let u = ops::dim_uncertainty(&doc, i).unwrap();
        assert!((1e-8..1e-5).contains(&u), "uncertainty {u}");
        let f = ops::derive(&doc, Target::Dim(i), 0, None).unwrap();
        assert!(f.exact, "{} is not exact", f.plain);
        assert!((f.value - 3.0).abs() < 1e-14 && (f.actual - 3.0).abs() > 1e-7, "{}: {} measured {}", f.plain, f.value, f.actual);
    }
    // a regular construction is known to rounding
    let doc = ops::Document::from_sgp(include_str!("law_of_cosines.sgp")).unwrap();
    let u = ops::dim_uncertainty(&doc, 3).unwrap();
    assert!(u < 1e-11, "uncertainty {u}");
    assert!(ops::dim_uncertainty(&doc, 99).is_err());
}

/// Formulas the derivation gives partly numerically: their tests check only
/// the value (`Kit::check_value`, or `allow_inexact`).
const NUMERIC: &[(&str, &str)] = &[
    ("gen_circle_tangency/malfatti_circles", "r1"),
    ("gen_conics/common_tangent_ellipse_circle", "x_C"),
    ("gen_conics/common_tangent_two_ellipses", "x_C"),
    ("gen_conics/steiner_inellipse", "a1"),
    ("gen_conics/steiner_inellipse", "b1"),
    ("gen_edge_cases/long_truss_chain_4", "d12"),
    ("gen_functions/dottie_number", "x_A"),
];

/// Exact formulas whose value and the measurement agree only to the given
/// tolerance (relative, as in `formulas_match_measurements`), and why.
const IMPRECISE: &[(&str, &str, f64, &str)] = &[
    // s² + c = 5s has a double root, which f64 pins down only to √ε
    ("gen_variables/solved_var_double_root", "d3", 1e-6, "double root"),
    // the contact of a tangent at an inflection is a triple root
    ("gen_functions/sine_inflection_tangent_unpinned", "d2", 1e-8, "triple root"),
    // the law of cosines at 0.001° evaluates 1 − cos θ with cancellation
    ("gen_edge_cases/tiny_angle_long_sides", "d3", 1e-8, "cancellation in the formula"),
];

/// The conditions for a formula to hold that its example is at the limit of
/// (`ops::Formula::limits`): no other formula of an example has one.
const LIMITS: &[(&str, &str, &str)] = &[
    // a = b: the foci are at the center, and pass to the other axis when b passes a
    ("gen_conics/ellipse_degenerate_circle_focus", "d1", "b1^2 - a1^2 ≥ 0"),
    ("gen_conics/ellipse_degenerate_circle_focus", "d2", "b1^2 - a1^2 ≥ 0"),
    // s² + c = 5s has a double root at c = 25/4, and no real one past it
    ("gen_variables/solved_var_double_root", "d3", "25 - 4c ≥ 0"),
];

/// Every exact formula of a reference dimension of an example equals the
/// measurement to rounding (1e-12 relative), and every formula is exact,
/// except those listed in `NUMERIC` and `IMPRECISE`; and the formulas at a
/// limit are those of `LIMITS`. A solver that leaves a
/// value a rounding away from a double root (and the measurement √ε away:
/// the foci of an ellipse with a = b) fails here, as does a measurement that
/// loses digits. Derives in parallel: all of them take about 10 s.
#[test]
fn formulas_match_measurements() {
    let files = example_files();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let found = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..std::thread::available_parallelism().map_or(4, |n| n.get()) {
            s.spawn(|| {
                derive::set_time_scale(1.0);
                while let Some((file, path)) = files.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed)) {
                    let sk = open(path);
                    for i in 0..sk.doc.dims.len() {
                        if sk.doc.dims[i].as_ref().is_none_or(|d| d.driving) {
                            continue;
                        }
                        let (name, target, _) = target_expr(&sk.doc, Target::Dim(i)).expect("bad target");
                        let r = derive::derive(&sk.doc, &target).map(|d| (d.exact, d.value, sk.doc.measure_value(i), sggeo::formula::limits(&sk.doc, &d.expr)));
                        found.lock().unwrap().push((file.clone(), name, r));
                    }
                }
            });
        }
    });
    let mut found = found.into_inner().unwrap();
    found.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    let mut bad = Vec::new();
    for (file, name, r) in &found {
        let what = format!("{file} {name}");
        // the conditions it is at the limit of, as listed
        let want: Vec<&str> = LIMITS.iter().filter(|x| x.0 == file && x.1 == name).map(|x| x.2).collect();
        if let Ok((.., limits)) = r
            && limits.iter().map(|l| l.0.as_str()).collect::<Vec<_>>() != want
        {
            bad.push(format!("{what}: at the limits {:?}, LIMITS lists {want:?}", limits.iter().map(|l| &l.0).collect::<Vec<_>>()));
        }
        let numeric = NUMERIC.contains(&(file.as_str(), name.as_str()));
        let imprecise = IMPRECISE.iter().find(|x| x.0 == file && x.1 == name);
        match r {
            Err(e) => bad.push(format!("{what}: no formula ({e})")),
            Ok((false, ..)) if !numeric => bad.push(format!("{what}: not exact")),
            Ok((true, ..)) if numeric => bad.push(format!("{what}: exact now, take it out of NUMERIC")),
            Ok((false, ..)) => {}
            Ok((true, value, measured, _)) => {
                let err = (value - measured).abs() / (1.0 + measured.abs());
                match imprecise {
                    _ if !(err <= 1.0) => bad.push(format!("{what}: formula {value}, measured {measured}")),
                    None if err > 1e-12 => bad.push(format!("{what}: formula {value}, measured {measured} (off by {err:.1e})")),
                    Some(x) if err > x.2 => bad.push(format!("{what}: formula {value}, measured {measured} (off by {err:.1e}, {} allows {:.0e})", x.3, x.2)),
                    Some(_) if err <= 1e-12 => bad.push(format!("{what}: precise now, take it out of IMPRECISE")),
                    _ => {}
                }
            }
        }
    }
    let seen = |f: &str, n: &str| found.iter().any(|x| x.0 == f && x.1 == n);
    for (f, n) in NUMERIC.iter().copied().chain(IMPRECISE.iter().map(|x| (x.0, x.1))).chain(LIMITS.iter().map(|x| (x.0, x.1))) {
        if !files.is_empty() && !seen(f, n) {
            bad.push(format!("{f} {n}: listed, but no such reference dimension"));
        }
    }
    assert!(bad.is_empty(), "{} of {} formulas:\n{}", bad.len(), found.len(), bad.join("\n"));
}

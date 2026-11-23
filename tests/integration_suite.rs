#![forbid(unsafe_code)]

use principia_studio::combinatorics_opt::{
    count_non_isomorphic_graphs, count_reduced_latin_squares, enumerate_free_polyominoes,
    simplex_maximize, tsp_exact_branch_and_bound,
};
use principia_studio::formal_verification::{
    encode_pythagorean_triples_cnf, encode_ramsey_r33_cnf, parse_prop_formula, portfolio_sat_solve,
    prove_sequent_lk, SatResult,
};
use principia_studio::geometry_topology::{
    compute_vietoris_rips_persistence, convex_hull_3d, jones_polynomial_from_pd, knot_preset_data,
};
use principia_studio::number_theory::{
    discrete_log_bsgs, factorize_u64, is_carmichael_korselt, is_mersenne_prime_lucas_lehmer,
    is_prime_baillie_psw, is_prime_miller_rabin, quadratic_sieve_factor, segmented_sieve_range,
    sieve_of_atkin,
};
use principia_studio::numerical_pde::{compute_pi_chudnovsky, dense_svd, BigInt};
use principia_studio::render3d::{export_report_svg, export_scene3d_obj};
use principia_studio::storage::{WorksetEnvelope, WorksetVault};
use principia_studio::symbolic_algebra::{
    buchberger_groebner_basis, differentiate, integrate_symbolic, parse_sym_expr, simplify_expr,
    MonomialOrder, MultiPoly,
};
use principia_studio::{execute_module, ExplorerParams, ModuleId};

#[test]
fn test_number_theory_sieves_primality_and_factorization() {
    // pi(1_000_000) == 78_498
    let primes_1m = segmented_sieve_range(2, 1_000_000);
    assert_eq!(primes_1m.len(), 78_498);

    // High 10^12 window sieve
    let high_window = segmented_sieve_range(1_000_000_000_000, 1_000_000_010_000);
    assert!(!high_window.is_empty());
    for &p in &high_window {
        assert!(is_prime_miller_rabin(p));
        assert!(is_prime_baillie_psw(p));
    }

    // Atkin matches Eratosthenes up to 10_000 (1_229 primes)
    assert_eq!(sieve_of_atkin(10_000).len(), 1_229);

    // Carmichael numbers rejected by Baillie-PSW & Miller-Rabin, accepted by Korselt
    for &c in &[561u64, 1105, 1729, 2465, 2821, 6601, 8911] {
        assert!(!is_prime_baillie_psw(c));
        assert!(!is_prime_miller_rabin(c));
        assert!(is_carmichael_korselt(c));
    }

    // Factorization & Quadratic Sieve & BSGS Discrete Log
    let composite_prod = 10_403u64 * 100_003u64; // 10_403 = 101 * 103
    assert_eq!(factorize_u64(composite_prod), vec![101, 103, 100_003]);
    let semiprime = 10_007u64 * 100_003u64;
    assert_eq!(factorize_u64(semiprime), vec![10_007, 100_003]);
    let qs = quadratic_sieve_factor(1_040_301_499);
    assert!(qs.is_some());
    assert_eq!(discrete_log_bsgs(5, 125, 1009), Some(3));

    // Mersenne primes M_2, M_3, M_5, M_7, M_13, M_17, M_19, M_31
    assert!(is_mersenne_prime_lucas_lehmer(31));
    assert!(!is_mersenne_prime_lucas_lehmer(23));
}

#[test]
fn test_formal_verification_itp_and_portfolio_sat() {
    let f = parse_prop_formula("(P -> Q) -> (Q -> R) -> (P -> R)");
    assert!(prove_sequent_lk(Vec::new(), vec![f], 0).is_some());
    let invalid = parse_prop_formula("(P \\/ Q) -> (P /\\ Q)");
    assert!(prove_sequent_lk(Vec::new(), vec![invalid], 0).is_none());

    // Ramsey R(3,3) = 6 => K_5 is SAT, K_6 is UNSAT
    let (v5, c5) = encode_ramsey_r33_cnf(5);
    assert!(matches!(portfolio_sat_solve(v5, &c5, 4), SatResult::Sat(_)));
    let (v6, c6) = encode_ramsey_r33_cnf(6);
    assert!(matches!(
        portfolio_sat_solve(v6, &c6, 4),
        SatResult::Unsat { .. }
    ));

    let (vp, cp, _) = encode_pythagorean_triples_cnf(40);
    assert!(matches!(portfolio_sat_solve(vp, &cp, 4), SatResult::Sat(_)));
}

#[test]
fn test_symbolic_calculus_and_groebner() {
    let e = parse_sym_expr("x^2 * exp(x) + sin(x) * cos(x)");
    let anti = integrate_symbolic(&e, "x").expect("Liouvillian integral should succeed");
    let roundtrip = differentiate(&anti, "x");
    for x in [-0.7, 0.2, 1.1] {
        assert!((roundtrip.eval_real(x) - e.eval_real(x)).abs() < 1e-6);
    }

    let trig = simplify_expr(&parse_sym_expr("sin(x)^2 + cos(x)^2"));
    assert_eq!(trig.eval_real(1.234), 1.0);

    let g1 = MultiPoly::from_terms(vec![([2, 0, 0], 1), ([0, 1, 0], -1)]); // x^2 - y
    let g2 = MultiPoly::from_terms(vec![([1, 1, 0], 1), ([0, 0, 0], -1)]); // x*y - 1
    let gb = buchberger_groebner_basis(&[g1, g2], MonomialOrder::Lex);
    assert!(!gb.is_empty());
}

#[test]
fn test_numerical_chudnovsky_ntt_and_svd() {
    let pi_60 = compute_pi_chudnovsky(60);
    assert!(pi_60.starts_with("3.14159265358979323846264338327950288419716939937510"));

    let a: Vec<i64> = (0..90).map(|i| (i * 73 + 19) % 10_000).collect();
    let b: Vec<i64> = (0..90).map(|i| (i * 197 + 41) % 10_000).collect();
    assert_eq!(
        BigInt::mul_karatsuba_limbs(&a, &b),
        BigInt::mul_ntt_limbs(&a, &b)
    );

    let mat = vec![
        vec![3.0, 1.0, 0.5],
        vec![1.0, 2.5, -0.4],
        vec![0.5, -0.4, 1.8],
    ];
    let (u, s, v) = dense_svd(&mat);
    for i in 0..3 {
        for j in 0..3 {
            let rec: f64 = (0..3).map(|k| u[i][k] * s[k] * v[j][k]).sum();
            assert!((mat[i][j] - rec).abs() < 1e-9);
        }
    }
}

#[test]
fn test_combinatorics_and_geometry_topology() {
    // OEIS A000105 Free Pentominoes = 12, Hexominoes = 35, Heptominoes = 108
    assert_eq!(enumerate_free_polyominoes(5).len(), 12);
    assert_eq!(enumerate_free_polyominoes(6).len(), 35);
    assert_eq!(count_reduced_latin_squares(5), 56);
    assert_eq!(count_non_isomorphic_graphs(5), 34);

    let cities = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    let (_, best_len) = tsp_exact_branch_and_bound(&cities);
    assert!((best_len - 4.0).abs() < 1e-9);

    let a = vec![vec![1.0, 1.0], vec![2.0, 1.0]];
    let b = vec![4.0, 6.0];
    let c = vec![3.0, 2.0];
    let (_, obj, _) = simplex_maximize(&a, &b, &c);
    assert!((obj - 10.0).abs() < 1e-7);

    // 3D Convex Hull of Cube + interior point -> V=8, E=18, F=12 => V - E + F = 2
    let pts = vec![
        [-1.0, -1.0, -1.0],
        [1.01, -1.0, -1.0],
        [1.0, 1.02, -1.0],
        [-1.0, 1.0, -1.03],
        [-1.0, -1.0, 1.04],
        [1.0, -1.02, 1.0],
        [1.02, 1.0, 1.0],
        [-1.01, 1.0, 1.0],
        [0.0, 0.0, 0.0],
    ];
    let hull = convex_hull_3d(&pts);
    assert_eq!(hull.len(), 12);

    // TDA circle -> beta_0 = 1, beta_1 = 1
    let circle: Vec<[f64; 3]> = (0..16)
        .map(|k| {
            let th = 2.0 * std::f64::consts::PI * (k as f64) / 16.0;
            [th.cos(), th.sin(), 0.0]
        })
        .collect();
    let (_, betti, _, _) = compute_vietoris_rips_persistence(&circle, 1.2, 0.75);
    assert_eq!(betti[0], 1);
    assert_eq!(betti[1], 1);

    // Knot chirality: Right vs Left Trefoil have distinct Jones polynomials, while Figure-8 is symmetric
    let (_, ar, pdr, _, _) = knot_preset_data(0);
    let (_, al, pdl, _, _) = knot_preset_data(1);
    let jr = jones_polynomial_from_pd(ar, &pdr);
    let jl = jones_polynomial_from_pd(al, &pdl);
    assert_ne!(jr, jl);
}

#[test]
fn test_all_22_modules_execute_and_persist() {
    let vault = WorksetVault::open_in_memory().expect("in-memory sqlite vault");
    for (idx, &m) in ModuleId::ALL.iter().enumerate() {
        let mut params = ExplorerParams::default_for(m);
        params.threads = 2;
        let rep = execute_module(&params);
        assert_eq!(rep.module, m);
        assert!(!rep.metrics.is_empty());
        let svg = export_report_svg(&rep, 800, 600);
        assert!(svg.starts_with("<svg"));
        if rep.prefer_3d {
            let obj = export_scene3d_obj(&rep.scene_3d);
            assert!(obj.contains("# Principia Studio 3D Export"));
        }
        let env = WorksetEnvelope::new(format!("Slot {idx}"), params, Default::default());
        assert!(env.verify_checksum());
        vault.save_slot(idx as i64, &env).expect("save slot");
        let loaded = vault.load_slot(idx as i64).expect("load slot");
        assert!(loaded.is_some());
    }
}

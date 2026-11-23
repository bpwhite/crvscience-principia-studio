#![forbid(unsafe_code)]

//! Comprehensive Headless Mathematical Algorithm Validation Suite for Principia Studio
//! Exercises all 6 domains (`N1..N4`, `F1..F3`, `S1..S3`, `D1..D3`, `C1..C3`, `G1..G3`)
//! and emits `validation/principia_studio_validation_summary.json` under the 384 KB cap.

mod common;

use principia_studio::combinatorics_opt::{
    bin_packing_ffd, count_non_isomorphic_graphs, count_reduced_latin_squares,
    enumerate_free_polyominoes, interior_point_central_path, simplex_maximize, tour_length,
    tsp_exact_branch_and_bound, tsp_parallel_annealing,
};
use principia_studio::formal_verification::{
    encode_pythagorean_triples_cnf, encode_ramsey_r33_cnf, eval_polynomial_pcf,
    parse_prop_formula, portfolio_sat_solve, prove_sequent_lk, SatResult,
};
use principia_studio::geometry_topology::{
    bowyer_watson_delaunay_2d, compute_vietoris_rips_persistence, convex_hull_3d,
    jones_polynomial_from_pd, knot_preset_data, LaurentPoly,
};
use principia_studio::number_theory::{
    aks_congruence_witness, collatz_stopping_time, discrete_log_bsgs, factorize_u64, gcd_u64,
    is_carmichael_korselt, is_mersenne_prime_lucas_lehmer, is_prime_baillie_psw,
    is_prime_miller_rabin, mod_mul, mod_pow, pollard_rho_brent, quadratic_sieve_factor,
    segmented_sieve_range, sieve_of_atkin, simple_sieve_u64,
};
use principia_studio::numerical_pde::{
    compute_pi_chudnovsky, dense_svd, integrate_lorenz_rk4, solve_2d_vorticity_fdm,
    solve_p1_fem_poisson, sparse_conjugate_gradient, BigInt, CsrMatrix,
};
use principia_studio::render3d::{export_report_svg, export_scene3d_obj};
use principia_studio::symbolic_algebra::{
    buchberger_groebner_basis, differentiate, integrate_symbolic, parse_sym_expr, s_polynomial,
    saturate_equivalences, simplify_expr, MonomialOrder, MultiPoly,
};
use principia_studio::{execute_module, ExplorerParams, ModuleId};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Mutex;

static SUMMARY_LOCK: Mutex<()> = Mutex::new(());

fn verify_cnf_assignment(clauses: &[Vec<i32>], model: &[bool]) -> bool {
    clauses.iter().all(|clause| {
        clause.iter().any(|&lit| {
            let var_idx = (lit.unsigned_abs() as usize) - 1;
            let val = model[var_idx];
            if lit > 0 {
                val
            } else {
                !val
            }
        })
    })
}

fn poly_eval_f64(p: &MultiPoly, x: f64, y: f64, z: f64) -> f64 {
    p.terms
        .iter()
        .map(|(&[ex, ey, ez], &(num, den))| {
            (num as f64 / den as f64) * x.powi(ex as i32) * y.powi(ey as i32) * z.powi(ez as i32)
        })
        .sum()
}

fn reflect_laurent_t(poly: &LaurentPoly) -> LaurentPoly {
    let mut coeffs = BTreeMap::new();
    for (&exp, &c) in &poly.coeffs {
        if c != 0 {
            coeffs.insert(-exp, c);
        }
    }
    LaurentPoly { coeffs }
}

#[test]
fn headless_domain1_number_theory_algorithms() {
    // 1. Cross-verify simple sieve, segmented sieve, and Sieve of Atkin
    let s1 = simple_sieve_u64(100_000);
    let s2 = segmented_sieve_range(2, 100_000);
    let s3 = sieve_of_atkin(100_000);
    assert_eq!(s1.len(), 9_592);
    assert_eq!(s1, s2);
    assert_eq!(s1, s3);
    assert_eq!(segmented_sieve_range(2, 1_000_000).len(), 78_498);

    // High window around 10^12: verify prime count & deterministic primality of every hit
    let high_primes = segmented_sieve_range(1_000_000_000_000, 1_000_000_050_000);
    assert!(high_primes.len() > 1_500);
    for &p in &high_primes {
        assert!(is_prime_miller_rabin(p));
        assert!(is_prime_baillie_psw(p));
    }

    // 2. Modular arithmetic and Euclid GCD
    assert_eq!(gcd_u64(48_640, 12_800), 2_560);
    assert_eq!(gcd_u64(36_288, 12_800), 64);
    assert_eq!(gcd_u64(1_000_000_007, 998_244_353), 1);
    let big_p = 1_000_000_007u64;
    for a in [2u64, 7, 13, 123_456_789] {
        assert_eq!(mod_pow(a, big_p - 1, big_p), 1);
    }
    assert_eq!(mod_mul(big_p - 1, big_p - 1, big_p), 1);

    // 3. Miller-Rabin, Baillie-PSW, Korselt Carmichael criterion, and AKS polynomial witness
    for n in 2u64..=2_500 {
        assert_eq!(is_prime_miller_rabin(n), is_prime_baillie_psw(n));
    }
    let carmichaels = [561u64, 1105, 1729, 2465, 2821, 6601, 8911, 41041];
    for &c in &carmichaels {
        assert!(!is_prime_miller_rabin(c));
        assert!(!is_prime_baillie_psw(c));
        assert!(is_carmichael_korselt(c));
    }
    for &non_carm in &[9u64, 15, 21, 91, 341, 1001, 17, 1_000_000_007] {
        assert!(!is_carmichael_korselt(non_carm));
    }
    assert!(aks_congruence_witness(31, 5, 1));
    assert!(aks_congruence_witness(97, 7, 2));
    assert!(aks_congruence_witness(101, 7, 3));
    assert!(!aks_congruence_witness(15, 5, 1));
    assert!(!aks_congruence_witness(561, 7, 1));

    // 4. Factorization (Pollard-Brent rho, complete factorization, Quadratic Sieve) & BSGS DLP
    let target_n = 10_007u64 * 100_003u64;
    let (rho_d, trace) = pollard_rho_brent(target_n, 1).expect("Brent rho should split semiprime");
    assert!(rho_d > 1 && rho_d < target_n && target_n % rho_d == 0);
    assert!(!trace.is_empty());

    let factors = factorize_u64(target_n);
    assert_eq!(factors, vec![10_007, 100_003]);
    assert_eq!(factors.iter().copied().product::<u64>(), target_n);
    assert!(factors.iter().all(|&f| is_prime_baillie_psw(f)));

    let qs_n = 1_040_301_499u64;
    let (qs_d, fb_len, rels) = quadratic_sieve_factor(qs_n).expect("Quadratic Sieve should split");
    assert!(qs_d > 1 && qs_d < qs_n && qs_n % qs_d == 0);
    assert!(fb_len > 0 && rels > 0);

    assert_eq!(discrete_log_bsgs(5, 125, 1009), Some(3));
    let dlp_p = 104_729u64;
    let dlp_g = 2u64;
    let dlp_h = mod_pow(dlp_g, 41_321, dlp_p);
    let solved_x = discrete_log_bsgs(dlp_g, dlp_h, dlp_p).expect("BSGS should find discrete log");
    assert_eq!(mod_pow(dlp_g, solved_x, dlp_p), dlp_h);

    // 5. Lucas-Lehmer Mersenne test & Collatz stopping-time record
    for &p in &[2u32, 3, 5, 7, 13, 17, 19, 31] {
        assert!(is_mersenne_prime_lucas_lehmer(p));
    }
    for &p in &[4u32, 11, 23, 29] {
        assert!(!is_mersenne_prime_lucas_lehmer(p));
    }
    assert_eq!(collatz_stopping_time(1), (0, 1));
    assert_eq!(collatz_stopping_time(27), (111, 9232));
}

#[test]
fn headless_domain2_formal_verification_algorithms() {
    // 1. Sequent-calculus prover on classical propositional tautologies and non-theorems
    let valid_tautologies = [
        "(P -> Q) -> (Q -> R) -> (P -> R)",
        "(P -> Q) -> (~Q -> ~P)",
        "((P -> Q) -> P) -> P",
        "P \\/ ~P",
        "(P /\\ Q) -> (Q /\\ P)",
    ];
    for raw in valid_tautologies {
        let f = parse_prop_formula(raw);
        assert!(
            prove_sequent_lk(Vec::new(), vec![f], 0).is_some(),
            "Expected valid sequent proof for tautology: {raw}"
        );
    }
    for invalid in ["(P \\/ Q) -> (P /\\ Q)", "P -> Q", "~(P -> P)"] {
        let f = parse_prop_formula(invalid);
        assert!(
            prove_sequent_lk(Vec::new(), vec![f], 0).is_none(),
            "Invalid formula unexpectedly proved: {invalid}"
        );
    }

    // 2. Portfolio CDCL SAT solver + explicit model verification across 1 and 4 workers
    let (v5, c5) = encode_ramsey_r33_cnf(5);
    for workers in [1usize, 4] {
        match portfolio_sat_solve(v5, &c5, workers) {
            SatResult::Sat(model) => {
                assert_eq!(model.len(), v5);
                assert!(
                    verify_cnf_assignment(&c5, &model),
                    "R(3,3;5) SAT assignment failed clause check with {workers} workers"
                );
            }
            other => panic!("Expected SAT for K_5 Ramsey R(3,3), got {other:?}"),
        }
    }
    let (v6, c6) = encode_ramsey_r33_cnf(6);
    assert!(matches!(
        portfolio_sat_solve(v6, &c6, 4),
        SatResult::Unsat { .. }
    ));

    let (vp, cp, triples) = encode_pythagorean_triples_cnf(40);
    assert!(!triples.is_empty());
    match portfolio_sat_solve(vp, &cp, 4) {
        SatResult::Sat(model) => {
            assert_eq!(model.len(), vp);
            assert!(verify_cnf_assignment(&cp, &model));
        }
        other => panic!("Expected SAT for Pythagorean triples N=40, got {other:?}"),
    }

    // 3. Polynomial continued fractions (`phi` and `sqrt(2)`)
    let phi = 0.5 * (1.0 + 5.0_f64.sqrt());
    let pcf_phi = eval_polynomial_pcf([1, 0, 0], [1, 0, 0], 80);
    assert!((pcf_phi - phi).abs() < 1e-12);

    let pcf_sqrt2 = eval_polynomial_pcf([2, 0, 0], [1, 0, 0], 60) - 1.0;
    assert!((pcf_sqrt2 - 2.0_f64.sqrt()).abs() < 1e-12);
}

#[test]
fn headless_domain3_symbolic_algebra_algorithms() {
    // 1. Symbolic differentiation matches central finite difference across functions
    let diff_cases = [
        "x^3 + 3*x^2 - 5*x + 2",
        "x^2 * exp(x)",
        "sin(x) * cos(x)",
    ];
    let h = 1e-6;
    for raw in diff_cases {
        let expr = parse_sym_expr(raw);
        let d_expr = differentiate(&expr, "x");
        for x in [-0.9f64, 0.35, 1.15] {
            let num_df = (expr.eval_real(x + h) - expr.eval_real(x - h)) / (2.0 * h);
            let sym_df = d_expr.eval_real(x);
            assert!(
                (sym_df - num_df).abs() < 1e-5,
                "Derivative mismatch for {raw} at x={x}: sym={sym_df}, num={num_df}"
            );
        }
    }

    // 2. Symbolic integration fundamental-theorem roundtrip & complex evaluation
    for raw in ["x^2 * exp(x) + sin(x) * cos(x)", "x^3 - 4*x + cos(x)"] {
        let expr = parse_sym_expr(raw);
        let anti = integrate_symbolic(&expr, "x").expect("Antiderivative should exist");
        let recovered = differentiate(&anti, "x");
        for x in [-0.8f64, 0.25, 1.25] {
            assert!((recovered.eval_real(x) - expr.eval_real(x)).abs() < 1e-6);
        }
    }
    let poly_i = parse_sym_expr("x^2 + 1");
    let (zr, zi) = poly_i.eval_complex(0.0, 1.0);
    assert!(zr.abs() < 1e-12 && zi.abs() < 1e-12);

    // 3. E-graph / multi-rule equality saturation & AST minimization
    let start = parse_sym_expr("sin(x)^2 + cos(x)^2 + (x + 0) * 1");
    let simplified = simplify_expr(&start);
    let (best, layers) = saturate_equivalences(&start, 5);
    assert!(!layers.is_empty());
    assert!(best.ast_nodes() <= start.ast_nodes());
    assert!(simplified.ast_nodes() <= start.ast_nodes());
    for x in [-1.1f64, 0.1, 0.85, 2.3] {
        assert!((best.eval_real(x) - start.eval_real(x)).abs() < 1e-9);
    }

    // 4. Buchberger Groebner basis & Buchberger's S-polynomial criterion under Lex and GRevLex
    let g1 = MultiPoly::from_terms(vec![([2, 0, 0], 1), ([0, 1, 0], -1)]); // x^2 - y
    let g2 = MultiPoly::from_terms(vec![([1, 1, 0], 1), ([0, 0, 0], -1)]); // x*y - 1
    let g3 = MultiPoly::from_terms(vec![([0, 0, 1], 1), ([1, 0, 0], -1)]); // z - x
    let gens = [g1, g2, g3];

    for order in [MonomialOrder::Lex, MonomialOrder::GRevLex] {
        let gb = buchberger_groebner_basis(&gens, order);
        assert!(!gb.is_empty());
        // Every input generator reduces to 0 modulo the Groebner basis
        for g in &gens {
            assert!(
                g.reduce_by(&gb, order).is_zero(),
                "Generator did not reduce to 0 modulo {order:?} Groebner basis"
            );
        }
        // Buchberger's criterion: every pairwise S-polynomial reduces to 0 modulo gb
        for i in 0..gb.len() {
            for j in (i + 1)..gb.len() {
                let s = s_polynomial(&gb[i], &gb[j], order);
                assert!(
                    s.reduce_by(&gb, order).is_zero(),
                    "Non-zero S-poly remainder for ({i},{j}) under {order:?}"
                );
            }
        }
        // Vanishing on root (x=1, y=1, z=1)
        for p in &gb {
            assert!(poly_eval_f64(p, 1.0, 1.0, 1.0).abs() < 1e-9);
        }
    }
}

#[test]
fn headless_domain4_numerical_pde_algorithms() {
    // 1. 2D Vorticity-Streamfunction FDM solver stability & finite enstrophy
    let (omega, enstrophy) = solve_2d_vorticity_fdm(24, 15, 0.015, 0.002);
    assert_eq!(omega.len(), 24 * 24);
    assert!(omega.iter().all(|v| v.is_finite()));
    assert!(enstrophy.is_finite() && enstrophy > 0.0);

    // 2. P1 FEM Poisson center value on [0,1]^2 (exact center u(0.5, 0.5) ~ 0.07367)
    let u9 = solve_p1_fem_poisson(9);
    let u17 = solve_p1_fem_poisson(17);
    assert!(u9.is_finite() && u9 > 0.06 && u9 < 0.09);
    assert!(u17.is_finite() && u17 > 0.06 && u17 < 0.09);

    // 3. RK4 Lorenz integrator boundedness & positive finite-time Lyapunov divergence
    let steps = 2_800usize;
    let init_eps = 1e-6;
    let tr_a = integrate_lorenz_rk4([1.0, 1.0, 1.0], steps, 0.01);
    let tr_b = integrate_lorenz_rk4([1.0 + init_eps, 1.0, 1.0], steps, 0.01);
    assert_eq!(tr_a.len(), steps + 1);
    assert_eq!(tr_b.len(), steps + 1);
    for p in tr_a.iter().chain(tr_b.iter()) {
        assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        assert!(p[0].abs() < 60.0 && p[1].abs() < 60.0 && p[2] < 80.0);
    }
    let last_a = tr_a[steps];
    let last_b = tr_b[steps];
    let sep = ((last_a[0] - last_b[0]).powi(2)
        + (last_a[1] - last_b[1]).powi(2)
        + (last_a[2] - last_b[2]).powi(2))
    .sqrt();
    assert!(
        sep > 100.0 * init_eps,
        "Expected Lorenz Lyapunov trajectory divergence, got {sep}"
    );

    // 4. BigInt Karatsuba/NTT parity, division, isqrt, and 100-digit Chudnovsky Pi
    let a_limbs: Vec<i64> = (0..120).map(|i| (i * 97 + 13) % 10_000).collect();
    let b_limbs: Vec<i64> = (0..120).map(|i| (i * 211 + 37) % 10_000).collect();
    assert_eq!(
        BigInt::mul_karatsuba_limbs(&a_limbs, &b_limbs),
        BigInt::mul_ntt_limbs(&a_limbs, &b_limbs)
    );

    let big_x = BigInt::from_i128(987_654_321_987_654_321);
    let big_y = BigInt::from_i128(123_456_789);
    let sum_xy = big_x.add(&big_y);
    assert_eq!(sum_xy.sub(&big_y).to_decimal_string(), "987654321987654321");
    let prod = big_x.mul(&big_y);
    assert_eq!(prod.div_bigint_positive(&big_y).to_decimal_string(), "987654321987654321");
    assert_eq!(BigInt::from_i128(1_000_000_000).div_small_u64(8).to_decimal_string(), "125000000");

    let n_sq = BigInt::from_i128(123_456_789_012_345_678);
    let r = n_sq.isqrt();
    let r2 = r.mul(&r);
    let rp1 = r.add(&BigInt::from_i128(1));
    let rp1_sq = rp1.mul(&rp1);
    assert!(n_sq.sub(&r2).sign >= 0);
    assert!(rp1_sq.sub(&n_sq).sub(&BigInt::from_i128(1)).sign >= 0);

    let pi_100 = compute_pi_chudnovsky(100);
    assert_eq!(
        pi_100,
        "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679"
    );

    // 5. CSR Sparse Matrix & Conjugate Gradient on 32x32 1D SPD Tridiagonal Laplacian
    let dim = 32usize;
    let mut row_ptr = Vec::with_capacity(dim + 1);
    let mut col_idx = Vec::new();
    let mut values = Vec::new();
    row_ptr.push(0);
    for r_idx in 0..dim {
        if r_idx > 0 {
            col_idx.push(r_idx - 1);
            values.push(-1.0);
        }
        col_idx.push(r_idx);
        values.push(2.0);
        if r_idx + 1 < dim {
            col_idx.push(r_idx + 1);
            values.push(-1.0);
        }
        row_ptr.push(col_idx.len());
    }
    let csr = CsrMatrix {
        nrows: dim,
        ncols: dim,
        row_ptr,
        col_idx,
        values,
    };
    let rhs: Vec<f64> = (1..=dim).map(|k| (k as f64) * 0.1).collect();
    let (sol, hist) = sparse_conjugate_gradient(&csr, &rhs, 200, 1e-12);
    let ax = csr.spmv(&sol);
    let res_norm = ax
        .iter()
        .zip(rhs.iter())
        .map(|(a_i, b_i)| (a_i - b_i).powi(2))
        .sum::<f64>()
        .sqrt();
    assert!(res_norm < 1e-9, "CG residual too high: {res_norm}");
    assert!(hist.last().copied().unwrap_or(1.0) < 1e-9);

    // 6. Dense One-Sided Jacobi SVD reconstruction and orthogonality on rectangular 4x3 matrix
    let mat = vec![
        vec![3.0, 1.0, 0.5],
        vec![1.0, 2.5, -0.4],
        vec![0.5, -0.4, 1.8],
        vec![-1.2, 0.9, 2.1],
    ];
    let m = mat.len();
    let n = mat[0].len();
    let (u, s, v) = dense_svd(&mat);
    assert_eq!(u.len(), m);
    assert_eq!(s.len(), n);
    assert_eq!(v.len(), n);
    for k in 0..(n - 1) {
        assert!(s[k] >= s[k + 1] && s[k] >= 0.0);
    }
    // Check ||U * diag(s) * V^T - A||_max < 1e-9
    for i in 0..m {
        for j in 0..n {
            let rec: f64 = (0..n).map(|k| u[i][k] * s[k] * v[j][k]).sum();
            assert!((rec - mat[i][j]).abs() < 1e-9);
        }
    }
    // Check ||U^T U - I||_max < 1e-9 and ||V^T V - I||_max < 1e-9
    for a_i in 0..n {
        for b_j in 0..n {
            let expected = if a_i == b_j { 1.0 } else { 0.0 };
            let dot_u: f64 = (0..m).map(|r_k| u[r_k][a_i] * u[r_k][b_j]).sum();
            let dot_v: f64 = (0..n).map(|r_k| v[r_k][a_i] * v[r_k][b_j]).sum();
            assert!((dot_u - expected).abs() < 1e-9);
            assert!((dot_v - expected).abs() < 1e-9);
        }
    }
}

#[test]
fn headless_domain5_combinatorics_opt_algorithms() {
    // 1. TSP Exact Branch-and-Bound vs Parallel Island Annealing + Tour Permutation Checks
    let hexagon: Vec<(f64, f64)> = (0..8)
        .map(|k| {
            let th = 2.0 * std::f64::consts::PI * (k as f64) / 8.0;
            (th.cos(), th.sin())
        })
        .collect();
    let expected_hex = 8.0 * 2.0 * (std::f64::consts::PI / 8.0).sin();
    let (exact_tour, exact_len) = tsp_exact_branch_and_bound(&hexagon);
    assert_eq!(exact_tour.len(), 8);
    let exact_set: BTreeSet<usize> = exact_tour.iter().copied().collect();
    assert_eq!(exact_set.len(), 8);
    assert!((exact_len - expected_hex).abs() < 1e-9);
    assert!((tour_length(&hexagon, &exact_tour) - exact_len).abs() < 1e-9);

    let (sa_tour, sa_len) = tsp_parallel_annealing(&hexagon, 6, 4_000);
    let sa_set: BTreeSet<usize> = sa_tour.iter().copied().collect();
    assert_eq!(sa_tour.len(), 8);
    assert_eq!(sa_set.len(), 8);
    assert!(sa_len >= exact_len - 1e-9);
    assert!(sa_len <= exact_len * 1.25);

    // 2. FFD 1D Bin Packing invariants
    let weights = [0.72, 0.55, 0.44, 0.41, 0.38, 0.29, 0.28, 0.25, 0.21, 0.19];
    let capacity = 1.0;
    let (num_bins, bins) = bin_packing_ffd(&weights, capacity);
    assert_eq!(num_bins, bins.len());
    let total_in: f64 = weights.iter().sum();
    let mut packed_count = 0usize;
    let mut total_out = 0.0f64;
    for b in &bins {
        let sum_b: f64 = b.iter().sum();
        assert!(sum_b <= capacity + 1e-9);
        packed_count += b.len();
        total_out += sum_b;
    }
    assert_eq!(packed_count, weights.len());
    assert!((total_out - total_in).abs() < 1e-9);
    assert!(num_bins >= (total_in / capacity).ceil() as usize);

    // 3. Simplex Maximize + Interior-Point Log-Barrier Central Path feasibility & optimality
    let a = vec![
        vec![1.0, 1.0, 1.0],
        vec![2.0, 1.0, 0.5],
        vec![0.5, 2.0, 1.5],
    ];
    let b = vec![4.7, 6.3, 6.8];
    let c = vec![5.0, 4.0, 6.0];
    let (lp_x, lp_obj, path) = simplex_maximize(&a, &b, &c);
    assert!(!path.is_empty());
    assert!(lp_x.iter().all(|&v| v >= -1e-7));
    for (row, &rhs) in a.iter().zip(b.iter()) {
        let lhs: f64 = row.iter().zip(lp_x.iter()).map(|(ai, xi)| ai * xi).sum();
        assert!(lhs <= rhs + 1e-7);
    }
    let calc_obj: f64 = c.iter().zip(lp_x.iter()).map(|(ci, xi)| ci * xi).sum();
    assert!((calc_obj - lp_obj).abs() < 1e-7);

    let central = interior_point_central_path(&a, &b, &c);
    assert!(central.len() >= 10);
    for pt in &central {
        assert!(pt[0] > 0.0 && pt[1] > 0.0 && pt[2] > 0.0);
        for i in 0..b.len() {
            assert!(a[i][0] * pt[0] + a[i][1] * pt[1] + a[i][2] * pt[2] < b[i]);
        }
    }

    // 4. OEIS Exact Combinatorial Enumeration Counts
    let expected_polyominoes = [1usize, 1, 2, 5, 12, 35];
    for (idx, &want) in expected_polyominoes.iter().enumerate() {
        assert_eq!(enumerate_free_polyominoes(idx + 1).len(), want);
    }
    let expected_latin = [1usize, 1, 1, 4, 56];
    for (idx, &want) in expected_latin.iter().enumerate() {
        assert_eq!(count_reduced_latin_squares(idx + 1), want);
    }
    let expected_graphs = [1usize, 2, 4, 11, 34];
    for (idx, &want) in expected_graphs.iter().enumerate() {
        assert_eq!(count_non_isomorphic_graphs(idx + 1), want);
    }
}

#[test]
fn headless_domain6_geometry_topology_algorithms() {
    // 1. Bowyer-Watson 2D Delaunay Empty-Circumcircle Invariant + Voronoi dual edges
    let sunflower: Vec<(f64, f64)> = (0..18)
        .map(|k| {
            let r = 0.85 * (((k as f64) + 0.5) / 18.0).sqrt();
            let theta = (k as f64) * 2.399963229728653;
            (r * theta.cos(), r * theta.sin())
        })
        .collect();
    let (tris, voronoi) = bowyer_watson_delaunay_2d(&sunflower);
    assert!(!tris.is_empty());
    assert!(!voronoi.is_empty());
    for &[i, j, k] in &tris {
        assert!(i < sunflower.len() && j < sunflower.len() && k < sunflower.len());
        assert!(i != j && j != k && i != k);
        let a = sunflower[i];
        let b = sunflower[j];
        let c = sunflower[k];
        let area2 = ((b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)).abs();
        assert!(area2 > 1e-9, "Degenerate Delaunay triangle");
        let denom = 2.0 * (a.0 * (b.1 - c.1) + b.0 * (c.1 - a.1) + c.0 * (a.1 - b.1));
        let a2 = a.0 * a.0 + a.1 * a.1;
        let b2 = b.0 * b.0 + b.1 * b.1;
        let c2 = c.0 * c.0 + c.1 * c.1;
        let ux = (a2 * (b.1 - c.1) + b2 * (c.1 - a.1) + c2 * (a.1 - b.1)) / denom;
        let uy = (a2 * (c.0 - b.0) + b2 * (a.0 - c.0) + c2 * (b.0 - a.0)) / denom;
        let r2 = (a.0 - ux).powi(2) + (a.1 - uy).powi(2);
        for (p_idx, &p) in sunflower.iter().enumerate() {
            if p_idx != i && p_idx != j && p_idx != k {
                let d2 = (p.0 - ux).powi(2) + (p.1 - uy).powi(2);
                assert!(
                    d2 >= r2 - 1e-8,
                    "Empty-circumcircle Delaunay violation: tri={tris:?} d2={d2} r2={r2}"
                );
            }
        }
    }

    // 2. 3D Convex Hull Euler Characteristic V - E + F = 2 and half-space containment
    let oct_pts = vec![
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, -1.0],
        [0.12, -0.18, 0.14],
        [-0.21, 0.09, -0.11],
    ];
    let hull = convex_hull_3d(&oct_pts);
    assert_eq!(hull.len(), 8); // Regular octahedron has F = 8 triangular faces
    let mut verts = BTreeSet::new();
    let mut edges = BTreeSet::new();
    for &[i, j, k] in &hull {
        verts.insert(i);
        verts.insert(j);
        verts.insert(k);
        for (u, v) in [(i, j), (j, k), (k, i)] {
            edges.insert((u.min(v), u.max(v)));
        }
        let a = oct_pts[i];
        let b = oct_pts[j];
        let c = oct_pts[k];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let nrm = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for p in &oct_pts {
            let signed = nrm[0] * (p[0] - a[0]) + nrm[1] * (p[1] - a[1]) + nrm[2] * (p[2] - a[2]);
            assert!(signed <= 1e-7, "Convex hull half-space violation: {signed}");
        }
    }
    let euler = (verts.len() as isize) - (edges.len() as isize) + (hull.len() as isize);
    assert_eq!(euler, 2);

    // 3. Vietoris-Rips Persistence Homology: separated clusters & S^1 circle
    let clusters = vec![
        [0.0, 0.0, 0.0],
        [0.2, 0.0, 0.0],
        [0.0, 0.2, 0.0],
        [3.0, 0.0, 0.0],
        [3.2, 0.0, 0.0],
        [3.0, 0.2, 0.0],
    ];
    let (bars_cl, betti_cl, _, _) = compute_vietoris_rips_persistence(&clusters, 1.0, 0.5);
    assert_eq!(betti_cl[0], 2);
    assert_eq!(betti_cl[1], 0);
    for bar in &bars_cl {
        if let Some(death) = bar.death {
            assert!(bar.birth <= death + 1e-12);
        }
    }

    let circle: Vec<[f64; 3]> = (0..18)
        .map(|k| {
            let th = 2.0 * std::f64::consts::PI * (k as f64) / 18.0;
            [th.cos(), th.sin(), 0.0]
        })
        .collect();
    let (bars_s1, betti_s1, _, _) = compute_vietoris_rips_persistence(&circle, 1.1, 0.7);
    assert_eq!(betti_s1[0], 1);
    assert_eq!(betti_s1[1], 1);
    for bar in &bars_s1 {
        if let Some(death) = bar.death {
            assert!(bar.birth <= death + 1e-12);
        }
    }

    // 4. LaurentPoly laws + Jones Polynomial mirror chirality & amphichirality
    let x_poly = LaurentPoly::term(1, 1).add(&LaurentPoly::term(0, 1));
    assert_eq!(
        x_poly.pow(2),
        LaurentPoly::term(2, 1)
            .add(&LaurentPoly::term(1, 2))
            .add(&LaurentPoly::term(0, 1))
    );

    let (_, ar, pdr, _, _) = knot_preset_data(0); // Right-Handed Trefoil
    let (_, al, pdl, _, _) = knot_preset_data(1); // Left-Handed Trefoil
    let (_, a4, pd4, _, _) = knot_preset_data(2); // Figure-Eight 4_1
    let (_, a5, pd5, _, _) = knot_preset_data(3); // Cinquefoil 5_1

    let jr = jones_polynomial_from_pd(ar, &pdr);
    let jl = jones_polynomial_from_pd(al, &pdl);
    let j4 = jones_polynomial_from_pd(a4, &pd4);
    let j5 = jones_polynomial_from_pd(a5, &pd5);

    assert_ne!(jr, jl);
    assert_eq!(jl, reflect_laurent_t(&jr), "Mirror trefoil must reflect t <-> t^-1");
    assert_eq!(j4, reflect_laurent_t(&j4), "Figure-8 knot must be amphichiral");
    assert_ne!(j5, LaurentPoly::term(0, 1));
}

#[test]
fn headless_all_19_modules_multithreaded_and_telemetry() {
    let mut rows_json = Vec::with_capacity(ModuleId::ALL.len());
    for &m in &ModuleId::ALL {
        let mut p1 = ExplorerParams::default_for(m);
        p1.threads = 1;
        let r1 = execute_module(&p1);
        assert_eq!(r1.module, m);
        assert_eq!(r1.threads_used, 1);
        assert!(r1.elapsed_ms.is_finite());
        assert!(!r1.headline.trim().is_empty());
        assert!(!r1.metrics.is_empty());

        let mut p4 = ExplorerParams::default_for(m);
        p4.threads = 4;
        let r4 = execute_module(&p4);
        assert_eq!(r4.module, m);
        assert_eq!(r4.threads_used, 4);
        assert!(r4.elapsed_ms.is_finite());
        assert!(!r4.metrics.is_empty());

        let svg = export_report_svg(&r4, 1024, 640);
        assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
        let obj_bytes = if r4.prefer_3d {
            let obj = export_scene3d_obj(&r4.scene_3d);
            assert!(obj.contains("# Principia Studio 3D Export"));
            obj.len()
        } else {
            0
        };

        rows_json.push(format!(
            concat!(
                "    {{\n",
                "      \"code\": \"{}\",\n",
                "      \"short_name\": \"{}\",\n",
                "      \"domain\": \"{}\",\n",
                "      \"prefer_3d\": {},\n",
                "      \"threads_1_ms\": {:.3},\n",
                "      \"threads_4_ms\": {:.3},\n",
                "      \"metric_count\": {},\n",
                "      \"svg_bytes\": {},\n",
                "      \"obj_bytes\": {},\n",
                "      \"headline\": \"{}\"\n",
                "    }}"
            ),
            common::json_escape(m.code()),
            common::json_escape(m.short_name()),
            common::json_escape(m.category().title()),
            r4.prefer_3d,
            r1.elapsed_ms,
            r4.elapsed_ms,
            r4.metrics.len(),
            svg.len(),
            obj_bytes,
            common::json_escape(&r4.headline)
        ));
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let val_dir = common::validation_dir(manifest_dir);
    let summary_path = val_dir.join("principia_studio_validation_summary.json");
    let payload = format!(
        concat!(
            "{{\n",
            "  \"schema_version\": \"1.0\",\n",
            "  \"repo\": \"crvscience-principia-studio\",\n",
            "  \"version\": \"{}\",\n",
            "  \"domains_verified\": 6,\n",
            "  \"modules_verified\": {},\n",
            "  \"modules\": [\n{}\n  ]\n",
            "}}\n"
        ),
        env!("CARGO_PKG_VERSION"),
        ModuleId::ALL.len(),
        rows_json.join(",\n")
    );
    common::write_and_mirror_summary_json(&SUMMARY_LOCK, &summary_path, None, &payload);
}

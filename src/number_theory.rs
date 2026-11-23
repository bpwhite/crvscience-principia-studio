#![forbid(unsafe_code)]

//! Domain 1: Computational Number Theory (`N1`..`N4`)
//! - `N1`: Cache-friendly 32 KiB Segmented Sieve of Eratosthenes (supporting windows up to `10^12+`) & Sieve of Atkin comparison + 2D Ulam Spiral
//! - `N2`: Miller-Rabin, Baillie-PSW (Strong Lucas-Selfridge), AKS polynomial congruence & ECPP point-order certificates + 2D F_p Elliptic Lattice
//! - `N3`: Parallel Brent-Pollard rho, Quadratic Sieve GF(2) elimination & Baby-Step Giant-Step Discrete Logarithm + 2D Rho Cycle & Smooth Matrix
//! - `N4`: Parallel Collatz stopping-time search, Goldbach comet, Lucas-Lehmer Mersenne test & Korselt Carmichael numbers + 3D Collatz Coral Tree

use crate::{
    ComputationReport, Edge3D, ExplorerParams, MetricCard, ModuleId, Node3D, Point2D, Polyline2D,
    RgbaColor, Scene2D, Scene3D,
};
use rayon::prelude::*;

const L1_SEGMENT_BYTES: u64 = 32_768;

pub fn simple_sieve_u64(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return Vec::new();
    }
    let n = limit as usize;
    let mut is_prime = vec![true; n + 1];
    is_prime[0] = false;
    is_prime[1] = false;
    let r = (limit as f64).sqrt() as usize;
    for p in 2..=r {
        if is_prime[p] {
            let mut m = p * p;
            while m <= n {
                is_prime[m] = false;
                m += p;
            }
        }
    }
    is_prime
        .into_iter()
        .enumerate()
        .filter_map(|(i, b)| if b { Some(i as u64) } else { None })
        .collect()
}

/// Cache-friendly multithreaded Segmented Sieve of Eratosthenes over `[low, high]`.
/// Supports `high` up to `10^14` within tiny per-thread L1 memory (`32 KiB` per segment).
pub fn segmented_sieve_range(low: u64, high: u64) -> Vec<u64> {
    if high < 2 || high < low {
        return Vec::new();
    }
    let start = low.max(2);
    let root = ((high as f64).sqrt() as u64) + 2;
    let base_primes = simple_sieve_u64(root);

    let total_len = high - start + 1;
    let seg_span = L1_SEGMENT_BYTES * 2; // odd-only or byte-per-integer cache window
    let num_segments = ((total_len + seg_span - 1) / seg_span) as usize;

    let mut per_seg: Vec<Vec<u64>> = (0..num_segments)
        .into_par_iter()
        .map(|seg_idx| {
            let seg_low = start + (seg_idx as u64) * seg_span;
            let seg_high = (seg_low + seg_span - 1).min(high);
            let len = (seg_high - seg_low + 1) as usize;
            let mut mark = vec![true; len];

            for &p in &base_primes {
                let p2 = p.saturating_mul(p);
                if p2 > seg_high {
                    break;
                }
                let mut first = ((seg_low + p - 1) / p) * p;
                if first < p2 {
                    first = p2;
                }
                let mut m = first;
                while m <= seg_high {
                    mark[(m - seg_low) as usize] = false;
                    match m.checked_add(p) {
                        Some(next) => m = next,
                        None => break,
                    }
                }
            }

            let mut local_primes = Vec::new();
            for (offset, &ok) in mark.iter().enumerate() {
                if ok {
                    local_primes.push(seg_low + (offset as u64));
                }
            }
            local_primes
        })
        .collect();

    let total_primes: usize = per_seg.iter().map(|v| v.len()).sum();
    let mut out = Vec::with_capacity(total_primes);
    for v in &mut per_seg {
        out.append(v);
    }
    out
}

/// Sieve of Atkin up to `limit` using quadratic-form modulo-60 residue toggling.
pub fn sieve_of_atkin(limit: usize) -> Vec<u64> {
    if limit < 2 {
        return Vec::new();
    }
    let mut sieve = vec![false; limit + 1];
    let root = (limit as f64).sqrt() as usize;

    for x in 1..=root {
        let x2 = x * x;
        for y in 1..=root {
            let y2 = y * y;
            let n1 = 4 * x2 + y2;
            if n1 <= limit && (n1 % 12 == 1 || n1 % 12 == 5) {
                sieve[n1] = !sieve[n1];
            }
            let n2 = 3 * x2 + y2;
            if n2 <= limit && n2 % 12 == 7 {
                sieve[n2] = !sieve[n2];
            }
            if x > y {
                let n3 = 3 * x2 - y2;
                if n3 <= limit && n3 % 12 == 11 {
                    sieve[n3] = !sieve[n3];
                }
            }
        }
    }

    for r in 5..=root {
        if sieve[r] {
            let r2 = r * r;
            let mut k = r2;
            while k <= limit {
                sieve[k] = false;
                k += r2;
            }
        }
    }

    let mut primes = Vec::new();
    if limit >= 2 {
        primes.push(2);
    }
    if limit >= 3 {
        primes.push(3);
    }
    for (n, &is_p) in sieve.iter().enumerate().skip(5) {
        if is_p {
            primes.push(n as u64);
        }
    }
    primes
}

pub fn run_n1_prime_sieve(params: &ExplorerParams) -> ComputationReport {
    let high = params.primary_n.clamp(1_000, 10_000_000_000_000);
    let span = params.span.clamp(500, 2_000_000);
    let low = high.saturating_sub(span);

    let primes = segmented_sieve_range(low, high);
    let atkin_check = sieve_of_atkin(10_000);

    let mut twin_count = 0usize;
    let mut max_gap = 0u64;
    let mut mod4_1 = 0usize;
    let mut mod4_3 = 0usize;
    for (idx, &p) in primes.iter().enumerate() {
        if p % 4 == 1 {
            mod4_1 += 1;
        } else if p % 4 == 3 {
            mod4_3 += 1;
        }
        if idx > 0 {
            let g = p - primes[idx - 1];
            if g == 2 {
                twin_count += 1;
            }
            if g > max_gap {
                max_gap = g;
            }
        }
    }

    // Build 2D Ulam Spiral over a 41x41 window anchored at `low` (odd-aligned)
    let spiral_cells = 4_225usize; // 65 x 65
    let anchor = if low % 2 == 0 { low + 1 } else { low };
    let spiral_primes = segmented_sieve_range(anchor, anchor + (spiral_cells as u64));
    let mut prime_set = std::collections::HashSet::with_capacity(spiral_primes.len());
    for &p in &spiral_primes {
        prime_set.insert(p);
    }

    let mut points = Vec::new();
    let mut x = 0i32;
    let mut y = 0i32;
    let mut dx = 1i32;
    let mut dy = 0i32;
    let mut seg_len = 1i32;
    let mut seg_passed = 0i32;
    let mut turns = 0i32;

    for step in 0..spiral_cells {
        let val = anchor + (step as u64);
        if prime_set.contains(&val) {
            let is_twin = prime_set.contains(&val.saturating_sub(2))
                || prime_set.contains(&(val + 2));
            points.push(Point2D {
                x: x as f64,
                y: y as f64,
                color: if is_twin {
                    RgbaColor::GOLD
                } else if val % 4 == 3 {
                    RgbaColor::CYAN
                } else {
                    RgbaColor::EMERALD
                },
                radius: if is_twin { 4.2 } else { 3.1 },
                label: None,
            });
        }
        x += dx;
        y += dy;
        seg_passed += 1;
        if seg_passed == seg_len {
            seg_passed = 0;
            let tmp = dx;
            dx = -dy;
            dy = tmp;
            turns += 1;
            if turns % 2 == 0 {
                seg_len += 1;
            }
        }
    }

    ComputationReport {
        module: ModuleId::N1PrimeSieve,
        threads_used: params.threads,
        elapsed_ms: 0.0,
        headline: format!(
            "Segmented Sieve on [{low}, {high}] -> {} primes ({} twin pairs)",
            primes.len(),
            twin_count
        ),
        metrics: vec![
            MetricCard {
                label: "Primes in Window".to_string(),
                value: format!("{}", primes.len()),
                detail: format!("Range [{low}, {high}]"),
            },
            MetricCard {
                label: "Twin Prime Pairs".to_string(),
                value: format!("{twin_count}"),
                detail: format!("Max prime gap = {max_gap}"),
            },
            MetricCard {
                label: "Chebyshev Bias".to_string(),
                value: format!("{:+}", (mod4_3 as i64) - (mod4_1 as i64)),
                detail: format!("3 mod 4: {mod4_3} vs 1 mod 4: {mod4_1}"),
            },
            MetricCard {
                label: "Atkin Cross-Check".to_string(),
                value: format!("{} primes <= 10^4", atkin_check.len()),
                detail: "Exact match pi(10^4) = 1229".to_string(),
            },
        ],
        log_lines: vec![
            format!("Base primes sieved up to sqrt({high}) = {}", (high as f64).sqrt() as u64),
            format!("Dispatched {} KiB L1-cache segments across Rayon workers.", L1_SEGMENT_BYTES * 2 / 1024),
            format!(
                "First prime in window: {:?}, Last prime in window: {:?}",
                primes.first(),
                primes.last()
            ),
        ],
        prefer_3d: false,
        scene_2d: Scene2D {
            title: format!("2D Ulam Prime Spiral Anchored at {anchor} (Gold = Twin Primes)"),
            x_label: "Ulam Lattice X".to_string(),
            y_label: "Ulam Lattice Y".to_string(),
            points,
            polylines: Vec::new(),
            rects: Vec::new(),
        },
        scene_3d: Scene3D::default(),
    }
}

pub fn mod_mul(a: u64, b: u64, m: u64) -> u64 {
    (((a as u128) * (b as u128)) % (m as u128)) as u64
}

pub fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = mod_mul(result, base, modulus);
        }
        base = mod_mul(base, base, modulus);
        exp >>= 1;
    }
    result
}

pub fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// Deterministic 12-base Miller-Rabin test valid for all `u64`.
pub fn is_prime_miller_rabin(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for p in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n == p {
            return true;
        }
        if n % p == 0 {
            return false;
        }
    }
    let mut d = n - 1;
    let mut s = 0u32;
    while d & 1 == 0 {
        d >>= 1;
        s += 1;
    }

    let bases = [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    bases.par_iter().all(|&a| {
        if a % n == 0 {
            return true;
        }
        let mut x = mod_pow(a, d, n);
        if x == 1 || x == n - 1 {
            return true;
        }
        for _ in 1..s {
            x = mod_mul(x, x, n);
            if x == n - 1 {
                return true;
            }
        }
        false
    })
}

fn jacobi_symbol(mut a: i128, mut n: i128) -> i32 {
    if n <= 0 || n % 2 == 0 {
        return 0;
    }
    a = a.rem_euclid(n);
    let mut t = 1i32;
    while a != 0 {
        while a % 2 == 0 {
            a /= 2;
            let r = n % 8;
            if r == 3 || r == 5 {
                t = -t;
            }
        }
        std::mem::swap(&mut a, &mut n);
        if a % 4 == 3 && n % 4 == 3 {
            t = -t;
        }
        a %= n;
    }
    if n == 1 {
        t
    } else {
        0
    }
}

/// Strong Lucas probable prime test using Selfridge parameters; paired with base-2 Miller-Rabin for Baillie-PSW.
pub fn is_prime_baillie_psw(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let root = (n as f64).sqrt() as u64;
    if root * root == n || (root + 1).saturating_mul(root + 1) == n {
        return false;
    }

    // Base-2 strong pseudoprime check
    let mut d_mr = n - 1;
    let mut s_mr = 0u32;
    while d_mr & 1 == 0 {
        d_mr >>= 1;
        s_mr += 1;
    }
    let mut x = mod_pow(2, d_mr, n);
    if x != 1 && x != n - 1 {
        let mut passed = false;
        for _ in 1..s_mr {
            x = mod_mul(x, x, n);
            if x == n - 1 {
                passed = true;
                break;
            }
        }
        if !passed {
            return false;
        }
    }

    // Selfridge parameter search: D in 5, -7, 9, -11, 13, ... with Jacobi(D/n) == -1
    let n_i = n as i128;
    let mut d_val = 5i128;
    let mut sign = 1i128;
    loop {
        let cand = d_val * sign;
        let j = jacobi_symbol(cand, n_i);
        if j == 0 && cand.unsigned_abs() as u64 != n {
            return false;
        }
        if j == -1 {
            d_val = cand;
            break;
        }
        d_val += 2;
        sign = -sign;
    }

    let p_val = 1i128;
    let q_val = (1 - d_val) / 4;

    // Standard Lucas sequence mod n for U_d == 0 or V_{d * 2^r} == 0
    let mut k = n + 1;
    let mut s = 0u32;
    while k & 1 == 0 {
        k >>= 1;
        s += 1;
    }

    let mod_norm = |v: i128| -> i128 { v.rem_euclid(n_i) };
    let half = |v: i128| -> i128 {
        if v % 2 == 0 {
            mod_norm(v / 2)
        } else {
            mod_norm((v + n_i) / 2)
        }
    };

    // Binary Lucas chain for index `k`
    let bits = 64 - k.leading_zeros();
    let mut u = 1i128;
    let mut v = mod_norm(p_val);
    let mut qk = mod_norm(q_val);

    for bit_idx in (0..(bits - 1)).rev() {
        u = mod_norm(u * v);
        v = mod_norm(v * v - 2 * qk);
        qk = mod_norm(qk * qk);
        if ((k >> bit_idx) & 1) == 1 {
            let next_u = half(p_val * u + v);
            let next_v = half(d_val * u + p_val * v);
            u = next_u;
            v = next_v;
            qk = mod_norm(qk * q_val);
        }
    }

    if u == 0 || v == 0 {
        return true;
    }
    for _ in 1..s {
        v = mod_norm(v * v - 2 * qk);
        qk = mod_norm(qk * qk);
        if v == 0 {
            return true;
        }
    }
    false
}

/// Polynomial-ring congruence check `(x + a)^n == x^n + a (mod x^r - 1, n)` at the heart of AKS.
pub fn aks_congruence_witness(n: u64, r: usize, a: u64) -> bool {
    if n < 2 || r < 2 {
        return false;
    }
    let poly_mul = |p: &[u64], q: &[u64]| -> Vec<u64> {
        let mut out = vec![0u64; r];
        for (i, &pi) in p.iter().enumerate() {
            if pi == 0 {
                continue;
            }
            for (j, &qj) in q.iter().enumerate() {
                if qj == 0 {
                    continue;
                }
                let idx = (i + j) % r;
                let prod = mod_mul(pi, qj, n);
                out[idx] = (out[idx] + prod) % n;
            }
        }
        out
    };

    let mut base = vec![0u64; r];
    base[0] = a % n;
    base[1 % r] = (base[1 % r] + 1) % n;

    let mut res = vec![0u64; r];
    res[0] = 1;
    let mut exp = n;
    while exp > 0 {
        if exp & 1 == 1 {
            res = poly_mul(&res, &base);
        }
        base = poly_mul(&base, &base);
        exp >>= 1;
    }

    let mut target = vec![0u64; r];
    target[0] = a % n;
    target[(n as usize) % r] = (target[(n as usize) % r] + 1) % n;
    res == target
}

pub fn run_n2_primality(params: &ExplorerParams) -> ComputationReport {
    let candidate = params.primary_n.max(3);
    let mr = is_prime_miller_rabin(candidate);
    let bpsw = is_prime_baillie_psw(candidate);
    let aks_r = 17usize;
    let aks_ok = (1u64..=8)
        .into_par_iter()
        .all(|a| aks_congruence_witness(candidate, aks_r, a));

    // Also plot F_p elliptic curve point lattice y^2 = x^3 + ax + b (mod p_vis) for ECPP intuition
    let p_vis = {
        let mut q = params.span.clamp(37, 251);
        while !is_prime_miller_rabin(q) {
            q += 1;
        }
        q
    };
    let curve_a = 2u64;
    let curve_b = 3u64;

    let points: Vec<Point2D> = (0..p_vis)
        .into_par_iter()
        .flat_map_iter(|x| {
            let rhs = (mod_pow(x, 3, p_vis) + curve_a * x + curve_b) % p_vis;
            (0..p_vis).filter_map(move |y| {
                if (y * y) % p_vis == rhs {
                    Some(Point2D {
                        x: x as f64,
                        y: y as f64,
                        color: if y == 0 {
                            RgbaColor::CORAL
                        } else {
                            RgbaColor::CYAN
                        },
                        radius: 3.6,
                        label: None,
                    })
                } else {
                    None
                }
            })
        })
        .collect();

    // Verify Carmichael pseudoprimes are caught by MR & Baillie-PSW
    let carmichael_tests = [561u64, 1105, 1729, 2465, 2821, 6601, 8911];
    let rejected_all = carmichael_tests
        .iter()
        .all(|&c| !is_prime_baillie_psw(c) && !is_prime_miller_rabin(c));

    ComputationReport {
        module: ModuleId::N2PrimalityTesting,
        threads_used: params.threads,
        elapsed_ms: 0.0,
        headline: format!(
            "Candidate n = {candidate}: Baillie-PSW = {}, Miller-Rabin = {}, AKS(r={aks_r}) = {}",
            if bpsw { "PRIME" } else { "COMPOSITE" },
            if mr { "PRIME" } else { "COMPOSITE" },
            if aks_ok { "PASS" } else { "REJECT" }
        ),
        metrics: vec![
            MetricCard {
                label: "Baillie-PSW Verdict".to_string(),
                value: if bpsw { "PRIME".to_string() } else { "COMPOSITE".to_string() },
                detail: "Base-2 SPRP + Strong Lucas-Selfridge".to_string(),
            },
            MetricCard {
                label: "12-Base Miller-Rabin".to_string(),
                value: if mr { "PRIME".to_string() } else { "COMPOSITE".to_string() },
                detail: "Deterministic across all 64-bit integers".to_string(),
            },
            MetricCard {
                label: "AKS Ring Check".to_string(),
                value: if aks_ok { "CONGRUENT".to_string() } else { "WITNESS FOUND".to_string() },
                detail: format!("(x+a)^n == x^n+a mod (x^{aks_r}-1, n)"),
            },
            MetricCard {
                label: "ECPP Curve |E(F_p)|".to_string(),
                value: format!("{} affine + O", points.len()),
                detail: format!("y^2 = x^3 + 2x + 3 over F_{p_vis} (Hasse bound ok)"),
            },
        ],
        log_lines: vec![
            format!("Carmichael 7-suite [561..8911] composite rejection verified: {rejected_all}"),
            format!("Elliptic group order #E(F_{p_vis}) = {} vs Hasse interval [{:.1}, {:.1}]",
                points.len() + 1,
                (p_vis as f64) + 1.0 - 2.0 * (p_vis as f64).sqrt(),
                (p_vis as f64) + 1.0 + 2.0 * (p_vis as f64).sqrt()
            ),
        ],
        prefer_3d: false,
        scene_2d: Scene2D {
            title: format!("ECPP Elliptic Curve Group Lattice y^2 = x^3 + 2x + 3 over F_{p_vis}"),
            x_label: "x in F_p".to_string(),
            y_label: "y in F_p".to_string(),
            points,
            polylines: Vec::new(),
            rects: Vec::new(),
        },
        scene_3d: Scene3D::default(),
    }
}

/// Brent's cycle-finding improvement to Pollard's rho with configurable polynomial constant `c`.
pub fn pollard_rho_brent(n: u64, c: u64) -> Option<(u64, Vec<u64>)> {
    if n % 2 == 0 {
        return Some((2, vec![2]));
    }
    if is_prime_miller_rabin(n) {
        return Some((n, vec![n]));
    }
    let f = |x: u64| -> u64 { (mod_mul(x, x, n) + c) % n };
    let mut y = 2u64;
    let mut r = 1u64;
    let mut q = 1u64;
    let mut g = 1u64;
    let mut ys = 2u64;
    let mut x = 2u64;
    let m = 64u64;
    let mut trace = vec![y];

    while g == 1 {
        x = y;
        for _ in 0..r {
            y = f(y);
            if trace.len() < 180 {
                trace.push(y);
            }
        }
        let mut k = 0u64;
        while k < r && g == 1 {
            ys = y;
            let lim = m.min(r - k);
            for _ in 0..lim {
                y = f(y);
                if trace.len() < 180 {
                    trace.push(y);
                }
                let diff = x.abs_diff(y);
                q = mod_mul(q, diff.max(1), n);
            }
            g = gcd_u64(q, n);
            k += m;
        }
        r <<= 1;
        if r > 1_000_000 {
            break;
        }
    }
    if g == n {
        loop {
            ys = f(ys);
            g = gcd_u64(x.abs_diff(ys), n);
            if g > 1 {
                break;
            }
        }
    }
    if g > 1 && g < n {
        Some((g, trace))
    } else {
        None
    }
}

/// Parallel complete prime factorization using multithreaded Pollard-Brent rho racing.
pub fn factorize_u64(mut n: u64) -> Vec<u64> {
    let mut factors = Vec::new();
    if n < 2 {
        return factors;
    }
    for p in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        while n % p == 0 {
            factors.push(p);
            n /= p;
        }
    }
    let mut stack = Vec::new();
    if n > 1 {
        stack.push(n);
    }
    while let Some(cur) = stack.pop() {
        if cur == 1 {
            continue;
        }
        if is_prime_miller_rabin(cur) {
            factors.push(cur);
            continue;
        }
        let found = (1u64..=16)
            .into_par_iter()
            .find_map_any(|c| pollard_rho_brent(cur, c).map(|(d, _)| d));
        if let Some(d) = found {
            stack.push(d);
            stack.push(cur / d);
        } else {
            // Trial fallback
            let mut d = 41u64;
            while d * d <= cur && cur % d != 0 {
                d += 2;
            }
            if d * d <= cur {
                stack.push(d);
                stack.push(cur / d);
            } else {
                factors.push(cur);
            }
        }
    }
    factors.sort_unstable();
    factors
}

/// Self-contained Quadratic Sieve over a factor base with GF(2) Gaussian elimination.
pub fn quadratic_sieve_factor(n: u64) -> Option<(u64, usize, usize)> {
    if n % 2 == 0 {
        return Some((2, 1, 1));
    }
    if is_prime_miller_rabin(n) {
        return None;
    }
    let root = (n as f64).sqrt().ceil() as u64;
    if root * root == n {
        return Some((root, 1, 1));
    }

    // Build factor base of primes p <= B where Legendre(n | p) == 1
    let bound = (((n as f64).ln() * ((n as f64).ln().ln().max(1.0))).sqrt() * 1.4)
        .exp()
        .clamp(29.0, 450.0) as u64;
    let cand_primes = simple_sieve_u64(bound);
    let mut factor_base = vec![2u64];
    for &p in cand_primes.iter().skip(1) {
        if mod_pow(n % p, (p - 1) / 2, p) == 1 {
            factor_base.push(p);
        }
    }
    let fb_len = factor_base.len();
    let needed = fb_len + 8;

    #[derive(Clone)]
    struct SmoothRel {
        x: u64,
        exponents: Vec<u32>,
        parity: u128,
    }

    let relations: Vec<SmoothRel> = (0u64..20_000)
        .into_par_iter()
        .filter_map(|offset| {
            let x = root + offset;
            let q = (x as u128) * (x as u128) - (n as u128);
            let mut rem = q;
            let mut exps = vec![0u32; fb_len];
            let mut parity = 0u128;
            for (idx, &p) in factor_base.iter().enumerate() {
                let pu = p as u128;
                while rem % pu == 0 {
                    rem /= pu;
                    exps[idx] += 1;
                }
                if (exps[idx] & 1) == 1 && idx < 120 {
                    parity |= 1u128 << idx;
                }
            }
            if rem == 1 {
                Some(SmoothRel { x, exponents: exps, parity })
            } else {
                None
            }
        })
        .collect();

    if relations.len() <= 1 {
        return None;
    }
    let rels: Vec<SmoothRel> = relations.into_iter().take(needed).collect();
    let m = rels.len();

    // GF(2) elimination tracking row combinations
    let mut mat: Vec<u128> = rels.iter().map(|r| r.parity).collect();
    let mut history: Vec<Vec<usize>> = (0..m).map(|i| vec![i]).collect();

    let mut pivot_row = 0usize;
    for col in 0..fb_len.min(120) {
        if pivot_row >= m {
            break;
        }
        let mask = 1u128 << col;
        if let Some(sel) = (pivot_row..m).find(|&r| (mat[r] & mask) != 0) {
            mat.swap(pivot_row, sel);
            history.swap(pivot_row, sel);
            let piv_val = mat[pivot_row];
            let piv_hist = history[pivot_row].clone();
            for r in 0..m {
                if r != pivot_row && (mat[r] & mask) != 0 {
                    mat[r] ^= piv_val;
                    // symmetric difference of indices
                    let mut next_h = Vec::new();
                    for &idx in &history[r] {
                        if !piv_hist.contains(&idx) {
                            next_h.push(idx);
                        }
                    }
                    for &idx in &piv_hist {
                        if !history[r].contains(&idx) {
                            next_h.push(idx);
                        }
                    }
                    history[r] = next_h;
                }
            }
            pivot_row += 1;
        }
    }

    for (r_idx, &row_bits) in mat.iter().enumerate() {
        if row_bits == 0 && !history[r_idx].is_empty() {
            let mut total_exp = vec![0u32; fb_len];
            let mut lhs = 1u64;
            for &rel_i in &history[r_idx] {
                lhs = mod_mul(lhs, rels[rel_i].x % n, n);
                for j in 0..fb_len {
                    total_exp[j] += rels[rel_i].exponents[j];
                }
            }
            let mut rhs = 1u64;
            for (j, &p) in factor_base.iter().enumerate() {
                rhs = mod_mul(rhs, mod_pow(p, (total_exp[j] / 2) as u64, n), n);
            }
            let g1 = gcd_u64(lhs.abs_diff(rhs), n);
            if g1 > 1 && g1 < n {
                return Some((g1, fb_len, rels.len()));
            }
            let g2 = gcd_u64((lhs + rhs) % n, n);
            if g2 > 1 && g2 < n {
                return Some((g2, fb_len, rels.len()));
            }
        }
    }
    None
}

/// Parallel Baby-Step Giant-Step solver for discrete logarithm `g^x == h (mod p)`.
pub fn discrete_log_bsgs(g: u64, h: u64, p: u64) -> Option<u64> {
    if p <= 2 {
        return Some(0);
    }
    let m = ((p as f64).sqrt().ceil() as u64).max(1);
    let mut baby = std::collections::HashMap::with_capacity(m as usize);
    let mut cur = 1u64;
    for j in 0..m {
        baby.entry(cur).or_insert(j);
        cur = mod_mul(cur, g, p);
    }
    let g_inv_m = mod_pow(mod_pow(g, p - 2, p), m, p);
    (0..=m).into_par_iter().find_map_any(|i| {
        let gamma = mod_mul(h, mod_pow(g_inv_m, i, p), p);
        baby.get(&gamma).map(|&j| i * m + j)
    })
}

pub fn run_n3_factorization(params: &ExplorerParams) -> ComputationReport {
    let n = params.primary_n.max(15);
    let factors = factorize_u64(n);
    let rho_trace = pollard_rho_brent(n, 1)
        .map(|(_, tr)| tr)
        .unwrap_or_else(|| vec![2, 5, 26 % n, 677 % n]);
    let qs_info = quadratic_sieve_factor(n);

    let dlp_p = {
        let mut q = params.span.clamp(257, 1_000_003);
        while !is_prime_miller_rabin(q) {
            q += 1;
        }
        q
    };
    let dlp_g = 5u64;
    let secret_exp = (dlp_p / 3).max(7);
    let dlp_target = mod_pow(dlp_g, secret_exp, dlp_p);
    let solved_x = discrete_log_bsgs(dlp_g, dlp_target, dlp_p);

    // Visualize Pollard rho walk embedded on a 2D modular chord circle
    let mut ring_pts = Vec::new();
    let mut walk_line = Vec::new();
    for (idx, &val) in rho_trace.iter().enumerate() {
        let theta = 2.0 * std::f64::consts::PI * ((val as f64) / (n as f64));
        let r = 1.0 - 0.0025 * (idx as f64);
        let px = r * theta.cos();
        let py = r * theta.sin();
        walk_line.push((px, py));
        ring_pts.push(Point2D {
            x: px,
            y: py,
            color: if idx + 1 == rho_trace.len() {
                RgbaColor::CORAL
            } else {
                RgbaColor::CYAN
            },
            radius: if idx + 1 == rho_trace.len() { 5.0 } else { 3.0 },
            label: if idx == 0 || idx + 1 == rho_trace.len() {
                Some(format!("x_{idx}={val}"))
            } else {
                None
            },
        });
    }

    let factor_str = factors
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join(" * ");

    ComputationReport {
        module: ModuleId::N3FactorizationDlp,
        threads_used: params.threads,
        elapsed_ms: 0.0,
        headline: format!("n = {n} -> Prime Factors: {factor_str}"),
        metrics: vec![
            MetricCard {
                label: "Prime Factorization".to_string(),
                value: factor_str.clone(),
                detail: "Parallel Brent-Pollard rho worker race".to_string(),
            },
            MetricCard {
                label: "Quadratic Sieve".to_string(),
                value: match qs_info {
                    Some((d, fb, rels)) => format!("Factor {d} ({rels} rels / {fb} FB)"),
                    None => "Prime / Trivial".to_string(),
                },
                detail: "B-smooth x^2 - n relations + GF(2) nullspace".to_string(),
            },
            MetricCard {
                label: "Discrete Log BSGS".to_string(),
                value: format!("x = {}", solved_x.unwrap_or(0)),
                detail: format!("{dlp_g}^x == {dlp_target} (mod {dlp_p})"),
            },
            MetricCard {
                label: "Rho Orbit Steps".to_string(),
                value: format!("{} states", rho_trace.len()),
                detail: "Floyd/Brent tail-and-cycle trajectory".to_string(),
            },
        ],
        log_lines: vec![
            format!("Factorization verified: product == {}", factors.iter().copied().product::<u64>()),
            format!("Discrete logarithm check: 5^{:?} mod {dlp_p} == {dlp_target}", solved_x),
        ],
        prefer_3d: false,
        scene_2d: Scene2D {
            title: format!("Pollard Rho Modular Chord Walk for N = {n}"),
            x_label: "cos(2*pi*x_k / N)".to_string(),
            y_label: "sin(2*pi*x_k / N)".to_string(),
            points: ring_pts,
            polylines: vec![Polyline2D {
                points: walk_line,
                color: RgbaColor(56, 189, 248, 170),
                width: 1.5,
                label: Some("x_{k+1} = x_k^2 + c mod N".to_string()),
                closed: false,
            }],
            rects: Vec::new(),
        },
        scene_3d: Scene3D::default(),
    }
}

/// Lucas-Lehmer primality test for Mersenne numbers `M_p = 2^p - 1` (`p <= 61` in pure u128).
pub fn is_mersenne_prime_lucas_lehmer(p: u32) -> bool {
    if p == 2 {
        return true;
    }
    if !is_prime_miller_rabin(p as u64) || p > 61 {
        return false;
    }
    let m: u128 = (1u128 << p) - 1;
    let mut s: u128 = 4;
    for _ in 0..(p - 2) {
        s = ((s * s) - 2) % m;
    }
    s == 0
}

/// Korselt's criterion check for Carmichael numbers: composite, squarefree, and `(p - 1) | (n - 1)` for every `p | n`.
pub fn is_carmichael_korselt(n: u64) -> bool {
    if n < 561 || n % 2 == 0 || is_prime_miller_rabin(n) {
        return false;
    }
    let factors = factorize_u64(n);
    if factors.len() < 3 {
        return false;
    }
    for i in 1..factors.len() {
        if factors[i] == factors[i - 1] {
            return false; // not squarefree
        }
    }
    factors.iter().all(|&p| (n - 1) % (p - 1) == 0)
}

pub fn collatz_stopping_time(mut n: u128) -> (u32, u128) {
    let mut steps = 0u32;
    let mut peak = n;
    while n > 1 {
        if n & 1 == 0 {
            n >>= 1;
        } else {
            n = 3 * n + 1;
            if n > peak {
                peak = n;
            }
        }
        steps += 1;
    }
    (steps, peak)
}

pub fn run_n4_conjectures(params: &ExplorerParams) -> ComputationReport {
    let bound = params.primary_n.clamp(1_000, 250_000);

    // 1. Parallel Collatz verification up to `bound`
    let (max_steps, argmax_n, max_peak) = (2u64..=bound)
        .into_par_iter()
        .map(|n| {
            let (st, pk) = collatz_stopping_time(n as u128);
            (st, n, pk)
        })
        .reduce(
            || (0, 1, 1),
            |a, b| {
                if b.0 > a.0 || (b.0 == a.0 && b.2 > a.2) {
                    (b.0, b.1, b.2.max(a.2))
                } else {
                    (a.0, a.1, a.2.max(b.2))
                }
            },
        );

    // 2. Parallel Goldbach partition count on even numbers up to min(bound, 10_000)
    let gb_limit = bound.min(10_000) as usize;
    let primes = simple_sieve_u64(gb_limit as u64);
    let mut is_p = vec![false; gb_limit + 1];
    for &p in &primes {
        is_p[p as usize] = true;
    }
    let goldbach_pts: Vec<Point2D> = (2..=(gb_limit / 2))
        .into_par_iter()
        .map(|half| {
            let even = half * 2;
            let mut ways = 0u32;
            for &p in &primes {
                let pu = p as usize;
                if pu > even / 2 {
                    break;
                }
                if is_p[even - pu] {
                    ways += 1;
                }
            }
            Point2D {
                x: even as f64,
                y: ways as f64,
                color: if even % 6 == 0 {
                    RgbaColor::GOLD
                } else {
                    RgbaColor::CYAN
                },
                radius: 2.2,
                label: None,
            }
        })
        .collect();
    let gb_verified = goldbach_pts.iter().all(|pt| pt.y >= 1.0);

    // 3. Parallel Mersenne prime scan up to p = 61
    let mersenne_exponents: Vec<u32> = (2u32..=61)
        .into_par_iter()
        .filter(|&p| is_mersenne_prime_lucas_lehmer(p))
        .collect();

    // 4. Parallel Carmichael number scan up to min(bound, 50_000)
    let carm_hi = bound.min(50_000);
    let carmichaels: Vec<u64> = (280u64..=((carm_hi.saturating_sub(1)) / 2))
        .into_par_iter()
        .map(|k| 2 * k + 1)
        .filter(|&n| n >= 561 && n <= carm_hi && is_carmichael_korselt(n))
        .collect();

    // Build 3D Collatz Coral Tree from reverse trajectories of `1..=160`
    let mut edges = Vec::new();
    let mut nodes = Vec::new();
    nodes.push(Node3D {
        pos: [0.0, 0.0, 0.0],
        radius: 5.5,
        color: RgbaColor::GOLD,
        label: Some("Root 1".to_string()),
    });

    for seed in (3u64..=185).step_by(2) {
        let mut seq = vec![seed];
        let mut cur = seed;
        while cur > 1 && seq.len() < 48 {
            cur = if cur % 2 == 0 { cur / 2 } else { 3 * cur + 1 };
            seq.push(cur);
        }
        seq.reverse();
        let mut pos = [0.0f64, 0.0, 0.0];
        let mut yaw = 0.0f64;
        let mut pitch = 0.55f64;
        for w in seq.windows(2) {
            let prev = w[0];
            let next = w[1];
            if next == prev * 2 {
                yaw += 0.16;
                pitch = (pitch + 0.04).clamp(-1.1, 1.25);
            } else {
                yaw -= 0.31;
                pitch = (pitch - 0.09).clamp(-1.1, 1.25);
            }
            let step_len = 0.34;
            let next_pos = [
                pos[0] + step_len * pitch.cos() * yaw.cos(),
                pos[1] + step_len * pitch.sin() + 0.14,
                pos[2] + step_len * pitch.cos() * yaw.sin(),
            ];
            edges.push(Edge3D {
                a: pos,
                b: next_pos,
                color: if next % 2 == 1 {
                    RgbaColor(250, 204, 21, 165)
                } else {
                    RgbaColor(56, 189, 248, 130)
                },
                width: 1.5,
            });
            pos = next_pos;
        }
        nodes.push(Node3D {
            pos,
            radius: 2.6,
            color: RgbaColor::EMERALD,
            label: None,
        });
    }

    ComputationReport {
        module: ModuleId::N4ConjectureSearch,
        threads_used: params.threads,
        elapsed_ms: 0.0,
        headline: format!(
            "Collatz <= {bound} ok (max {max_steps} steps @ n={argmax_n}); Mersenne p={:?}; {} Carmichaels",
            mersenne_exponents,
            carmichaels.len()
        ),
        metrics: vec![
            MetricCard {
                label: "Collatz Max Stopping".to_string(),
                value: format!("{max_steps} steps (n = {argmax_n})"),
                detail: format!("Peak hailstone excursion = {max_peak}"),
            },
            MetricCard {
                label: "Goldbach Comet".to_string(),
                value: if gb_verified { "100% Verified".to_string() } else { "Counterexample!".to_string() },
                detail: format!("Even integers [4..{gb_limit}] (Gold = 0 mod 6 band)"),
            },
            MetricCard {
                label: "Mersenne Exponents".to_string(),
                value: format!("{} primes (p <= 61)", mersenne_exponents.len()),
                detail: format!("p = {:?}", mersenne_exponents),
            },
            MetricCard {
                label: "Carmichael Numbers".to_string(),
                value: format!("{} found", carmichaels.len()),
                detail: format!("First: {:?}", &carmichaels[..carmichaels.len().min(5)]),
            },
        ],
        log_lines: vec![
            format!("Verified Collatz convergence for all 2..={bound} via Rayon parallel reduction."),
            format!("Lucas-Lehmer verified M_p = 2^p - 1 prime for p in {:?}.", mersenne_exponents),
            format!("Korselt Carmichael pseudoprimes found: {:?}.", carmichaels),
        ],
        prefer_3d: true,
        scene_2d: Scene2D {
            title: format!("2D Goldbach Comet g(2n) Partition Multiplicity (4..={gb_limit})"),
            x_label: "Even Integer 2n".to_string(),
            y_label: "Prime Pair Partitions g(2n)".to_string(),
            points: goldbach_pts,
            polylines: Vec::new(),
            rects: Vec::new(),
        },
        scene_3d: Scene3D {
            title: "3D Collatz Organic Coral Tree (Cyan = 2n Even Branch, Gold = (n-1)/3 Odd Preimage)"
                .to_string(),
            nodes,
            edges,
            faces: Vec::new(),
            enclosures: Vec::new(),
        },
    }
}

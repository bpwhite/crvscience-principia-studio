#![forbid(unsafe_code)]

//! Principia Studio (`principia-studio` v1.2.0)
//! Multithreaded Safe-Rust Computational Mathematics Explorer covering:
//! 1. Computational Number Theory (`N1`..`N4`)
//! 2. Formal Verification & Automated Theorem Proving (`F1`..`F3`)
//! 3. Symbolic Computation & Computer Algebra (`S1`..`S3`)
//! 4. Numerical Analysis & Differential Equations (`D1`..`D3`)
//! 5. Combinatorics, Graph Theory & Optimization (`C1`..`C3`)
//! 6. Computational Geometry & Topology (`G1`..`G3`)
//! 7. Geometric Proofs & Visualizations (`P1`..`P3`)

pub mod cli;
pub mod combinatorics_opt;
pub mod formal_verification;
pub mod geometric_proofs;
pub mod geometry_topology;
pub mod gui;
pub mod icon;
pub mod number_theory;
pub mod numerical_pde;
pub mod render3d;
pub mod storage;
pub mod symbolic_algebra;

use rayon::ThreadPoolBuilder;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainCategory {
    NumberTheory,
    FormalVerification,
    SymbolicAlgebra,
    NumericalAnalysis,
    CombinatoricsOpt,
    GeometryTopology,
    GeometricProofs,
}

impl DomainCategory {
    pub const ALL: [DomainCategory; 7] = [
        DomainCategory::NumberTheory,
        DomainCategory::FormalVerification,
        DomainCategory::SymbolicAlgebra,
        DomainCategory::NumericalAnalysis,
        DomainCategory::CombinatoricsOpt,
        DomainCategory::GeometryTopology,
        DomainCategory::GeometricProofs,
    ];

    pub fn title(self) -> &'static str {
        match self {
            DomainCategory::NumberTheory => "1. Computational Number Theory",
            DomainCategory::FormalVerification => "2. Formal Verification & ATP",
            DomainCategory::SymbolicAlgebra => "3. Symbolic Computation & CAS",
            DomainCategory::NumericalAnalysis => "4. Numerical Analysis & PDEs",
            DomainCategory::CombinatoricsOpt => "5. Combinatorics, Graphs & Opt",
            DomainCategory::GeometryTopology => "6. Computational Geometry & Topology",
            DomainCategory::GeometricProofs => "7. Geometric Proofs & Visualizations",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleId {
    // 1. Computational Number Theory
    N1PrimeSieve,
    N2PrimalityTesting,
    N3FactorizationDlp,
    N4ConjectureSearch,
    // 2. Formal Verification & ATP
    F1InteractiveTheoremProving,
    F2SatSmtSolving,
    F3ConjectureGeneration,
    // 3. Symbolic Computation & Computer Algebra
    S1SymbolicDiffInt,
    S2GroebnerPolynomials,
    S3ExpressionSimplification,
    // 4. Numerical Analysis & Differential Equations
    D1PdeOdeSolvers,
    D2ArbitraryPrecision,
    D3LinearAlgebraScale,
    // 5. Combinatorics, Graph Theory & Optimization
    C1NpHardOptimization,
    C2LinearIntegerProgramming,
    C3ExhaustiveEnumeration,
    // 6. Computational Geometry & Topology
    G1SpatialMeshing,
    G2TopologicalDataAnalysis,
    G3KnotTheory,
    // 7. Geometric Proofs & Visualizations
    P1AreaDissectionProofs,
    P2CircleConicNonEuclidean,
    P3SolidCavalieriPolyhedra,
}

impl ModuleId {
    pub const ALL: [ModuleId; 22] = [
        ModuleId::N1PrimeSieve,
        ModuleId::N2PrimalityTesting,
        ModuleId::N3FactorizationDlp,
        ModuleId::N4ConjectureSearch,
        ModuleId::F1InteractiveTheoremProving,
        ModuleId::F2SatSmtSolving,
        ModuleId::F3ConjectureGeneration,
        ModuleId::S1SymbolicDiffInt,
        ModuleId::S2GroebnerPolynomials,
        ModuleId::S3ExpressionSimplification,
        ModuleId::D1PdeOdeSolvers,
        ModuleId::D2ArbitraryPrecision,
        ModuleId::D3LinearAlgebraScale,
        ModuleId::C1NpHardOptimization,
        ModuleId::C2LinearIntegerProgramming,
        ModuleId::C3ExhaustiveEnumeration,
        ModuleId::G1SpatialMeshing,
        ModuleId::G2TopologicalDataAnalysis,
        ModuleId::G3KnotTheory,
        ModuleId::P1AreaDissectionProofs,
        ModuleId::P2CircleConicNonEuclidean,
        ModuleId::P3SolidCavalieriPolyhedra,
    ];

    pub fn code(self) -> &'static str {
        match self {
            ModuleId::N1PrimeSieve => "n1",
            ModuleId::N2PrimalityTesting => "n2",
            ModuleId::N3FactorizationDlp => "n3",
            ModuleId::N4ConjectureSearch => "n4",
            ModuleId::F1InteractiveTheoremProving => "f1",
            ModuleId::F2SatSmtSolving => "f2",
            ModuleId::F3ConjectureGeneration => "f3",
            ModuleId::S1SymbolicDiffInt => "s1",
            ModuleId::S2GroebnerPolynomials => "s2",
            ModuleId::S3ExpressionSimplification => "s3",
            ModuleId::D1PdeOdeSolvers => "d1",
            ModuleId::D2ArbitraryPrecision => "d2",
            ModuleId::D3LinearAlgebraScale => "d3",
            ModuleId::C1NpHardOptimization => "c1",
            ModuleId::C2LinearIntegerProgramming => "c2",
            ModuleId::C3ExhaustiveEnumeration => "c3",
            ModuleId::G1SpatialMeshing => "g1",
            ModuleId::G2TopologicalDataAnalysis => "g2",
            ModuleId::G3KnotTheory => "g3",
            ModuleId::P1AreaDissectionProofs => "p1",
            ModuleId::P2CircleConicNonEuclidean => "p2",
            ModuleId::P3SolidCavalieriPolyhedra => "p3",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        let lower = s.trim().to_ascii_lowercase();
        Self::ALL
            .iter()
            .copied()
            .find(|m| m.code() == lower || m.short_name().to_ascii_lowercase() == lower)
    }

    pub fn category(self) -> DomainCategory {
        match self {
            ModuleId::N1PrimeSieve
            | ModuleId::N2PrimalityTesting
            | ModuleId::N3FactorizationDlp
            | ModuleId::N4ConjectureSearch => DomainCategory::NumberTheory,
            ModuleId::F1InteractiveTheoremProving
            | ModuleId::F2SatSmtSolving
            | ModuleId::F3ConjectureGeneration => DomainCategory::FormalVerification,
            ModuleId::S1SymbolicDiffInt
            | ModuleId::S2GroebnerPolynomials
            | ModuleId::S3ExpressionSimplification => DomainCategory::SymbolicAlgebra,
            ModuleId::D1PdeOdeSolvers
            | ModuleId::D2ArbitraryPrecision
            | ModuleId::D3LinearAlgebraScale => DomainCategory::NumericalAnalysis,
            ModuleId::C1NpHardOptimization
            | ModuleId::C2LinearIntegerProgramming
            | ModuleId::C3ExhaustiveEnumeration => DomainCategory::CombinatoricsOpt,
            ModuleId::G1SpatialMeshing
            | ModuleId::G2TopologicalDataAnalysis
            | ModuleId::G3KnotTheory => DomainCategory::GeometryTopology,
            ModuleId::P1AreaDissectionProofs
            | ModuleId::P2CircleConicNonEuclidean
            | ModuleId::P3SolidCavalieriPolyhedra => DomainCategory::GeometricProofs,
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            ModuleId::N1PrimeSieve => "N1 Segmented Prime Sieve",
            ModuleId::N2PrimalityTesting => "N2 Primality & ECPP",
            ModuleId::N3FactorizationDlp => "N3 Factorization & DLP",
            ModuleId::N4ConjectureSearch => "N4 Conjecture Verification",
            ModuleId::F1InteractiveTheoremProving => "F1 Interactive Theorem Proving",
            ModuleId::F2SatSmtSolving => "F2 Portfolio CDCL SAT/SMT",
            ModuleId::F3ConjectureGeneration => "F3 Automated Conjecture Gen",
            ModuleId::S1SymbolicDiffInt => "S1 Symbolic Diff & Risch Int",
            ModuleId::S2GroebnerPolynomials => "S2 Gröbner Bases & Varieties",
            ModuleId::S3ExpressionSimplification => "S3 E-Graph Term Rewriting",
            ModuleId::D1PdeOdeSolvers => "D1 PDE/FEM & RK45 Solvers",
            ModuleId::D2ArbitraryPrecision => "D2 BigInt NTT & Chudnovsky Pi",
            ModuleId::D3LinearAlgebraScale => "D3 Sparse Krylov, QR & SVD",
            ModuleId::C1NpHardOptimization => "C1 TSP, Coloring & Bin Packing",
            ModuleId::C2LinearIntegerProgramming => "C2 Simplex, Barrier & MILP",
            ModuleId::C3ExhaustiveEnumeration => "C3 Symmetry-Broken Enumeration",
            ModuleId::G1SpatialMeshing => "G1 3D Hull, Voronoi & Delaunay",
            ModuleId::G2TopologicalDataAnalysis => "G2 Vietoris-Rips TDA & Betti",
            ModuleId::G3KnotTheory => "G3 3D Knots & Jones Polynomial",
            ModuleId::P1AreaDissectionProofs => "P1 Area & Dissection Proofs",
            ModuleId::P2CircleConicNonEuclidean => "P2 Circle, Conic & Hyperbolic",
            ModuleId::P3SolidCavalieriPolyhedra => "P3 Cavalieri & 3D Polyhedra",
        }
    }

    pub fn is_3d_primary(self) -> bool {
        matches!(
            self,
            ModuleId::N4ConjectureSearch
                | ModuleId::F1InteractiveTheoremProving
                | ModuleId::F2SatSmtSolving
                | ModuleId::S1SymbolicDiffInt
                | ModuleId::S2GroebnerPolynomials
                | ModuleId::D1PdeOdeSolvers
                | ModuleId::D3LinearAlgebraScale
                | ModuleId::C2LinearIntegerProgramming
                | ModuleId::G1SpatialMeshing
                | ModuleId::G2TopologicalDataAnalysis
                | ModuleId::G3KnotTheory
                | ModuleId::P3SolidCavalieriPolyhedra
        )
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            ModuleId::N1PrimeSieve => {
                "Cache-friendly 32 KiB segmented Eratosthenes & Atkin comparison up to 10^12 windows (2D Ulam Spiral)"
            }
            ModuleId::N2PrimalityTesting => {
                "Miller-Rabin, Baillie-PSW, AKS polynomial ring & ECPP elliptic witness chains (2D F_p Lattice)"
            }
            ModuleId::N3FactorizationDlp => {
                "Parallel Brent-Pollard rho, Quadratic Sieve GF(2) matrix & Baby-Step Giant-Step DLP (2D Cycle/Matrix)"
            }
            ModuleId::N4ConjectureSearch => {
                "Parallel Collatz stopping records, Goldbach comet, Lucas-Lehmer & Carmichael scan (3D Collatz Coral)"
            }
            ModuleId::F1InteractiveTheoremProving => {
                "Sequent/Tactic proof kernel & Four-Color / Kepler / FLT formal dependency DAGs (3D Universe DAG)"
            }
            ModuleId::F2SatSmtSolving => {
                "Diversified portfolio CDCL SAT & LRA-SMT on Pythagorean Triples & Ramsey graphs (3D Simplex Shell)"
            }
            ModuleId::F3ConjectureGeneration => {
                "Ramanujan-style continued-fraction constant hunter & spectral graph search (2D Pareto Phase Plot)"
            }
            ModuleId::S1SymbolicDiffInt => {
                "Exact AST calculus, Hermite/Liouvillian Risch-Norman integration & complex phase surface (3D Domain Surface)"
            }
            ModuleId::S2GroebnerPolynomials => {
                "Multivariate Q[x,y,z] Buchberger Gröbner bases under Lex/GRevLex & algebraic varieties (3D Variety)"
            }
            ModuleId::S3ExpressionSimplification => {
                "Bounded E-graph equality saturation & trigonometric/algebraic canonicalizer (2D E-Class Enclosures)"
            }
            ModuleId::D1PdeOdeSolvers => {
                "Parallel 2D Vorticity/Heat FDM, P1 Galerkin FEM & RK45 3-Body/Lorenz ensembles (3D Field/Attractor)"
            }
            ModuleId::D2ArbitraryPrecision => {
                "Karatsuba & NTT multiplication + parallel Binary-Splitting Chudnovsky Pi (2D Pi Walk & Scaling)"
            }
            ModuleId::D3LinearAlgebraScale => {
                "CSR Sparse CG, Lanczos, Implicit-Shift QR & Golub-Kahan SVD stability (3D Singular Ellipsoid)"
            }
            ModuleId::C1NpHardOptimization => {
                "Exact Branch-and-Bound vs Parallel Island Annealing on TSP, Coloring & Packing (2D Tour & Bounds)"
            }
            ModuleId::C2LinearIntegerProgramming => {
                "Two-Phase Simplex, Log-Barrier Central Path & MILP branch-and-bound (3D Feasible Polytope)"
            }
            ModuleId::C3ExhaustiveEnumeration => {
                "Parallel symmetry-broken Redelmeier polyominoes, Latin/Magic squares & graphs (2D Tiled Gallery)"
            }
            ModuleId::G1SpatialMeshing => {
                "Exact Bowyer-Watson 2D Delaunay/Voronoi & 3D Quickhull polyhedra (3D Convex Mesh + 2D Dual)"
            }
            ModuleId::G2TopologicalDataAnalysis => {
                "Parallel Vietoris-Rips filtration, Z2 boundary reduction & Betti barcodes (3D Complex + Barcodes)"
            }
            ModuleId::G3KnotTheory => {
                "3D tubular prime/torus knots, planar writhe, Alexander & Kauffman/Jones invariants (3D Knot Ribbon)"
            }
            ModuleId::P1AreaDissectionProofs => {
                "Visual area proofs: Pythagorean dissection, Euclid I.47 windmill, geometric series tiling & Nicomachus cubes (2D + 3D Prisms)"
            }
            ModuleId::P2CircleConicNonEuclidean => {
                "Archimedes pi exhaustion, Thales/Ptolemy cyclic theorems, 3D Dandelin spheres & Poincare disk geodesics (2D + 3D Cone)"
            }
            ModuleId::P3SolidCavalieriPolyhedra => {
                "Archimedes 1:2:3 Cavalieri slice proof, Platonic Euler/Descartes defect, 3D Viviani simplex & Pappus torus (3D + 2D Slices)"
            }
        }
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.short_name(), self.code())
    }
}

/// Unified user-configurable parameters shared across CLI, GUI, and Workset saves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerParams {
    pub module: ModuleId,
    pub threads: usize,
    pub preset_index: usize,
    pub primary_n: u64,
    pub span: u64,
    pub secondary_param: f64,
    pub expression_input: String,
    pub seed: u64,
}

impl Default for ExplorerParams {
    fn default() -> Self {
        Self::default_for(ModuleId::N1PrimeSieve)
    }
}

impl ExplorerParams {
    pub fn default_for(module: ModuleId) -> Self {
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let (primary_n, span, secondary_param, expr) = match module {
            ModuleId::N1PrimeSieve => (1_000_000_000_000, 100_000, 0.0, String::new()),
            ModuleId::N2PrimalityTesting => (1_000_000_007, 561, 0.0, String::new()),
            ModuleId::N3FactorizationDlp => (1_040_301_499, 65537, 0.0, String::new()),
            ModuleId::N4ConjectureSearch => (50_000, 2_500, 0.0, String::new()),
            ModuleId::F1InteractiveTheoremProving => (0, 12, 0.0, "(P -> Q) -> (Q -> R) -> (P -> R)".to_string()),
            ModuleId::F2SatSmtSolving => (60, 6, 0.0, String::new()),
            ModuleId::F3ConjectureGeneration => (80, 12, 0.0, String::new()),
            ModuleId::S1SymbolicDiffInt => (2, 40, 1.0, "x^2 * exp(x) + sin(x) * cos(x)".to_string()),
            ModuleId::S2GroebnerPolynomials => (0, 28, 1.0, "x^2 + y^2 + z^2 - 4; x*y - 1; z - x^2".to_string()),
            ModuleId::S3ExpressionSimplification => (6, 20, 0.0, "sin(x)^2 + cos(x)^2 + (x + 0) * 1 + (x^2 - 1)/(x - 1)".to_string()),
            ModuleId::D1PdeOdeSolvers => (48, 400, 0.015, String::new()),
            ModuleId::D2ArbitraryPrecision => (500, 2048, 0.0, String::new()),
            ModuleId::D3LinearAlgebraScale => (64, 40, 0.25, String::new()),
            ModuleId::C1NpHardOptimization => (26, 800, 0.985, String::new()),
            ModuleId::C2LinearIntegerProgramming => (0, 32, 0.1, String::new()),
            ModuleId::C3ExhaustiveEnumeration => (8, 4, 0.0, String::new()),
            ModuleId::G1SpatialMeshing => (64, 48, 1.0, String::new()),
            ModuleId::G2TopologicalDataAnalysis => (72, 24, 1.15, String::new()),
            ModuleId::G3KnotTheory => (0, 160, 0.22, String::new()),
            ModuleId::P1AreaDissectionProofs => (3, 4, 1.0, String::new()),
            ModuleId::P2CircleConicNonEuclidean => (3, 48, 0.72, String::new()),
            ModuleId::P3SolidCavalieriPolyhedra => (4, 36, 0.45, String::new()),
        };
        Self {
            module,
            threads,
            preset_index: 0,
            primary_n,
            span,
            secondary_param,
            expression_input: expr,
            seed: 42,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RgbaColor(pub u8, pub u8, pub u8, pub u8);

impl RgbaColor {
    pub const CYAN: Self = Self(56, 189, 248, 255);
    pub const GOLD: Self = Self(250, 204, 21, 255);
    pub const EMERALD: Self = Self(52, 211, 153, 255);
    pub const CORAL: Self = Self(251, 113, 133, 255);
    pub const VIOLET: Self = Self(167, 139, 250, 255);
    pub const SLATE: Self = Self(148, 163, 184, 200);
    pub const AMBER: Self = Self(245, 158, 11, 255);

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
    pub color: RgbaColor,
    pub radius: f32,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polyline2D {
    pub points: Vec<(f64, f64)>,
    pub color: RgbaColor,
    pub width: f32,
    pub label: Option<String>,
    pub closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect2D {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub fill: RgbaColor,
    pub stroke: RgbaColor,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Scene2D {
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub points: Vec<Point2D>,
    pub polylines: Vec<Polyline2D>,
    pub rects: Vec<Rect2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node3D {
    pub pos: [f64; 3],
    pub radius: f32,
    pub color: RgbaColor,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge3D {
    pub a: [f64; 3],
    pub b: [f64; 3],
    pub color: RgbaColor,
    pub width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face3D {
    pub verts: [[f64; 3]; 3],
    pub normal: [f64; 3],
    pub color: RgbaColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enclosure3D {
    pub center: [f64; 3],
    pub half_extents: [f64; 3],
    pub label: String,
    pub color: RgbaColor,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Scene3D {
    pub title: String,
    pub nodes: Vec<Node3D>,
    pub edges: Vec<Edge3D>,
    pub faces: Vec<Face3D>,
    pub enclosures: Vec<Enclosure3D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCard {
    pub label: String,
    pub value: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationReport {
    pub module: ModuleId,
    pub threads_used: usize,
    pub elapsed_ms: f64,
    pub headline: String,
    pub metrics: Vec<MetricCard>,
    pub log_lines: Vec<String>,
    pub prefer_3d: bool,
    pub scene_2d: Scene2D,
    pub scene_3d: Scene3D,
}

/// Runs any module inside a dedicated Rayon thread pool configured to `params.threads`.
pub fn execute_module(params: &ExplorerParams) -> ComputationReport {
    let threads = params.threads.clamp(1, 128);
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("Failed to construct Rayon worker pool");

    let t0 = Instant::now();
    let mut report = pool.install(|| match params.module {
        ModuleId::N1PrimeSieve => number_theory::run_n1_prime_sieve(params),
        ModuleId::N2PrimalityTesting => number_theory::run_n2_primality(params),
        ModuleId::N3FactorizationDlp => number_theory::run_n3_factorization(params),
        ModuleId::N4ConjectureSearch => number_theory::run_n4_conjectures(params),
        ModuleId::F1InteractiveTheoremProving => formal_verification::run_f1_itp(params),
        ModuleId::F2SatSmtSolving => formal_verification::run_f2_sat_smt(params),
        ModuleId::F3ConjectureGeneration => formal_verification::run_f3_conjecture_gen(params),
        ModuleId::S1SymbolicDiffInt => symbolic_algebra::run_s1_symbolic_calculus(params),
        ModuleId::S2GroebnerPolynomials => symbolic_algebra::run_s2_groebner(params),
        ModuleId::S3ExpressionSimplification => symbolic_algebra::run_s3_rewriting(params),
        ModuleId::D1PdeOdeSolvers => numerical_pde::run_d1_pde_ode(params),
        ModuleId::D2ArbitraryPrecision => numerical_pde::run_d2_arbitrary_precision(params),
        ModuleId::D3LinearAlgebraScale => numerical_pde::run_d3_linear_algebra(params),
        ModuleId::C1NpHardOptimization => combinatorics_opt::run_c1_np_hard(params),
        ModuleId::C2LinearIntegerProgramming => combinatorics_opt::run_c2_lp_milp(params),
        ModuleId::C3ExhaustiveEnumeration => combinatorics_opt::run_c3_enumeration(params),
        ModuleId::G1SpatialMeshing => geometry_topology::run_g1_spatial_meshing(params),
        ModuleId::G2TopologicalDataAnalysis => geometry_topology::run_g2_tda(params),
        ModuleId::G3KnotTheory => geometry_topology::run_g3_knot_theory(params),
        ModuleId::P1AreaDissectionProofs => geometric_proofs::run_p1_area_dissection(params),
        ModuleId::P2CircleConicNonEuclidean => {
            geometric_proofs::run_p2_circle_conic_non_euclidean(params)
        }
        ModuleId::P3SolidCavalieriPolyhedra => {
            geometric_proofs::run_p3_solid_cavalieri_polyhedra(params)
        }
    });
    report.threads_used = threads;
    report.elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
    report
}

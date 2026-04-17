# APS-V1-0002 — Metrics Catalog

**Version**: 1.0.0
**Status**: Normative reference — referenced by [01_spec.md](./01_spec.md)

This document defines all architectural metrics used by the Architecture Fitness Functions standard. Each metric includes its mathematical formula, original author, scope, industry-standard thresholds, and rationale for inclusion.

---

## Availability Matrix

| Metric | Dimension | Computed by Topology | Derivable | Needs New Tooling |
|--------|-----------|---------------------|-----------|-------------------|
| Cyclomatic Complexity | MT01 | Yes | — | — |
| Cognitive Complexity | MT01 | Yes | — | — |
| Halstead Volume | MT01 | Yes | — | — |
| Halstead Difficulty | MT01 | Yes | — | — |
| Halstead Effort | MT01 | Yes | — | — |
| Halstead Time | MT01 | Yes | — | — |
| Halstead Bugs | MT01 | Yes | — | — |
| LOC / SLOC | MT01 | Yes | — | — |
| Maintainability Index | MT01 | Schema defined | — | Calculation needed |
| Afferent Coupling (Ca) | MD01 | Yes | — | — |
| Efferent Coupling (Ce) | MD01 | Yes | — | — |
| Instability (I) | MD01 | Yes | — | — |
| Abstractness (A) | MD01 | Yes | — | — |
| Distance from Main Sequence (D) | MD01 | Yes | — | — |
| Composite Coupling | MD01 | Yes | — | — |
| Coupling Density | MD01 | Yes | — | — |
| Fan-in / Fan-out | MD01 | — | Yes (call graph) | — |
| Depth of Inheritance Tree | ST01 | — | — | Yes |
| Coupling Between Objects | ST01 | — | — | Yes |
| Response For a Class | ST01 | — | — | Yes |
| Weighted Methods per Class | ST01 | — | Partial | — |
| Lack of Cohesion in Methods | ST01 | — | — | Yes |

---

## 1. Maintainability Metrics (MT01)

### 1.1 Cyclomatic Complexity (CC)

**Author**: McCabe, T.J. (1976). "A Complexity Measure." *IEEE Transactions on Software Engineering*, SE-2(4), 308–320.

**Formula**:

```
CC(f) = E - N + 2P
```

Where E = edges in the control flow graph, N = nodes, P = connected components. Simplified for a single function:

```
CC(f) = 1 + d
```

Where d = number of decision points (if, elif, for, while, case, &&, ||, catch, ternary).

**Scope**: Function

**Measures**: The number of linearly independent paths through source code. Equals the minimum number of test cases needed for full branch coverage.

**Why it matters**: CC directly predicts testability and defect density. Functions with high CC have exponentially more execution paths, making exhaustive testing impractical. It protects **testability**, **maintainability**, and **reliability**.

**Thresholds**:

| Range | Risk Level | Source |
|-------|-----------|--------|
| 1–10 | Low — simple, well-structured | McCabe (1976); NIST SP 500-235 |
| 11–20 | Moderate — more complex | NIST SP 500-235 (1996) |
| 21–50 | High — complex, difficult to test | SEI/CMU |
| 51+ | Very high — untestable, error-prone | Industry consensus |

**Default rule**: `max = 10` (error), with `max = 15` as a common relaxed alternative.

**Computed by topology**: Yes — `complexity.rs`, decision-point counting method.

---

### 1.2 Cognitive Complexity (CogC)

**Author**: Campbell, G.A. / SonarSource (2017). "Cognitive Complexity: A new way of measuring understandability." Specification v1.4 (2021).

**Formula**: Incremental algorithm — no closed-form expression.

For each control structure *s* at nesting depth *d*:

```
increment(s) = B(s) + N(s) × d
```

Where:
- B(s) = 1 if *s* is a basic increment (break in linear flow)
- N(s) = 1 if *s* carries a nesting penalty
- d = current nesting depth

Basic increments (+1, no nesting penalty): logical operator sequence breaks (&&, ||)
Nesting increments (+1 + depth): if, for, while, catch, switch, lambda, nested function

**Scope**: Function

**Measures**: How difficult code is for a *human* to understand. Unlike CC which measures paths, CogC measures cognitive load by penalizing nesting depth.

**Why it matters**: A deeply nested `if` inside a `for` inside a `while` has the same CC as three flat `if` statements, yet is dramatically harder to understand. CogC catches this. It protects **readability** and **maintainability**.

**Thresholds**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| 0–5 | Easy to understand | SonarSource |
| 6–15 | Moderate effort | SonarSource (default rule threshold) |
| 16–25 | Hard to understand, refactor recommended | SonarSource |
| 26+ | Very hard, strong refactoring signal | SonarSource |

**Default rule**: `max = 15` (error).

**Computed by topology**: Yes — `complexity.rs`, incremental nesting-penalty algorithm.

---

### 1.3 Halstead Metrics Suite

**Author**: Halstead, M.H. (1977). *Elements of Software Science*. Elsevier North-Holland.

**Primitive measurements**:

| Symbol | Name | Definition |
|--------|------|-----------|
| η₁ | Distinct operators | Count of unique operators |
| η₂ | Distinct operands | Count of unique operands (identifiers, literals) |
| N₁ | Total operators | Total occurrences of all operators |
| N₂ | Total operands | Total occurrences of all operands |

**Derived metrics**:

| Metric | Formula | Interpretation |
|--------|---------|---------------|
| **Vocabulary (η)** | η = η₁ + η₂ | Size of the "alphabet" used |
| **Program Length (N)** | N = N₁ + N₂ | Total token count |
| **Volume (V)** | V = N × log₂(η) | Information content in bits |
| **Difficulty (D)** | D = (η₁ / 2) × (N₂ / η₂) | Error-proneness |
| **Effort (E)** | E = D × V | Total mental effort |
| **Time (T)** | T = E / 18 seconds | Estimated coding time (Stroud's number = 18) |
| **Estimated Bugs (B)** | B = V / 3000 | Estimated defect count (empirically derived) |

**Scope**: Function

**Measures**: The information-theoretic complexity of code. Volume measures content density; Difficulty measures error-proneness; Effort combines both into a workload estimate.

**Why it matters**: Halstead metrics provide a language-neutral size/complexity measure based on information theory. Volume is more meaningful than raw LOC because it accounts for vocabulary richness. Difficulty flags code that is error-prone. Effort estimates maintenance cost. Volume feeds into the Maintainability Index.

**Thresholds (Volume)**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| < 100 | Simple function | SEI/CMU |
| 100–300 | Moderate | Industry practice |
| 300–1000 | Complex | Verifysoft Technology |
| > 1000 | Very complex, refactor | Verifysoft Technology |

**Thresholds (Estimated Bugs)**:

| Range | Interpretation |
|-------|---------------|
| < 0.5 | Low risk |
| 0.5–1.0 | Moderate risk |
| > 1.0 | High risk — likely contains defects |

**Default rules**: Volume `max = 1000` (warning); Estimated Bugs `max = 1.0` (warning).

**Computed by topology**: Yes — `complexity.rs` (collection) and `lib.rs` (`HalsteadMetrics::calculate()`).

---

### 1.4 Lines of Code (LOC / SLOC)

**Author**: No single author. Formalized by Brooks (1975), Boehm (1981, COCOMO), and the SEI.

**Variants**:

| Variant | Definition |
|---------|-----------|
| **LOC** | Total lines including blanks and comments |
| **SLOC** | Non-blank, non-comment source lines |
| **CLOC** | Comment lines |
| **Comment Ratio** | CLOC / LOC |

**Scope**: File, Module

**Measures**: Raw size. Despite being the simplest metric, LOC is one of the strongest predictors of defect counts — larger code has more places for bugs to hide.

**Why it matters**: LOC protects against **bloat** and serves as a normalization denominator for other metrics (defects per KLOC). Fitness thresholds on file LOC prevent god-objects and mega-files.

**Thresholds (per file)**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| < 200 | Good | Martin, *Clean Code* (2008) |
| 200–500 | Moderate | McConnell, *Code Complete* (2004) |
| 500+ | Large, consider splitting | Industry consensus |

**Default rule**: File LOC `max = 500` (warning).

**Computed by topology**: Yes — `complexity.rs`, `count_lines()`.

---

### 1.5 Maintainability Index (MI)

**Author**: Coleman, D., Ash, D., Lowther, B., & Oman, P. (1994). "Using Metrics to Evaluate Software System Maintainability." *IEEE Computer*, 27(8), 44–49.

**Formula (original)**:

```
MI = 171 - 5.2 × ln(avgV) - 0.23 × avgCC - 16.2 × ln(avgLOC)
```

Where:
- avgV = average Halstead Volume per function
- avgCC = average Cyclomatic Complexity per function
- avgLOC = average lines of code per function

**Formula (Microsoft 0–100 scale)**:

```
MI = max(0, (171 - 5.2 × ln(V) - 0.23 × CC - 16.2 × ln(LOC)) × 100 / 171)
```

**Scope**: File, Module

**Measures**: A composite score estimating how maintainable code is. Higher is better. Combines volume, complexity, and size into a single number.

**Why it matters**: MI is the quintessential composite fitness function — it catches degradation that no single metric would flag. The coefficients were derived through regression against expert maintainability assessments. It protects **maintainability** as a holistic characteristic.

**Thresholds (0–100 scale)**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| 85–100 | High maintainability | Microsoft Visual Studio |
| 65–84 | Moderate maintainability | Microsoft Visual Studio |
| 0–64 | Low maintainability, refactor needed | Microsoft Visual Studio |

**Default rule**: MI `min = 40` (warning), `min = 20` (error).

**Computed by topology**: Schema defined (`maintainability_index` in proto); calculation at aggregation stage.

---

## 2. Modularity & Coupling Metrics (MD01)

### 2.1 Afferent Coupling (Ca)

**Author**: Martin, R.C. (1994). "OO Design Quality Metrics: An Analysis of Dependencies." *OOPSLA*. Expanded in: *Agile Software Development* (2003), Chapter 20.

**Formula**:

```
Ca(M) = |{ X : X depends on M, X ≠ M }|
```

The count of external modules that depend on module M (incoming dependencies).

**Scope**: Module

**Measures**: How many other modules depend on this module — its "responsibility" in the system. The term "afferent" comes from neuroscience (nerves carrying signals *toward* the brain).

**Why it matters**: High Ca means many modules depend on you — changes have high **blast radius**. Per Martin's Stable Dependencies Principle, high-Ca modules should be stable and therefore abstract. Ca protects **stability** awareness and identifies modules where changes carry the most risk.

**Thresholds**: Relative to system size. No universal numeric threshold. Common heuristics: Ca > 20 warrants review.

**Default rule**: `max = 40` (warning) — context-dependent.

**Computed by topology**: Yes — `lib.rs`, `MartinMetrics::calculate()`.

---

### 2.2 Efferent Coupling (Ce)

**Author**: Martin, R.C. (1994/2003). Same as Ca.

**Formula**:

```
Ce(M) = |{ X : M depends on X, X ≠ M }|
```

The count of external modules that module M depends on (outgoing dependencies).

**Scope**: Module

**Measures**: How many other modules this module depends on — its "fragility" or sensitivity to external changes.

**Why it matters**: High Ce means this module has many reasons to change because any dependency might change. Ce protects **changeability** and **isolation**. Fitness thresholds on Ce prevent modules from accumulating too many dependencies — a primary cause of architectural erosion.

**Thresholds**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| ≤ 20 | Acceptable | Lakos (1996), *Large-Scale C++ Software Design* |
| > 20 | Excessive, review needed | Industry practice |

**Default rule**: `max = 20` (error).

**Computed by topology**: Yes — `lib.rs`, `MartinMetrics::calculate()`.

---

### 2.3 Instability (I)

**Author**: Martin, R.C. (1994/2003).

**Formula**:

```
I(M) = Ce / (Ca + Ce)
```

When Ca + Ce = 0, I = 0 by convention. Range: [0, 1].

- I = 0: Maximally **stable** — only depended upon, depends on nothing.
- I = 1: Maximally **unstable** — depends on others, nobody depends on it.

**Scope**: Module

**Measures**: The module's susceptibility to change. Stable modules resist change because many others depend on them. Unstable modules are easy to change because few depend on them.

**Why it matters**: Martin's **Stable Dependencies Principle (SDP)** states that dependencies should run in the direction of stability. Instability quantifies this. The key fitness insight is that I should be *appropriate* for the module's role — framework modules should be stable (low I), application modules should be unstable (high I). Extremes in either direction indicate potential problems.

**Thresholds**:

| Range | Interpretation |
|-------|---------------|
| I < 0.1 | Overly rigid — may resist necessary change |
| 0.1 ≤ I ≤ 0.9 | Healthy range |
| I > 0.9 | Overly unstable — may be too volatile |

**Default rule**: `min = 0.1`, `max = 0.9` (warning).

**Computed by topology**: Yes — `lib.rs`.

---

### 2.4 Abstractness (A)

**Author**: Martin, R.C. (1994/2003).

**Formula**:

```
A(M) = Na / Nc
```

Where Na = number of abstract types (traits, interfaces, abstract classes) in the module, Nc = total types. When Nc = 0, A = 0. Range: [0, 1].

**Scope**: Module

**Measures**: The ratio of abstractions to concretions. High abstractness means the module defines contracts rather than implementations.

**Why it matters**: Martin's **Stable Abstractions Principle (SAP)** states that stable modules should be abstract — if many things depend on you and you're concrete, any implementation change breaks everyone. This protects the **Dependency Inversion Principle** and **architectural resilience**.

**Thresholds**: Meaningful only in combination with Instability via Distance from Main Sequence.

**Computed by topology**: Yes — `lib.rs`, via `TypeInfo.is_abstract`.

---

### 2.5 Distance from the Main Sequence (D)

**Author**: Martin, R.C. (1994/2003).

**Formula**:

```
D(M) = |A + I - 1|
```

Range: [0, 1]. D = 0 means the module sits on the "Main Sequence" — the ideal line where A + I = 1.

**The Main Sequence** is the line from (A=1, I=0) to (A=0, I=1):

```
A (Abstractness)
1.0 ┌─────────────────────────┐
    │ Zone of       ╲         │
    │ Uselessness    ╲        │
    │ (abstract,      ╲ Main  │
    │  nobody uses)    ╲ Seq. │
    │                   ╲     │
    │          Zone of   ╲    │
    │          Pain       ╲   │
    │  (concrete, stable =  ╲ │
    │   rigid, hard to change)│
0.0 └─────────────────────────┘
   0.0          I          1.0
```

**The two danger zones**:
- **Zone of Pain** (low A, low I): Concrete AND stable. Rigid — hard to change because many depend on it, yet contains implementation details. Examples: database schemas, utility libraries without abstractions.
- **Zone of Uselessness** (high A, high I): Abstract AND unstable. Over-engineered interfaces nobody depends on.

**Scope**: Module

**Measures**: How far a module deviates from the ideal balance between abstractness and instability.

**Why it matters**: D is arguably the single most powerful module-level fitness function. It detects both the Zone of Pain and Zone of Uselessness with a single number. It protects **architectural balance** and **long-term evolvability**.

**Thresholds**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| 0.0–0.1 | Excellent | Martin (2003) |
| 0.1–0.3 | Good | Industry practice |
| 0.3–0.5 | Warning | APS CI01 substandard |
| 0.5–0.7 | Poor | Industry practice |
| 0.7–1.0 | Critical | APS CI01 substandard |

**Default rule**: `max = 0.5` (warning), `max = 0.7` (error).

**Computed by topology**: Yes — `lib.rs`.

---

### 2.6 Composite Coupling

**Author**: APS-V1-0001 (custom metric).

**Formula**:

```
composite(A, B) = 0.30 × norm(import) + 0.25 × norm(call) + 0.20 × norm(symbol) + 0.15 × norm(type) + 0.10 × norm(change)
```

Normalization uses logarithmic percentile ranking: ln(v + 1) followed by percentile rank. When change coupling is unavailable, weights redistribute proportionally.

**Scope**: Module-pair (pairwise coupling matrix)

**Measures**: Overall coupling strength between two modules, combining multiple structural signals.

**Why it matters**: No single coupling signal tells the full story. Import coupling misses runtime coupling; call coupling misses type dependencies; change coupling captures implicit coupling invisible to static analysis. The weighted composite captures the full picture.

**Computed by topology**: Yes — coupling matrix pipeline.

---

### 2.7 Coupling Density

**Author**: General graph theory, applied by Baldwin & Clark (2000) and Martin (2003).

**Formula**:

```
coupling_density = |E| / (n × (n - 1))
```

Where n = number of modules, E = dependency edges.

**Scope**: System

**Measures**: What fraction of all possible inter-module dependencies actually exist. Rising density is the primary symptom of architectural erosion.

**Thresholds**: Highly context-dependent. Key metric is **trend** — density should not increase over time.

**Computed by topology**: Yes — snapshot summaries.

---

### 2.8 Fan-in / Fan-out

**Author**: Henry, S. & Kafura, D. (1981). "Software Structure Metrics Based on Information Flow." *IEEE TSE*, SE-7(5). Also: Yourdon, E. & Constantine, L. (1979). *Structured Design*.

**Formula**:

```
Fan-in(f) = |{ g : g calls f }|    (callers)
Fan-out(f) = |{ g : f calls g }|    (callees)
```

Henry-Kafura complexity: `HK(f) = length(f) × (fan_in × fan_out)²`

**Scope**: Function, Module

**Measures**: Fan-in = reuse/centrality. Fan-out = coordination complexity. The Henry-Kafura product identifies hub nodes where information flow converges and diverges.

**Why it matters**: High fan-out indicates a function orchestrating too many things (violating Single Responsibility). Fan-in is the function-level analog of Ca; fan-out of Ce.

**Thresholds**:

| Metric | Warning | Source |
|--------|---------|--------|
| Fan-out | > 7 | Card & Glass (1990) |
| Fan-out | > 15 | Conservative industry practice |

**Default rule**: Fan-out `max = 15` (warning).

**Computed by topology**: Derivable from call graph (in-degree = fan-in, out-degree = fan-out). Not yet surfaced as named metrics.

---

## 3. Structural Integrity Metrics (ST01)

These metrics are from the Chidamber-Kemerer (CK) suite — the most cited object-oriented metrics in software engineering. They require class-level analysis not currently provided by the topology standard.

**Status**: All CK metrics are marked as **planned**. They will be available when class-level analysis is added to APS-V1-0001 or implemented as a dedicated analyzer.

### 3.1 Depth of Inheritance Tree (DIT)

**Author**: Chidamber, S.R. & Kemerer, C.F. (1994). "A Metrics Suite for Object Oriented Design." *IEEE TSE*, 20(6), 476–493.

**Formula**:

```
DIT(C) = max path length from C to the root of its inheritance hierarchy
```

**Scope**: Class

**Measures**: How deep a class is in the inheritance tree. Deeper classes inherit more behavior but are harder to understand and more sensitive to parent changes.

**Why it matters**: Deep inheritance creates the **fragile base class problem** — changes to base classes cascade unpredictably. Modern practice favors composition over inheritance. DIT protects **understandability** and **resilience to change**.

**Thresholds**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| 0–2 | Good | Chidamber & Kemerer (1994) |
| 3–4 | Moderate | Industry consensus |
| 5–6 | Deep, fragile base class risk | Lorenz & Kidd (1994) |
| 7+ | Excessive | Rosenberg (1998), NASA/GSFC |

**Default rule**: `max = 4` (warning), `max = 6` (error).

**Computed by topology**: No — needs inheritance hierarchy tracking.

---

### 3.2 Coupling Between Objects (CBO)

**Author**: Chidamber & Kemerer (1994).

**Formula**:

```
CBO(C) = |{ D : C is coupled to D, D ≠ C }|
```

Coupling = uses methods, fields, or types of another class (bidirectional).

**Scope**: Class

**Measures**: The breadth of a class's coupling — how many other classes it interacts with.

**Why it matters**: CBO is the most direct measure of class isolation. Empirical studies (Basili et al., 1996; Subramanyam & Krishnan, 2003) consistently find CBO to be one of the strongest predictors of defect probability. It protects **modularity**, **testability**, and **reusability**.

**Thresholds**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| 0–5 | Good | Industry practice |
| 6–14 | Moderate | Rosenberg (1998) |
| 14+ | Excessive | Sahraoui et al. (2000) |

**Default rule**: `max = 14` (warning).

**Computed by topology**: No — needs class-level coupling tracking.

---

### 3.3 Response For a Class (RFC)

**Author**: Chidamber & Kemerer (1994).

**Formula**:

```
RFC(C) = |M(C)| + |{ m : m called by some method in M(C), m ∉ M(C) }|
```

The response set = all methods of C plus all methods directly called by C's methods.

**Scope**: Class

**Measures**: The potential scope of behavior triggered by a single message to the class. High RFC means a single call can trigger a wide cascade.

**Why it matters**: RFC protects **testability** (more response means more test cases) and **predictability**. It is complementary to CBO — CBO measures breadth of coupling, RFC measures depth of the response chain.

**Thresholds**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| < 50 | Good | Lorenz & Kidd (1994) |
| 50–100 | Moderate | Industry practice |
| > 100 | Excessive | Rosenberg (1998) |

**Default rule**: `max = 50` (warning).

**Computed by topology**: No — needs class-level method resolution.

---

### 3.4 Weighted Methods per Class (WMC)

**Author**: Chidamber & Kemerer (1994).

**Formula**:

```
WMC(C) = Σ c_i  for all methods m_i in class C
```

Where c_i = complexity of method m_i. Typically c_i = CC(m_i). If unweighted (c_i = 1), WMC = method count.

**Scope**: Class

**Measures**: The total complexity burden of a class. Predicts maintenance effort and identifies god classes.

**Why it matters**: High WMC indicates a class doing too much or having individually too-complex methods. When weighted by CC, it combines size and complexity into a single class-level indicator. It protects against **god classes** and excessive **class-level cognitive load**.

**Thresholds**:

| Range (CC-weighted) | Interpretation | Source |
|---------------------|---------------|--------|
| < 20 | Good | Rosenberg (1998) |
| 20–50 | Moderate | Industry practice |
| > 50 | Excessive | Rosenberg (NASA/GSFC) |

**Default rule**: `max = 50` (warning).

**Computed by topology**: Partially — `total_cyclomatic` per file/module exists. Class-level aggregation needs new tooling.

---

### 3.5 Lack of Cohesion in Methods (LCOM)

**Authors**:
- LCOM1: Chidamber & Kemerer (1994).
- LCOM-HS: Henderson-Sellers, B. (1996). *Object-Oriented Metrics: Measures of Complexity*. Prentice Hall.
- LCOM3/4: Hitz, M. & Montazeri, B. (1995). "Measuring Coupling and Cohesion in Object-Oriented Systems."

**Formula (Henderson-Sellers, recommended)**:

```
LCOM-HS(C) = ((1/|F|) × Σ m(f_j) - |M|) / (1 - |M|)
```

Where |M| = number of methods, |F| = number of instance variables, m(f_j) = number of methods accessing variable f_j. Range: [0, 1] where 0 = perfect cohesion, 1 = no cohesion.

**Formula (LCOM3 — connected components)**:

```
LCOM3(C) = number of connected components in the method–variable access graph
```

If LCOM3 = 1, the class is cohesive. If LCOM3 = 2, the class should probably be split into 2 classes.

**Scope**: Class

**Measures**: Whether a class's methods work together on shared data or are independent. Low cohesion suggests the class should be split.

**Why it matters**: LCOM detects classes violating the **Single Responsibility Principle** by doing unrelated things. A class with low cohesion is harder to understand, harder to reuse, and harder to test. It protects **cohesion** and **modularity**.

**Thresholds (Henderson-Sellers, 0–1)**:

| Range | Interpretation | Source |
|-------|---------------|--------|
| 0.0–0.3 | Cohesive | Henderson-Sellers (1996) |
| 0.3–0.5 | Moderate | Industry practice |
| 0.5–0.8 | Low cohesion, consider splitting | Industry practice |
| 0.8–1.0 | Very low, class should be split | Industry practice |

**Default rule**: LCOM-HS `max = 0.8` (warning).

**Computed by topology**: No — needs method-to-field access tracking.

---

## 4. External Dimension Metrics

Metrics for SC01, LG01, AC01, PF01, and AV01 are defined by their respective substandards and the external tools that produce them. The adapter contract (spec section 9) normalizes these into the standard metrics format.

External dimension metrics are not defined in this catalog because they are tool-specific and evolve with their respective ecosystems. Each substandard's `01_spec.md` defines the metrics it consumes and the adapter contract for normalizing them.

---

## References

- Basili, V.R., Briand, L.C., & Melo, W.L. (1996). "A Validation of Object-Oriented Design Metrics as Quality Indicators." *IEEE TSE*, 22(10).
- Boehm, B.W. (1981). *Software Engineering Economics*. Prentice Hall.
- Card, D.N. & Glass, R.L. (1990). *Measuring Software Design Quality*. Prentice Hall.
- Chidamber, S.R. & Kemerer, C.F. (1994). "A Metrics Suite for Object Oriented Design." *IEEE TSE*, 20(6), 476–493.
- Coleman, D. et al. (1994). "Using Metrics to Evaluate Software System Maintainability." *IEEE Computer*, 27(8), 44–49.
- Ford, N., Parsons, R., & Kua, P. (2017). *Building Evolutionary Architectures*. O'Reilly Media.
- Halstead, M.H. (1977). *Elements of Software Science*. Elsevier.
- Henderson-Sellers, B. (1996). *Object-Oriented Metrics: Measures of Complexity*. Prentice Hall.
- Henry, S. & Kafura, D. (1981). "Software Structure Metrics Based on Information Flow." *IEEE TSE*, SE-7(5).
- Lakos, J. (1996). *Large-Scale C++ Software Design*. Addison-Wesley.
- Lorenz, M. & Kidd, J. (1994). *Object-Oriented Software Metrics*. Prentice Hall.
- Martin, R.C. (2003). *Agile Software Development, Principles, Patterns, and Practices*. Prentice Hall.
- McCabe, T.J. (1976). "A Complexity Measure." *IEEE TSE*, SE-2(4), 308–320.
- McConnell, S. (2004). *Code Complete*. 2nd Edition. Microsoft Press.
- Rosenberg, L.H. (1998). "Applying and Interpreting Object Oriented Metrics." NASA/GSFC.
- Sahraoui, H.A. et al. (2000). "A metrics suite for measuring reusability of OO software components."
- Subramanyam, R. & Krishnan, M.S. (2003). "Empirical Analysis of CK Metrics for Object-Oriented Design Complexity." *IEEE TSE*, 29(4).
- Watson, A.H. & McCabe, T.J. (1996). *Structured Testing*. NIST SP 500-235.

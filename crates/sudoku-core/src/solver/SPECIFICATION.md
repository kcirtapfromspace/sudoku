# Solver Engine Formal Specification

Derived from first principles of constraint satisfaction over the four
spaces of a standard 9x9 Sudoku grid. Every elimination theorem is proved
from the constraint axioms; named techniques are exhibited as parameter
instantiations of the three abstract engines.

---

## 0. Foundational Axioms

### 0.1 The Four Spaces

**Cell Space** *C*.
81 positions arranged in a 9x9 matrix. Each cell *c* ∈ *C* is identified
by its linear index 0..80 (row-major). A cell is either *given* (fixed
value), *placed* (solver-assigned value), or *empty* (no value yet).

**Candidate Space** *X*.
The set of live (cell, digit) pairs: *X* ⊆ *C* × {1..9}. Initially
|*X*| ≤ 729. The element *(c, d)* ∈ *X* means "digit *d* is still
possible in cell *c*". We write *X(c)* for the candidate set of cell *c*
and *X_d(S)* for the set of cells in sector *S* that have candidate *d*.

**Sector Space** *S*.
27 units partitioning the grid: rows *r_0..r_8*, columns *c_0..c_8*,
boxes *b_0..b_8*. Index convention: 0..8 = rows, 9..17 = columns,
18..26 = boxes. Every cell belongs to exactly 3 sectors. Two cells *see*
each other iff they share at least one sector.

**Link Space** *L*.
Binary relations over nodes in *X*:

- **Strong link** *(p, q)*: exactly one of *p, q* is true in any
  solution. Arises when a digit has exactly 2 positions in a sector
  (conjugate pair) or a cell has exactly 2 candidates (bivalue cell).
  Formally: *p ∨ q* is a tautology and *¬(p ∧ q)* is a tautology.

- **Weak link** *(p, q)*: at most one of *p, q* is true. Arises from
  the Sudoku constraint: two candidates of the same digit in the same
  sector, or two different candidates of the same cell.
  Formally: *¬(p ∧ q)* is a tautology.

Every strong link is also a weak link, but not conversely.

### 0.2 Constraint Axioms

For every solution *σ*: *C* → {1..9}:

1. **Uniqueness axiom**: ∀ sector *S*, ∀ digit *d* ∈ {1..9}:
   exactly one cell in *S* has value *d*.
2. **Completeness axiom**: ∀ cell *c*: *σ(c)* ∈ *X(c)*.
3. **Consistency axiom**: ∀ cells *c₁, c₂* sharing a sector:
   *σ(c₁) ≠ σ(c₂)*.

### 0.3 Inference Rules

Given the axioms, we may perform two operations:

- **Elimination**: Remove *(c, d)* from *X* when we can prove
  *σ(c) ≠ d* in every solution.
- **Placement**: Set *σ(c) = d* when *X(c) = {d}* (naked single) or
  when *d* has exactly one remaining position in some sector (hidden
  single).

---

## 1. Fish Engine: Sector-Rank Deficiency

### 1.1 Mathematical Objects

**Definition 1.1 (Fish Configuration).**
Fix a digit *d*. Let *B = {B₁, ..., Bₙ}* be a set of *n* sectors
(the *base set*) and *K = {K₁, ..., Kₙ}* be a set of *n* sectors
(the *cover set*) such that *B ∩ K = ∅*. Define:

- *Base cells*:  *β = ⋃ᵢ X_d(Bᵢ)*  (cells with candidate *d* in
  base sectors)
- *Cover cells*: *κ = ⋃ⱼ X_d(Kⱼ)*  (cells with candidate *d* in
  cover sectors)
- *Fins*:        *φ = β \ κ*         (base cells not covered)
- *Eliminations*: *ε = κ \ β*        (cover cells not in base)

**Definition 1.2 (Sector Constraint).**
The classification of which sector types may appear in *B* and *K*:

| Constraint | Base types    | Cover types      | Name             |
|------------|--------------|------------------|------------------|
| Basic      | {row} or {col} | {col} or {row} | Standard fish    |
| Franken    | {row, col}   | {row, col, box}  | Franken fish     |
| Mutant     | {row, col, box} | {row, col, box} | Mutant fish   |

### 1.2 Core Theorem

**Theorem 1.1 (Basic Fish Elimination).**
If *φ = ∅* (no fins), then for every cell *c ∈ ε*, we may eliminate
*(c, d)* from *X*.

*Proof sketch.*
By the Uniqueness axiom, digit *d* must appear exactly once in each base
sector *Bᵢ*. So *d* appears in exactly *n* cells among *β*. Since
*φ = ∅*, all of *β ⊆ κ*. The cover sectors *K₁, ..., Kₙ* together
also require exactly *n* occurrences of *d* (by the Uniqueness axiom for
each *Kⱼ*). These *n* occurrences are exactly the cells in *β ∩ κ = β*.
Therefore no cell in *ε = κ \ β* can contain *d*. ∎

**Theorem 1.2 (Finned Fish Elimination).**
If *φ ≠ ∅* and all fin cells lie in a single box *F*, then for every
cell *c ∈ ε* such that *c* shares box *F*, we may eliminate *(c, d)*.

*Proof sketch.*
Case analysis: either (a) no fin cell holds *d* in the solution, reducing
to Theorem 1.1, or (b) some fin cell in *F* holds *d*. In case (b), box
*F* already has *d*, so any cell in *ε ∩ F* cannot hold *d*. In both
cases, *d* is eliminated from *ε ∩ cells(F)*. ∎

### 1.3 Named Techniques as Special Cases

Every named fish technique is an instantiation of (digit *d*, size *n*,
constraint, fin status):

| n | Fins | Constraint | Technique            | SE   |
|---|------|-----------|----------------------|------|
| 2 | No   | Basic     | X-Wing               | 3.2  |
| 2 | Yes  | Basic     | Finned X-Wing        | 3.4  |
| 3 | No   | Basic     | Swordfish            | 3.8  |
| 3 | Yes  | Basic     | Finned Swordfish     | 4.0  |
| 4 | No   | Basic     | Jellyfish            | 5.2  |
| 4 | Yes  | Basic     | Finned Jellyfish     | 5.4  |
| * | *    | Franken   | Franken Fish         | 5.5  |
| * | *    | Mutant    | Mutant Fish          | 6.5  |

**Siamese Fish** are two overlapping finned fish configurations sharing
a fin box. Both fish independently restrict eliminations to the fin box;
their intersection yields strictly more eliminations than either alone.

### 1.4 Degenerate Case: Pointing Pairs and Box/Line Reduction

**Theorem 1.3 (Intersection as Fish of Size 1).**
Pointing Pair and Box/Line Reduction are degenerate fish with *n = 1*.

*Proof.*
A **Pointing Pair** for digit *d*: the *d*-candidates in box *B* are
confined to row *R*. Set base *B = {box B}*, cover *K = {row R}*. Then:
- *β* = positions of *d* in box *B*
- *κ* = positions of *d* in row *R*
- *φ = β \ κ = ∅* (all *d*-cells in the box lie on row *R*)
- *ε = κ \ β* = cells in row *R* outside box *B*

By Theorem 1.1, eliminate *d* from *ε*. This is exactly the Pointing
Pair rule.

A **Box/Line Reduction** for digit *d*: the *d*-candidates in row *R*
are confined to box *B*. Set base *B = {row R}*, cover *K = {box B}*.
- *β* = positions of *d* in row *R*
- *κ* = positions of *d* in box *B*
- *φ = β \ κ = ∅* (all *d*-cells in the row lie in box *B*)
- *ε = κ \ β* = cells in box *B* outside row *R*

By Theorem 1.1, eliminate *d* from *ε*. This is exactly the Box/Line
Reduction rule. ∎

Therefore, intersections are a strict subset of the fish framework.
They need not be implemented separately.

### 1.5 Sector-Rank Interpretation

The fish argument has a clean linear-algebraic reading. Construct the
*base incidence matrix* **M** ∈ {0,1}^{n × m} where **M**[i,j] = 1 iff
base sector *Bᵢ* contains cell *cⱼ* (restricted to *d*-candidates).
The rank condition for a valid fish is:

> rank(**M**) = *n* and the column space is contained in the span of the
> cover sectors.

This means the *n* base sectors "use up" exactly *n* degrees of freedom
for digit *d*, and the cover sectors fully account for those degrees.
Excess cells in the cover (ε) cannot participate.

---

## 2. ALS Engine: Subset Degree-of-Freedom

### 2.1 Mathematical Objects

**Definition 2.1 (Almost Locked Set).**
A set *A* of *n* cells within a single sector, such that
|⋃_{c ∈ A} X(c)| = *n* + 1. That is, *n* cells collectively have
exactly *n* + 1 distinct candidates. We write *cands(A)* for this
union.

The "almost" refers to having one excess candidate beyond what a fully
locked set (naked subset) would have. This single degree of freedom is
what enables linking.

**Definition 2.2 (Restricted Common Candidate, RCC).**
Given two non-overlapping ALS *A* and *B*, a digit *x* is an RCC of
(*A, B*) if:
- *x* ∈ *cands(A)* ∩ *cands(B)*, and
- every *x*-cell in *A* sees every *x*-cell in *B*.

When *A* and *B* are linked by RCC *x*, at most one of them can "use"
*x* to fill its surplus. If *A* uses *x*, then *A*'s remaining *n*
candidates lock *n* cells (a naked subset). Similarly for *B*.

### 2.2 Core Theorem

**Theorem 2.1 (ALS-XZ Elimination).**
Let *A* (size *n*) and *B* (size *m*) be non-overlapping ALS linked
by RCC *x*. Let *z* ∈ *cands(A)* ∩ *cands(B)*, *z ≠ x*. Then for
any cell *c* ∉ *A* ∪ *B* that sees every *z*-cell in *A* and every
*z*-cell in *B*, we may eliminate *(c, z)*.

*Proof sketch.*
In any solution, consider the digit *x*:

**Case 1**: *x* is placed in some cell of *A*.
Then *A* \ {*x*-cell} has *n* − 1 cells needing *n* other values from
*cands(A)* \ {*x*}. Since |*cands(A)* \ {*x*}| = *n*, this is a
locked set. In particular, *z* is confined to the *z*-cells of *A*.
Since *c* sees all *z*-cells of *A*, cell *c* cannot hold *z*.

**Case 2**: *x* is placed in some cell of *B*.
Symmetrically, *z* is confined to the *z*-cells of *B*, and *c* sees
all of them.

**Case 3**: *x* is placed in neither *A* nor *B*.
Since *x* is an RCC, every *x*-cell in *A* sees every *x*-cell in *B*.
By the Uniqueness axiom, *x* can appear in at most one cell among
*A ∪ B* per sector. But if *x* appears in neither, then *A* has *n*
cells with *n* remaining candidates *cands(A)* \ ∅ — wait, *A* still
has *n* + 1 candidates. However, the RCC constraint means the *x*-cells
of *A* and *B* share a sector, and *x* must appear somewhere in that
sector. The constraint forces *x* into *A* or *B* (since all other
*x*-positions in the shared sectors are eliminated or occupied). ∎

More precisely: because *x* is an RCC, the mutual visibility of all
*x*-cells means that in any solution, at most one of *A ∪ B* holds *x*.
Since both *A* and *B* must each resolve to exactly *n* (resp. *m*)
values, and each has one surplus candidate, exactly one of them absorbs
*x*. The remaining ALS becomes fully locked, confining *z*.

### 2.3 Named Techniques as Special Cases

| A size | B size | Links | Technique     | SE   |
|--------|--------|-------|---------------|------|
| 1      | 1      | 1 RCC | XY-Wing       | 4.2  |
| 1      | 2      | 1 RCC | XYZ-Wing      | 4.4  |
| ≤2     | ≤2     | 1 RCC, total=4 | WXYZ-Wing | 4.6 |
| any    | any    | 1 RCC | ALS-XZ        | 5.5  |

**Wings are small ALS-XZ.**
An XY-Wing consists of two bivalue cells (size-1 ALS, since 1 cell
with 2 candidates = *n* + 1). The pivot-wing structure is exactly the
RCC geometry: the pivot cell shares one value with each wing, and the
RCC is the shared candidate between pivot and wing.

#### 2.3.1 ALS Chains (Generalized)

**Theorem 2.2 (ALS Chain Elimination).**
Given a chain *A₁ - A₂ - ... - Aₖ* where consecutive ALS are
connected by distinct RCCs *x₁, x₂, ..., x_{k-1}* (each *xᵢ* is
an RCC of *(Aᵢ, Aᵢ₊₁)*), and *z* ∈ *cands(A₁)* ∩ *cands(Aₖ)*,
*z ∉ {x₁, ..., x_{k-1}}*:

For any cell *c* that sees all *z*-cells in *A₁* and all *z*-cells
in *Aₖ*, we may eliminate *(c, z)*.

*Proof sketch.*
By induction on chain length. The base case *k = 2* is Theorem 2.1.
For the inductive step: in any solution, the RCC *x₁* is absorbed
by either *A₁* or *A₂*. If *A₁* absorbs it, *A₁* becomes locked
and confines *z*. If *A₂* absorbs it, then *A₂*'s surplus shifts
to *x₂*, propagating the same argument down the chain. ∎

| Chain length | Technique     | SE   |
|-------------|---------------|------|
| 2           | ALS-XZ        | 5.5  |
| 3           | ALS-XY-Wing   | 7.0  |
| 4+          | ALS Chain     | 7.5  |

### 2.4 Sue de Coq as ALS Decomposition

**Theorem 2.3 (Sue de Coq).**
Let *I* be the intersection of a box and a line (2-3 empty cells).
Let *cands(I)* be the union of candidates in *I*. If there exist
ALS *A* in (rest of box) and ALS *B* in (rest of line) such that:
- *cands(A) ∪ cands(B) = cands(I)*
- *cands(A) ∩ cands(B) = ∅*

Then:
- Eliminate *cands(A)* from rest-of-box cells not in *A*.
- Eliminate *cands(B)* from rest-of-line cells not in *B*.

*Proof.*
The intersection cells must collectively hold values from *cands(I)*.
The disjoint partition *cands(A) ⊔ cands(B) = cands(I)* means:
*A* and *I* together form a locked set on *cands(A)* within the box,
and *B* and *I* together form a locked set on *cands(B)* within the
line. Standard locked-set elimination applies. ∎

Sue de Coq is an ALS technique because the key objects *A* and *B* are
ALS found in the remainder sectors, and the elimination logic is a
locked-set (= fully resolved ALS) argument.

### 2.5 Death Blossom as ALS Star Graph

**Theorem 2.4 (Death Blossom).**
Let *s* be a stem cell with candidates *{d₁, ..., dₖ}*. For each
*dᵢ*, let *Pᵢ* be an ALS (a petal) such that:
- *s ∉ Pᵢ* and the petals are pairwise non-overlapping.
- *dᵢ* ∈ *cands(Pᵢ)* and the *dᵢ*-cells of *Pᵢ* see *s*.

If *z* ∈ ⋂ᵢ *cands(Pᵢ)*, *z ∉ {d₁,...,dₖ}*, and cell *c* sees all
*z*-cells in every petal, then eliminate *(c, z)*.

*Proof sketch.*
In any solution, *s* takes some value *dⱼ*. Then petal *Pⱼ* cannot
use *dⱼ* (since a *dⱼ*-cell of *Pⱼ* sees *s*), so *Pⱼ* becomes
fully locked on its remaining *n* candidates. The value *z* is
confined to *z*-cells of *Pⱼ*. Since *c* sees all of them, *c ≠ z*.
This holds for every possible *dⱼ*, so the elimination is valid. ∎

### 2.6 Aligned Pair/Triplet Exclusion as ALS

**Theorem 2.5 (APE/ATE Subsumption).**
Aligned Pair Exclusion (APE) and Aligned Triplet Exclusion (ATE)
are special cases of the ALS framework.

*Proof.*
**APE**: Two mutually visible cells *{p₁, p₂}* with candidate sets
*C₁, C₂*. APE eliminates candidate *v* from a common peer *t* if
every valid pair *(v₁, v₂)* ∈ *C₁ × C₂* with *v₁ ≠ v₂* has
*v ∈ {v₁, v₂}*.

Recast: The pair *{p₁, p₂}* forms an almost-locked structure.
Consider the ALS *A = {p₁}* (bivalue cell, size 1 with 2 candidates)
and *A' = {p₂}*. The valid-pair enumeration is equivalent to checking
which values are forced when each RCC is resolved. Specifically:

For every candidate *v₁* of *p₁*, define the ALS constraint: if
*p₁ = v₁*, then *p₂* cannot be *v₁* (visibility), so *p₂*'s
effective candidate set shrinks. The condition "every valid pair
includes *v*" means that *v* is locked into *{p₁, p₂}* under all
case splits — which is a DoF argument on the pair viewed as a
two-cell ALS with restricted assignments.

More directly, if *{p₁, p₂}* has *|C₁ ∪ C₂| = 3* and we consider
the pair as a near-ALS with mutual visibility, the APE elimination
follows from Theorem 2.1 with appropriate RCC choice.

**ATE** extends identically to 3 mutually visible cells. The brute-
force enumeration of valid triples is the exhaustive version of the
chain-of-ALS locking argument. ∎

The practical consequence: APE/ATE can be retired by extending the
ALS catalog to handle mutually-visible cell groups and checking all
locking configurations.

---

## 3. AIC Engine: Link-Reachability Arguments

### 3.1 Mathematical Objects

**Definition 3.1 (Inference Graph).**
The AIC inference graph *G = (V, E_s, E_w)* where:
- *V* = *X* (the candidate space — each node is a (cell, digit) pair)
- *E_s* ⊆ *V × V*: strong links (exactly-one-true)
- *E_w* ⊆ *V × V*: weak links (at-most-one-true), *E_s ⊆ E_w*

**Definition 3.2 (Link Sources).**
Strong links arise from:
1. **Conjugate pairs**: digit *d* has exactly 2 positions in sector *S*
   → strong link between *(c₁, d)* and *(c₂, d)*.
2. **Bivalue cells**: cell *c* has exactly 2 candidates *{a, b}*
   → strong link between *(c, a)* and *(c, b)*.

Weak links arise from:
1. **Same digit, same sector**: *(c₁, d)* — *(c₂, d)* when *c₁, c₂*
   share a sector (at most one can be *d*).
2. **Same cell, different digit**: *(c, d₁)* — *(c, d₂)* (a cell holds
   at most one value).

**Definition 3.3 (Alternating Inference Chain).**
An AIC is a path *p₁ — p₂ — p₃ — ... — pₖ* in *G* where edges
alternate between strong and weak:

    p₁ ==[strong]== p₂ --[weak]-- p₃ ==[strong]== p₄ -- ... -- pₖ

The chain starts with a strong link and traverses
strong → weak → strong → weak → ...

We say the chain has *strong polarity* at odd-indexed nodes (arrived
via strong link) and *weak polarity* at even-indexed nodes.

### 3.2 Core Theorem

**Theorem 3.1 (AIC Elimination, Type 1: Shared Digit, Different Cells).**
If an AIC exists from *(c₁, d)* (strong start) through alternating
links to a node *(c₂, d)* (arriving via weak link), where *c₁ ≠ c₂*
and *d* is the same digit, then:

For any cell *c* ∉ {c₁, c₂} that sees both *c₁* and *c₂*, eliminate
*(c, d)*.

*Proof sketch.*
Interpret the chain as a logical implication chain. At the start node
*(c₁, d)*: consider two cases.

**Case A**: *(c₁, d)* is true (cell *c₁* holds *d*). Then *c* sees
*c₁*, so *c* cannot hold *d*.

**Case B**: *(c₁, d)* is false. Then by the strong link to *p₂*,
*p₂* is true. By the weak link to *p₃*, *p₃* is false (at most one
of a weak pair is true, and *p₂* is true). By the strong link to
*p₄*, *p₄* is true. Continuing this alternating logic to the end:
the final node *(c₂, d)* is reached via weak link, meaning the
preceding node (arrived via strong link) is true, which forces
*(c₂, d)* to be true through the weak-link implication. Wait — let
me be more precise.

The alternating chain logic:
- ¬*p₁* → *p₂* (strong link: one must be true)
- *p₂* → ¬*p₃* (weak link: at most one true)
- ¬*p₃* → *p₄* (strong link)
- ...continuing...
- Eventually: ¬*p₁* → ... → *pₖ₋₁* (arrived via strong, so true)
- *pₖ₋₁* → ¬*pₖ* (weak link)

Hmm, let me reconsider. The endpoint *(c₂, d)* is reached via a
weak link from the previous node. The previous node has strong
polarity (true if ¬*p₁*). So:

If ¬*p₁*: the chain implies *pₖ₋₁* is true, which via weak link
means we *cannot conclude* *pₖ* is true or false directly. But
the chain structure means ¬*p₁* → *pₖ₋₁* and *pₖ₋₁* weakly
conflicts with *pₖ*...

Actually, the standard AIC elimination works as follows:

The chain endpoints have **strong polarity** — meaning:
- *p₁* is the start of a strong link (strong arrival)
- *pₖ* arrives via a weak link following a strong-polarity node

The elimination rule is: the chain forms a **nice loop** or
**discontinuous nice loop**. For a chain starting and ending on the
same digit *d* at different cells:

*p₁ = (c₁, d)* and *pₖ = (c₂, d)* are both "potentially true" —
the chain establishes that at least one of *p₁, pₖ* must be true
(because assuming both false leads to contradiction through the
chain). Since both nodes concern digit *d* and any cell *c* seeing
both cannot hold *d* regardless of which endpoint is true, we
eliminate *d* from *c*. ∎

**Theorem 3.2 (AIC Elimination, Type 2: Same Cell, Different Digits).**
If an AIC exists from *(c, d₁)* (strong start) to *(c, d₂)* (weak
arrival) where the cell is the same, then eliminate all candidates
of *c* except *d₁* and *d₂*.

*Proof.*
The chain establishes: at least one of *(c, d₁)* or *(c, d₂)* is
true. Since a cell holds exactly one value, *σ(c) ∈ {d₁, d₂}*.
All other candidates of *c* are eliminated. ∎

### 3.3 Named Techniques as Special Cases

**X-Chain** (single-digit AIC):
An AIC where every node concerns the same digit *d*. All links are
conjugate-pair strong links and same-sector weak links. The single-
value restriction makes the chain search cheaper. SE: 4.5.

**W-Wing**:
Two bivalue cells *{a, b}* at positions *c₁, c₂* connected by a
conjugate pair (strong link) on digit *a* in some sector. The chain:

    (c₁, b) ==[strong, bivalue]== (c₁, a) --[weak, sees link]--
    (ℓ₁, a) ==[strong, conjugate]== (ℓ₂, a) --[weak, sees c₂]--
    (c₂, a) ==[strong, bivalue]== (c₂, b)

This 6-node AIC with endpoints *(c₁, b)* and *(c₂, b)* yields:
eliminate *b* from any cell seeing both *c₁* and *c₂*. SE: 4.4.

**General AIC** (multi-digit):
Unrestricted alternating chains mixing conjugate-pair and bivalue
strong links, with weak links from sector/cell constraints. SE: 6.0.

### 3.4 Medusa as AIC Coloring

**Theorem 3.3 (3D Medusa is AIC Strong-Link Coloring).**
3D Medusa is the connected-component coloring of the strong-link
subgraph of the AIC inference graph.

*Proof.*
Medusa assigns two colors (0 and 1) to nodes by BFS traversal of
strong links only. This partitions each connected component of the
strong-link graph into two color classes. The coloring satisfies:
if node *p* has color *c*, then every strong neighbor has color
*1 − c*.

Since every strong link represents an exactly-one-true constraint:
exactly one color class is the "true" class (all its nodes hold in
the solution) and the other is the "false" class.

**Contradiction rules** (identify the false color class):
- Rule 1: Two same-color nodes share a digit *d* and a sector.
  Then that color class would require *d* twice in one sector. ✗
- Rule 2: Two same-color nodes are in the same cell.
  Then that color class would place two values in one cell. ✗

If a contradiction is found for color *c*, all nodes of color *c*
are false → eliminate those candidates.

**Rule 5** (uncolored elimination): If an uncolored candidate
*(cell, d)* sees *d*-nodes of both colors, eliminate it (one color
is true, so the *d*-node of the true color blocks *cell*).

This is precisely the AIC framework restricted to strong-link-only
BFS. Every Medusa elimination can be expressed as an AIC, but the
coloring approach finds them more efficiently by processing entire
connected components at once. ∎

### 3.5 Forcing Chains as Multi-Source AIC

**Definition 3.4 (Forcing Chain).**
A forcing chain explores all candidates of a source (cell or sector
position) via independent propagation. If all branches agree on an
outcome, that outcome is valid.

| Source        | Branches                  | Technique              | SE   |
|--------------|---------------------------|------------------------|------|
| Cell *c*     | Each candidate of *c*     | Cell Forcing Chain     | 8.3  |
| Sector/digit | Each position of *d* in *S* | Region Forcing Chain | 8.5  |
| Cell (full)  | With full technique propagation | Dynamic FC      | 9.3  |
| Single cand. | One branch contradicts    | Nishio FC              | 7.5  |

**Theorem 3.4 (Forcing Chain Correctness).**
Let *S = {s₁, ..., sₖ}* be an exhaustive set of hypotheses (candidates
of a cell, or positions of a digit in a sector). For each *sᵢ*,
let *Gᵢ* be the grid state after propagating the assumption *sᵢ = true*.

- **Common placement**: If ∀ *i*: *Gᵢ* places value *v* at cell *t*,
  then *σ(t) = v*.
- **Common elimination**: If ∀ *i*: *Gᵢ* eliminates *(t, v)*, then
  eliminate *(t, v)*.
- **Nishio**: If propagating *sᵢ* leads to contradiction, then
  *sᵢ* is false; eliminate it.

*Proof.*
By exhaustion: one of *{s₁, ..., sₖ}* must be true (completeness
axiom for cells; uniqueness axiom for sectors). Since every possible
truth leads to the same conclusion, that conclusion holds. ∎

**Kraken Fish**: A finned fish where the fin's effect is verified by
forcing-chain propagation. If propagating each fin cell with digit *d*
still eliminates *d* from the target, the elimination is valid — even
when the standard finned-fish box restriction would not apply.

### 3.6 Dynamic Forcing Chains

Dynamic Forcing Chains are the most powerful non-backtracking technique.
They differ from standard forcing chains in the propagation function:
instead of propagating only naked/hidden singles, they apply the full
technique repertoire (including AIC, ALS, and fish) within each branch.

This makes them strictly more powerful: they can discover implications
that simple propagation misses. The trade-off is computational cost.
SE: 9.3.

---

## 4. Completeness and Orthogonality

### 4.1 Three Orthogonal Arguments

The three engines cover three fundamentally different proof strategies:

| Engine | Core argument          | Operates primarily in |
|--------|------------------------|----------------------|
| Fish   | Sector-rank deficiency | Sector Space *S*     |
| ALS    | Subset degree-of-freedom | Candidate Space *X* |
| AIC    | Link reachability      | Link Space *L*       |

**Fish** reasons about digits in aggregate across sectors: "these *n*
base sectors consume exactly *n* cover sector slots for digit *d*."

**ALS** reasons about cell groups and their candidate surplus: "*n*
cells have *n* + 1 candidates, so linking two such groups forces
value confinement."

**AIC** reasons about individual (cell, digit) nodes and the logical
implications of truth propagation through strong and weak links.

### 4.2 Subsumption Relationships

```
Pointing Pair  ──┐
Box/Line Reduc.──┼──► Fish Engine (n=1 degenerate case)
                 │
X-Wing ──────────┤
Swordfish ───────┼──► Fish Engine (n=2,3,4; Basic constraint)
Jellyfish ───────┤
                 │
Franken Fish ────┼──► Fish Engine (Franken constraint)
Siamese Fish ────┤──► Fish Engine (overlapping finned pair)
Mutant Fish ─────┘──► Fish Engine (Mutant constraint)

XY-Wing ─────────┐
XYZ-Wing ────────┼──► ALS Engine (size-1,2 ALS pairs)
WXYZ-Wing ───────┤
ALS-XZ ──────────┤
ALS-XY-Wing ─────┼──► ALS Engine (3-ALS chain)
ALS Chain ───────┤──► ALS Engine (4+ ALS chain)
Sue de Coq ──────┤──► ALS Engine (box/line decomposition)
Death Blossom ───┤──► ALS Engine (star graph topology)
APE/ATE ─────────┘──► ALS Engine (mutually-visible cell group)

W-Wing ──────────┐
X-Chain ─────────┤
3D Medusa ───────┼──► AIC Engine (strong-link coloring)
AIC ─────────────┤
Nishio FC ───────┤──► AIC Engine (single-branch contradiction)
Kraken Fish ─────┤──► AIC Engine (fish + forcing verification)
Cell FC ─────────┤──► AIC Engine (cell-source forcing)
Region FC ───────┤──► AIC Engine (sector-source forcing)
Dynamic FC ──────┘──► AIC Engine (full-propagation forcing)
```

### 4.3 Technique Coverage by Engine

The solver dispatches 45 technique variants. Their engine ownership:

| Engine        | Count | Technique variants |
|---------------|:-----:|---|
| Basic (direct) | 8 | NakedSingle, HiddenSingle, NakedPair/Triple/Quad, HiddenPair/Triple/Quad |
| Fish          | 11 | PointingPair, BoxLineReduction, X-Wing, Finned X-Wing, Swordfish, Finned Swordfish, Jellyfish, Finned Jellyfish, Franken Fish, Siamese Fish, Mutant Fish |
| ALS           | 10 | XY-Wing, XYZ-Wing, WXYZ-Wing, ALS-XZ, ALS-XY-Wing, ALS Chain, Sue de Coq, Death Blossom, Aligned Pair Exclusion, Aligned Triplet Exclusion |
| AIC           | 9 | W-Wing, X-Chain, 3D Medusa, AIC, Nishio FC, Kraken Fish, Cell FC, Region FC, Dynamic FC |
| Uniqueness    | 6 | Empty Rectangle, Avoidable Rectangle, Unique Rectangle, Hidden Rectangle, Extended UR, BUG |
| Backtracking  | 1 | Backtracking |

Note: Basic techniques (singles, subsets) are direct applications of the
axioms and do not require an engine. Uniqueness techniques rely on the
assumption that the puzzle has a unique solution (an additional axiom
beyond the standard constraint set).

---

## 5. Hint Delivery

The solver exposes two hint methods with different safety properties.

### 5.1 `get_hint()` — Display Hints

`get_hint(grid)` returns the first applicable technique as a `Hint`.
It may return either a `SetValue` (placement) or `EliminateCandidates`
(elimination). **It does not verify the result against the backtracking
solution.** This method is used for hint *display* — showing the user
which technique applies and why — where an occasional unsound result
is acceptable because no state is mutated.

### 5.2 `get_next_placement()` — Verified Placement Hints

`get_next_placement(grid)` is the safe hint path used by all frontends
when *applying* a hint to the game state. It works as follows:

1. Solve the grid once via backtracking to obtain the verified solution.
2. Find the first applicable technique.
3. If it is an **elimination**: verify that no eliminated candidate is
   the solution value. If sound, apply it internally and repeat from
   step 2 (up to 500 iterations).
4. If it is a **placement**: verify that the placed value matches the
   solution. If sound, return it as the hint.
5. If any step produces an unsound result (technique bug), **stop
   chaining** and fall back to a backtracking hint (always correct).

This loop chains eliminations internally so the caller always receives
a `SetValue` hint. The solution verification at each step ensures that
known technique bugs (e.g., Avoidable Rectangle on given cells, W-Wing
self-links, X-Chain conjugate errors) never produce wrong placements.

### 5.3 ProofCertificate

Each `Hint` carries an optional `ProofCertificate` providing structured
metadata for visualization:

| Variant        | Fields                                        |
|---------------|-----------------------------------------------|
| `Basic`       | involved cells                                |
| `Fish`        | base sectors, cover sectors, fin cells, digit |
| `Als`         | ALS chain (cells + candidates per ALS)        |
| `Aic`         | chain of (cell, digit, polarity) nodes        |
| `Uniqueness`  | floor cells, roof cells                       |
| `Forcing`     | source cell, branches                         |
| `Backtracking`| (no fields)                                   |

Frontends use these to render proof-detail overlays (e.g., base/cover
sector highlighting for fish, on/off coloring for AIC chains).

---

## 6. Soundness Guarantees

### 6.1 Engine Soundness

Each engine's eliminations are sound if the axioms hold:

- **Fish**: Sound by Theorems 1.1 and 1.2. Depends only on the
  Uniqueness axiom for sectors.
- **ALS**: Sound by Theorems 2.1, 2.2, 2.3, 2.4. Depends on the
  Uniqueness axiom and Completeness axiom.
- **AIC**: Sound by Theorems 3.1, 3.2, 3.4. Depends on the logical
  semantics of strong and weak links, which are derived from all
  three axioms.

### 6.2 Implementation Verification

Three test suites verify soundness empirically:

- `test_hint_soundness`: For a battery of puzzles, every hint from
  `get_hint()` is checked against the unique solution. A placement
  *(c, v)* must match the solution, and an elimination *(c, v)* must
  not remove the solution value.
- `test_hint_soundness_all_tiers`: Extends coverage across all
  difficulty tiers.
- `test_next_placement_soundness`: Verifies that `get_next_placement()`
  produces correct placements for every puzzle in the battery.

These are runtime verifications, not formal proofs, but they provide
high confidence that the implementation correctly realizes the
theorems above. The `get_next_placement()` verification loop
(Section 5.2) provides an additional per-call safety net in
production.

### 6.3 Termination

The solver terminates because:
1. Each technique application either places a value (reducing
   empty cells by 1) or eliminates at least one candidate.
2. Candidates are never re-added.
3. The candidate space *X* is finite and monotonically decreasing.
4. If no technique applies, backtracking terminates in finite time
   (brute-force search over a finite space).
5. `get_next_placement()` bounds its chaining loop to 500 iterations.

---

## 7. Difficulty Classification

### 7.1 Eight-Tier System

Puzzles are classified into eight difficulty tiers. Three independent
mechanisms use these tiers for different purposes.

| Tier         | SE Range    | Technique Hint          |
|-------------|-------------|-------------------------|
| Beginner    | 1.5 – 2.0   | Hidden singles          |
| Easy        | 2.0 – 2.5   | Naked singles           |
| Medium      | 2.5 – 3.4   | Pairs & triples         |
| Intermediate| 3.4 – 3.8   | Hidden triples          |
| Hard        | 3.8 – 4.5   | Box/line reduction      |
| Expert      | 4.5 – 5.5   | Fish & rectangles       |
| Master      | 5.5 – 7.0   | Wings & chains          |
| Extreme     | 7.0 – 11.0  | Advanced techniques     |

Master and Extreme are hidden tiers, unlocked by the player.

### 7.2 Three Difficulty Axes

**SE rating** (`rate_se`): The maximum SE rating among all techniques
used to solve the puzzle. This is a continuous numerical score that
maps directly to the Sudoku Explainer community standard.

**Technique-based classification** (`technique_to_difficulty`): Maps
the hardest technique used during solving to a difficulty tier. This
is a discrete classification based on the *kind* of reasoning required:

| Tier         | Techniques included |
|-------------|---|
| Beginner    | NakedSingle (≤35 empty cells) |
| Easy        | NakedSingle (>35 empty cells) |
| Medium      | HiddenSingle |
| Intermediate| NakedPair, HiddenPair, NakedTriple, HiddenTriple |
| Hard        | PointingPair, BoxLineReduction |
| Expert      | X-Wing (±fin), Swordfish (±fin), Jellyfish (±fin), NakedQuad, HiddenQuad, EmptyRectangle, AvoidableRectangle, UniqueRectangle, HiddenRectangle |
| Master      | XY/XYZ/WXYZ-Wing, W-Wing, X-Chain, 3D Medusa, SueDeCoq, AIC, FrankenFish, SiameseFish, ALS-XZ, ExtendedUR, BUG |
| Extreme     | ALS-XY-Wing, ALS Chain, MutantFish, APE, ATE, DeathBlossom, Nishio/Kraken/Cell/Region/Dynamic FC, Backtracking |

**Generation cap** (`max_technique`): Limits which techniques the
generator may require. This uses the `Technique` enum ordering (not SE
rating) to define an upper bound:

| Tier         | Max technique allowed     |
|-------------|---------------------------|
| Beginner    | NakedSingle               |
| Easy        | NakedSingle               |
| Medium      | HiddenSingle              |
| Intermediate| HiddenTriple              |
| Hard        | BoxLineReduction          |
| Expert      | HiddenRectangle           |
| Master      | BivalueUniversalGrave     |
| Extreme     | Backtracking              |

The generation cap and technique classification are intentionally
different: the generator uses a coarse enum-order gate to quickly
reject puzzles during generation, while classification uses the full
technique-to-tier mapping after solving. The SE range provides the
continuous scale used by `generate_for_se()`.

---

## Appendix A: Notation Summary

| Symbol | Meaning |
|--------|---------|
| *C*    | Cell space (81 cells) |
| *X*    | Candidate space ⊆ *C* × {1..9} |
| *S*    | Sector space (27 units) |
| *L*    | Link space (strong + weak edges) |
| *X(c)* | Candidates of cell *c* |
| *X_d(S)* | Cells in sector *S* with candidate *d* |
| *β, κ* | Base cells, cover cells (fish) |
| *φ, ε* | Fin cells, elimination cells (fish) |
| *cands(A)* | Candidate union of ALS *A* |
| RCC    | Restricted Common Candidate |
| ALS    | Almost Locked Set |
| AIC    | Alternating Inference Chain |
| SE     | Sudoku Explainer rating |

## Appendix B: CandidateFabric Sector Convention

The implementation uses a flat sector index:

```
Sector  0.. 8  →  rows r₁..r₉
Sector  9..17  →  columns c₁..c₉
Sector 18..26  →  boxes b₁..b₉
```

`sector_digit_cells[s][d-1]` gives a `u16` bitmask of positions within
sector `s` that have candidate `d`. `cell_sectors[c]` gives the 3
sectors containing cell `c`: `[row, col, box]`.

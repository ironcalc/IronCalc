import Mathlib.Data.Finset.Basic
import Mathlib.Data.List.Basic

/-!
# The sheet and the specification

This file is `cold-evaluation.md`, section 1: what a sheet is, what a formula
is, and what "consistent" means, without reference to any algorithm.

## Abstractions

* A position is an abstract type `Pos` with decidable equality. The finite
  workbook grid is given by a `Universe`: the list of all positions in natural
  `(sheet, row, column)` order, which is what `Model::all_positions` returns.
* A value is an abstract type `Value` with three distinguished elements: what
  an empty position reads as, `#CIRC!` and `#SPILL!` (`ValueSort`).
* A formula is a strategy tree (`Formula`): either it has its result, or it
  reads a position and continues with what it found. This is the abstraction
  the design document asks for: a formula "reads some positions and produces a
  result", and "which positions it reads may depend on what it finds". The
  frame condition, that the result depends only on what was read, holds by
  construction.
* An array result carries its area as a list of positions instead of a
  rectangle. Worksheet bounds and merged cells are not modelled: an array
  always fits the sheet.
* A dynamic anchor's spill cells are found by scanning the universe for cells
  that point at it, rather than by remembering the old area (`r` in
  `Cell::ArrayFormula`). The Rust only scans the old area; the two agree
  whenever every own spill cell lies in the old area, which the editing paths
  maintain.
-/

namespace IronCalcEval

/-- The values the algorithm produces itself. -/
class ValueSort (Value : Type) where
  /-- What an empty position reads as (`CalcResult::EmptyCell`). -/
  emptyValue : Value
  /-- `#CIRC!` -/
  circ : Value
  /-- `#SPILL!` -/
  spillError : Value

export ValueSort (emptyValue circ spillError)

/-- All the positions of the workbook, in natural order (`Model::all_positions`). -/
structure Universe (Pos : Type) where
  positions : List Pos
  complete : ∀ p, p ∈ positions
  nodup : positions.Nodup

/-- What a formula produces: a scalar, or an array occupying `area` (the
anchor's own position included) with `vals` giving the value at each position. -/
inductive Result (Pos Value : Type) where
  | scalar (v : Value)
  | array (area : List Pos) (vals : Pos → Value)

/-- The value a result puts at a position: the scalar, or the array's entry
there. A scalar formula cell that receives an array stores the entry at its
own position. -/
def Result.valueAt {Pos Value : Type} : Result Pos Value → Pos → Value
  | .scalar v, _ => v
  | .array _ vals, p => vals p

/-- A formula as a strategy: it either has its result, or reads a position and
continues with what it found. Every formula reads finitely many positions. -/
inductive Formula (Pos Value : Type) where
  | done (r : Result Pos Value)
  | read (p : Pos) (k : Value → Formula Pos Value)

/-- Runs a formula with reads performed in a monad. This is how the pass runs
it: reads go through `evalCell`. -/
def Formula.run {Pos Value : Type} {m : Type → Type} [Monad m] (readFn : Pos → m Value) :
    Formula Pos Value → m (Result Pos Value)
  | .done r => pure r
  | .read p k => do
      let v ← readFn p
      Formula.run readFn (k v)

/-- Runs a formula against a fixed reader. This is how the specification runs
it: reads are the values stored in the sheet. -/
def Formula.runPure {Pos Value : Type} (readFn : Pos → Value) : Formula Pos Value → Result Pos Value
  | .done r => r
  | .read p k => Formula.runPure readFn (k (readFn p))

/-- The positions a formula reads when run against a fixed reader. -/
def Formula.reads {Pos Value : Type} (readFn : Pos → Value) : Formula Pos Value → List Pos
  | .done _ => []
  | .read p k => p :: Formula.reads readFn (k (readFn p))

/-- What a position holds (`Cell`, reduced to what evaluation cares about). -/
inductive Content (Pos Value : Type) where
  /-- No cell, or `Cell::EmptyCell`. -/
  | empty
  /-- A constant: number, string, boolean, error. -/
  | const (v : Value)
  /-- `Cell::CellFormula` with its stored value. -/
  | formula (t : Formula Pos Value) (stored : Value)
  /-- `Cell::ArrayFormula` of kind `Cse`, with its fixed area. -/
  | cseAnchor (t : Formula Pos Value) (area : List Pos) (stored : Value)
  /-- `Cell::ArrayFormula` of kind `Dynamic`. -/
  | dynAnchor (t : Formula Pos Value) (stored : Value)
  /-- `Cell::SpillCell`, pointing at its anchor. -/
  | spill (anchor : Pos) (v : Value)

/-- A sheet assigns a content to every position. -/
abbrev Sheet (Pos Value : Type) := Pos → Content Pos Value

variable {Pos Value : Type}

def Content.isEmpty : Content Pos Value → Bool
  | .empty => true
  | _ => false

def Content.spillAnchor? : Content Pos Value → Option Pos
  | .spill a _ => some a
  | _ => none

def Content.isDynAnchor : Content Pos Value → Bool
  | .dynAnchor .. => true
  | _ => false

def Content.isAnchor : Content Pos Value → Bool
  | .dynAnchor .. => true
  | .cseAnchor .. => true
  | _ => false

/-- The formula of a formula cell or anchor. -/
def Content.formula? : Content Pos Value → Option (Formula Pos Value)
  | .formula t _ => some t
  | .cseAnchor t _ _ => some t
  | .dynAnchor t _ => some t
  | _ => none

/-- Whether the anchor `p` may spill into a cell with this content: empty, or
its own spill cell. Anything else blocks (`spill_dynamic_array`). -/
def Content.freeFor [DecidableEq Pos] (c : Content Pos Value) (p : Pos) : Bool :=
  match c with
  | .empty => true
  | .spill a _ => decide (a = p)
  | _ => false

/-- The value stored at a position (`Model::stored_value`). -/
def valueAt [ValueSort Value] (S : Sheet Pos Value) (p : Pos) : Value :=
  match S p with
  | .empty => emptyValue
  | .const v => v
  | .formula _ v => v
  | .cseAnchor _ _ v => v
  | .dynAnchor _ v => v
  | .spill _ v => v

/-- The spill cells that point at `p`. -/
def ownSpillCells [DecidableEq Pos] (U : Universe Pos) (S : Sheet Pos Value) (p : Pos) : List Pos :=
  U.positions.filter fun q => (S q).spillAnchor? = some p

/-- The area of `p` is blocked: some cell of it, other than `p`, is neither
empty nor `p`'s own spill cell. -/
def blocked [DecidableEq Pos] (S : Sheet Pos Value) (p : Pos) (area : List Pos) : Prop :=
  ∃ q ∈ area, q ≠ p ∧ (S q).freeFor p = false

/-- No spill cell points at a position that is not an anchor. The editing
paths maintain this (`evaluation.md` 4.6); the algorithm reads an orphan as
empty, which the specification below does not. -/
def NoOrphans (S : Sheet Pos Value) : Prop :=
  ∀ q a v, S q = .spill a v → (S a).isAnchor = true

/-- The cells of a CSE anchor's area are its spill cells. The editing paths
set them up when the formula is entered, and nothing else ever writes them:
they block every other array, and only their anchor rewrites them. -/
def CseAreasFixed (S : Sheet Pos Value) : Prop :=
  ∀ p t area v, S p = .cseAnchor t area v → ∀ q ∈ area, q ≠ p → ∃ w, S q = .spill p w

/-- A CSE anchor has no spill cell outside its area. -/
def CseSpillsInArea (S : Sheet Pos Value) : Prop :=
  ∀ q a w, S q = .spill a w → ∀ t area v, S a = .cseAnchor t area v → q ∈ area

/-- What the editing paths guarantee of a sheet before evaluation. -/
def WellFormed (S : Sheet Pos Value) : Prop :=
  NoOrphans S ∧ CseAreasFixed S ∧ CseSpillsInArea S

/-- A formula run against two readers that agree on what it reads gives the
same result. -/
theorem Formula.runPure_congr {f g : Pos → Value} :
    ∀ t : Formula Pos Value, (∀ q ∈ t.reads f, f q = g q) → t.runPure f = t.runPure g
  | .done _, _ => rfl
  | .read p k, h => by
      simp only [Formula.reads, List.mem_cons, forall_eq_or_imp] at h
      simp only [Formula.runPure]
      rw [← h.1]
      exact Formula.runPure_congr (k (f p)) h.2

/-- And reads the same positions. -/
theorem Formula.reads_congr {f g : Pos → Value} :
    ∀ t : Formula Pos Value, (∀ q ∈ t.reads f, f q = g q) → t.reads f = t.reads g
  | .done _, _ => rfl
  | .read p k, h => by
      simp only [Formula.reads, List.mem_cons, forall_eq_or_imp] at h
      simp only [Formula.reads]
      rw [← h.1]
      exact congrArg _ (Formula.reads_congr (k (f p)) h.2)

/-- The clause of consistency at one position, against a reader and a notion
of "the area is blocked". The specification uses `blocked S`; the pass
invariant uses a stronger notion whose blockers no later step can remove. -/
def ConsistentAtWith [DecidableEq Pos] [ValueSort Value] (reader : Pos → Value)
    (blockedBy : Pos → List Pos → Prop) (S : Sheet Pos Value) (p : Pos) : Prop :=
  match S p with
  | .formula t stored =>
      stored ≠ circ → stored = (t.runPure reader).valueAt p
  | .cseAnchor t area stored =>
      stored ≠ circ →
        let r := t.runPure reader
        stored = r.valueAt p ∧ ∀ q ∈ area, q ≠ p → S q = .spill p (r.valueAt q)
  | .dynAnchor t stored =>
      stored ≠ circ →
        match t.runPure reader with
        | .scalar v => stored = v ∧ ∀ q, (S q).spillAnchor? ≠ some p
        | .array area vals =>
            (stored = vals p ∧
              ∀ q, q ≠ p →
                (q ∈ area → S q = .spill p (vals q)) ∧
                (q ∉ area → (S q).spillAnchor? ≠ some p))
            ∨ (blockedBy p area ∧ stored = spillError ∧ ∀ q, (S q).spillAnchor? ≠ some p)
  | _ => True

/-- The clause of consistency at one position, against a reader. With the
reader `valueAt S` this is `cold-evaluation.md`, section 1. -/
def ConsistentAt [DecidableEq Pos] [ValueSort Value] (reader : Pos → Value)
    (S : Sheet Pos Value) (p : Pos) : Prop :=
  ConsistentAtWith reader (blocked S) S p

/-- A stronger notion of blocked gives a stronger clause. -/
theorem ConsistentAtWith.mono [DecidableEq Pos] [ValueSort Value] {reader : Pos → Value}
    {b₁ b₂ : Pos → List Pos → Prop} (h : ∀ p area, b₁ p area → b₂ p area)
    {S : Sheet Pos Value} {p : Pos} (hc : ConsistentAtWith reader b₁ S p) :
    ConsistentAtWith reader b₂ S p := by
  unfold ConsistentAtWith at hc ⊢
  split at hc <;> try exact hc
  · rename_i t stored
    intro hne
    specialize hc hne
    revert hc
    split
    · exact id
    · rintro (h1 | ⟨hb, h2, h3⟩)
      · exact Or.inl h1
      · exact Or.inr ⟨h _ _ hb, h2, h3⟩

/-- `cold-evaluation.md`, section 1: a sheet is consistent when every formula
cell that does not hold `#CIRC!` stores what its formula gives against the
sheet itself, and every array either occupies its area or is blocked and
reports `#SPILL!`. -/
def Consistent [DecidableEq Pos] [ValueSort Value] (S : Sheet Pos Value) : Prop :=
  ∀ p, ConsistentAt (valueAt S) S p

end IronCalcEval

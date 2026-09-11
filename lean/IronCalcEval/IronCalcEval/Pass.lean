import IronCalcEval.Sheet

/-!
# A pass

`cold-evaluation.md`, sections 3.2 and 3.4 to 3.6; `base/src/evaluation.rs`
and `spill_dynamic_array` in `base/src/model.rs`. Names follow the Rust.

The recursion of `evaluate_cell` is modelled with fuel: `evalCell fuel p`
gives up (returns the empty value) when the fuel runs out. `runPass` provides
`2 * |positions| + 2`, and `Correctness.lean` states that this is never
exhausted: every nested `evaluate_formula_cell` puts a distinct cell in the
`Evaluating` state, and a spill cell forwards to its anchor at most once
between two of them.

`in_pass` is not modelled: everything here happens inside a pass. Evaluation
outside a pass (conditional formatting, formula helpers) is out of scope.
-/

namespace IronCalcEval

/-- `CellState`: the state of a formula cell within the current pass. Absent
(`none` in the map) means not evaluated yet. -/
inductive CellState where
  | evaluating
  | evaluated
  deriving DecidableEq, Repr

/-- `Seen`: what a formula found at a position when it read it, as far as
spills are concerned. -/
inductive Seen where
  | empty
  | occupied
  deriving DecidableEq, Repr

/-- `Restart`: why the current pass is abandoned. -/
inductive Restart (Pos : Type) where
  | staleRead (anchor : Pos)
  | conflict (anchor : Pos)
  | selfContradiction (anchor : Pos)
  /-- The spill cells of `anchor` at `cells`, left by a previous evaluation,
  blocked an array evaluated on its own behalf and nothing else. They were
  history, not a dependency: the driver drops them and starts again. -/
  | staleCells (anchor : Pos) (cells : List Pos)
  deriving DecidableEq, Repr

variable {Pos Value : Type}

def Restart.anchor : Restart Pos → Pos
  | .staleRead a => a
  | .conflict a => a
  | .selfContradiction a => a
  | .staleCells a _ => a

def Restart.isSelfContradiction : Restart Pos → Bool
  | .selfContradiction _ => true
  | _ => false

def Restart.isStaleCells : Restart Pos → Bool
  | .staleCells _ _ => true
  | _ => false

/-- The per-pass fields of `Evaluation`, plus the sheet. -/
structure PassState (Pos Value : Type) where
  sheet : Sheet Pos Value
  /-- `cells`: formula cells touched in this pass. -/
  cells : Pos → Option CellState
  /-- `stack`: cells being evaluated, innermost **first**. The root of the
  recursion (`stack.first()` in Rust) is the last element. -/
  stack : List Pos
  /-- `circular`: cells known to be circular. -/
  circular : Finset Pos
  /-- `seen`, split by kind: positions read as empty, and positions found
  occupied by another array's spill cell, each with the root of the
  recursion the read or the scan was made in. The Rust keeps a single record
  per position and drops the second kind when the first is present; the
  design document records both, and the consistency proof needs both. -/
  seenEmpty : Pos → Option Pos
  seenOccupied : Pos → Option Pos
  /-- `restart`: set when the pass must be abandoned. -/
  restart : Option (Restart Pos)

/-- The state at the start of a pass (`run_pass`). -/
def PassState.initial (S : Sheet Pos Value) (circular : Finset Pos) : PassState Pos Value :=
  { sheet := S
    cells := fun _ => none
    stack := []
    circular := circular
    seenEmpty := fun _ => none
    seenOccupied := fun _ => none
    restart := none }

/-- The root of the current recursion: the cell the driver is evaluating. -/
def PassState.root (st : PassState Pos Value) : Option Pos :=
  st.stack.getLast?

abbrev PassM (Pos Value : Type) := StateM (PassState Pos Value)

section
variable [DecidableEq Pos] [ValueSort Value]

/-- `stored_value`: the value currently stored at a position. -/
def storedValue (p : Pos) : PassM Pos Value Value := do
  return valueAt (← get).sheet p

/-- `record_seen`: records what the formula being evaluated found at a
position, together with the root of the current recursion. The first record
of each kind for a position is kept. Reads made by the driver itself (empty
stack) are nobody's dependency. -/
def recordSeen (q : Pos) (s : Seen) : PassM Pos Value Unit := do
  let st ← get
  match st.root with
  | none => pure ()
  | some r =>
    match s with
    | .empty =>
      match st.seenEmpty q with
      | some _ => pure ()
      | none => set { st with seenEmpty := Function.update st.seenEmpty q (some r) }
    | .occupied =>
      match st.seenOccupied q with
      | some _ => pure ()
      | none => set { st with seenOccupied := Function.update st.seenOccupied q (some r) }

/-- `mark_cycle`: a read closed a loop at `origin`; every cell on the stack
from `origin` to the top is on the loop and is marked circular. -/
def markCycle (origin : Pos) : PassM Pos Value Unit :=
  modify fun st =>
    if origin ∈ st.stack then
      let above := st.stack.takeWhile fun c => decide (c ≠ origin)
      { st with circular := st.circular ∪ (insert origin above.toFinset) }
    else st

/-- `spill_contradicts_a_read`: on behalf of the anchor about to commit,
whether writing spill cells at `writes` and removing its own spill cells at
`clears` would contradict what a formula read earlier in this pass. If so the
pass is abandoned and `true` is returned: the anchor must not write anything.
When every contradicted read was on the anchor's own behalf, it is circular,
unless only the removals contradict: then the cells were stale, and they go. -/
def spillContradictsARead (anchor : Pos) (writes clears : List Pos) : PassM Pos Value Bool := do
  let st ← get
  let roots := writes.filterMap st.seenEmpty ++ clears.filterMap st.seenOccupied
  if roots.isEmpty then
    return false
  let onlyOwnReads := roots.all fun r => decide (r = anchor)
  set { st with
    restart := some (if onlyOwnReads then
      (if (writes.filterMap st.seenEmpty).isEmpty then .staleCells anchor clears
        else .selfContradiction anchor)
      else .conflict anchor) }
  return true

/-- Writes several contents at once. -/
def Sheet.writeAll (S : Sheet Pos Value) (ws : List (Pos × Content Pos Value)) : Sheet Pos Value :=
  ws.foldl (fun S ⟨q, c⟩ => Function.update S q c) S

theorem Sheet.writeAll_map (S : Sheet Pos Value) (f : Pos → Content Pos Value) :
    ∀ (l : List Pos) (q : Pos),
      Sheet.writeAll S (l.map fun x => (x, f x)) q = if q ∈ l then f q else S q
  | [], q => by simp [Sheet.writeAll]
  | x :: l, q => by
      simp only [List.map_cons, Sheet.writeAll, List.foldl_cons]
      rw [show List.foldl (fun S (⟨q, c⟩ : Pos × Content Pos Value) => Function.update S q c)
          (Function.update S x (f x)) (l.map fun x => (x, f x)) =
          Sheet.writeAll (Function.update S x (f x)) (l.map fun x => (x, f x)) from rfl]
      rw [Sheet.writeAll_map]
      by_cases hql : q ∈ l
      · simp [hql]
      · by_cases hqx : q = x
        · subst hqx
          simp [hql]
        · simp [hql, hqx, Function.update_of_ne hqx]

/-- The scalar path of `set_cells_with_result` for a dynamic anchor: a dynamic
anchor without an array to spill keeps no spill cells
(`retire_own_spill_cells`), with the contradiction check. -/
def storeScalar (U : Universe Pos) (p : Pos) (t : Formula Pos Value) (v : Value) :
    PassM Pos Value Unit := do
  let st ← get
  let clears := (ownSpillCells U st.sheet p).filter fun q => decide (q ≠ p)
  if ← spillContradictsARead p [] clears then
    return
  modify fun st =>
    let sheet := Function.update st.sheet p (.dynAnchor t v)
    let sheet := Sheet.writeAll sheet (clears.map fun q => (q, .empty))
    { st with sheet := sheet }

/-- The blocking scan's records: every target holding another array's spill
cell is put on record as occupied. `st` is the state the scan reads the
sheet from (the sheet does not change during the scan). -/
def recordBlockers (st : PassState Pos Value) (p : Pos) : List Pos → PassM Pos Value Unit
  | [] => pure ()
  | q :: l => do
      (match (st.sheet q).spillAnchor? with
        | some a => if a ≠ p then recordSeen q .occupied else pure ()
        | none => pure () : PassM Pos Value Unit)
      recordBlockers st p l

/-- `spill_dynamic_array`: spills `array` from the dynamic anchor `p`. Another
array's spill cell blocks and is recorded as seen occupied; anything else that
is not empty blocks too. Own spill cells never block: they are rewritten or,
outside the new area, removed. -/
def spillDynamicArray (U : Universe Pos) (p : Pos) (t : Formula Pos Value)
    (area : List Pos) (vals : Pos → Value) : PassM Pos Value Unit := do
  let st ← get
  let targets := area.filter fun q => decide (q ≠ p)
  recordBlockers st p targets
  if targets.any fun q => !(st.sheet q).freeFor p then
    storeScalar U p t spillError
    return
  let writes := targets
  let clears := (ownSpillCells U st.sheet p).filter fun q => decide (q ∉ area)
  if ← spillContradictsARead p writes clears then
    return
  modify fun st =>
    let sheet := Function.update st.sheet p (.dynAnchor t (vals p))
    let sheet := Sheet.writeAll sheet (writes.map fun q => (q, .spill p (vals q)))
    let sheet := Sheet.writeAll sheet (clears.map fun q => (q, .empty))
    { st with sheet := sheet }

/-- `set_cells_with_result`: stores a result at a formula cell. -/
def commit (U : Universe Pos) (p : Pos) (r : Result Pos Value) : PassM Pos Value Unit := do
  let st ← get
  match st.sheet p with
  | .formula t _ =>
      set { st with sheet := Function.update st.sheet p (.formula t (r.valueAt p)) }
  | .cseAnchor t area _ =>
      -- A CSE area is fixed: fill it, whatever the result.
      let others := area.filter fun q => decide (q ≠ p)
      let sheet := Function.update st.sheet p (.cseAnchor t area (r.valueAt p))
      let sheet := Sheet.writeAll sheet (others.map fun q => (q, .spill p (r.valueAt q)))
      set { st with sheet := sheet }
  | .dynAnchor t _ =>
      match r with
      | .scalar v => storeScalar U p t v
      | .array area vals => spillDynamicArray U p t area vals
  | _ => pure ()

/-- `evaluate_spill_cell`, with `rec` standing for the recursive
`evaluate_cell`. Only a value written by the anchor in the current pass is a
value. -/
def evalSpillCell (rec : Pos → PassM Pos Value Value) (p a : Pos) : PassM Pos Value Value := do
  let st ← get
  match st.sheet a with
  | .cseAnchor .. =>
      match st.cells a with
      | some .evaluated => storedValue p
      -- The anchor is running: this read closes a cycle, which `rec` reports.
      | some .evaluating => rec a
      | none => do
          let _ ← rec a
          storedValue p
  | .dynAnchor .. =>
      match st.cells a with
      | some .evaluated => storedValue p
      -- The anchor is running and this read is on its behalf: the area counts
      -- as empty until the anchor commits.
      | some .evaluating => do
          recordSeen p .empty
          return emptyValue
      -- Left over from a previous evaluation: the anchor should have run first.
      | none => do
          set { st with restart := some (.staleRead a) }
          return emptyValue
  -- An orphan: its anchor is gone. Nothing will ever write it again.
  | _ => do
      recordSeen p .empty
      return emptyValue

/-- The cell on top of the stack has finished: it is `Evaluated` and leaves
the stack. -/
def markEvaluated (st : PassState Pos Value) (p : Pos) : PassState Pos Value :=
  { st with cells := Function.update st.cells p (some .evaluated), stack := st.stack.tail }

/-- The tail of `evaluate_formula_cell` once the formula has run: store the
result unless the pass is being abandoned, pop the stack, mark the cell
evaluated and return what was stored. -/
def finishFormulaCell (U : Universe Pos) (p : Pos) (r : Result Pos Value) :
    PassM Pos Value Value := do
  let st ← get
  -- The cell may have been found on a cycle while it ran.
  let r := if p ∈ st.circular then .scalar circ else r
  -- Abandoned pass: store nothing.
  let _ ← (if st.restart.isNone then commit U p r else pure () : PassM Pos Value Unit)
  let st ← get
  if st.restart.isSome then
    return emptyValue
  modify fun st => markEvaluated st p
  -- Return what was stored, so that a dependent sees the same value it
  -- would read from the sheet later.
  storedValue p

/-- `evaluate_formula_cell`, with `rec` standing for the recursive
`evaluate_cell` used for the formula's reads. -/
def evalFormulaCell (U : Universe Pos) (rec : Pos → PassM Pos Value Value)
    (p : Pos) (t : Formula Pos Value) : PassM Pos Value Value := do
  let st ← get
  match st.cells p with
  | some .evaluating => do
      markCycle p
      return circ
  | some .evaluated => storedValue p
  | none => do
      modify fun st =>
        { st with cells := Function.update st.cells p (some .evaluating), stack := p :: st.stack }
      let st ← get
      -- A cell marked circular by the driver skips its formula.
      let r ← (if p ∈ st.circular then pure (.scalar circ) else t.run rec :
        PassM Pos Value (Result Pos Value))
      finishFormulaCell U p r

/-- `evaluate_cell`: the value of a cell, evaluating it first if it is a
formula that has not been evaluated in this pass. Reads are performed on
behalf of the cell on top of the stack. Every recursive call spends one unit
of fuel. -/
def evalCell (U : Universe Pos) : Nat → Pos → PassM Pos Value Value
  | 0, _ => pure emptyValue
  | fuel + 1, p => do
      let st ← get
      if st.restart.isSome then
        -- The pass is being abandoned; nothing computed now is kept.
        return emptyValue
      match st.sheet p with
      | .empty => do
          recordSeen p .empty
          return emptyValue
      | .const v => return v
      | .spill a _ => evalSpillCell (evalCell U fuel) p a
      | .formula t _ => evalFormulaCell U (evalCell U fuel) p t
      | .cseAnchor t _ _ => evalFormulaCell U (evalCell U fuel) p t
      | .dynAnchor t _ => evalFormulaCell U (evalCell U fuel) p t

/-- Fuel that `Correctness.lean` claims is never exhausted. -/
def passFuel (U : Universe Pos) : Nat :=
  2 * U.positions.length + 2

/-- The body of `run_pass`: evaluates the cells in turn, and stops at the
first restart. (A `for` loop with a `break` in the Rust.) -/
def passBody (U : Universe Pos) : List Pos → PassM Pos Value Unit
  | [] => pure ()
  | c :: l => do
      let st ← get
      if st.restart.isSome then
        pure ()
      else
        let _ ← evalCell U (passFuel U) c
        passBody U l

/-- `run_pass`: one pass over the workbook, starting from `S` with the anchors
in `order` and the given circular set. Returns the sheet and the reason the
pass had to be abandoned, if any. -/
def runPass (U : Universe Pos) (S : Sheet Pos Value) (order : List Pos) (circular : Finset Pos) :
    Sheet Pos Value × Option (Restart Pos) :=
  let (_, st) := (passBody (Value := Value) U (order ++ U.positions)).run (PassState.initial S circular)
  (st.sheet, st.restart)

end

end IronCalcEval

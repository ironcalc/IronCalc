import IronCalcEval.Pass

/-!
# The driver

`cold-evaluation.md`, section 3.3; `RestartLog` and `Model::evaluate` in
`base/src/evaluation.rs`.

Every pass starts from the same sheet. The Rust achieves this by restoring the
dynamic anchors and their spill cells from a snapshot; formula cells keep the
values an abandoned pass stored, which does not matter because a pass never
reads a formula cell's stored value before recomputing it. The model simply
starts every pass from the original sheet. The one change to that sheet
within an evaluation is the dropping of stale spill cells an anchor gave up
(`Restart.staleCells`): the Rust removes them from the snapshot, the model
empties them (`dropStale`).

The loop is written with fuel, because its termination is one of the theorems
(`Termination.lean`), not something the definition may assume.
-/

namespace IronCalcEval

variable {Pos Value : Type}

/-- `RestartLog`: the driver's memory of the restarts of one evaluation. -/
structure RestartLog (Pos : Type) where
  /-- Anchors marked circular so far; passed to every pass. -/
  circular : Finset Pos
  /-- The orders seen since the circular set or the starting sheet last
  changed. -/
  ordersSeen : List (List Pos)
  /-- The anchor moved to the front to go from each seen order to the next. -/
  moves : List Pos
  /-- Every restart of this evaluation. -/
  restarts : Nat
  /-- The restarts that count against the budget: all but the stale-cell ones. -/
  spent : Nat
  budget : Nat

/-- `RestartLog::new`. -/
def RestartLog.new (initialOrder : List Pos) : RestartLog Pos :=
  { circular := ∅
    ordersSeen := [initialOrder]
    moves := []
    restarts := 0
    spent := 0
    budget := initialOrder.length * initialOrder.length + 2 }

section
variable [DecidableEq Pos]

/-- What a restart proves circular: the anchor itself on a self-contradiction
or once the budget is spent, every anchor moved since the order was first
seen when the order repeats, nothing otherwise. -/
def RestartLog.newlyCircular (log : RestartLog Pos) (r : Restart Pos) (order : List Pos) :
    List Pos :=
  if r.isSelfContradiction || decide (log.budget ≤ log.spent) then
    [r.anchor]
  else
    match log.ordersSeen.findIdx? (fun seen => decide (seen = order)) with
    | some firstSeen => (log.moves ++ [r.anchor]).drop firstSeen
    | none => []

/-- `RestartLog::record`: records a restart whose anchor has just been moved
to the front, giving `order`, and marks whatever it proves circular. -/
def RestartLog.record (log : RestartLog Pos) (r : Restart Pos) (order : List Pos) :
    RestartLog Pos :=
  if r.isStaleCells then
    -- The starting sheet changed: what a pass does changed with it.
    { log with
        ordersSeen := [order]
        moves := []
        restarts := log.restarts + 1 }
  else
    let newlyCircular := log.newlyCircular r order
    if newlyCircular.isEmpty then
      { log with
          ordersSeen := log.ordersSeen ++ [order]
          moves := log.moves ++ [r.anchor]
          restarts := log.restarts + 1
          spent := log.spent + 1 }
    else
      -- The circular set changed: what a pass does changed with it.
      { log with
          circular := log.circular ∪ newlyCircular.toFinset
          ordersSeen := [order]
          moves := []
          restarts := log.restarts + 1
          spent := log.spent + 1 }

/-- What `restore_dynamic_spills` does once the stale cells of `a` at `cells`
have left the snapshot: they are empty. Only spill cells of `a` are touched. -/
def dropStale (S : Sheet Pos Value) (a : Pos) (cells : List Pos) : Sheet Pos Value :=
  fun q => if q ∈ cells ∧ (S q).spillAnchor? = some a then .empty else S q

/-- The sheet the next pass starts from. -/
def Restart.nextSheet (r : Restart Pos) (S : Sheet Pos Value) : Sheet Pos Value :=
  match r with
  | .staleCells a cells => dropStale S a cells
  | _ => S

theorem Restart.nextSheet_of_not_stale {r : Restart Pos} (h : r.isStaleCells = false)
    (S : Sheet Pos Value) : r.nextSheet S = S := by
  cases r <;> simp_all [Restart.isStaleCells, Restart.nextSheet]

theorem dropStale_eq_or (S : Sheet Pos Value) (a : Pos) (cells : List Pos) (q : Pos) :
    dropStale S a cells q = S q ∨
      (dropStale S a cells q = .empty ∧ (S q).spillAnchor? = some a) := by
  unfold dropStale
  split
  · rename_i h
    exact Or.inr ⟨rfl, h.2⟩
  · exact Or.inl rfl

theorem dropStale_isDynAnchor (S : Sheet Pos Value) (a : Pos) (cells : List Pos) (q : Pos) :
    (dropStale S a cells q).isDynAnchor = (S q).isDynAnchor := by
  rcases dropStale_eq_or S a cells q with h | ⟨h, hs⟩
  · rw [h]
  · rw [h]
    cases hq : S q <;> simp_all [Content.spillAnchor?, Content.isDynAnchor]

theorem dropStale_isAnchor (S : Sheet Pos Value) (a : Pos) (cells : List Pos) (q : Pos) :
    (dropStale S a cells q).isAnchor = (S q).isAnchor := by
  rcases dropStale_eq_or S a cells q with h | ⟨h, hs⟩
  · rw [h]
  · rw [h]
    cases hq : S q <;> simp_all [Content.spillAnchor?, Content.isAnchor]

theorem dropStale_spill {S : Sheet Pos Value} {a : Pos} {cells : List Pos} {q b : Pos} {v : Value}
    (h : dropStale S a cells q = .spill b v) : S q = .spill b v := by
  rcases dropStale_eq_or S a cells q with h' | ⟨h', _⟩
  · rw [← h']
    exact h
  · rw [h'] at h
    cases h

theorem dropStale_cse {S : Sheet Pos Value} {a : Pos} {cells : List Pos} {q : Pos}
    {t : Formula Pos Value} {area : List Pos} {v : Value}
    (h : dropStale S a cells q = .cseAnchor t area v) : S q = .cseAnchor t area v := by
  rcases dropStale_eq_or S a cells q with h' | ⟨h', _⟩
  · rw [← h']
    exact h
  · rw [h'] at h
    cases h

/-- Dropping spill cells of a dynamic anchor keeps the sheet well-formed. -/
theorem WellFormed.dropStale {S : Sheet Pos Value} (hwf : WellFormed S) {a : Pos}
    (ha : (S a).isDynAnchor = true) (cells : List Pos) : WellFormed (dropStale S a cells) := by
  obtain ⟨hno, hcse, hin⟩ := hwf
  refine ⟨?_, ?_, ?_⟩
  · intro q b v hq
    rw [dropStale_isAnchor]
    exact hno q b v (dropStale_spill hq)
  · intro p t area v hp q hq hqp
    have hp' := dropStale_cse hp
    obtain ⟨w, hw⟩ := hcse p t area v hp' q hq hqp
    refine ⟨w, ?_⟩
    rcases dropStale_eq_or S a cells q with h | ⟨_, hs⟩
    · rw [h, hw]
    · -- A cell of a CSE area is the CSE anchor's, not a dynamic anchor's.
      rw [hw] at hs
      simp only [Content.spillAnchor?, Option.some.injEq] at hs
      subst hs
      rw [hp'] at ha
      simp [Content.isDynAnchor] at ha
  · intro q b w hq t area v hb
    exact hin q b w (dropStale_spill hq) t area v (dropStale_cse hb)

theorem WellFormed.nextSheet {S : Sheet Pos Value} (hwf : WellFormed S) {r : Restart Pos}
    (hr : (S r.anchor).isDynAnchor = true) : WellFormed (r.nextSheet S) := by
  cases r with
  | staleCells a cells => exact hwf.dropStale hr cells
  | _ => exact hwf

/-- Moves an anchor to the front of the order. -/
def moveToFront (order : List Pos) (a : Pos) : List Pos :=
  a :: order.filter fun b => decide (b ≠ a)

/-- `sync_anchor_order`: anchors that are gone are dropped, new ones are
appended in natural order, the rest keep their relative order. -/
def syncAnchorOrder (U : Universe Pos) (S : Sheet Pos Value) (order : List Pos) : List Pos :=
  let kept := order.filter fun a => (S a).isDynAnchor
  let current := U.positions.filter fun a => (S a).isDynAnchor
  kept ++ current.filter fun a => decide (a ∉ kept)

variable [ValueSort Value]

/-- The loop of `Model::evaluate`, with fuel. `none` means the fuel ran out.
The sheet is the one every pass starts from; it changes only when stale
cells are dropped. -/
def evaluateLoop (U : Universe Pos) :
    Nat → Sheet Pos Value → List Pos → RestartLog Pos →
      Option (Sheet Pos Value × List Pos × RestartLog Pos)
  | 0, _, _, _ => none
  | k + 1, S, order, log =>
      match runPass U S order log.circular with
      | (S', none) => some (S', order, log)
      | (_, some r) =>
          let order' := moveToFront order r.anchor
          evaluateLoop U k (r.nextSheet S) order' (log.record r order')

/-- `Model::evaluate`: evaluates every formula, starting from the sheet `S`
with the remembered `anchorOrder`. Returns the consistent sheet, the new
anchor order and the log, or `none` if `fuel` passes were not enough. -/
def evaluate (U : Universe Pos) (S : Sheet Pos Value) (anchorOrder : List Pos) (fuel : Nat) :
    Option (Sheet Pos Value × List Pos × RestartLog Pos) :=
  let order := syncAnchorOrder U S anchorOrder
  evaluateLoop U fuel S order (RestartLog.new order)

end

end IronCalcEval

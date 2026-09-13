import IronCalcEval.Pass

/-!
# The driver

`cold-evaluation.md`, section 3.3; `RestartLog` and `Model::evaluate` in
`base/src/evaluation.rs`.

Every pass starts from the same sheet. The Rust achieves this by restoring the
dynamic anchors and their spill cells from a snapshot; formula cells keep the
values an abandoned pass stored, which does not matter because a pass never
reads a formula cell's stored value before recomputing it. The model simply
starts every pass from the original sheet. The only changes to that sheet
within an evaluation are the dropping of stale spill cells: an anchor's own
when it gave them up (`Restart.staleCells`), a marked anchor's when it is
marked. The Rust removes them from the snapshot, the model empties them
(`dropStale`, `dropMarked`).

A restart says that some readers read an anchor's area before it ran: a
fact, "the anchor runs before each of them", that the current order breaks.
The driver keeps the facts (`RestartLog.facts`) and repairs the order after
each restart (`RestartLog.learn`). `Termination.lean` shows that every
restart adds a fact the order broke, hence one not yet known, or marks an
anchor, or drops a stale cell.

The loop is written with fuel, because its termination is one of the theorems
(`Termination.lean`), not something the definition may assume.
-/

namespace IronCalcEval

variable {Pos Value : Type}

/-- `RestartLog`: the driver's memory of one evaluation: the facts learned
about the order, and the anchors marked circular. -/
structure RestartLog (Pos : Type) where
  /-- Anchors marked circular so far; passed to every pass. -/
  circular : Finset Pos
  /-- `(a, r)`: `a` runs before `r`. Learned from restarts, never broken by
  the order. -/
  facts : List (Pos × Pos)
  /-- Every restart of this evaluation. -/
  restarts : Nat

/-- `RestartLog::new`. -/
def RestartLog.new : RestartLog Pos :=
  { circular := ∅, facts := [], restarts := 0 }

section
variable [DecidableEq Pos]

/-- One step of saturation: every target of an edge from a seen element. -/
def closureStep (edges : List (Pos × Pos)) (seen : List Pos) : List Pos :=
  seen ++ (edges.filterMap fun e => if e.1 ∈ seen ∧ e.2 ∉ seen then some e.2 else none).dedup

/-- Everything reachable from `start` along `edges`, `start` included. The
Rust walks the graph; the model saturates, which reaches the same set once
no step adds anything, and `edges.length + 1` steps are enough. -/
def closure (edges : List (Pos × Pos)) (start : Pos) : List Pos :=
  (edges.length + 1).iterate (closureStep edges) [start]

/-- What the facts place before `a`, `a` included. -/
def RestartLog.before (log : RestartLog Pos) (a : Pos) : List Pos :=
  closure (log.facts.map fun f => (f.2, f.1)) a

/-- What the facts place after `r`, `r` included. -/
def RestartLog.after (log : RestartLog Pos) (r : Pos) : List Pos :=
  closure log.facts r

/-- Marks an anchor circular; its facts are spent. -/
def RestartLog.mark (log : RestartLog Pos) (cell : Pos) : RestartLog Pos :=
  { log with
      circular := insert cell log.circular
      facts := log.facts.filter fun (a, r) => decide (a ≠ cell ∧ r ≠ cell) }

/-- `learn`: the anchor runs before the reader. If the facts already place
the reader before the anchor, everything the facts place between them is
on a loop of anchors reading each other's areas: all of it is marked.
Otherwise the fact is kept and the order repaired: the anchor and whatever
the facts place before it move from behind the reader to just before it, in
their present order. Returns the log, the order, and the anchors marked. -/
def RestartLog.learn (log : RestartLog Pos) (anchor reader : Pos) (order : List Pos) :
    RestartLog Pos × List Pos × List Pos :=
  if reader = anchor then (log, order, []) else
  let before := log.before anchor
  if reader ∈ before then
    let onLoop := before.filter fun c => decide (c ∈ log.after reader ∧ c ∉ log.circular)
    (onLoop.foldl RestartLog.mark log, order, onLoop)
  else
    let i := order.idxOf reader
    let R := order.drop (i + 1)
    ({ log with facts := log.facts ++ [(anchor, reader)] },
      order.take i ++ R.filter (fun c => decide (c ∈ before)) ++
        reader :: R.filter (fun c => decide (c ∉ before)),
      [])

/-- One reader of a restart: learned unless the anchor is marked by now (its
facts are spent) or the reader is (on a loop, nothing to order it against). -/
def RestartLog.learnStep (anchor : Pos) (acc : RestartLog Pos × List Pos × List Pos)
    (reader : Pos) : RestartLog Pos × List Pos × List Pos :=
  if anchor ∈ acc.1.circular ∨ reader ∈ acc.1.circular then acc
  else
    let (log, order, more) := acc.1.learn anchor reader acc.2.1
    (log, order, acc.2.2 ++ more)

/-- A self-contradiction marks its anchor before anything is learned. -/
def RestartLog.premark (log : RestartLog Pos) (r : Restart Pos) : RestartLog Pos × List Pos :=
  match r with
  | .selfContradiction a => if a ∈ log.circular then (log, []) else (log.mark a, [a])
  | _ => (log, [])

/-- `RestartLog::record`: learns the facts of a restart, one reader at a
time. Returns the log, the repaired order, and the anchors newly marked. -/
def RestartLog.record (log : RestartLog Pos) (r : Restart Pos) (order : List Pos) :
    RestartLog Pos × List Pos × List Pos :=
  let (log, marked) := RestartLog.premark { log with restarts := log.restarts + 1 } r
  r.readers.foldl (RestartLog.learnStep r.anchor) (log, order, marked)

/-- What `restore_dynamic_spills` does once the stale cells of `a` at `cells`
have left the snapshot: they are empty. Only spill cells of `a` are touched. -/
def dropStale (S : Sheet Pos Value) (a : Pos) (cells : List Pos) : Sheet Pos Value :=
  fun q => if q ∈ cells ∧ (S q).spillAnchor? = some a then .empty else S q

/-- What `restore_dynamic_spills` does once the spill cells of the anchors
in `marked` have left the snapshot: they are empty. Only spill cells of
dynamic anchors are touched (the marked anchors are dynamic anchors; the
check spares the proofs a side condition). -/
def dropMarked (S : Sheet Pos Value) (marked : List Pos) : Sheet Pos Value :=
  fun q => match S q with
    | .spill a _ => if a ∈ marked ∧ (S a).isDynAnchor = true then .empty else S q
    | _ => S q

/-- The sheet the next pass starts from: without the stale cells a restart
gave up, and without the spill cells of the anchors it marked. -/
def Restart.nextSheet (r : Restart Pos) (marked : List Pos) (S : Sheet Pos Value) :
    Sheet Pos Value :=
  dropMarked (match r with
    | .staleCells a cells => dropStale S a cells
    | _ => S) marked

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

theorem dropMarked_eq_or (S : Sheet Pos Value) (marked : List Pos) (q : Pos) :
    dropMarked S marked q = S q ∨
      (dropMarked S marked q = .empty ∧
        ∃ a v, S q = .spill a v ∧ a ∈ marked ∧ (S a).isDynAnchor = true) := by
  unfold dropMarked
  split
  · rename_i a v h
    split
    · rename_i hm
      exact Or.inr ⟨rfl, a, v, h, hm.1, hm.2⟩
    · exact Or.inl rfl
  · exact Or.inl rfl

theorem dropMarked_isDynAnchor (S : Sheet Pos Value) (marked : List Pos) (q : Pos) :
    (dropMarked S marked q).isDynAnchor = (S q).isDynAnchor := by
  rcases dropMarked_eq_or S marked q with h | ⟨h, a, v, hs, _⟩
  · rw [h]
  · rw [h, hs]
    rfl

theorem dropMarked_isAnchor (S : Sheet Pos Value) (marked : List Pos) (q : Pos) :
    (dropMarked S marked q).isAnchor = (S q).isAnchor := by
  rcases dropMarked_eq_or S marked q with h | ⟨h, a, v, hs, _⟩
  · rw [h]
  · rw [h, hs]
    rfl

theorem dropMarked_spill {S : Sheet Pos Value} {marked : List Pos} {q b : Pos} {v : Value}
    (h : dropMarked S marked q = .spill b v) :
    S q = .spill b v ∧ ¬ (b ∈ marked ∧ (S b).isDynAnchor = true) := by
  unfold dropMarked at h
  revert h
  split
  · rename_i a w hs
    split
    · intro h
      cases h
    · rename_i hna
      intro h
      refine ⟨h, ?_⟩
      rw [hs] at h
      cases h
      exact hna
  · rename_i hns
    intro h
    exact absurd h (hns b v)

theorem dropMarked_cse {S : Sheet Pos Value} {marked : List Pos} {q : Pos}
    {t : Formula Pos Value} {area : List Pos} {v : Value}
    (h : dropMarked S marked q = .cseAnchor t area v) : S q = .cseAnchor t area v := by
  rcases dropMarked_eq_or S marked q with h' | ⟨h', _⟩
  · rw [← h']
    exact h
  · rw [h'] at h
    cases h

/-- Dropping the spill cells of dynamic anchors keeps the sheet well-formed. -/
theorem WellFormed.dropMarked {S : Sheet Pos Value} (hwf : WellFormed S) (marked : List Pos) :
    WellFormed (dropMarked S marked) := by
  obtain ⟨hno, hcse, hin⟩ := hwf
  refine ⟨?_, ?_, ?_⟩
  · intro q b v hq
    rw [dropMarked_isAnchor]
    exact hno q b v (dropMarked_spill hq).1
  · intro p t area v hp q hq hqp
    have hp' := dropMarked_cse hp
    obtain ⟨w, hw⟩ := hcse p t area v hp' q hq hqp
    refine ⟨w, ?_⟩
    rcases dropMarked_eq_or S marked q with h | ⟨_, a, w', hs, _, hdyn⟩
    · rw [h, hw]
    · -- A cell of a CSE area is the CSE anchor's, not a dynamic anchor's.
      rw [hw] at hs
      cases hs
      rw [hp'] at hdyn
      simp [Content.isDynAnchor] at hdyn
  · intro q b w hq t area v hb
    exact hin q b w (dropMarked_spill hq).1 t area v (dropMarked_cse hb)

theorem WellFormed.nextSheet {S : Sheet Pos Value} (hwf : WellFormed S) {r : Restart Pos}
    (hr : (S r.anchor).isDynAnchor = true) (marked : List Pos) :
    WellFormed (r.nextSheet marked S) := by
  unfold Restart.nextSheet
  cases r with
  | staleCells a cells => exact (hwf.dropStale hr cells).dropMarked marked
  | _ => exact hwf.dropMarked marked

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
          let (log', order', marked) := log.record r order
          evaluateLoop U k (r.nextSheet marked S) order' log'

/-- `Model::evaluate`: evaluates every formula, starting from the sheet `S`
with the remembered `anchorOrder`. Returns the consistent sheet, the new
anchor order and the log, or `none` if `fuel` passes were not enough. -/
def evaluate (U : Universe Pos) (S : Sheet Pos Value) (anchorOrder : List Pos) (fuel : Nat) :
    Option (Sheet Pos Value × List Pos × RestartLog Pos) :=
  let order := syncAnchorOrder U S anchorOrder
  evaluateLoop U fuel S order RestartLog.new

end

end IronCalcEval

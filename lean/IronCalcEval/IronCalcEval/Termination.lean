import IronCalcEval.Loop
import Mathlib.Data.Nat.Factorial.Basic
import Mathlib.Data.Finset.Card
import Mathlib.Data.List.Permutation
import Mathlib.Tactic.Ring

/-!
# Termination

`cold-evaluation.md`, section 5.3, with one correction.

The design document argues: "a marked anchor never restarts again: wherever
it is in the order it stores `#CIRC!` without spilling and keeps no spill
cells". The second half is not what the code does. Every pass restores the
spill cells the anchor left from a *previous* evaluation, and a marked anchor
only removes them when its turn comes (`retire_own_spill_cells`, with the
contradiction check). So an unmarked anchor placed before it can be blocked
by those cells (`Occupied`, then a `Conflict` when they are removed) or read
one of them (`StaleRead`). Both restart the marked anchor and move it to the
front. `marked_restart_has_unmarked_before` is the statement that survives:
a marked anchor restarts only if an unmarked anchor precedes it, and then it
jumps in front of that anchor.

Termination then follows from the budget alone. Once the budget is spent every
restart marks its anchor: an unmarked one grows the circular set (at most
`n` times), a marked one moves in front of every unmarked anchor and creates
no new such pair, so the number of (unmarked before marked) pairs drops. The
pigeonhole argument of section 5.3 is what keeps the budget from being spent
in the first place; it is stated separately.

A stale-cells restart (`Restart.staleCells`) marks nothing and does not count
against the budget; it empties at least one spill cell of the sheet the
passes start from, and nothing ever puts one back. The number of spill cells
of that sheet is therefore the leading component of the measure.
-/

namespace IronCalcEval

variable {Pos Value : Type} [DecidableEq Pos] [ValueSort Value]

/-- A marked anchor restarts only when an unmarked anchor precedes it in the
order. -/
theorem marked_restart_has_unmarked_before (U : Universe Pos) (S S' : Sheet Pos Value)
    (order : List Pos) (circular : Finset Pos) (r : Restart Pos)
    (hwf : WellFormed S) (horder : OrderOf S order)
    (h : runPass U S order circular = (S', some r)) (hm : r.anchor ∈ circular) :
    ∃ a ∈ order, a ∉ circular ∧ order.idxOf a < order.idxOf r.anchor := by
  rw [runPass_eq] at h
  simp only [Prod.mk.injEq] at h
  obtain ⟨_, hrest⟩ := h
  have hloop : LoopInv S order circular [] (PassState.initial S circular) :=
    { inv := PassInv.initial S circular hwf
      stack := rfl
      circ := fun x hx => hx
      done := by simp
      records := fun ⟨x, r', h⟩ => by simp [PassState.initial] at h }
  exact passBody_marked S U order horder.1 horder circular (order ++ U.positions) []
    (PassState.initial S circular) hloop rfl r hrest hm

/-- The restarting anchor is always an anchor of the order. -/
theorem restart_anchor_mem (U : Universe Pos) (S S' : Sheet Pos Value)
    (order : List Pos) (circular : Finset Pos) (r : Restart Pos)
    (hwf : WellFormed S) (horder : OrderOf S order)
    (h : runPass U S order circular = (S', some r)) :
    r.anchor ∈ order := by
  rw [runPass_eq] at h
  simp only [Prod.mk.injEq] at h
  obtain ⟨_, hrest⟩ := h
  have hok := passBody_restartOk S U (order ++ U.positions) (PassState.initial S circular)
    (PassInv.initial S circular hwf) rfl r hrest
  exact (horder.2 r.anchor).mpr hok.1

/-! ## The log -/

/-- What `RestartLog.record` maintains: the orders seen since the circular
set last changed are distinct, and all are permutations of the initial order.
There is one move fewer than orders seen (the first order was not reached by
a move), and every anchor mentioned is an anchor of the initial order. -/
structure LogInv (order₀ : List Pos) (log : RestartLog Pos) : Prop where
  ordersSeen_nodup : log.ordersSeen.Nodup
  ordersSeen_perm : ∀ o ∈ log.ordersSeen, o.Perm order₀
  moves_length : log.moves.length + 1 = log.ordersSeen.length
  moves_sub : ∀ a ∈ log.moves, a ∈ order₀
  circular_sub : ∀ a ∈ log.circular, a ∈ order₀

omit [DecidableEq Pos] in
theorem LogInv.new (order₀ : List Pos) : LogInv order₀ (RestartLog.new order₀) where
  ordersSeen_nodup := List.nodup_singleton _
  ordersSeen_perm := by
    intro o ho
    simp only [RestartLog.new, List.mem_singleton] at ho
    exact ho ▸ List.Perm.refl _
  moves_length := rfl
  moves_sub := by simp [RestartLog.new]
  circular_sub := by simp [RestartLog.new]

/-- Everything a restart marks is an anchor of the initial order. -/
theorem newlyCircular_sub (order₀ order : List Pos) (log : RestartLog Pos) (r : Restart Pos)
    (hmoves : ∀ a ∈ log.moves, a ∈ order₀) (hmem : r.anchor ∈ order₀) :
    ∀ a ∈ log.newlyCircular r order, a ∈ order₀ := by
  intro a ha
  have hall : ∀ a ∈ log.moves ++ [r.anchor], a ∈ order₀ := by
    intro a ha
    rcases List.mem_append.mp ha with ha | ha
    · exact hmoves a ha
    · exact List.mem_singleton.mp ha ▸ hmem
  unfold RestartLog.newlyCircular at ha
  split at ha
  · exact List.mem_singleton.mp ha ▸ hmem
  · split at ha
    · exact hall a (List.mem_of_mem_drop ha)
    · simp at ha

/-- An order that repeats always marks something: the move that produced it
is among those since it was first seen. -/
theorem newlyCircular_ne_nil_of_mem (order : List Pos) (log : RestartLog Pos) (r : Restart Pos)
    (hlen : log.moves.length + 1 = log.ordersSeen.length) (hin : order ∈ log.ordersSeen) :
    (log.newlyCircular r order).isEmpty = false := by
  unfold RestartLog.newlyCircular
  split
  · rfl
  · split
    · rename_i i hi
      have hlt : i < log.ordersSeen.length :=
        (List.findIdx?_eq_some_iff_getElem.mp hi).1
      rw [List.isEmpty_eq_false_iff]
      apply List.ne_nil_of_length_pos
      simp only [List.length_drop, List.length_append, List.length_singleton]
      omega
    · rename_i hnone
      rw [List.findIdx?_eq_none_iff] at hnone
      have := hnone order hin
      simp at this

theorem LogInv.record (order₀ order : List Pos) (log : RestartLog Pos) (r : Restart Pos)
    (hinv : LogInv order₀ log) (hperm : order.Perm order₀) (hmem : r.anchor ∈ order₀) :
    LogInv order₀ (log.record r order) := by
  unfold RestartLog.record
  dsimp only
  split
  · exact
      { ordersSeen_nodup := List.nodup_singleton _
        ordersSeen_perm := by
          intro o ho
          exact List.mem_singleton.mp ho ▸ hperm
        moves_length := rfl
        moves_sub := by simp
        circular_sub := hinv.circular_sub }
  split
  · rename_i hempty
    have hnew : order ∉ log.ordersSeen := fun hin => by
      rw [newlyCircular_ne_nil_of_mem order log r hinv.moves_length hin] at hempty
      exact Bool.false_ne_true hempty
    exact
      { ordersSeen_nodup := by
          rw [List.nodup_append]
          refine ⟨hinv.ordersSeen_nodup, List.nodup_singleton _, ?_⟩
          intro a ha b hb hab
          exact hnew (List.mem_singleton.mp hb ▸ hab ▸ ha)
        ordersSeen_perm := by
          intro o ho
          rcases List.mem_append.mp ho with ho | ho
          · exact hinv.ordersSeen_perm o ho
          · exact List.mem_singleton.mp ho ▸ hperm
        moves_length := by simp [hinv.moves_length]
        moves_sub := by
          intro a ha
          rcases List.mem_append.mp ha with ha | ha
          · exact hinv.moves_sub a ha
          · exact List.mem_singleton.mp ha ▸ hmem
        circular_sub := hinv.circular_sub }
  · exact
      { ordersSeen_nodup := List.nodup_singleton _
        ordersSeen_perm := by
          intro o ho
          exact List.mem_singleton.mp ho ▸ hperm
        moves_length := rfl
        moves_sub := by simp
        circular_sub := by
          intro a ha
          rcases Finset.mem_union.mp ha with ha | ha
          · exact hinv.circular_sub a ha
          · exact newlyCircular_sub order₀ order log r hinv.moves_sub hmem a
              (List.mem_toFinset.mp ha) }

/-- Pigeonhole (5.3): between two changes of the circular set there are at
most `n!` orders. -/
theorem ordersSeen_le_factorial (order₀ : List Pos) (log : RestartLog Pos)
    (hinv : LogInv order₀ log) :
    log.ordersSeen.length ≤ order₀.length.factorial := by
  rw [← List.length_permutations]
  exact (List.subperm_of_subset hinv.ordersSeen_nodup fun o ho =>
    List.mem_permutations.mpr (hinv.ordersSeen_perm o ho)).length_le

/-- The circular set never shrinks. -/
theorem record_circular_mono (log : RestartLog Pos) (r : Restart Pos) (order : List Pos) :
    log.circular ⊆ (log.record r order).circular := by
  unfold RestartLog.record
  dsimp only
  split
  · exact Finset.Subset.refl _
  split
  · exact Finset.Subset.refl _
  · exact Finset.subset_union_left

/-- More fuel never changes a result that was reached. -/
theorem evaluateLoop_mono (U : Universe Pos) (k k' : Nat) (S : Sheet Pos Value)
    (order : List Pos) (log : RestartLog Pos)
    (hk : k ≤ k') (h : (evaluateLoop U k S order log).isSome) :
    evaluateLoop U k' S order log = evaluateLoop U k S order log := by
  induction k generalizing k' S order log with
  | zero => simp [evaluateLoop] at h
  | succ k ih =>
    obtain ⟨k'', rfl⟩ : ∃ k'', k' = k'' + 1 := ⟨k' - 1, by omega⟩
    rcases hrun : runPass U S order log.circular with ⟨S', _ | r⟩
    · simp [evaluateLoop, hrun]
    · simp only [evaluateLoop, hrun] at h ⊢
      exact ih _ _ _ _ (by omega) h

/-! ## The potential

After the budget is spent, every restart marks its anchor. A restart by an
unmarked anchor marks a new anchor. A restart by a marked anchor moves it in
front of an unmarked anchor that preceded it (`marked_restart_has_unmarked_before`),
which removes at least one pair (unmarked before marked) and creates none.
Before the budget is spent, the count of restarts left does the work. -/

section potential
variable (circ : Finset Pos)

/-- How many anchors of the order are unmarked. -/
def unmarked (l : List Pos) : Nat := l.countP fun x => decide (x ∉ circ)

/-- How many are marked. -/
def marked (l : List Pos) : Nat := l.countP fun x => decide (x ∈ circ)

/-- Pairs of an unmarked anchor before a marked one: for each unmarked
anchor, the marked ones after it. -/
def pot : List Pos → Nat
  | [] => 0
  | x :: l => (if x ∈ circ then 0 else marked circ l) + pot l

/-- The unmarked anchors before `a`. -/
def unmarkedBefore (a : Pos) : List Pos → Nat
  | [] => 0
  | x :: l => if x = a then 0 else (if x ∈ circ then 0 else 1) + unmarkedBefore a l

omit [ValueSort Value] in
theorem marked_le_length (l : List Pos) : marked circ l ≤ l.length :=
  List.countP_le_length

omit [ValueSort Value] in
theorem pot_le (l : List Pos) : pot circ l ≤ l.length * l.length := by
  induction l with
  | nil => simp [pot]
  | cons x l ih =>
      simp only [pot, List.length_cons]
      have h1 := marked_le_length circ l
      have h2 : (l.length + 1) * (l.length + 1) = l.length * l.length + 2 * l.length + 1 := by ring
      rw [h2]
      split <;> omega

omit [ValueSort Value] in
theorem marked_perm {l l' : List Pos} (h : l.Perm l') : marked circ l = marked circ l' :=
  h.countP_eq _

omit [ValueSort Value] in
theorem unmarked_perm {l l' : List Pos} (h : l.Perm l') : unmarked circ l = unmarked circ l' :=
  h.countP_eq _

omit [ValueSort Value] in
theorem marked_erase {a : Pos} {l : List Pos} (ha : a ∈ l) (hac : a ∈ circ) :
    marked circ l = marked circ (l.erase a) + 1 := by
  rw [marked_perm circ (List.perm_cons_erase ha)]
  simp [marked, List.countP_cons, hac]

omit [ValueSort Value] in
/-- Removing a marked anchor removes the pairs it closes. -/
theorem pot_erase {a : Pos} : ∀ {l : List Pos}, a ∈ l → a ∈ circ → l.Nodup →
    pot circ l = pot circ (l.erase a) + unmarkedBefore circ a l
  | [], h, _, _ => absurd h List.not_mem_nil
  | x :: l, ha, hac, hnd => by
      by_cases hxa : x = a
      · subst hxa
        simp [pot, unmarkedBefore, hac]
      · have ha' : a ∈ l := by
          rcases List.mem_cons.mp ha with h | h
          · exact absurd h.symm hxa
          · exact h
        rw [List.erase_cons_tail (a := a) (b := x) (l := l) (by simp [hxa])]
        simp only [pot, unmarkedBefore, hxa, ↓reduceIte]
        rw [pot_erase ha' hac (List.nodup_cons.mp hnd).2, marked_erase circ ha' hac]
        split <;> omega

omit [ValueSort Value] in
/-- An unmarked anchor before `a` is counted. -/
theorem unmarkedBefore_pos {a u : Pos} : ∀ {l : List Pos}, u ∈ l → u ∉ circ →
    l.idxOf u < l.idxOf a → 1 ≤ unmarkedBefore circ a l
  | [], h, _, _ => absurd h List.not_mem_nil
  | x :: l, hu, huc, hlt => by
      by_cases hxa : x = a
      · subst hxa
        simp at hlt
      · simp only [unmarkedBefore, hxa, ↓reduceIte]
        by_cases hxu : x = u
        · subst hxu
          simp [huc]
        · have hu' : u ∈ l := by
            rcases List.mem_cons.mp hu with h | h
            · exact absurd h.symm hxu
            · exact h
          rw [List.idxOf_cons_ne _ hxu, List.idxOf_cons_ne _ hxa] at hlt
          have := unmarkedBefore_pos hu' huc (Nat.lt_of_succ_lt_succ hlt)
          split <;> omega

omit [ValueSort Value] in
/-- Marking an unmarked anchor of the order lowers the count by one. -/
theorem unmarked_insert {a : Pos} {l : List Pos} (ha : a ∈ l) (hac : a ∉ circ) (hnd : l.Nodup) :
    unmarked (insert a circ) l + 1 = unmarked circ l := by
  rw [unmarked_perm circ (List.perm_cons_erase ha),
    unmarked_perm (insert a circ) (List.perm_cons_erase ha)]
  have hnot : a ∉ l.erase a :=
    fun h => (List.nodup_cons.mp ((List.perm_cons_erase ha).nodup_iff.mp hnd)).1 h
  have hcongr : (l.erase a).countP (fun x => decide (x ∉ insert a circ)) =
      (l.erase a).countP (fun x => decide (x ∉ circ)) :=
    List.countP_congr fun x hx => by
      have hxa : x ≠ a := fun h => hnot (h ▸ hx)
      simp [hxa]
  simp only [unmarked, List.countP_cons, hcongr]
  simp [hac]

omit [ValueSort Value] in
theorem unmarked_pos {a : Pos} {l : List Pos} (ha : a ∈ l) (hac : a ∉ circ) : 1 ≤ unmarked circ l :=
  List.countP_pos_iff.mpr ⟨a, ha, by simp [hac]⟩

omit [ValueSort Value] in
theorem unmarked_le_length (l : List Pos) : unmarked circ l ≤ l.length :=
  List.countP_le_length

end potential

omit [ValueSort Value] in
/-- On a list without duplicates, moving to the front is `cons` after `erase`. -/
theorem moveToFront_eq {l : List Pos} (a : Pos) (hnd : l.Nodup) :
    moveToFront l a = a :: l.erase a := by
  unfold moveToFront
  congr 1
  induction l with
  | nil => rfl
  | cons x l ih =>
      by_cases hxa : x = a
      · subst hxa
        simp only [List.filter_cons, ne_eq, not_true_eq_false, decide_false, ↓reduceIte,
          List.erase_cons_head]
        exact List.filter_eq_self.mpr fun y hy => by
          have := (List.nodup_cons.mp hnd).1
          simp [show y ≠ x from fun h => this (h ▸ hy)]
      · simp only [List.filter_cons, ne_eq, hxa, not_false_eq_true, decide_true, ↓reduceIte,
          List.erase_cons_tail (a := a) (b := x) (l := l) (by simp [hxa])]
        rw [ih (List.nodup_cons.mp hnd).2]

omit [ValueSort Value] in
theorem moveToFront_perm {l : List Pos} {a : Pos} (ha : a ∈ l) (hnd : l.Nodup) :
    (moveToFront l a).Perm l := by
  rw [moveToFront_eq a hnd]
  exact (List.perm_cons_erase ha).symm

omit [ValueSort Value] in
/-- `record` once the budget is spent, by a restart that counts: the anchor
is marked. -/
theorem record_post_budget (log : RestartLog Pos) (r : Restart Pos) (order : List Pos)
    (hns : r.isStaleCells = false) (hb : log.budget ≤ log.spent) :
    (log.record r order).circular = insert r.anchor log.circular := by
  have hnew : log.newlyCircular r order = [r.anchor] := by
    unfold RestartLog.newlyCircular
    rw [if_pos (by simp [hb])]
  unfold RestartLog.record
  simp only [hns, Bool.false_eq_true, ↓reduceIte, hnew, List.isEmpty_cons, List.toFinset_cons,
    List.toFinset_nil]
  ext x
  simp [or_comm]

omit [ValueSort Value] in
theorem record_restarts (log : RestartLog Pos) (r : Restart Pos) (order : List Pos) :
    (log.record r order).restarts = log.restarts + 1 ∧ (log.record r order).budget = log.budget := by
  unfold RestartLog.record
  dsimp only
  split
  · exact ⟨rfl, rfl⟩
  split <;> exact ⟨rfl, rfl⟩

omit [ValueSort Value] in
/-- A restart that counts spends one unit of the budget. -/
theorem record_spent (log : RestartLog Pos) (r : Restart Pos) (order : List Pos)
    (hns : r.isStaleCells = false) : (log.record r order).spent = log.spent + 1 := by
  unfold RestartLog.record
  simp only [hns, Bool.false_eq_true, ↓reduceIte]
  split <;> rfl

omit [ValueSort Value] in
/-- A stale-cells restart marks nothing and spends nothing. -/
theorem record_stale (log : RestartLog Pos) (r : Restart Pos) (order : List Pos)
    (hs : r.isStaleCells = true) :
    (log.record r order).spent = log.spent ∧ (log.record r order).circular = log.circular := by
  unfold RestartLog.record
  simp [hs]

/-- The spill cells of the sheet the passes start from: what dropping stale
cells consumes. -/
def staleCount (U : Universe Pos) (S : Sheet Pos Value) : Nat :=
  (U.positions.toFinset.filter fun q => (S q).spillAnchor?.isSome).card

omit [ValueSort Value] in
theorem staleCount_dropStale_lt (U : Universe Pos) (S : Sheet Pos Value) (a : Pos)
    (cells : List Pos) (hex : ∃ q ∈ cells, ∃ v, S q = .spill a v) :
    staleCount U (dropStale S a cells) < staleCount U S := by
  obtain ⟨q₀, hq₀, v, hv⟩ := hex
  apply Finset.card_lt_card
  rw [Finset.ssubset_iff_of_subset]
  · refine ⟨q₀, ?_, ?_⟩
    · exact Finset.mem_filter.mpr ⟨List.mem_toFinset.mpr (U.complete q₀), by rw [hv]; rfl⟩
    · intro hmem
      have hdrop : dropStale S a cells q₀ = .empty := by
        simp [dropStale, hq₀, hv, Content.spillAnchor?]
      have := (Finset.mem_filter.mp hmem).2
      rw [hdrop] at this
      cases this
  · intro q hq
    rw [Finset.mem_filter] at hq ⊢
    refine ⟨hq.1, ?_⟩
    rcases dropStale_eq_or S a cells q with h | ⟨h, _⟩
    · rw [h] at hq
      exact hq.2
    · rw [h] at hq
      simp [Content.spillAnchor?] at hq

/-- The measure: spill cells of the starting sheet, then restarts left before
the budget, then the potential. -/
def measure (U : Universe Pos) (n : Nat) (S : Sheet Pos Value) (order : List Pos)
    (log : RestartLog Pos) : Nat :=
  staleCount U S * ((log.budget + 1) * ((n + 1) * (n * n + 1))) +
    ((log.budget - log.spent) * ((n + 1) * (n * n + 1)) +
      (unmarked log.circular order * (n * n + 1) + pot log.circular order))

omit [ValueSort Value] in
theorem postM_lt {n : Nat} {order : List Pos} {circ : Finset Pos} (hlen : order.length ≤ n) :
    unmarked circ order * (n * n + 1) + pot circ order < (n + 1) * (n * n + 1) := by
  have h1 := unmarked_le_length circ order
  have h2 := pot_le circ order
  have h3 : order.length * order.length ≤ n * n := Nat.mul_le_mul hlen hlen
  have h4 : (n + 1) * (n * n + 1) = n * (n * n + 1) + n * n + 1 := by ring
  have h5 : unmarked circ order * (n * n + 1) ≤ n * (n * n + 1) :=
    Nat.mul_le_mul_right _ (h1.trans hlen)
  omega

/-- The driver's invariant: the order is a permutation of the initial one, and
the log is well-formed. -/
structure DriverInv (order₀ order : List Pos) (log : RestartLog Pos) : Prop where
  perm : order.Perm order₀
  log : LogInv order₀ log

omit [ValueSort Value] in
theorem OrderOf.of_perm {S : Sheet Pos Value} {order₀ order : List Pos} (h : OrderOf S order₀)
    (hp : order.Perm order₀) : OrderOf S order :=
  ⟨hp.nodup_iff.mpr h.1, fun a => (hp.mem_iff).trans (h.2 a)⟩

omit [ValueSort Value] in
/-- Dropping stale cells changes no anchor. -/
theorem OrderOf.nextSheet {S : Sheet Pos Value} {order : List Pos} (h : OrderOf S order)
    (r : Restart Pos) : OrderOf (r.nextSheet S) order := by
  cases r with
  | staleCells a cells =>
      exact ⟨h.1, fun b => by
        change b ∈ order ↔ (dropStale S a cells b).isDynAnchor = true
        rw [dropStale_isDynAnchor]
        exact h.2 b⟩
  | _ => exact h

/-- One restart lowers the measure. -/
theorem measure_lt (U : Universe Pos) (S S' : Sheet Pos Value) (hwf : WellFormed S)
    (order₀ : List Pos) (horder₀ : OrderOf S order₀) (order : List Pos) (log : RestartLog Pos)
    (hinv : DriverInv order₀ order log) (r : Restart Pos)
    (h : runPass U S order log.circular = (S', some r)) :
    measure U order₀.length (r.nextSheet S) (moveToFront order r.anchor)
        (log.record r (moveToFront order r.anchor)) <
      measure U order₀.length S order log := by
  set n := order₀.length with hn
  have horder : OrderOf S order := horder₀.of_perm hinv.perm
  have hnd : order.Nodup := horder.1
  have ha : r.anchor ∈ order := restart_anchor_mem U S S' order log.circular r hwf horder h
  have hlen : order.length = n := hinv.perm.length_eq
  set order' := moveToFront order r.anchor with horder'
  have hperm' : order'.Perm order := moveToFront_perm ha hnd
  have hlen' : order'.length = n := hperm'.length_eq.trans hlen
  obtain ⟨-, hbud⟩ := record_restarts log r order'
  unfold measure
  rw [hbud]
  set K := (n + 1) * (n * n + 1) with hK
  cases hs : r.isStaleCells
  swap
  · -- Stale cells dropped: the starting sheet has fewer spill cells, and
    -- everything below is bounded.
    obtain ⟨a, cells, rfl⟩ : ∃ a cells, r = .staleCells a cells := by
      cases r <;> simp [Restart.isStaleCells] at hs
      exact ⟨_, _, rfl⟩
    have hstale : ∃ q ∈ cells, ∃ v, S q = .spill a v :=
      (runPass_restartOk U S S' order log.circular _ hwf h).2
    obtain ⟨hspent, hcirc⟩ := record_stale log (.staleCells a cells) order' rfl
    rw [hspent, hcirc]
    simp only [Restart.nextSheet]
    have hlt := staleCount_dropStale_lt U S a cells hstale
    have hpost := postM_lt (n := n) (order := order') (circ := log.circular) (by rw [hlen'])
    rw [← hK] at hpost
    have hle : log.budget - log.spent ≤ log.budget := Nat.sub_le _ _
    have hB : (log.budget - log.spent) * K + K ≤ (log.budget + 1) * K := by
      rw [Nat.succ_mul]
      exact Nat.add_le_add_right (Nat.mul_le_mul_right _ hle) _
    have hup : (staleCount U (dropStale S a cells) + 1) * ((log.budget + 1) * K) ≤
        staleCount U S * ((log.budget + 1) * K) :=
      Nat.mul_le_mul_right _ hlt
    rw [Nat.succ_mul] at hup
    omega
  -- A restart that counts: the starting sheet is unchanged.
  rw [Restart.nextSheet_of_not_stale hs, record_spent log r order' hs]
  by_cases hb : log.budget ≤ log.spent
  · -- The budget is spent: the potential decides.
    have hcirc := record_post_budget log r order' hs hb
    rw [hcirc]
    have h0 : log.budget - log.spent = 0 := by omega
    have h0' : log.budget - (log.spent + 1) = 0 := by omega
    rw [h0, h0']
    simp only [Nat.zero_mul, Nat.zero_add]
    by_cases hac : r.anchor ∈ log.circular
    · -- A marked anchor: it jumps in front of an unmarked one.
      rw [Finset.insert_eq_of_mem hac]
      obtain ⟨u, hu, huc, hlt⟩ :=
        marked_restart_has_unmarked_before U S S' order log.circular r hwf horder h hac
      rw [unmarked_perm _ hperm']
      have hpot : pot log.circular order' < pot log.circular order := by
        rw [horder', moveToFront_eq _ hnd]
        simp only [pot, hac, ↓reduceIte, Nat.zero_add]
        rw [pot_erase log.circular ha hac hnd]
        have := unmarkedBefore_pos log.circular hu huc hlt
        omega
      omega
    · -- An unmarked anchor: one fewer.
      have hu := unmarked_insert log.circular ha hac hnd
      rw [unmarked_perm _ hperm']
      have hbound := pot_le (insert r.anchor log.circular) order'
      rw [hlen'] at hbound
      have hpos := unmarked_pos log.circular ha hac
      have h3 : n * n ≤ n * n := le_refl _
      have : unmarked (insert r.anchor log.circular) order * (n * n + 1) + n * n <
          unmarked log.circular order * (n * n + 1) := by
        rw [← hu]
        ring_nf
        omega
      omega
  · -- Before the budget: one restart fewer to go.
    have hsub : log.budget - (log.spent + 1) + 1 = log.budget - log.spent := by omega
    have hpost := postM_lt (n := n) (order := order')
      (circ := (log.record r order').circular) (by rw [hlen'])
    rw [← hK] at hpost
    have hK' : (log.budget - log.spent) * K = (log.budget - (log.spent + 1)) * K + K := by
      rw [← hsub]
      ring
    rw [hK']
    omega

/-- The loop returns once the measure's worth of fuel is provided. The sheet
the passes start from stays well-formed with the same anchors. -/
theorem evaluateLoop_of_measure (U : Universe Pos) (order₀ : List Pos) :
    ∀ (m : Nat) (S : Sheet Pos Value) (order : List Pos) (log : RestartLog Pos),
      WellFormed S → OrderOf S order₀ → DriverInv order₀ order log →
      measure U order₀.length S order log = m → (evaluateLoop U (m + 1) S order log).isSome := by
  intro m
  induction m using Nat.strong_induction_on with
  | _ m ih =>
    intro S order log hwf horder₀ hinv hm
    rcases hrun : runPass U S order log.circular with ⟨S', _ | r⟩
    · simp [evaluateLoop, hrun]
    · simp only [evaluateLoop, hrun]
      have hlt := measure_lt U S S' hwf order₀ horder₀ order log hinv r hrun
      rw [hm] at hlt
      have horder : OrderOf S order := horder₀.of_perm hinv.perm
      have ha : r.anchor ∈ order := restart_anchor_mem U S S' order log.circular r hwf horder hrun
      have hdyn : (S r.anchor).isDynAnchor = true := (horder.2 r.anchor).mp ha
      have hinv' : DriverInv order₀ (moveToFront order r.anchor)
          (log.record r (moveToFront order r.anchor)) :=
        { perm := (moveToFront_perm ha horder.1).trans hinv.perm
          log := hinv.log.record _ _ _ _ ((moveToFront_perm ha horder.1).trans hinv.perm)
            (hinv.perm.mem_iff.mp ha) }
      have := ih _ hlt _ _ _ (hwf.nextSheet hdyn) (horder₀.nextSheet r) hinv' rfl
      rw [evaluateLoop_mono U _ m _ _ _ (by omega) this]
      exact this

/-! ## The driver -/


/-- The driver terminates: some amount of fuel is enough. -/
theorem evaluate_terminates (U : Universe Pos) (S : Sheet Pos Value) (anchorOrder : List Pos)
    (hwf : WellFormed S) (hnd : anchorOrder.Nodup) :
    ∃ fuel, (evaluate U S anchorOrder fuel).isSome := by
  unfold evaluate
  set order₀ := syncAnchorOrder U S anchorOrder with horder₀
  have horder : OrderOf S order₀ := syncAnchorOrder_orderOf U S anchorOrder hnd
  refine ⟨measure U order₀.length S order₀ (RestartLog.new order₀) + 1, ?_⟩
  exact evaluateLoop_of_measure U order₀ _ S order₀ (RestartLog.new order₀) hwf horder
    ⟨List.Perm.refl _, LogInv.new order₀⟩ rfl

end IronCalcEval

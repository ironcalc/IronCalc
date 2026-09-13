import IronCalcEval.Loop
import Mathlib.Data.Finset.Card
import Mathlib.Data.List.ProdSigma
import Mathlib.Data.List.Perm.Basic

/-!
# Termination

`cold-evaluation.md`, section 5.3.

Every pass starts from the same sheet, so a pass is a function of the anchor
order and of the circular set. A restart names an anchor and the readers
that read its area before it ran (`RestartFacts`, from `Loop.lean`): each
is a fact, "the anchor runs before that reader", that the current order
breaks. The driver keeps the facts and repairs the order so that all of them
hold (`RestartLog.learn`). So every restart does one of three things:

* it adds a fact the order broke, hence one the driver did not know; facts
  are pairs of distinct anchors, so at most `n²` restarts do this;
* it marks an anchor circular, when the fact would close a loop among the
  facts or on a self-contradiction; the anchor of a restart is unmarked
  (a marked anchor's stale cells were dropped when it was marked, so it is
  neither read stale nor contradicted), and so are the readers (a marked
  anchor makes no records), so at most `n` restarts do this;
* it drops a stale cell of the starting sheet, at most once per cell.

The measure is lexicographic in that order, reversed: stale cells, then
unmarked anchors, then facts still to learn. No cap on restarts is needed.
-/

namespace IronCalcEval

open scoped List

variable {Pos Value : Type} [DecidableEq Pos] [ValueSort Value]

/-! ## Closure -/

section closure
variable (edges : List (Pos × Pos))

omit [ValueSort Value] in
theorem mem_closureStep {seen : List Pos} {x : Pos} (h : x ∈ seen) : x ∈ closureStep edges seen :=
  List.mem_append_left _ h

omit [ValueSort Value] in
theorem closureStep_sub {seen : List Pos} {x : Pos} (h : x ∈ closureStep edges seen) :
    x ∈ seen ∨ ∃ e ∈ edges, x = e.2 := by
  unfold closureStep at h
  rcases List.mem_append.mp h with h | h
  · exact Or.inl h
  · right
    rw [List.mem_dedup, List.mem_filterMap] at h
    obtain ⟨e, he, hx⟩ := h
    refine ⟨e, he, ?_⟩
    split at hx
    · cases hx
      rfl
    · cases hx

omit [ValueSort Value] in
theorem closureStep_nodup {seen : List Pos} (h : seen.Nodup) : (closureStep edges seen).Nodup := by
  unfold closureStep
  rw [List.nodup_append]
  refine ⟨h, List.nodup_dedup _, ?_⟩
  intro a ha b hb hab
  subst hab
  rw [List.mem_dedup, List.mem_filterMap] at hb
  obtain ⟨e, _, hx⟩ := hb
  split at hx
  · rename_i hc
    cases hx
    exact hc.2 ha
  · cases hx

omit [ValueSort Value] in
theorem closureStep_eq_or_lt (seen : List Pos) :
    closureStep edges seen = seen ∨ seen.length < (closureStep edges seen).length := by
  unfold closureStep
  rcases h : (edges.filterMap fun e => if e.1 ∈ seen ∧ e.2 ∉ seen then some e.2 else none).dedup
    with _ | ⟨z, l⟩
  · left
    rw [h, List.append_nil]
  · right
    rw [h]
    simp

omit [ValueSort Value] in
/-- A fixpoint of the step is closed under the edges. -/
theorem closureStep_fix_closed {seen : List Pos} (hfix : closureStep edges seen = seen)
    {e : Pos × Pos} (he : e ∈ edges) (hx : e.1 ∈ seen) : e.2 ∈ seen := by
  by_contra hy
  have hmem : e.2 ∈ (edges.filterMap fun e => if e.1 ∈ seen ∧ e.2 ∉ seen then some e.2
      else none).dedup := by
    rw [List.mem_dedup, List.mem_filterMap]
    exact ⟨e, he, by simp [hx, hy]⟩
  have := congrArg List.length hfix
  unfold closureStep at this
  rw [List.length_append] at this
  have hpos := List.length_pos_of_mem hmem
  omega

omit [ValueSort Value] in
theorem iterate_nodup (start : Pos) : ∀ k, ((closureStep edges)^[k] [start]).Nodup
  | 0 => List.nodup_singleton _
  | k + 1 => by
      rw [Function.iterate_succ_apply']
      exact closureStep_nodup edges (iterate_nodup start k)

omit [ValueSort Value] in
theorem iterate_sub (start : Pos) :
    ∀ k, ∀ x ∈ (closureStep edges)^[k] [start], x = start ∨ ∃ e ∈ edges, x = e.2
  | 0, x, hx => Or.inl (List.mem_singleton.mp hx)
  | k + 1, x, hx => by
      rw [Function.iterate_succ_apply'] at hx
      rcases closureStep_sub edges hx with h | h
      · exact iterate_sub start k x h
      · exact Or.inr h

omit [ValueSort Value] in
theorem iterate_length_le (start : Pos) (k : Nat) :
    ((closureStep edges)^[k] [start]).length ≤ edges.length + 1 := by
  have hsub : (closureStep edges)^[k] [start] ⊆ start :: edges.map Prod.snd := by
    intro x hx
    rcases iterate_sub edges start k x hx with rfl | ⟨e, he, rfl⟩
    · exact List.mem_cons_self
    · exact List.mem_cons_of_mem _ (List.mem_map.mpr ⟨e, he, rfl⟩)
  have := (List.subperm_of_subset (iterate_nodup edges start k) hsub).length_le
  simpa using this

omit [ValueSort Value] in
theorem iterate_mem_self (start : Pos) : ∀ k, start ∈ (closureStep edges)^[k] [start]
  | 0 => List.mem_singleton_self _
  | k + 1 => by
      rw [Function.iterate_succ_apply']
      exact mem_closureStep edges (iterate_mem_self start k)

omit [ValueSort Value] in
/-- Either the iteration has reached a fixpoint, or it has grown at every
step. -/
theorem iterate_fix_or_long (start : Pos) :
    ∀ k, closureStep edges ((closureStep edges)^[k] [start]) = (closureStep edges)^[k] [start] ∨
      k + 1 ≤ ((closureStep edges)^[k] [start]).length
  | 0 => Or.inr (by simp)
  | k + 1 => by
      rcases iterate_fix_or_long start k with h | h
      · left
        rw [Function.iterate_succ_apply', h, h]
      · rcases closureStep_eq_or_lt edges ((closureStep edges)^[k] [start]) with h' | h'
        · left
          rw [Function.iterate_succ_apply', h', h']
        · right
          rw [Function.iterate_succ_apply']
          omega

omit [ValueSort Value] in
theorem closure_fix (start : Pos) :
    closureStep edges (closure edges start) = closure edges start := by
  unfold closure
  rcases iterate_fix_or_long edges start (edges.length + 1) with h | h
  · exact h
  · have := iterate_length_le edges start (edges.length + 1)
    omega

omit [ValueSort Value] in
theorem mem_closure_self (start : Pos) : start ∈ closure edges start :=
  iterate_mem_self edges start _

omit [ValueSort Value] in
theorem closure_sub {start x : Pos} (h : x ∈ closure edges start) :
    x = start ∨ ∃ e ∈ edges, x = e.2 :=
  iterate_sub edges start _ x h

omit [ValueSort Value] in
theorem closure_closed {start : Pos} {e : Pos × Pos} (he : e ∈ edges)
    (hx : e.1 ∈ closure edges start) : e.2 ∈ closure edges start :=
  closureStep_fix_closed edges (closure_fix edges start) he hx

end closure

omit [ValueSort Value] in
theorem RestartLog.mem_before_self (log : RestartLog Pos) (a : Pos) : a ∈ log.before a :=
  mem_closure_self _ a

omit [ValueSort Value] in
theorem RestartLog.before_sub {log : RestartLog Pos} {a x : Pos} (h : x ∈ log.before a) :
    x = a ∨ ∃ f ∈ log.facts, x = f.1 := by
  rcases closure_sub _ h with h | ⟨e, he, rfl⟩
  · exact Or.inl h
  · right
    obtain ⟨f, hf, rfl⟩ := List.mem_map.mp he
    exact ⟨f, hf, rfl⟩

omit [ValueSort Value] in
/-- What the facts place before `a` is closed under the facts. -/
theorem RestartLog.before_closed {log : RestartLog Pos} {a : Pos} {f : Pos × Pos}
    (hf : f ∈ log.facts) (h : f.2 ∈ log.before a) : f.1 ∈ log.before a :=
  closure_closed _ (e := (f.2, f.1)) (List.mem_map.mpr ⟨f, hf, rfl⟩) h

omit [ValueSort Value] in
theorem RestartLog.mem_after_self (log : RestartLog Pos) (r : Pos) : r ∈ log.after r :=
  mem_closure_self _ r

/-! ## Marking -/

omit [ValueSort Value] in
theorem RestartLog.mark_circular_mono (log : RestartLog Pos) (c : Pos) :
    log.circular ⊆ (log.mark c).circular :=
  Finset.subset_insert _ _

omit [ValueSort Value] in
theorem foldl_mark_circular_mono (l : List Pos) :
    ∀ log : RestartLog Pos, log.circular ⊆ (l.foldl RestartLog.mark log).circular := by
  induction l with
  | nil => intro log; exact Finset.Subset.refl _
  | cons c l ih =>
      intro log
      exact Finset.Subset.trans (log.mark_circular_mono c) (ih _)

omit [ValueSort Value] in
theorem foldl_mark_mem (l : List Pos) :
    ∀ (log : RestartLog Pos) (x : Pos), x ∈ l → x ∈ (l.foldl RestartLog.mark log).circular := by
  induction l with
  | nil => intro _ _ h; exact absurd h List.not_mem_nil
  | cons c l ih =>
      intro log x hx
      rcases List.mem_cons.mp hx with rfl | hx
      · exact foldl_mark_circular_mono l _ (Finset.mem_insert_self _ _)
      · exact ih _ x hx

omit [ValueSort Value] in
theorem foldl_mark_circular_sub (l : List Pos) :
    ∀ (log : RestartLog Pos) (x : Pos), x ∈ (l.foldl RestartLog.mark log).circular →
      x ∈ log.circular ∨ x ∈ l := by
  induction l with
  | nil => intro _ _ h; exact Or.inl h
  | cons c l ih =>
      intro log x hx
      rcases ih _ x hx with h | h
      · rcases Finset.mem_insert.mp h with rfl | h
        · exact Or.inr List.mem_cons_self
        · exact Or.inl h
      · exact Or.inr (List.mem_cons_of_mem _ h)

omit [ValueSort Value] in
theorem foldl_mark_facts (l : List Pos) :
    ∀ log : RestartLog Pos, (l.foldl RestartLog.mark log).facts <+ log.facts := by
  induction l with
  | nil => intro _; exact List.Sublist.refl _
  | cons c l ih =>
      intro log
      exact (ih _).trans List.filter_sublist

/-! ## "Before" in a list: the two-element sublist -/

section pairs
variable {a b : Pos}

omit [DecidableEq Pos] [ValueSort Value] in
theorem pair_sublist_append_iff {L M : List Pos} :
    [a, b] <+ L ++ M ↔ [a, b] <+ L ∨ [a, b] <+ M ∨ (a ∈ L ∧ b ∈ M) := by
  constructor
  · intro h
    obtain ⟨l₁, l₂, hl, h₁, h₂⟩ := List.sublist_append_iff.mp h
    match l₁, l₂, hl with
    | [], _, hl =>
        simp only [List.nil_append] at hl
        subst hl
        exact Or.inr (Or.inl h₂)
    | [x], [y], hl =>
        simp only [List.cons_append, List.nil_append, List.cons.injEq] at hl
        obtain ⟨rfl, rfl, -⟩ := hl
        exact Or.inr (Or.inr ⟨List.singleton_sublist.mp h₁, List.singleton_sublist.mp h₂⟩)
    | [x, y], [], hl =>
        simp only [List.cons_append, List.nil_append, List.cons.injEq] at hl
        obtain ⟨rfl, rfl, -⟩ := hl
        exact Or.inl h₁
    | [x], y :: z :: _, hl => simp at hl
    | [x, y], z :: _, hl => simp at hl
    | x :: y :: z :: _, _, hl => simp at hl
  · rintro (h | h | ⟨ha, hb⟩)
    · exact h.trans (List.sublist_append_left L M)
    · exact h.trans (List.sublist_append_right L M)
    · exact List.Sublist.append (List.singleton_sublist.mpr ha) (List.singleton_sublist.mpr hb)

omit [DecidableEq Pos] [ValueSort Value] in
theorem pair_sublist_cons_iff {x : Pos} {R : List Pos} :
    [a, b] <+ x :: R ↔ [a, b] <+ R ∨ (a = x ∧ b ∈ R) := by
  rw [List.sublist_cons_iff]
  constructor
  · rintro (h | ⟨r, hr, hr'⟩)
    · exact Or.inl h
    · simp only [List.cons.injEq] at hr
      obtain ⟨rfl, rfl⟩ := hr
      exact Or.inr ⟨rfl, List.singleton_sublist.mp hr'⟩
  · rintro (h | ⟨rfl, hb⟩)
    · exact Or.inl h
    · exact Or.inr ⟨[b], rfl, List.singleton_sublist.mpr hb⟩

omit [DecidableEq Pos] [ValueSort Value] in
theorem pair_sublist_filter {R : List Pos} {p : Pos → Bool} (h : [a, b] <+ R) (ha : p a = true)
    (hb : p b = true) : [a, b] <+ R.filter p := by
  have := h.filter p
  simpa [ha, hb] using this

omit [DecidableEq Pos] [ValueSort Value] in
theorem pair_sublist_mem {l : List Pos} (h : [a, b] <+ l) : a ∈ l ∧ b ∈ l :=
  ⟨h.subset List.mem_cons_self, h.subset (List.mem_cons_of_mem _ List.mem_cons_self)⟩

omit [DecidableEq Pos] [ValueSort Value] in
theorem pair_sublist_ne {l : List Pos} (hnd : l.Nodup) (h : [a, b] <+ l) : a ≠ b := by
  have := hnd.sublist h
  simp at this
  exact this

omit [ValueSort Value] in
/-- On a list without duplicates, the sublist says the indices. -/
theorem idxOf_lt_of_pair_sublist : ∀ {l : List Pos}, l.Nodup → [a, b] <+ l →
    l.idxOf a < l.idxOf b
  | [], _, h => absurd (pair_sublist_mem h).1 List.not_mem_nil
  | x :: l, hnd, h => by
      have hx := List.nodup_cons.mp hnd
      rcases pair_sublist_cons_iff.mp h with h | ⟨rfl, hb⟩
      · obtain ⟨ha, hb⟩ := pair_sublist_mem h
        have hax : a ≠ x := fun e => hx.1 (e ▸ ha)
        have hbx : b ≠ x := fun e => hx.1 (e ▸ hb)
        rw [List.idxOf_cons_ne _ hax.symm, List.idxOf_cons_ne _ hbx.symm]
        exact Nat.succ_lt_succ (idxOf_lt_of_pair_sublist hx.2 h)
      · have hbx : b ≠ a := fun e => hx.1 (e ▸ hb)
        rw [List.idxOf_cons_self, List.idxOf_cons_ne _ hbx.symm]
        exact Nat.zero_lt_succ _

omit [ValueSort Value] in
theorem pair_sublist_of_idxOf_lt : ∀ {l : List Pos}, l.Nodup → a ∈ l → b ∈ l →
    l.idxOf a < l.idxOf b → [a, b] <+ l
  | [], _, ha, _, _ => absurd ha List.not_mem_nil
  | x :: l, hnd, ha, hb, hlt => by
      have hx := List.nodup_cons.mp hnd
      by_cases hax : a = x
      · subst hax
        have hbx : b ≠ a := by
          rintro rfl
          simp at hlt
        have hb' : b ∈ l := by
          rcases List.mem_cons.mp hb with h | h
          · exact absurd h hbx
          · exact h
        exact pair_sublist_cons_iff.mpr (Or.inr ⟨rfl, hb'⟩)
      · have hbx : b ≠ x := by
          rintro rfl
          rw [List.idxOf_cons_self] at hlt
          omega
        have ha' : a ∈ l := by
          rcases List.mem_cons.mp ha with h | h
          · exact absurd h hax
          · exact h
        have hb' : b ∈ l := by
          rcases List.mem_cons.mp hb with h | h
          · exact absurd h hbx
          · exact h
        rw [List.idxOf_cons_ne _ (Ne.symm hax), List.idxOf_cons_ne _ (Ne.symm hbx)] at hlt
        exact pair_sublist_cons_iff.mpr
          (Or.inl (pair_sublist_of_idxOf_lt hx.2 ha' hb' (Nat.lt_of_succ_lt_succ hlt)))

omit [ValueSort Value] in
theorem pair_sublist_asymm {l : List Pos} (hnd : l.Nodup) (h₁ : [a, b] <+ l)
    (h₂ : [b, a] <+ l) : False := by
  have := idxOf_lt_of_pair_sublist hnd h₁
  have := idxOf_lt_of_pair_sublist hnd h₂
  omega

omit [DecidableEq Pos] [ValueSort Value] in
/-- Both in the left part of a duplicate-free append: the sublist is there. -/
theorem pair_sublist_left {L M : List Pos} (hnd : (L ++ M).Nodup) (h : [a, b] <+ L ++ M)
    (ha : a ∈ L) (hb : b ∈ L) : [a, b] <+ L := by
  have hdisj := (List.nodup_append.mp hnd).2.2
  rcases pair_sublist_append_iff.mp h with h | h | ⟨_, hbM⟩
  · exact h
  · exact absurd rfl (hdisj a ha a (pair_sublist_mem h).1)
  · exact absurd rfl (hdisj b hb b hbM)

omit [DecidableEq Pos] [ValueSort Value] in
theorem pair_sublist_right {L M : List Pos} (hnd : (L ++ M).Nodup) (h : [a, b] <+ L ++ M)
    (ha : a ∈ M) (hb : b ∈ M) : [a, b] <+ M := by
  have hdisj := (List.nodup_append.mp hnd).2.2
  rcases pair_sublist_append_iff.mp h with h | h | ⟨haL, _⟩
  · exact absurd rfl (hdisj a (pair_sublist_mem h).1 a ha)
  · exact h
  · exact absurd rfl (hdisj a haL a ha)

end pairs

/-! ## The potential -/

section potential
variable (circ : Finset Pos)

/-- How many anchors of the order are unmarked. -/
def unmarked (l : List Pos) : Nat := l.countP fun x => decide (x ∉ circ)

omit [ValueSort Value] in
theorem unmarked_le_length (l : List Pos) : unmarked circ l ≤ l.length :=
  List.countP_le_length

omit [ValueSort Value] in
theorem unmarked_perm {l l' : List Pos} (h : l.Perm l') : unmarked circ l = unmarked circ l' :=
  h.countP_eq _

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
theorem unmarked_mono {circ' : Finset Pos} (h : circ ⊆ circ') (l : List Pos) :
    unmarked circ' l ≤ unmarked circ l :=
  List.countP_mono_left fun x _ hx => by
    simp only [decide_eq_true_eq] at hx ⊢
    exact fun hc => hx (h hc)

omit [ValueSort Value] in
/-- Marking a new anchor of the order, and possibly more, lowers the count. -/
theorem unmarked_lt_of_mark {circ' : Finset Pos} (h : circ ⊆ circ') {a : Pos} {l : List Pos}
    (ha : a ∈ l) (hac : a ∉ circ) (hac' : a ∈ circ') (hnd : l.Nodup) :
    unmarked circ' l < unmarked circ l := by
  have h1 := unmarked_insert circ ha hac hnd
  have h2 : unmarked circ' l ≤ unmarked (insert a circ) l :=
    unmarked_mono _ (Finset.insert_subset hac' h) l
  omega

end potential

/-- The spill cells of the sheet the passes start from: what dropping stale
cells consumes. -/
def staleCount (U : Universe Pos) (S : Sheet Pos Value) : Nat :=
  (U.positions.toFinset.filter fun q => (S q).spillAnchor?.isSome).card

omit [ValueSort Value] in
theorem staleCount_le_of_pointwise (U : Universe Pos) {S S' : Sheet Pos Value}
    (h : ∀ q, (S' q).spillAnchor?.isSome → (S q).spillAnchor?.isSome) :
    staleCount U S' ≤ staleCount U S := by
  apply Finset.card_le_card
  intro q hq
  rw [Finset.mem_filter] at hq ⊢
  exact ⟨hq.1, h q hq.2⟩

omit [ValueSort Value] in
theorem staleCount_dropStale_le (U : Universe Pos) (S : Sheet Pos Value) (a : Pos)
    (cells : List Pos) : staleCount U (dropStale S a cells) ≤ staleCount U S := by
  apply staleCount_le_of_pointwise
  intro q hq
  rcases dropStale_eq_or S a cells q with h | ⟨h, _⟩
  · rw [h] at hq
    exact hq
  · rw [h] at hq
    simp [Content.spillAnchor?] at hq

omit [ValueSort Value] in
theorem staleCount_dropMarked_le (U : Universe Pos) (S : Sheet Pos Value) (marked : List Pos) :
    staleCount U (dropMarked S marked) ≤ staleCount U S := by
  apply staleCount_le_of_pointwise
  intro q hq
  rcases dropMarked_eq_or S marked q with h | ⟨h, _⟩
  · rw [h] at hq
    exact hq
  · rw [h] at hq
    simp [Content.spillAnchor?] at hq

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

omit [ValueSort Value] in
theorem staleCount_nextSheet_lt (U : Universe Pos) (S : Sheet Pos Value) (r : Restart Pos)
    (marked : List Pos) (hs : r.isStaleCells = true) (hstale : r.StaleOk S) :
    staleCount U (r.nextSheet marked S) < staleCount U S := by
  rcases r with ⟨a, rd⟩ | ⟨a, rs⟩ | a | ⟨a, cells⟩
  · simp [Restart.isStaleCells] at hs
  · simp [Restart.isStaleCells] at hs
  · simp [Restart.isStaleCells] at hs
  · unfold Restart.nextSheet
    exact (staleCount_dropMarked_le U _ marked).trans_lt
      (staleCount_dropStale_lt U S a cells hstale)

omit [ValueSort Value] in
theorem staleCount_nextSheet_le (U : Universe Pos) (S : Sheet Pos Value) (r : Restart Pos)
    (marked : List Pos) : staleCount U (r.nextSheet marked S) ≤ staleCount U S := by
  unfold Restart.nextSheet
  refine (staleCount_dropMarked_le U _ marked).trans ?_
  cases r with
  | staleCells a cells => exact staleCount_dropStale_le U S a cells
  | _ => exact le_refl _

/-- Facts still to learn and unmarked anchors, weighted so that a mark
outweighs any change of the facts. -/
def potential (n : Nat) (order₀ : List Pos) (log : RestartLog Pos) : Nat :=
  unmarked log.circular order₀ * (n * n + 2) + (n * n + 1 - log.facts.length)

omit [ValueSort Value] in
theorem potential_lt {n : Nat} {order₀ : List Pos} (log : RestartLog Pos)
    (hlen : order₀.length = n) : potential n order₀ log < (n + 1) * (n * n + 2) := by
  unfold potential
  have h1 := unmarked_le_length log.circular order₀
  rw [hlen] at h1
  have h2 : unmarked log.circular order₀ * (n * n + 2) ≤ n * (n * n + 2) :=
    Nat.mul_le_mul_right _ h1
  have h3 : (n + 1) * (n * n + 2) = n * (n * n + 2) + (n * n + 2) := Nat.succ_mul _ _
  omega

omit [ValueSort Value] in
/-- Marking a new anchor lowers the potential whatever happens to the facts. -/
theorem potential_lt_of_mark {n : Nat} {order₀ : List Pos} {log log' : RestartLog Pos}
    (h : unmarked log'.circular order₀ < unmarked log.circular order₀) :
    potential n order₀ log' < potential n order₀ log := by
  unfold potential
  have h1 : (unmarked log'.circular order₀ + 1) * (n * n + 2) ≤
      unmarked log.circular order₀ * (n * n + 2) := Nat.mul_le_mul_right _ h
  rw [Nat.succ_mul] at h1
  omega

/-- The measure: stale cells first, then the potential. -/
def measure (U : Universe Pos) (n : Nat) (S : Sheet Pos Value) (order₀ : List Pos)
    (log : RestartLog Pos) : Nat :=
  staleCount U S * ((n + 1) * (n * n + 2)) + potential n order₀ log

/-! ## The driver invariant -/

/-- What the driver maintains between passes. -/
structure DriverInv (U : Universe Pos) (order₀ : List Pos) (S : Sheet Pos Value)
    (order : List Pos) (log : RestartLog Pos) : Prop where
  wf : WellFormed S
  horder : OrderOf S order₀
  perm : order.Perm order₀
  /-- The order respects every fact. -/
  facts_ok : ∀ f ∈ log.facts, [f.1, f.2] <+ order
  facts_nodup : log.facts.Nodup
  circ_sub : ∀ x ∈ log.circular, x ∈ order
  /-- A marked anchor has no spill cell left in the starting sheet. -/
  no_stale_marked : ∀ q a v, S q = .spill a v → a ∉ log.circular

omit [ValueSort Value] in
theorem DriverInv.nodup {U : Universe Pos} {order₀ : List Pos} {S : Sheet Pos Value}
    {order : List Pos} {log : RestartLog Pos} (h : DriverInv U order₀ S order log) :
    order.Nodup :=
  h.perm.nodup_iff.mpr h.horder.1

omit [ValueSort Value] in
theorem DriverInv.facts_mem {U : Universe Pos} {order₀ : List Pos} {S : Sheet Pos Value}
    {order : List Pos} {log : RestartLog Pos} (h : DriverInv U order₀ S order log) :
    ∀ f ∈ log.facts, f.1 ∈ order ∧ f.2 ∈ order :=
  fun f hf => pair_sublist_mem (h.facts_ok f hf)

omit [ValueSort Value] in
/-- Duplicate-free pairs of anchors of the order: at most `n²` of them. -/
theorem facts_length_le {order : List Pos} {facts : List (Pos × Pos)} (hnd : facts.Nodup)
    (hmem : ∀ f ∈ facts, f.1 ∈ order ∧ f.2 ∈ order) :
    facts.length ≤ order.length * order.length := by
  have hsub : facts ⊆ order ×ˢ order := by
    intro f hf
    obtain ⟨h1, h2⟩ := hmem f hf
    exact List.pair_mem_product.mpr ⟨h1, h2⟩
  have := (List.subperm_of_subset hnd hsub).length_le
  rwa [List.length_product] at this

/-! ## One reader -/

/-- The log-and-order half of the invariant, for the steps of `record`. -/
structure LogInv (order₀ order : List Pos) (log : RestartLog Pos) : Prop where
  perm : order.Perm order₀
  facts_ok : ∀ f ∈ log.facts, [f.1, f.2] <+ order
  facts_nodup : log.facts.Nodup
  circ_sub : ∀ x ∈ log.circular, x ∈ order

omit [ValueSort Value] in
theorem LogInv.nodup {order₀ order : List Pos} {log : RestartLog Pos} (hnd₀ : order₀.Nodup)
    (h : LogInv order₀ order log) : order.Nodup :=
  h.perm.nodup_iff.mpr hnd₀

omit [ValueSort Value] in
theorem LogInv.facts_mem {order₀ order : List Pos} {log : RestartLog Pos}
    (h : LogInv order₀ order log) : ∀ f ∈ log.facts, f.1 ∈ order ∧ f.2 ∈ order :=
  fun f hf => pair_sublist_mem (h.facts_ok f hf)

omit [ValueSort Value] in
/-- The order after a repair respects every fact, old and new. -/
theorem repair_ok {log : RestartLog Pos} {order : List Pos} {anchor reader : Pos}
    (hnd : order.Nodup) (hfacts : ∀ f ∈ log.facts, [f.1, f.2] <+ order)
    (hr : reader ∈ order) (hne : reader ≠ anchor) (hnot : reader ∉ log.before anchor)
    (ha : anchor ∈ order) :
    ∀ f ∈ log.facts ++ [(anchor, reader)], [f.1, f.2] <+
      order.take (order.idxOf reader) ++
        (order.drop (order.idxOf reader + 1)).filter (fun c => decide (c ∈ log.before anchor)) ++
        reader :: (order.drop (order.idxOf reader + 1)).filter
          (fun c => decide (c ∉ log.before anchor)) := by
  intro f hf
  rw [List.append_assoc]
  set before := log.before anchor with hbefore
  set L := order.take (order.idxOf reader) with hL
  set R := order.drop (order.idxOf reader + 1) with hR
  have hi : order.idxOf reader < order.length := List.idxOf_lt_length_iff.mpr hr
  have hget : order[order.idxOf reader] = reader := List.getElem_idxOf hi
  have hsplit : order = L ++ reader :: R := by
    rw [hL, hR]
    conv_lhs => rw [← List.take_append_drop (order.idxOf reader) order]
    rw [List.drop_eq_getElem_cons hi, hget]
  have hnd' : (L ++ reader :: R).Nodup := hsplit ▸ hnd
  have hdisj := (List.nodup_append.mp hnd').2.2
  have hndR : R.Nodup := ((List.nodup_append.mp hnd').2.1).of_cons
  have hrR : reader ∉ R := (List.nodup_cons.mp (List.nodup_append.mp hnd').2.1).1
  -- Every element of the old tail is in the new tail.
  have htail : ∀ x, x ∈ reader :: R → x ∈ R.filter (fun c => decide (c ∈ before)) ++
      reader :: R.filter (fun c => decide (c ∉ before)) := by
    intro x hx
    rcases List.mem_cons.mp hx with rfl | hx
    · exact List.mem_append_right _ List.mem_cons_self
    · by_cases hb : x ∈ before
      · exact List.mem_append_left _ (List.mem_filter.mpr ⟨hx, by simp [hb]⟩)
      · exact List.mem_append_right _ (List.mem_cons_of_mem _ (List.mem_filter.mpr ⟨hx, by simp [hb]⟩))
  have hclosed : ∀ f ∈ log.facts, f.2 ∈ before → f.1 ∈ before :=
    fun f hf h => log.before_closed hf h
  rcases List.mem_append.mp hf with hf | hf
  · -- An old fact.
    have h := hfacts f hf
    rw [hsplit] at h
    rcases pair_sublist_append_iff.mp h with h | h | ⟨h1, h2⟩
    · exact h.trans (List.sublist_append_left _ _)
    · rcases pair_sublist_cons_iff.mp h with h | ⟨h1, h2⟩
      · -- Both in the tail.
        have hm := pair_sublist_mem h
        refine List.Sublist.trans ?_ (List.sublist_append_right L _)
        by_cases hb1 : f.1 ∈ before
        · by_cases hb2 : f.2 ∈ before
          · exact (pair_sublist_filter h (by simp [hb1]) (by simp [hb2])).trans
              (List.sublist_append_left _ _)
          · exact List.Sublist.append (List.singleton_sublist.mpr
              (List.mem_filter.mpr ⟨hm.1, by simp [hb1]⟩))
              (List.singleton_sublist.mpr (List.mem_cons_of_mem _
                (List.mem_filter.mpr ⟨hm.2, by simp [hb2]⟩)))
        · have hb2 : f.2 ∉ before := fun h2 => hb1 (hclosed f hf h2)
          exact (List.Sublist.cons _ (pair_sublist_filter h (by simp [hb1]) (by simp [hb2]))).trans
            (List.sublist_append_right _ _)
      · -- From the reader into the tail: the target is not placed before
        -- the anchor, else the reader would be.
        have hb2 : f.2 ∉ before := fun h => hnot (h1 ▸ hclosed f hf h)
        refine List.Sublist.trans ?_ (List.sublist_append_right L _)
        refine List.Sublist.trans ?_ (List.sublist_append_right _ _)
        rw [h1]
        exact pair_sublist_cons_iff.mpr (Or.inr ⟨rfl, List.mem_filter.mpr ⟨h2, by simp [hb2]⟩⟩)
    · -- From the head into the tail.
      exact List.Sublist.append (List.singleton_sublist.mpr h1)
        (List.singleton_sublist.mpr (htail _ h2))
  · -- The new fact.
    rw [List.mem_singleton] at hf
    subst hf
    show [anchor, reader] <+ _
    rw [hsplit] at ha
    rcases List.mem_append.mp ha with haL | haR
    · exact List.Sublist.append (List.singleton_sublist.mpr haL)
        (List.singleton_sublist.mpr (List.mem_append_right _ List.mem_cons_self))
    · rcases List.mem_cons.mp haR with h | haR
      · exact absurd h.symm hne
      · refine List.Sublist.trans ?_ (List.sublist_append_right L _)
        exact List.Sublist.append (List.singleton_sublist.mpr (List.mem_filter.mpr
          ⟨haR, by simp [hbefore, log.mem_before_self]⟩))
          (List.singleton_sublist.mpr List.mem_cons_self)

omit [ValueSort Value] in
/-- The repaired order is a permutation of the old one. -/
theorem repair_perm {log : RestartLog Pos} {order : List Pos} {anchor reader : Pos}
    (hr : reader ∈ order) :
    (order.take (order.idxOf reader) ++
      (order.drop (order.idxOf reader + 1)).filter (fun c => decide (c ∈ log.before anchor)) ++
      reader :: (order.drop (order.idxOf reader + 1)).filter
        (fun c => decide (c ∉ log.before anchor))).Perm order := by
  set R := order.drop (order.idxOf reader + 1) with hR
  have hi : order.idxOf reader < order.length := List.idxOf_lt_length_iff.mpr hr
  have hget : order[order.idxOf reader] = reader := List.getElem_idxOf hi
  have hsplit : order = order.take (order.idxOf reader) ++ reader :: R := by
    rw [hR]
    conv_lhs => rw [← List.take_append_drop (order.idxOf reader) order]
    rw [List.drop_eq_getElem_cons hi, hget]
  conv_rhs => rw [hsplit]
  rw [List.append_assoc]
  refine List.Perm.append_left _ ?_
  refine List.perm_middle.trans (List.Perm.cons _ ?_)
  have := List.filter_append_perm (fun c => decide (c ∈ log.before anchor)) R
  refine List.Perm.trans ?_ this
  refine List.Perm.append_left _ ?_
  refine List.Perm.of_eq ?_
  congr 1
  funext c
  simp

omit [ValueSort Value] in
/-- One reader learned: either the fact is added and the order repaired, or
the reader is marked, with everything the facts place between it and the
anchor. -/
theorem learn_spec {order₀ order : List Pos} {log : RestartLog Pos} (hnd₀ : order₀.Nodup)
    (hinv : LogInv order₀ order log) {anchor reader : Pos}
    (ha : anchor ∈ order) (hr : reader ∈ order) (hne : reader ≠ anchor)
    (hnew : (anchor, reader) ∉ log.facts) (hrc : reader ∉ log.circular)
    {log' : RestartLog Pos} {order' marked : List Pos}
    (h : log.learn anchor reader order = (log', order', marked)) :
    LogInv order₀ order' log' ∧ log.circular ⊆ log'.circular ∧
      (∀ x ∈ log'.circular, x ∈ log.circular ∨ x ∈ marked) ∧ (∀ x ∈ marked, x ∈ order) ∧
      ((log'.circular = log.circular ∧ log'.facts = log.facts ++ [(anchor, reader)]) ∨
        (reader ∈ log'.circular ∧ log'.facts <+ log.facts)) := by
  have hnd : order.Nodup := hinv.nodup hnd₀
  unfold RestartLog.learn at h
  simp only [hne, ↓reduceIte] at h
  split at h
  · -- A loop: mark.
    rename_i hin
    simp only [Prod.mk.injEq] at h
    obtain ⟨rfl, rfl, rfl⟩ := h
    set onLoop := (log.before anchor).filter fun c =>
      decide (c ∈ log.after reader ∧ c ∉ log.circular) with honLoop
    have hloop_sub : ∀ x ∈ onLoop, x ∈ order := by
      intro x hx
      have hx := (List.mem_filter.mp hx).1
      rcases log.before_sub hx with rfl | ⟨f, hf, rfl⟩
      · exact ha
      · exact (hinv.facts_mem f hf).1
    have hreader : reader ∈ onLoop :=
      List.mem_filter.mpr ⟨hin, by simp [log.mem_after_self, hrc]⟩
    have hfacts := foldl_mark_facts onLoop log
    refine ⟨⟨hinv.perm, fun f hf => hinv.facts_ok f (hfacts.subset hf),
      hinv.facts_nodup.sublist hfacts, ?_⟩, foldl_mark_circular_mono onLoop log,
      foldl_mark_circular_sub onLoop log, hloop_sub,
      Or.inr ⟨foldl_mark_mem onLoop log reader hreader, hfacts⟩⟩
    intro x hx
    rcases foldl_mark_circular_sub onLoop log x hx with hx | hx
    · exact hinv.circ_sub x hx
    · exact hloop_sub x hx
  · -- A new fact: repair.
    rename_i hnot
    simp only [Prod.mk.injEq] at h
    obtain ⟨rfl, rfl, rfl⟩ := h
    have hperm := repair_perm (log := log) (anchor := anchor) hr
    refine ⟨⟨hperm.trans hinv.perm, repair_ok hnd hinv.facts_ok hr hne hnot ha, ?_,
      fun x hx => hperm.mem_iff.mpr (hinv.circ_sub x hx)⟩, Finset.Subset.refl _,
      fun x hx => Or.inl hx, by simp, Or.inl ⟨rfl, rfl⟩⟩
    rw [List.nodup_append]
    refine ⟨hinv.facts_nodup, List.nodup_singleton _, ?_⟩
    intro f hf g hg hfg
    rw [List.mem_singleton] at hg
    subst hg
    exact hnew (hfg ▸ hf)

/-! ## All the readers of a restart -/

omit [ValueSort Value] in
/-- The fold over the readers, from the first one on: the log invariant
holds throughout, the potential never rises, and it drops at the first
reader, which is learned. -/
theorem fold_spec (n : Nat) {order₀ : List Pos} (hnd₀ : order₀.Nodup) (hlen : order₀.length = n)
    (anchor : Pos) (facts₀ : List (Pos × Pos)) :
    ∀ (rs done : List Pos) (log : RestartLog Pos) (order marked : List Pos),
      LogInv order₀ order log → (done ++ rs).Nodup →
      (∀ x ∈ rs, x ∈ order ∧ x ≠ anchor ∧ (anchor, x) ∉ facts₀) → anchor ∈ order →
      (∀ f ∈ log.facts, f ∈ facts₀ ∨ (f.1 = anchor ∧ f.2 ∈ done)) →
      (∀ x ∈ marked, x ∈ order) →
      let out := rs.foldl (RestartLog.learnStep anchor) (log, order, marked)
      LogInv order₀ out.2.1 out.1 ∧ log.circular ⊆ out.1.circular ∧
        (∀ x ∈ out.1.circular, x ∈ log.circular ∨ x ∈ out.2.2) ∧
        (∀ x ∈ out.2.2, x ∈ order) ∧ (∀ x ∈ marked, x ∈ out.2.2) ∧
        potential n order₀ out.1 ≤ potential n order₀ log ∧
        (∀ x ∈ rs, anchor ∉ log.circular → x ∉ log.circular → rs.head? = some x →
          potential n order₀ out.1 < potential n order₀ log)
  | [], done, log, order, marked, hinv, _, _, _, _, hmarked => by
      refine ⟨hinv, Finset.Subset.refl _, fun x hx => Or.inl hx, hmarked, fun x hx => hx,
        le_refl _, ?_⟩
      intro x hx
      exact absurd hx List.not_mem_nil
  | x :: rs, done, log, order, marked, hinv, hnd, hrs, ha, hfacts, hmarked => by
      simp only [List.foldl_cons]
      unfold RestartLog.learnStep
      split
      · -- Skipped: the anchor or the reader is marked by now.
        rename_i hskip
        have hnd' : (done ++ rs).Nodup := by
          have := hnd.sublist (List.Sublist.append (List.Sublist.refl done) (List.sublist_cons_self x rs))
          exact this
        obtain ⟨h1, h2, h3, h4, h4', h5, h6⟩ := fold_spec n hnd₀ hlen anchor facts₀ rs done log
          order marked hinv hnd' (fun y hy => hrs y (List.mem_cons_of_mem _ hy)) ha hfacts hmarked
        refine ⟨h1, h2, h3, h4, h4', h5, ?_⟩
        intro y hy hac hyc hhead
        simp only [List.head?_cons, Option.some.injEq] at hhead
        subst hhead
        exact absurd hskip (by simp [hac, hyc])
      · rename_i hskip
        simp only [not_or] at hskip
        obtain ⟨hac, hxc⟩ := hskip
        obtain ⟨hx_order, hx_ne, hx_new⟩ := hrs x List.mem_cons_self
        have hnew : (anchor, x) ∉ log.facts := by
          intro hf
          rcases hfacts _ hf with hf | ⟨_, hf⟩
          · exact hx_new hf
          · have := (List.nodup_append.mp hnd).2.2 x hf x List.mem_cons_self
            exact this rfl
        rcases hl : log.learn anchor x order with ⟨log₁, order₁, more⟩
        simp only
        obtain ⟨hinv₁, hcirc₁, hcirc₁', hmore, hcase⟩ :=
          learn_spec hnd₀ hinv ha hx_order hx_ne hnew hxc hl
        have hnd' : (done ++ [x] ++ rs).Nodup := by
          rw [List.append_assoc]
          exact hnd
        have hperm₁ : order₁.Perm order := hinv₁.perm.trans hinv.perm.symm
        have hlen₁ : order₁.length = n := hinv₁.perm.length_eq.trans hlen
        -- The potential drops at this reader.
        have hdrop : potential n order₀ log₁ < potential n order₀ log := by
          rcases hcase with ⟨hc, hf⟩ | ⟨hxm, hf⟩
          · unfold potential
            rw [hc, hf, List.length_append, List.length_singleton]
            have hbound := facts_length_le hinv₁.facts_nodup (hinv₁.facts_mem)
            rw [hlen₁, hf, List.length_append, List.length_singleton] at hbound
            omega
          · apply potential_lt_of_mark
            exact unmarked_lt_of_mark _ hcirc₁ (hinv.perm.mem_iff.mp hx_order) hxc hxm hnd₀
        obtain ⟨h1, h2, h3, h4, h4', h5, -⟩ := fold_spec n hnd₀ hlen anchor facts₀ rs (done ++ [x])
          log₁
          order₁ (marked ++ more) hinv₁ hnd'
          (fun y hy =>
            let ⟨hy1, hy2, hy3⟩ := hrs y (List.mem_cons_of_mem _ hy)
            ⟨hperm₁.mem_iff.mpr hy1, hy2, hy3⟩)
          (hperm₁.mem_iff.mpr ha)
          (by
            intro f hf
            rcases hcase with ⟨_, hfs⟩ | ⟨_, hfs⟩
            · rw [hfs] at hf
              rcases List.mem_append.mp hf with hf | hf
              · rcases hfacts f hf with h | ⟨h1, h2⟩
                · exact Or.inl h
                · exact Or.inr ⟨h1, List.mem_append_left _ h2⟩
              · rw [List.mem_singleton] at hf
                subst hf
                exact Or.inr ⟨rfl, List.mem_append_right _ (List.mem_singleton_self _)⟩
            · rcases hfacts f (hfs.subset hf) with h | ⟨h1, h2⟩
              · exact Or.inl h
              · exact Or.inr ⟨h1, List.mem_append_left _ h2⟩)
          (by
            intro y hy
            rcases List.mem_append.mp hy with hy | hy
            · exact hperm₁.mem_iff.mpr (hmarked y hy)
            · exact hperm₁.mem_iff.mpr (hmore y hy))
        refine ⟨h1, Finset.Subset.trans hcirc₁ h2, ?_, ?_, ?_, h5.trans hdrop.le,
          fun _ _ _ _ _ => h5.trans_lt hdrop⟩
        · intro y hy
          rcases h3 y hy with hy | hy
          · rcases hcirc₁' y hy with hy | hy
            · exact Or.inl hy
            · exact Or.inr (h4' y (List.mem_append_right _ hy))
          · exact Or.inr hy
        · intro y hy
          exact hperm₁.mem_iff.mp (h4 y hy)
        · intro y hy
          exact h4' y (List.mem_append_left _ hy)

/-! ## One restart -/

omit [ValueSort Value] in
theorem premark_spec {order₀ order : List Pos} {log : RestartLog Pos} (hnd₀ : order₀.Nodup)
    (hinv : LogInv order₀ order log) (r : Restart Pos) (ha : r.anchor ∈ order)
    {log' : RestartLog Pos} {marked : List Pos} (h : log.premark r = (log', marked)) :
    LogInv order₀ order log' ∧ log.circular ⊆ log'.circular ∧
      (∀ x ∈ log'.circular, x ∈ log.circular ∨ x ∈ marked) ∧ (∀ x ∈ marked, x ∈ order) ∧
      log'.facts <+ log.facts ∧ ∀ n, potential n order₀ log' ≤ potential n order₀ log ∧
      (∀ a, r = .selfContradiction a → a ∉ log.circular →
        potential n order₀ log' < potential n order₀ log) := by
  have hid : log' = log → marked = [] → LogInv order₀ order log' ∧ log.circular ⊆ log'.circular ∧
      (∀ x ∈ log'.circular, x ∈ log.circular ∨ x ∈ marked) ∧ (∀ x ∈ marked, x ∈ order) ∧
      log'.facts <+ log.facts ∧ ∀ n, potential n order₀ log' ≤ potential n order₀ log := by
    rintro rfl rfl
    exact ⟨hinv, Finset.Subset.refl _, fun x hx => Or.inl hx, by simp, List.Sublist.refl _,
      fun _ => le_refl _⟩
  rcases r with ⟨a, rd⟩ | ⟨a, rs⟩ | a | ⟨a, cells⟩
  · simp only [RestartLog.premark, Prod.mk.injEq] at h
    obtain ⟨h1, h2, h3, h4, h5, h6⟩ := hid h.1.symm h.2.symm
    exact ⟨h1, h2, h3, h4, h5, fun n => ⟨h6 n, fun _ h' => by cases h'⟩⟩
  · simp only [RestartLog.premark, Prod.mk.injEq] at h
    obtain ⟨h1, h2, h3, h4, h5, h6⟩ := hid h.1.symm h.2.symm
    exact ⟨h1, h2, h3, h4, h5, fun n => ⟨h6 n, fun _ h' => by cases h'⟩⟩
  · simp only [RestartLog.premark] at h
    split at h
    · rename_i hm
      simp only [Prod.mk.injEq] at h
      obtain ⟨h1, h2, h3, h4, h5, h6⟩ := hid h.1.symm h.2.symm
      refine ⟨h1, h2, h3, h4, h5, fun n => ⟨h6 n, ?_⟩⟩
      intro a' ha' hnm
      cases ha'
      exact absurd hm hnm
    · rename_i hm
      simp only [Prod.mk.injEq] at h
      obtain ⟨rfl, rfl⟩ := h
      have hfacts : (log.mark a).facts <+ log.facts := List.filter_sublist
      have hlt : ∀ n, potential n order₀ (log.mark a) < potential n order₀ log := by
        intro n
        apply potential_lt_of_mark
        exact unmarked_lt_of_mark _ (log.mark_circular_mono a) (hinv.perm.mem_iff.mp ha) hm
          (Finset.mem_insert_self _ _) hnd₀
      refine ⟨⟨hinv.perm, fun f hf => hinv.facts_ok f (hfacts.subset hf),
        hinv.facts_nodup.sublist hfacts, ?_⟩, log.mark_circular_mono a, ?_, ?_, hfacts,
        fun n => ⟨(hlt n).le, fun _ _ _ => hlt n⟩⟩
      · intro x hx
        rcases Finset.mem_insert.mp hx with rfl | hx
        · exact ha
        · exact hinv.circ_sub x hx
      · intro x hx
        rcases Finset.mem_insert.mp hx with rfl | hx
        · exact Or.inr (List.mem_singleton_self _)
        · exact Or.inl hx
      · intro x hx
        rw [List.mem_singleton] at hx
        subst hx
        exact ha
  · simp only [RestartLog.premark, Prod.mk.injEq] at h
    obtain ⟨h1, h2, h3, h4, h5, h6⟩ := hid h.1.symm h.2.symm
    exact ⟨h1, h2, h3, h4, h5, fun n => ⟨h6 n, fun _ h' => by cases h'⟩⟩

omit [ValueSort Value] in
/-- Recording a restart keeps the log invariant, marks only anchors of the
order, and lowers the potential unless the restart drops stale cells. -/
theorem record_spec (n : Nat) {order₀ order : List Pos} {log : RestartLog Pos}
    (hnd₀ : order₀.Nodup) (hlen : order₀.length = n) (hinv : LogInv order₀ order log)
    {r : Restart Pos} (hf : RestartFacts S order log.circular r) (hac : r.anchor ∉ log.circular)
    {log' : RestartLog Pos} {order' marked : List Pos}
    (h : log.record r order = (log', order', marked)) :
    LogInv order₀ order' log' ∧ log.circular ⊆ log'.circular ∧
      (∀ x ∈ log'.circular, x ∈ log.circular ∨ x ∈ marked) ∧ (∀ x ∈ marked, x ∈ order) ∧
      potential n order₀ log' ≤ potential n order₀ log ∧
      (r.isStaleCells = false → potential n order₀ log' < potential n order₀ log) := by
  unfold RestartLog.record at h
  have hinv₀ : LogInv order₀ order { log with restarts := log.restarts + 1 } :=
    ⟨hinv.perm, hinv.facts_ok, hinv.facts_nodup, hinv.circ_sub⟩
  rcases hpre : RestartLog.premark { log with restarts := log.restarts + 1 } r with ⟨log₁, marked₁⟩
  rw [hpre] at h
  simp only at h
  obtain ⟨hinv₁, hcirc₁, hcirc₁', hmarked₁, hfacts₁, hpot₁⟩ :=
    premark_spec hnd₀ hinv₀ r hf.anchor_mem hpre
  have hcirc₁ : log.circular ⊆ log₁.circular := hcirc₁
  have hcirc₁' : ∀ x ∈ log₁.circular, x ∈ log.circular ∨ x ∈ marked₁ := hcirc₁'
  have hfacts₁ : log₁.facts <+ log.facts := hfacts₁
  obtain ⟨hpot₁_le, hpot₁_lt⟩ := hpot₁ n
  have hpot₁_le : potential n order₀ log₁ ≤ potential n order₀ log := hpot₁_le
  have hpot₁_lt : ∀ a, r = .selfContradiction a → a ∉ log.circular →
      potential n order₀ log₁ < potential n order₀ log := hpot₁_lt
  have hnd := hinv.nodup hnd₀
  -- Every reader is fit to be learned from the state after the premark.
  have hrs : ∀ x ∈ r.readers, x ∈ order ∧ x ≠ r.anchor ∧ (r.anchor, x) ∉ log₁.facts := by
    intro x hx
    obtain ⟨hx1, hx2, hx3⟩ := hf.readers x hx
    refine ⟨hx1, fun e => by rw [e] at hx3; exact lt_irrefl _ hx3, ?_⟩
    intro hfx
    have := hinv.facts_ok _ (hfacts₁.subset hfx)
    exact pair_sublist_asymm hnd this (pair_sublist_of_idxOf_lt hnd hx1 hf.anchor_mem hx3)
  have hnd_rs : ([] ++ r.readers).Nodup := by simpa using hf.nodup
  obtain ⟨h1, h2, h3, h4, h4', h5, h6⟩ := fold_spec n hnd₀ hlen r.anchor log₁.facts r.readers []
    log₁ order marked₁ hinv₁ hnd_rs hrs hf.anchor_mem (fun f hf => Or.inl hf) hmarked₁
  rw [h] at h1 h2 h3 h4 h4' h5 h6
  simp only at h1 h2 h3 h4 h4' h5 h6
  refine ⟨h1, Finset.Subset.trans hcirc₁ h2, ?_, h4, h5.trans hpot₁_le, ?_⟩
  · intro x hx
    rcases h3 x hx with hx | hx
    · rcases hcirc₁' x hx with hx | hx
      · exact Or.inl hx
      · exact Or.inr (h4' x hx)
    · exact Or.inr hx
  · intro hns
    cases r with
    | staleCells a cells => simp [Restart.isStaleCells] at hns
    | selfContradiction a =>
        exact h5.trans_lt (hpot₁_lt a rfl hac)
    | staleRead a reader =>
        have hmarked_nil : marked₁ = [] := by
          simp only [RestartLog.premark, Prod.mk.injEq] at hpre
          exact hpre.2.symm
        subst hmarked_nil
        have hac₁ : a ∉ log₁.circular := by
          intro hm
          rcases hcirc₁' a hm with hm | hm
          · exact hac hm
          · exact absurd hm List.not_mem_nil
        have hrc₁ : reader ∉ log₁.circular := by
          intro hm
          rcases hcirc₁' reader hm with hm | hm
          · exact (hf.readers reader (by simp [Restart.readers])).2.1 hm
          · exact absurd hm List.not_mem_nil
        exact (h6 reader (by simp [Restart.readers]) hac₁ hrc₁ (by simp [Restart.readers])).trans_le
          hpot₁_le
    | conflict a readers =>
        have hne : readers ≠ [] := hf.nonempty rfl
        obtain ⟨x, rs, hx⟩ := List.exists_cons_of_ne_nil hne
        have hmarked_nil : marked₁ = [] := by
          simp only [RestartLog.premark, Prod.mk.injEq] at hpre
          exact hpre.2.symm
        subst hmarked_nil
        have hac₁ : a ∉ log₁.circular := by
          intro hm
          rcases hcirc₁' a hm with hm | hm
          · exact hac hm
          · exact absurd hm List.not_mem_nil
        have hxr : x ∈ (Restart.conflict a readers).readers := by
          show x ∈ readers
          rw [hx]
          exact List.mem_cons_self
        have hrc₁ : x ∉ log₁.circular := by
          intro hm
          rcases hcirc₁' x hm with hm | hm
          · exact (hf.readers x hxr).2.1 hm
          · exact absurd hm List.not_mem_nil
        have hhead : (Restart.conflict a readers).readers.head? = some x := by
          show readers.head? = some x
          rw [hx]
          rfl
        exact (h6 x hxr hac₁ hrc₁ hhead).trans_le hpot₁_le

/-! ## The driver -/

omit [ValueSort Value] in
theorem OrderOf.of_perm {S : Sheet Pos Value} {order₀ order : List Pos} (h : OrderOf S order₀)
    (hp : order.Perm order₀) : OrderOf S order :=
  ⟨hp.nodup_iff.mpr h.1, fun a => (hp.mem_iff).trans (h.2 a)⟩

omit [ValueSort Value] in
theorem OrderOf.nextSheet {S : Sheet Pos Value} {order : List Pos} (h : OrderOf S order)
    (r : Restart Pos) (marked : List Pos) : OrderOf (r.nextSheet marked S) order := by
  refine ⟨h.1, fun b => ?_⟩
  unfold Restart.nextSheet
  rw [dropMarked_isDynAnchor]
  cases r with
  | staleCells a cells =>
      change b ∈ order ↔ (dropStale S a cells b).isDynAnchor = true
      rw [dropStale_isDynAnchor]
      exact h.2 b
  | _ => exact h.2 b

omit [ValueSort Value] in
/-- A spill cell of the next sheet is one of this sheet, of an anchor that
was not just marked. -/
theorem nextSheet_spill {S : Sheet Pos Value} {r : Restart Pos} {marked : List Pos} {q a : Pos}
    {v : Value} (h : r.nextSheet marked S q = .spill a v) :
    S q = .spill a v ∧ ¬ (a ∈ marked ∧ (S a).isDynAnchor = true) := by
  rcases r with ⟨b, rd⟩ | ⟨b, rs⟩ | b | ⟨b, cells⟩ <;> simp only [Restart.nextSheet] at h
  · exact dropMarked_spill h
  · exact dropMarked_spill h
  · exact dropMarked_spill h
  · obtain ⟨h1, h2⟩ := dropMarked_spill h
    refine ⟨dropStale_spill h1, ?_⟩
    rw [dropStale_isDynAnchor] at h2
    exact h2

/-- One restart keeps the driver invariant and lowers the measure. -/
theorem restart_step (U : Universe Pos) (order₀ : List Pos) (S S' : Sheet Pos Value)
    (order : List Pos) (log : RestartLog Pos) (hinv : DriverInv U order₀ S order log)
    (r : Restart Pos) (h : runPass U S order log.circular = (S', some r))
    {log' : RestartLog Pos} {order' marked : List Pos}
    (hrec : log.record r order = (log', order', marked)) :
    DriverInv U order₀ (r.nextSheet marked S) order' log' ∧
      measure U order₀.length (r.nextSheet marked S) order₀ log' <
        measure U order₀.length S order₀ log := by
  set n := order₀.length with hn
  have hnd₀ : order₀.Nodup := hinv.horder.1
  have horder : OrderOf S order := OrderOf.of_perm hinv.horder hinv.perm
  have hfacts : RestartFacts S order log.circular r :=
    runPass_facts U S S' order log.circular r hinv.wf horder hinv.circ_sub h
  -- The anchor is unmarked: a marked anchor has no stale cell to restart over.
  have hac : r.anchor ∉ log.circular := by
    intro hm
    obtain ⟨q, v, hq⟩ := hfacts.marked_spill hm
    exact hinv.no_stale_marked q _ v hq hm
  have hloginv : LogInv order₀ order log :=
    ⟨hinv.perm, hinv.facts_ok, hinv.facts_nodup, hinv.circ_sub⟩
  obtain ⟨hinv', hcirc, hcirc', hmarked, hpot_le, hpot_lt⟩ :=
    record_spec (S := S) n hnd₀ rfl hloginv hfacts hac hrec
  have hdyn : (S r.anchor).isDynAnchor = true := (horder.2 r.anchor).mp hfacts.anchor_mem
  refine ⟨⟨WellFormed.nextSheet hinv.wf hdyn marked, OrderOf.nextSheet hinv.horder r marked,
    hinv'.perm,
    hinv'.facts_ok, hinv'.facts_nodup, hinv'.circ_sub, ?_⟩, ?_⟩
  · -- No spill cell of a marked anchor is left.
    intro q a v hq hm
    obtain ⟨hq, hnot⟩ := nextSheet_spill hq
    rcases hcirc' a hm with hm | hm
    · exact hinv.no_stale_marked q a v hq hm
    · exact hnot ⟨hm, (horder.2 a).mp (hmarked a hm)⟩
  · -- The measure.
    unfold measure
    have hstale_le := staleCount_nextSheet_le U S r marked
    have hK : potential n order₀ log' < (n + 1) * (n * n + 2) := potential_lt log' rfl
    cases hs : r.isStaleCells
    · -- A restart that counts: the potential drops.
      have hlt := hpot_lt hs
      have := Nat.mul_le_mul_right ((n + 1) * (n * n + 2)) hstale_le
      omega
    · -- Stale cells dropped.
      have hstale_lt := staleCount_nextSheet_lt U S r marked hs hfacts.stale
      have := Nat.mul_le_mul_right ((n + 1) * (n * n + 2)) hstale_lt
      rw [Nat.succ_mul] at this
      omega

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
    · rcases hrec : log.record r order with ⟨log', order', marked⟩
      simp only [evaluateLoop, hrun, hrec] at h ⊢
      exact ih _ _ _ _ (by omega) h

/-- The loop returns once the measure's worth of fuel is provided. -/
theorem evaluateLoop_of_measure (U : Universe Pos) (order₀ : List Pos) :
    ∀ (m : Nat) (S : Sheet Pos Value) (order : List Pos) (log : RestartLog Pos),
      DriverInv U order₀ S order log →
      measure U order₀.length S order₀ log = m → (evaluateLoop U (m + 1) S order log).isSome := by
  intro m
  induction m using Nat.strong_induction_on with
  | _ m ih =>
    intro S order log hinv hm
    rcases hrun : runPass U S order log.circular with ⟨S', _ | r⟩
    · simp [evaluateLoop, hrun]
    · rcases hrec : log.record r order with ⟨log', order', marked⟩
      simp only [evaluateLoop, hrun, hrec]
      obtain ⟨hinv', hlt⟩ := restart_step U order₀ S S' order log hinv r hrun hrec
      rw [hm] at hlt
      have := ih _ hlt _ _ _ hinv' rfl
      rw [evaluateLoop_mono U _ m _ _ _ (by omega) this]
      exact this

/-- The driver terminates: some amount of fuel is enough. -/
theorem evaluate_terminates (U : Universe Pos) (S : Sheet Pos Value) (anchorOrder : List Pos)
    (hwf : WellFormed S) (hnd : anchorOrder.Nodup) :
    ∃ fuel, (evaluate U S anchorOrder fuel).isSome := by
  unfold evaluate
  set order₀ := syncAnchorOrder U S anchorOrder with horder₀
  have horder : OrderOf S order₀ := syncAnchorOrder_orderOf U S anchorOrder hnd
  refine ⟨measure U order₀.length S order₀ RestartLog.new + 1, ?_⟩
  exact evaluateLoop_of_measure U order₀ _ S order₀ RestartLog.new
    ⟨hwf, horder, List.Perm.refl _, by simp [RestartLog.new], by simp [RestartLog.new],
      by simp [RestartLog.new], by simp [RestartLog.new]⟩ rfl

end IronCalcEval

import IronCalcEval.Driver
import IronCalcEval.Read

/-!
# Partial correctness

`cold-evaluation.md`, sections 5.1 and 5.2: a completed pass leaves the sheet
consistent, hence so does `evaluate` whenever it returns.

The invariant `PassInv` is the formal counterpart of the three invariants of
section 5.1 (exact records, no stale value is ever read, a completed pass
contradicts none of its records), stated about the state after every step of
a pass that has not been abandoned. Everything in this file is proved.
-/

namespace IronCalcEval

variable {Pos Value : Type} [DecidableEq Pos] [ValueSort Value]

/-- The anchor order lists exactly the dynamic anchors, once each. This is
what `sync_anchor_order` establishes. -/
def OrderOf (S : Sheet Pos Value) (order : List Pos) : Prop :=
  order.Nodup ∧ ∀ a, a ∈ order ↔ (S a).isDynAnchor = true

theorem syncAnchorOrder_orderOf (U : Universe Pos) (S : Sheet Pos Value) (order : List Pos)
    (hnd : order.Nodup) : OrderOf S (syncAnchorOrder U S order) := by
  unfold syncAnchorOrder OrderOf
  constructor
  · rw [List.nodup_append]
    refine ⟨hnd.filter _, U.nodup.filter _ |>.filter _, ?_⟩
    intro a ha b hb hab
    subst hab
    have := (List.mem_filter.mp hb).2
    simp [ha] at this
  · intro a
    simp only [List.mem_append, List.mem_filter, U.complete a, true_and, decide_eq_true_eq]
    constructor
    · rintro (⟨_, h⟩ | ⟨h, _⟩) <;> exact h
    · intro h
      by_cases hk : a ∈ order.filter fun a => (S a).isDynAnchor
      · exact Or.inl (by simpa using List.mem_filter.mp hk)
      · exact Or.inr ⟨h, fun h' => hk (List.mem_filter.mpr ⟨h'.1, by simp [h'.2]⟩)⟩

/-! ## The invariant of a pass (5.1): see `Invariant.lean` -/

theorem PassInv.initial (S : Sheet Pos Value) (circular : Finset Pos) (hwf : WellFormed S) :
    PassInv S (PassState.initial S circular) where
  not_abandoned := rfl
  shape := by
    intro p
    cases h : S p <;> simp [PassState.initial, h, Content.isEmpty, Content.spillAnchor?]
  no_orphans := hwf.1
  cse_areas := hwf.2.1
  cse_spills := hwf.2.2
  stack_nodup := List.nodup_nil
  stack_evaluating := by simp [PassState.initial]
  seen_empty := by simp [PassState.initial]
  seen_occupied := by simp [PassState.initial]
  orig_spill := fun _ _ _ hq _ => hq
  evaluated_consistent := by simp [PassState.initial]
  reads_protected := by simp [PassState.initial]
  root_cell := by simp [PassState.initial]

omit [DecidableEq Pos] [ValueSort Value] in
theorem SameShape.isDynAnchor_eq {S₀ S : Sheet Pos Value} (h : SameShape S₀ S) (a : Pos) :
    (S a).isDynAnchor = (S₀ a).isDynAnchor := by
  have := h a
  revert this
  cases S₀ a with
  | const v => intro h; rw [h]
  | formula t _ => rintro ⟨v, hv⟩; rw [hv]; simp [Content.isDynAnchor]
  | cseAnchor t area _ => rintro ⟨v, hv⟩; rw [hv]; simp [Content.isDynAnchor]
  | dynAnchor t _ => rintro ⟨v, hv⟩; rw [hv]; simp [Content.isDynAnchor]
  | empty =>
      intro h
      cases hs : S a <;> simp_all [Content.isEmpty, Content.spillAnchor?, Content.isDynAnchor]
  | spill _ _ =>
      intro h
      cases hs : S a <;> simp_all [Content.isEmpty, Content.spillAnchor?, Content.isDynAnchor]

/-! ## The root, set by the pass body before each cell -/

/-- The state `passBody` hands to `evalCell c`. -/
def setRoot (st : PassState Pos Value) (c : Pos) : PassState Pos Value :=
  { st with root := some c }

theorem PassInv.setRoot {S₀ : Sheet Pos Value} {st : PassState Pos Value} (hinv : PassInv S₀ st)
    (hstack : st.stack = []) (c : Pos) : PassInv S₀ (setRoot st c) :=
  { hinv with root_cell := fun _ _ h => absurd hstack h }

theorem RootOk.setRoot (st : PassState Pos Value) (c : Pos) : RootOk (setRoot st c) c :=
  ⟨rfl, fun _ c' hc' => Or.inl (by cases hc'; rfl)⟩

/-- What the pass body does between two cells, root changes included. -/
structure BodyStep (a b : PassState Pos Value) : Prop where
  cells_mono : ∀ q, a.cells q = some .evaluated → b.cells q = some .evaluated
  circular_mono : a.circular ⊆ b.circular

theorem BodyStep.refl (a : PassState Pos Value) : BodyStep a a :=
  ⟨fun _ h => h, Finset.Subset.refl _⟩

theorem BodyStep.trans {a b c : PassState Pos Value} (h₁ : BodyStep a b) (h₂ : BodyStep b c) :
    BodyStep a c :=
  ⟨fun q h => h₂.1 q (h₁.1 q h), Finset.Subset.trans h₁.2 h₂.2⟩

theorem PassStep.toBody {S₀ : Sheet Pos Value} {a b : PassState Pos Value} (h : PassStep S₀ a b) :
    BodyStep a b :=
  ⟨h.cells_mono, h.circular_mono⟩

theorem BodyStep.setRoot (st : PassState Pos Value) (c : Pos) : BodyStep st (setRoot st c) :=
  ⟨fun _ h => h, Finset.Subset.refl _⟩

/-- The cells map only grows during a pass: an evaluated cell stays evaluated. -/
theorem evalCell_cells_mono (U : Universe Pos) (fuel : Nat) (p : Pos)
    (st st' : PassState Pos Value) (v : Value)
    (h : (evalCell U fuel p).run st = (v, st')) :
    ∀ q, st.cells q = some .evaluated → st'.cells q = some .evaluated := by
  have := evalCell_cells U fuel p st
  rw [h] at this
  exact this

/-- One `evaluate_cell`, started from a state satisfying the invariant with
enough fuel, either abandons the pass or preserves the invariant. This is the
state half of `evalCell_spec`; the value half is what the callers use. -/
theorem evalCell_preserves (S₀ : Sheet Pos Value) (U : Universe Pos) (fuel : Nat) (p : Pos)
    (st st' : PassState Pos Value) (v : Value)
    (hinv : PassInv S₀ st) (hroot : RootOk st p) (hfuel : FuelOk U st p fuel)
    (h : (evalCell U fuel p).run st = (v, st')) :
    st'.restart.isSome ∨ PassInv S₀ st' := by
  have := evalCell_spec U fuel p st hinv hroot hfuel
  rw [h] at this
  rcases this.2.1 with h | ⟨h, _⟩
  · exact Or.inl h
  · exact Or.inr h

/-- The fuel `run_pass` provides is enough for every read it makes: the stack
is empty at the top level, and a read needs at most two units per cell. -/
theorem passFuel_ok (S₀ : Sheet Pos Value) (U : Universe Pos) (st : PassState Pos Value)
    (hinv : PassInv S₀ st) (hstack : st.stack = []) (q : Pos) : FuelOk U st q (passFuel U) := by
  unfold FuelOk passFuel
  rw [hstack]
  simp only [List.length_nil, Nat.sub_zero]
  split <;> omega

/-- Once the pass is abandoned, the body does nothing more. -/
theorem passBody_abandoned (U : Universe Pos) :
    ∀ (l : List Pos) (s : PassState Pos Value), s.restart.isSome →
      (passBody U l).run s = ((), s)
  | [], _, _ => rfl
  | c :: l, s, h => by
      simp only [passBody, StateM.run_getBind, h, ↓reduceIte, StateT.run_pure]
      rfl

/-- One step of the pass body: unless the pass is abandoned, the root is set
to the cell, the cell is read, and the body goes on. -/
theorem passBody_cons (U : Universe Pos) (c : Pos) (l : List Pos) (s : PassState Pos Value) :
    (passBody U (c :: l)).run s =
      if s.restart.isSome then ((), s)
      else (passBody U l).run ((evalCell U (passFuel U) c).run (setRoot s c)).2 := by
  simp only [passBody, StateT.run_bind, StateT.run_get, StateT.run_modify, Id.pure_apply,
    Id.bind_apply]
  split <;> rfl

/-- The pass body from a state satisfying the invariant with an empty stack:
either the pass is abandoned, or the invariant holds, the stack is empty and
every formula cell it visited is evaluated. -/
theorem passBody_spec (S₀ : Sheet Pos Value) (U : Universe Pos) :
    ∀ (l : List Pos) (s : PassState Pos Value), PassInv S₀ s → s.stack = [] →
      BodyStep s ((passBody U l).run s).2 ∧
        ((((passBody U l).run s).2).restart.isSome ∨
          (PassInv S₀ ((passBody U l).run s).2 ∧ (((passBody U l).run s).2).stack = [] ∧
            ∀ c ∈ l, (s.sheet c).formula?.isSome →
              (((passBody U l).run s).2).cells c = some .evaluated))
  | [], s, hinv, hstack =>
      ⟨BodyStep.refl s, Or.inr ⟨hinv, hstack, fun c hc => absurd hc List.not_mem_nil⟩⟩
  | c :: l, s, hinv, hstack => by
      rw [passBody_cons, if_neg (by simp [hinv.not_abandoned])]
      have hinv₀ : PassInv S₀ (setRoot s c) := hinv.setRoot hstack c
      have hstack₀ : (setRoot s c).stack = [] := hstack
      obtain ⟨hstep₁, h₁, _⟩ := evalCell_spec U (passFuel U) c (setRoot s c) hinv₀
        (RootOk.setRoot s c) (passFuel_ok S₀ U (setRoot s c) hinv₀ hstack₀ c)
      set out₁ := (evalCell U (passFuel U) c).run (setRoot s c) with hout₁
      have hstep₁' : BodyStep s out₁.2 := (BodyStep.setRoot s c).trans hstep₁.toBody
      rcases h₁ with hab | ⟨hinv₁, hstack₁, _, hev⟩
      · rw [passBody_abandoned U l out₁.2 hab]
        exact ⟨hstep₁', Or.inl hab⟩
      · have hnot : (setRoot s c).cells c ≠ some .evaluating := by
          intro h
          have := (hinv.stack_evaluating c).mpr h
          rw [hstack] at this
          exact List.not_mem_nil this
        obtain ⟨hstep₂, h₂⟩ := passBody_spec S₀ U l out₁.2 hinv₁ (hstack₁.trans hstack)
        refine ⟨hstep₁'.trans hstep₂, ?_⟩
        rcases h₂ with hab | ⟨hinv₂, hstack₂, hev₂⟩
        · exact Or.inl hab
        · right
          refine ⟨hinv₂, hstack₂, fun x hx hf => ?_⟩
          rcases List.mem_cons.mp hx with rfl | hx
          · exact hstep₂.cells_mono x (hev hf hnot)
          · have hf₁ : (out₁.2.sheet x).formula?.isSome := by
              rcases SameShape.kind_stable hinv.shape hinv₁.shape x with h | h
              · rw [h.formula_eq.1]
                exact hf
              · rw [h] at hf
                cases hf
            exact hev₂ x hx hf₁

/-- A restart of the pass body names a dynamic anchor of the original sheet,
and if it drops stale cells, one of them is a spill cell of that anchor in
the original sheet. (What else a restart says is the subject of
`Loop.lean`.) -/
theorem passBody_restart_dyn (S₀ : Sheet Pos Value) (U : Universe Pos) :
    ∀ (l : List Pos) (s : PassState Pos Value), PassInv S₀ s → s.stack = [] →
      ∀ r, (((passBody U l).run s).2).restart = some r →
        (S₀ r.anchor).isDynAnchor = true ∧ r.StaleOk S₀
  | [], s, hinv, _, r, hr => by
      rw [passBody] at hr
      simp only [StateT.run_pure, Id.pure_apply] at hr
      rw [hinv.not_abandoned] at hr
      cases hr
  | c :: l, s, hinv, hstack, r, hr => by
      rw [passBody_cons, if_neg (by simp [hinv.not_abandoned])] at hr
      have hinv₀ : PassInv S₀ (setRoot s c) := hinv.setRoot hstack c
      have hstack₀ : (setRoot s c).stack = [] := hstack
      obtain ⟨_, h₁, hr₁⟩ := evalCell_spec U (passFuel U) c (setRoot s c) hinv₀
        (RootOk.setRoot s c) (passFuel_ok S₀ U (setRoot s c) hinv₀ hstack₀ c)
      set out₁ := (evalCell U (passFuel U) c).run (setRoot s c) with hout₁
      rcases h₁ with hab | ⟨hinv₁, hstack₁, _, _⟩
      · rw [passBody_abandoned U l out₁.2 hab] at hr
        have hok := hr₁ r hr
        refine ⟨?_, hok.stale⟩
        rw [← SameShape.isDynAnchor_eq hinv₀.shape]
        exact hok.dyn
      · exact passBody_restart_dyn S₀ U l out₁.2 hinv₁ (hstack₁.trans hstack) r hr

/-- When every anchor has committed and no spill cell is an orphan, the pass
reads the sheet as the specification does. -/
theorem passView_eq_valueAt (st : PassState Pos Value) (hno : NoOrphans st.sheet)
    (hall : ∀ p, (st.sheet p).formula?.isSome → st.cells p = some .evaluated) :
    passView st = valueAt st.sheet := by
  funext q
  unfold passView
  split
  · rename_i a v h
    have hanchor := hno q a v h
    have hform : (st.sheet a).formula?.isSome := by
      cases hs : st.sheet a <;> simp [hs, Content.isAnchor, Content.formula?] at hanchor ⊢
    simp [hall a hform]
  · rfl

/-! ## 5.2 Partial correctness -/

/-- `runPass` in terms of the pass body. -/
theorem runPass_eq (U : Universe Pos) (S : Sheet Pos Value) (order : List Pos)
    (circular : Finset Pos) :
    runPass U S order circular =
      ((((passBody U (order ++ U.positions)).run (PassState.initial S circular)).2).sheet,
        (((passBody U (order ++ U.positions)).run (PassState.initial S circular)).2).restart) :=
  rfl

/-- A completed pass leaves a consistent sheet. -/
theorem runPass_consistent (U : Universe Pos) (S S' : Sheet Pos Value) (order : List Pos)
    (circular : Finset Pos) (hwf : WellFormed S)
    (h : runPass U S order circular = (S', none)) :
    Consistent S' := by
  rw [runPass_eq] at h
  simp only [Prod.mk.injEq] at h
  obtain ⟨hS', hrest⟩ := h
  obtain ⟨_, hspec⟩ := passBody_spec S U (order ++ U.positions) (PassState.initial S circular)
    (PassInv.initial S circular hwf) rfl
  rcases hspec with hab | ⟨hinv, hstack, hev⟩
  · rw [hrest] at hab
    cases hab
  · subst hS'
    intro p
    by_cases hf : ((((passBody U (order ++ U.positions)).run
        (PassState.initial S circular)).2).sheet p).formula?.isSome
    · have hev' := hev p (List.mem_append_right _ (U.complete p)) (by
        rcases SameShape.kind_stable hinv.shape (PassInv.initial S circular hwf).shape p with h | h
        · rw [h.formula_eq.1]
          exact hf
        · rw [h] at hf
          cases hf)
      have := hinv.consistentAt hev'
      rwa [passView_eq_valueAt _ hinv.no_orphans (fun q hq => hev q
        (List.mem_append_right _ (U.complete q)) (by
          rcases SameShape.kind_stable hinv.shape (PassInv.initial S circular hwf).shape q with h | h
          · rw [h.formula_eq.1]
            exact hq
          · rw [h] at hq
            cases hq))] at this
    · exact ConsistentAtWith.of_not_formula (Option.not_isSome_iff_eq_none.mp hf)

/-- What the driver learns from a restart: it names a dynamic anchor of the
sheet the pass started from, and if it drops stale cells, one of them is a
spill cell of that anchor in that sheet. -/
theorem runPass_restart_dyn (U : Universe Pos) (S S' : Sheet Pos Value) (order : List Pos)
    (circular : Finset Pos) (r : Restart Pos) (hwf : WellFormed S)
    (h : runPass U S order circular = (S', some r)) :
    (S r.anchor).isDynAnchor = true ∧ r.StaleOk S := by
  rw [runPass_eq] at h
  simp only [Prod.mk.injEq] at h
  obtain ⟨_, hrest⟩ := h
  exact passBody_restart_dyn S U (order ++ U.positions) (PassState.initial S circular)
    (PassInv.initial S circular hwf) rfl r hrest

/-- Whatever the loop returns is consistent. The sheet a pass starts from is
well-formed throughout: dropping stale cells keeps it so. -/
theorem evaluateLoop_consistent (U : Universe Pos) :
    ∀ (fuel : Nat) (S : Sheet Pos Value) (order : List Pos) (log : RestartLog Pos)
      (S' : Sheet Pos Value) (order' : List Pos) (log' : RestartLog Pos), WellFormed S →
      evaluateLoop U fuel S order log = some (S', order', log') → Consistent S'
  | 0, _, _, _, _, _, _, _, h => by simp [evaluateLoop] at h
  | k + 1, S, order, log, S', order', log', hwf, h => by
      rcases hrun : runPass U S order log.circular with ⟨S₁, _ | r⟩
      · simp only [evaluateLoop, hrun, Option.some.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, _, _⟩ := h
        exact runPass_consistent U S S₁ order log.circular hwf hrun
      · rcases hrec : log.record r order with ⟨log₁, order₁, marked⟩
        simp only [evaluateLoop, hrun, hrec] at h
        exact evaluateLoop_consistent U k _ _ _ S' order' log'
          (hwf.nextSheet (runPass_restart_dyn U S S₁ order log.circular r hwf hrun).1 marked) h

/-- Whatever `evaluate` returns is consistent. -/
theorem evaluate_consistent (U : Universe Pos) (S S' : Sheet Pos Value)
    (anchorOrder order' : List Pos) (log : RestartLog Pos) (fuel : Nat)
    (hwf : WellFormed S) (h : evaluate U S anchorOrder fuel = some (S', order', log)) :
    Consistent S' :=
  evaluateLoop_consistent U fuel S _ _ S' order' log hwf h

end IronCalcEval

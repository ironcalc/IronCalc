import IronCalcEval.Correctness

/-!
# The pass, seen from the driver

What the driver's termination argument needs to know about one pass:

* the anchor a restart names is a dynamic anchor of the sheet
  (`passBody_restartOk`, in `Correctness.lean`), hence in the anchor order;
* a marked anchor restarts only when an unmarked anchor precedes it in the
  order (`passBody_marked`).

The second is the corrected form of "a marked anchor never restarts again"
(§5.3). The argument: a marked cell runs no formula, so it makes no reads and
no records, and the only restart it can cause is the removal of its own
leftover cells contradicting a record made earlier. Records are made only
under unmarked top-level cells, which precede it in the order. Any other
restart names an anchor that has not been evaluated, which lies after the
current top-level cell, itself unmarked.
-/

namespace IronCalcEval

variable {Pos Value : Type} [DecidableEq Pos] [ValueSort Value]

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

section marked
variable (U : Universe Pos)

/-- The tail of a formula cell whose result is `#CIRC!`: records are
untouched, and a restart names the cell and needs an occupied record that
was already there. -/
theorem finish_circ_facts (s : PassState Pos Value) (c : Pos) (hrest : s.restart = none)
    (hf : (s.sheet c).formula?.isSome) :
    (((finishFormulaCell U c (.scalar circ)).run s).2).seenEmpty = s.seenEmpty ∧
    (((finishFormulaCell U c (.scalar circ)).run s).2).seenOccupied = s.seenOccupied ∧
    ∀ r, (((finishFormulaCell U c (.scalar circ)).run s).2).restart = some r →
      r.anchor = c ∧ ∃ x r', s.seenOccupied x = some r' := by
  have hnone : s.restart.isNone = true := by simp [hrest]
  simp only [finishFormulaCell, StateM.run_getBind, hnone, ↓reduceIte]
  try dsimp only
  have hr' : (if c ∈ s.circular then Result.scalar circ else Result.scalar circ) =
      (Result.scalar circ : Result Pos Value) := by
    split <;> rfl
  rw [hr', StateT.run_bind, Id.bind_apply, StateM.run_getBind]
  cases hsc : s.sheet c with
  | formula t v =>
      rw [commit_run_formula U s c _ t v hsc]
      simp only [hrest, Option.isSome_none, Bool.false_eq_true, ↓reduceIte, StateT.run_bind,
        StateT.run_modify, Id.pure_apply, Id.bind_apply]
      exact ⟨rfl, rfl, fun r hr => by cases hr⟩
  | cseAnchor t area v =>
      rw [commit_run_cse U s c _ t area v hsc]
      simp only [hrest, Option.isSome_none, Bool.false_eq_true, ↓reduceIte, StateT.run_bind,
        StateT.run_modify, Id.pure_apply, Id.bind_apply]
      exact ⟨rfl, rfl, fun r hr => by cases hr⟩
  | dynAnchor t v =>
      rw [commit_run_dyn U s c _ t v hsc]
      try dsimp only
      rw [storeScalar_run]
      try dsimp only
      split
      · simp only [hrest, Option.isSome_none, Bool.false_eq_true, ↓reduceIte, StateT.run_bind,
          StateT.run_modify, Id.pure_apply, Id.bind_apply]
        exact ⟨rfl, rfl, fun r hr => by cases hr⟩
      · rename_i hroots
        simp only [Option.isSome_some, ↓reduceIte, StateT.run_pure, Id.pure_apply]
        refine ⟨by simp, by simp, fun r hr => ?_⟩
        simp only [Option.some.injEq] at hr
        subst hr
        refine ⟨by split <;> rfl, ?_⟩
        rw [Bool.not_eq_true, List.isEmpty_eq_false_iff_exists_mem] at hroots
        obtain ⟨r', hr'⟩ := hroots
        obtain ⟨x, _, hx⟩ := List.mem_filterMap.mp hr'
        exact ⟨x, r', hx⟩
  | empty => simp [hsc, Content.formula?] at hf
  | const _ => simp [hsc, Content.formula?] at hf
  | spill _ _ => simp [hsc, Content.formula?] at hf

/-- A marked formula cell read at top level: no formula runs, records are
untouched, and a restart names the cell and needs an existing occupied
record. -/
theorem evalCell_marked_top {S₀ : Sheet Pos Value} (st : PassState Pos Value) (c : Pos)
    (hinv : PassInv S₀ st) (hstack : st.stack = []) (hc : c ∈ st.circular)
    (hf : (st.sheet c).formula?.isSome) :
    (((evalCell U (passFuel U) c).run st).2).seenEmpty = st.seenEmpty ∧
    (((evalCell U (passFuel U) c).run st).2).seenOccupied = st.seenOccupied ∧
    ∀ r, (((evalCell U (passFuel U) c).run st).2).restart = some r →
      r.anchor = c ∧ ∃ x r', st.seenOccupied x = some r' := by
  obtain ⟨k, hk⟩ : ∃ k, passFuel U = k + 1 := ⟨2 * U.positions.length + 1, rfl⟩
  rw [hk]
  simp only [evalCell, StateM.run_getBind, hinv.not_abandoned, Option.isSome_none,
    Bool.false_eq_true, ↓reduceIte]
  have hnot : st.cells c ≠ some .evaluating := by
    intro h
    have := (hinv.stack_evaluating c).mpr h
    rw [hstack] at this
    exact List.not_mem_nil this
  -- The body shared by the three kinds of formula cell.
  have body : ∀ t, (((evalFormulaCell U (evalCell U k) c t).run st).2).seenEmpty = st.seenEmpty ∧
      (((evalFormulaCell U (evalCell U k) c t).run st).2).seenOccupied = st.seenOccupied ∧
      ∀ r, (((evalFormulaCell U (evalCell U k) c t).run st).2).restart = some r →
        r.anchor = c ∧ ∃ x r', st.seenOccupied x = some r' := by
    intro t
    simp only [evalFormulaCell, StateM.run_getBind]
    rcases hcc : st.cells c with _ | (_ | _)
    · try simp only []
      rw [StateT.run_bind, StateT.run_modify, Id.pure_apply, Id.bind_apply, StateM.run_getBind]
      show (((if c ∈ (pushState st c).circular then pure (Result.scalar circ)
        else t.run (evalCell U k) : PassM Pos Value (Result Pos Value)) >>=
          fun r => finishFormulaCell U c r).run (pushState st c)).2.seenEmpty = _ ∧ _
      simp only [pushState, hc, ↓reduceIte, StateT.run_bind, StateT.run_pure, Id.bind_apply,
        Id.pure_apply]
      exact finish_circ_facts U (pushState st c) c hinv.not_abandoned hf
    · exact absurd hcc hnot
    · try simp only []
      refine ⟨rfl, rfl, fun r hr => ?_⟩
      have hr' : st.restart = some r := hr
      rw [hinv.not_abandoned] at hr'
      cases hr'
  cases hsc : st.sheet c with
  | formula t v => exact body t
  | cseAnchor t area v => exact body t
  | dynAnchor t v => exact body t
  | empty => simp [hsc, Content.formula?] at hf
  | const _ => simp [hsc, Content.formula?] at hf
  | spill _ _ => simp [hsc, Content.formula?] at hf

end marked

/-! ## The restart of a pass -/

section loop
variable (S₀ : Sheet Pos Value) (U : Universe Pos)

/-- The loop invariant of the marked-anchor argument: the processed prefix
`P` is evaluated, the driver's marks are marks of the pass, and if any record
exists, some processed cell is unmarked, unless the whole order is processed. -/
structure LoopInv (order : List Pos) (circ₀ : Finset Pos) (P : List Pos)
    (s : PassState Pos Value) : Prop where
  inv : PassInv S₀ s
  stack : s.stack = []
  circ : circ₀ ⊆ s.circular
  done : ∀ c ∈ P, (s.sheet c).formula?.isSome → s.cells c = some .evaluated
  records : (∃ x r, s.seenEmpty x = some r ∨ s.seenOccupied x = some r) →
    (∃ u ∈ P, u ∉ circ₀) ∨ order ⊆ P

omit [ValueSort Value] in
/-- Once the processed prefix is as long as the order, the order is processed. -/
theorem order_sub_of_le {order positions P l : List Pos}
    (h : order ++ positions = P ++ l) (hlen : order.length ≤ P.length) : order ⊆ P :=
  (List.prefix_of_prefix_length_le (List.prefix_append order positions)
    (h ▸ List.prefix_append P l) hlen).subset

omit [ValueSort Value] in
/-- Before that, the current cell is the next of the order. -/
theorem order_split {order positions P l : List Pos} {c : Pos}
    (h : order ++ positions = P ++ c :: l) (hlen : P.length < order.length) :
    ∃ rest, order = P ++ c :: rest := by
  have hPc : P ++ [c] <+: order ++ positions := by
    rw [h]
    exact ⟨l, by simp⟩
  obtain ⟨rest, hrest⟩ := List.prefix_of_prefix_length_le hPc (List.prefix_append order positions)
    (by simp; omega)
  exact ⟨rest, by rw [← hrest]; simp⟩

omit [ValueSort Value] in
theorem idxOf_prefix_lt {P rest : List Pos} {c u : Pos} (hnd : (P ++ c :: rest).Nodup)
    (hu : u ∈ P) : (P ++ c :: rest).idxOf u < (P ++ c :: rest).idxOf c := by
  have hcP : c ∉ P := by
    intro h
    have := List.nodup_append.mp hnd
    exact this.2.2 c h c List.mem_cons_self rfl
  rw [List.idxOf_append, List.idxOf_append]
  simp only [hu, ↓reduceIte, hcP, List.idxOf_cons_self, Nat.zero_add]
  exact List.idxOf_lt_length_iff.mpr hu

omit [ValueSort Value] in
theorem idxOf_lt_of_rest {P rest : List Pos} {c a : Pos} (hnd : (P ++ c :: rest).Nodup)
    (haP : a ∉ P) (hac : a ≠ c) :
    (P ++ c :: rest).idxOf c < (P ++ c :: rest).idxOf a := by
  have hcP : c ∉ P := by
    intro h
    have := List.nodup_append.mp hnd
    exact this.2.2 c h c List.mem_cons_self rfl
  rw [List.idxOf_append, List.idxOf_append, if_neg hcP, if_neg haP, List.idxOf_cons_self,
    List.idxOf_cons_ne _ hac.symm]
  omega

/-- The marked-anchor argument, along the pass body. -/
theorem passBody_marked (order : List Pos) (hnd : order.Nodup) (horder : OrderOf S₀ order)
    (circ₀ : Finset Pos) :
    ∀ (l P : List Pos) (s : PassState Pos Value), LoopInv S₀ order circ₀ P s →
      order ++ U.positions = P ++ l →
      ∀ r, (((passBody U l).run s).2).restart = some r → r.anchor ∈ circ₀ →
        ∃ u ∈ order, u ∉ circ₀ ∧ order.idxOf u < order.idxOf r.anchor
  | [], _, s, hinv, _, r, hr, _ => by
      rw [passBody] at hr
      simp only [StateT.run_pure, Id.pure_apply] at hr
      rw [hinv.inv.not_abandoned] at hr
      cases hr
  | c :: l, P, s, hloop, hsplit, r, hr, hmarked => by
      have hinv := hloop.inv
      have hstack := hloop.stack
      simp only [passBody, StateT.run_bind, StateT.run_get, Id.pure_apply, Id.bind_apply,
        hinv.not_abandoned, Option.isSome_none, Bool.false_eq_true, ↓reduceIte] at hr
      obtain ⟨hstep₁, h₁, hr₁⟩ :=
        evalCell_spec U (passFuel U) c s hinv (passFuel_ok S₀ U s hinv hstack c)
      set out₁ := (evalCell U (passFuel U) c).run s with hout₁
      -- Facts about anchors, relative to the initial sheet.
      have hdyn_order : ∀ a, (s.sheet a).isDynAnchor = true → a ∈ order := fun a ha =>
        (horder.2 a).mpr (by rw [← SameShape.isDynAnchor_eq hinv.shape a]; exact ha)
      have hformula_of_order : ∀ a ∈ order, (s.sheet a).formula?.isSome := by
        intro a ha
        have := (horder.2 a).mp ha
        rw [← SameShape.isDynAnchor_eq hinv.shape a] at this
        cases hs : s.sheet a <;> simp_all [Content.isDynAnchor, Content.formula?]
      have hnotP' : ∀ r : Restart Pos, RestartOk S₀ s r → r.anchor ∉ P := by
        intro r ⟨hdyn, hev, _⟩ haP
        apply hev
        apply hloop.done _ haP
        cases hs : s.sheet r.anchor <;> simp_all [Content.isDynAnchor, Content.formula?]
      rcases h₁ with hab | ⟨hinv₁, hstack₁, _, hev⟩
      · -- The read itself restarted.
        rw [passBody_abandoned U l out₁.2 hab] at hr
        have hok := hr₁ r hr
        have ha_order : r.anchor ∈ order := hdyn_order _ hok.1
        have ha_notP : r.anchor ∉ P := hnotP' r hok
        have hlen : P.length < order.length := by
          by_contra hle
          exact ha_notP (order_sub_of_le hsplit (by omega) ha_order)
        obtain ⟨rest, hrest⟩ := order_split hsplit hlen
        have hc_order : c ∈ order := by rw [hrest]; simp
        have hf : (s.sheet c).formula?.isSome := hformula_of_order c hc_order
        have hnd' : (P ++ c :: rest).Nodup := hrest ▸ hnd
        by_cases hcm : c ∈ s.circular
        · -- A marked cell restarted on its own leftovers: an earlier unmarked
          -- cell made the record it contradicted.
          obtain ⟨hrc, hrec⟩ := (evalCell_marked_top U s c hinv hstack hcm hf).2.2 r hr
          obtain ⟨x, r', hx⟩ := hrec
          rcases hloop.records ⟨x, r', Or.inr hx⟩ with ⟨u, huP, hu⟩ | hsub
          · refine ⟨u, by rw [hrest]; exact List.mem_append_left _ huP, hu, ?_⟩
            rw [hrc, hrest]
            exact idxOf_prefix_lt hnd' huP
          · exact absurd (hsub ha_order) ha_notP
        · -- An unmarked cell: it precedes the anchor, which was not evaluated.
          have hac : r.anchor ≠ c := by
            rintro heq
            rw [heq] at hmarked
            exact hcm (hloop.circ hmarked)
          refine ⟨c, hc_order, fun h => hcm (hloop.circ h), ?_⟩
          rw [hrest]
          exact idxOf_lt_of_rest hnd' ha_notP hac
      · -- The read completed: continue with `c` processed.
        have hfs : ∀ x, (out₁.2.sheet x).formula?.isSome → (s.sheet x).formula?.isSome := by
          intro x hx
          rcases SameShape.kind_stable hinv₁.shape hinv.shape x with h | h
          · rw [← h.formula_eq.1] at hx
            exact hx
          · rw [h] at hx
            cases hx
        have hnot : s.cells c ≠ some .evaluating := by
          intro h
          have := (hinv.stack_evaluating c).mpr h
          rw [hstack] at this
          exact List.not_mem_nil this
        have hloop₁ : LoopInv S₀ order circ₀ (P ++ [c]) out₁.2 :=
          { inv := hinv₁
            stack := hstack₁.trans hstack
            circ := Finset.Subset.trans hloop.circ hstep₁.circular_mono
            done := by
              intro x hx hfx
              rcases List.mem_append.mp hx with hx | hx
              · exact hstep₁.cells_mono x (hloop.done x hx (hfs x hfx))
              · rw [List.mem_singleton] at hx
                subst hx
                exact hev (hfs x hfx) hnot
            records := by
              intro hrec
              by_cases hlen : order.length ≤ P.length
              · exact Or.inr (fun x hx => List.mem_append_left _ (order_sub_of_le hsplit hlen hx))
              · obtain ⟨rest, hrest⟩ := order_split hsplit (by omega)
                have hc_order : c ∈ order := by rw [hrest]; simp
                have hf : (s.sheet c).formula?.isSome := hformula_of_order c hc_order
                by_cases hcm : c ∈ s.circular
                · obtain ⟨hse, hso, _⟩ := evalCell_marked_top U s c hinv hstack hcm hf
                  rw [hse, hso] at hrec
                  rcases hloop.records hrec with ⟨u, huP, hu⟩ | hsub
                  · exact Or.inl ⟨u, List.mem_append_left _ huP, hu⟩
                  · exact Or.inr (fun x hx => List.mem_append_left _ (hsub hx))
                · exact Or.inl ⟨c, List.mem_append_right _ (List.mem_singleton_self c),
                    fun h => hcm (hloop.circ h)⟩ }
        exact passBody_marked order hnd horder circ₀ l (P ++ [c]) out₁.2 hloop₁
          (by rw [hsplit, List.append_assoc]; rfl) r hr hmarked

end loop

end IronCalcEval

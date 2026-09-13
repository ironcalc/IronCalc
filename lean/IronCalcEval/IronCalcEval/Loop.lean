import IronCalcEval.Correctness

/-!
# The pass, seen from the driver

What the driver's termination argument needs to know about one pass
(`RestartFacts`): the anchor a restart names is a dynamic anchor of the
order; its readers are anchors of the order placed before it, none of them
marked; a stale read or a conflict has at least one reader; a marked anchor
restarts only over a stale cell of its own, which the driver never leaves in
the sheet; and a stale-cells restart names a cell of the sheet.

The argument: every record is made on behalf of the root, the cell the
driver is evaluating; a marked cell runs no formula, so it makes no records
and reads nothing stale; so the readers of a restart are top-level cells
processed before the anchor, unmarked, and the restart happens while the
anchors are being processed, so all of them are anchors of the order.
-/

namespace IronCalcEval

variable {Pos Value : Type} [DecidableEq Pos] [ValueSort Value]

section marked
variable (U : Universe Pos)

/-- The tail of a formula cell whose result is `#CIRC!`: records are
untouched, and a restart's readers are roots of occupied records that were
already there. -/
theorem finish_circ_facts (s : PassState Pos Value) (c : Pos) (hrest : s.restart = none)
    (hf : (s.sheet c).formula?.isSome) :
    (((finishFormulaCell U c (.scalar circ)).run s).2).seenEmpty = s.seenEmpty ∧
    (((finishFormulaCell U c (.scalar circ)).run s).2).seenOccupied = s.seenOccupied ∧
    ∀ r, (((finishFormulaCell U c (.scalar circ)).run s).2).restart = some r →
      ∀ x ∈ r.readers, ∃ q, s.seenOccupied q = some x := by
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
      · simp only [Option.isSome_some, ↓reduceIte, StateT.run_pure, Id.pure_apply]
        refine ⟨by simp, by simp, fun r hr => ?_⟩
        simp only [Option.some.injEq] at hr
        subst hr
        split
        · simp [Restart.readers]
        · intro x hx
          simp only [Restart.readers, List.mem_dedup, List.mem_filter] at hx
          obtain ⟨hx, _⟩ := hx
          obtain ⟨q, _, hq⟩ := List.mem_filterMap.mp hx
          exact ⟨q, hq⟩
  | empty => simp [hsc, Content.formula?] at hf
  | const _ => simp [hsc, Content.formula?] at hf
  | spill _ _ => simp [hsc, Content.formula?] at hf

/-- A marked formula cell read at top level: no formula runs, records are
untouched, and a restart's readers are roots of occupied records that were
already there. -/
theorem evalCell_marked_top {S₀ : Sheet Pos Value} (st : PassState Pos Value) (c : Pos)
    (hinv : PassInv S₀ st) (hstack : st.stack = []) (hc : c ∈ st.circular)
    (hf : (st.sheet c).formula?.isSome) :
    (((evalCell U (passFuel U) c).run st).2).seenEmpty = st.seenEmpty ∧
    (((evalCell U (passFuel U) c).run st).2).seenOccupied = st.seenOccupied ∧
    ∀ r, (((evalCell U (passFuel U) c).run st).2).restart = some r →
      ∀ x ∈ r.readers, ∃ q, st.seenOccupied q = some x := by
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
        ∀ x ∈ r.readers, ∃ q, st.seenOccupied q = some x := by
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

/-- What the driver learns from a restart of a pass started from the sheet
`S₀` with the anchors in `order` and `circ₀` marked. -/
structure RestartFacts (S₀ : Sheet Pos Value) (order : List Pos) (circ₀ : Finset Pos)
    (r : Restart Pos) : Prop where
  anchor_mem : r.anchor ∈ order
  /-- A marked anchor restarts only over a stale cell of its own. -/
  marked_spill : r.anchor ∈ circ₀ → ∃ q v, S₀ q = .spill r.anchor v
  /-- Every reader is an unmarked anchor placed before the restarting one. -/
  readers : ∀ x ∈ r.readers, x ∈ order ∧ x ∉ circ₀ ∧ order.idxOf x < order.idxOf r.anchor
  nonempty : r.learns = true → r.readers ≠ []
  nodup : r.readers.Nodup
  stale : r.StaleOk S₀

section loop
variable (S₀ : Sheet Pos Value) (U : Universe Pos)

/-- The loop invariant: the processed prefix `P` is evaluated, the driver's
marks are marks of the pass, and every record was made on behalf of an
unmarked processed cell. -/
structure LoopInv (order : List Pos) (circ₀ : Finset Pos) (P : List Pos)
    (s : PassState Pos Value) : Prop where
  inv : PassInv S₀ s
  stack : s.stack = []
  circ : circ₀ ⊆ s.circular
  done : ∀ c ∈ P, (s.sheet c).formula?.isSome → s.cells c = some .evaluated
  roots : ∀ x r, (s.seenEmpty x = some r ∨ s.seenOccupied x = some r) → r ∈ P ∧ r ∉ circ₀

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

omit [ValueSort Value] in
theorem not_mem_of_prefix_nodup {P rest : List Pos} {c : Pos} (hnd : (P ++ c :: rest).Nodup) :
    c ∉ P := by
  intro h
  have := List.nodup_append.mp hnd
  exact this.2.2 c h c List.mem_cons_self rfl

/-- The facts of a restart, along the pass body. -/
theorem passBody_facts (order : List Pos) (horder : OrderOf S₀ order) (circ₀ : Finset Pos)
    (hcirc : ∀ x ∈ circ₀, x ∈ order) :
    ∀ (l P : List Pos) (s : PassState Pos Value), LoopInv S₀ order circ₀ P s →
      order ++ U.positions = P ++ l →
      ∀ r, (((passBody U l).run s).2).restart = some r → RestartFacts S₀ order circ₀ r
  | [], _, s, hinv, _, r, hr => by
      rw [passBody] at hr
      simp only [StateT.run_pure, Id.pure_apply] at hr
      rw [hinv.inv.not_abandoned] at hr
      cases hr
  | c :: l, P, s, hloop, hsplit, r, hr => by
      have hinv := hloop.inv
      have hstack := hloop.stack
      have hnd : order.Nodup := horder.1
      rw [passBody_cons, if_neg (by simp [hinv.not_abandoned])] at hr
      set s₀ := setRoot s c with hs₀
      have hinv₀ : PassInv S₀ s₀ := hinv.setRoot hstack c
      have hstack₀ : s₀.stack = [] := hstack
      obtain ⟨hstep₁, h₁, hr₁⟩ := evalCell_spec U (passFuel U) c s₀ hinv₀
        (RootOk.setRoot s c) (passFuel_ok S₀ U s₀ hinv₀ hstack₀ c)
      set out₁ := (evalCell U (passFuel U) c).run s₀ with hout₁
      -- Facts about anchors, relative to the initial sheet.
      have hdyn_order : ∀ a, (s.sheet a).isDynAnchor = true → a ∈ order := fun a ha =>
        (horder.2 a).mpr (by rw [← SameShape.isDynAnchor_eq hinv.shape a]; exact ha)
      have hformula_of_order : ∀ a ∈ order, (s.sheet a).formula?.isSome := by
        intro a ha
        have := (horder.2 a).mp ha
        rw [← SameShape.isDynAnchor_eq hinv.shape a] at this
        cases hs : s.sheet a <;> simp_all [Content.isDynAnchor, Content.formula?]
      rcases h₁ with hab | ⟨hinv₁, hstack₁, _, hev⟩
      · -- The read of `c` restarted.
        rw [passBody_abandoned U l out₁.2 hab] at hr
        have hok : RestartOk S₀ s₀ r := hr₁ r hr
        have ha_order : r.anchor ∈ order := hdyn_order _ hok.dyn
        have ha_notP : r.anchor ∉ P := fun haP =>
          hok.not_evaluated (hloop.done _ haP (hformula_of_order _ ha_order))
        have hlen : P.length < order.length := by
          by_contra hle
          exact ha_notP (order_sub_of_le hsplit (by omega) ha_order)
        obtain ⟨rest, hrest⟩ := order_split hsplit hlen
        have hnd' : (P ++ c :: rest).Nodup := hrest ▸ hnd
        have hcP : c ∉ P := not_mem_of_prefix_nodup hnd'
        have hc_order : c ∈ order := by rw [hrest]; simp
        -- A reader is a processed cell, or `c` itself; either way unmarked
        -- and before the anchor.
        have hreader : ∀ x ∈ r.readers, x ∈ order ∧ x ∉ circ₀ ∧
            order.idxOf x < order.idxOf r.anchor := by
          intro x hx
          obtain ⟨hxa, hrec⟩ := hok.readers x hx
          have hlt_c : ∀ u ∈ P, order.idxOf u < order.idxOf r.anchor := by
            intro u hu
            rw [hrest]
            by_cases hac : r.anchor = c
            · rw [hac]
              exact idxOf_prefix_lt hnd' hu
            · exact (idxOf_prefix_lt hnd' hu).trans (idxOf_lt_of_rest hnd' ha_notP hac)
          rcases hrec with ⟨q, hq⟩ | hq
          · obtain ⟨hxP, hxc⟩ := hloop.roots q x hq
            exact ⟨by rw [hrest]; exact List.mem_append_left _ hxP, hxc, hlt_c x hxP⟩
          · -- The root, `c` itself.
            have hxc : x = c := by
              have : (setRoot s c).root = some x := hq
              simp [setRoot] at this
              exact this.symm
            subst hxc
            refine ⟨hc_order, ?_, ?_⟩
            · -- A marked `c` makes no records and reads nothing: its
              -- readers were roots of records already there, processed cells.
              intro hxm
              have hcm : x ∈ s₀.circular := hloop.circ hxm
              obtain ⟨-, -, hrd⟩ := evalCell_marked_top U s₀ x hinv₀ hstack₀ hcm
                (hformula_of_order x hc_order)
              obtain ⟨q, hq⟩ := hrd r hr x hx
              exact hcP (hloop.roots q x (Or.inr hq)).1
            · rw [hrest]
              exact idxOf_lt_of_rest hnd' ha_notP (Ne.symm hxa)
        exact
          { anchor_mem := ha_order
            marked_spill := fun hm => hok.marked_spill (hloop.circ hm)
            readers := hreader
            nonempty := hok.nonempty
            nodup := hok.nodup
            stale := hok.stale }
      · -- The read completed: continue with `c` processed.
        have hfs : ∀ x, (out₁.2.sheet x).formula?.isSome → (s.sheet x).formula?.isSome := by
          intro x hx
          rcases SameShape.kind_stable hinv₁.shape hinv.shape x with h | h
          · rw [← h.formula_eq.1] at hx
            exact hx
          · rw [h] at hx
            cases hx
        have hnot : s₀.cells c ≠ some .evaluating := by
          intro h
          have := (hinv.stack_evaluating c).mpr h
          rw [hstack] at this
          exact List.not_mem_nil this
        -- New records are on behalf of `c`; a marked `c` makes none.
        have hnew_root : ∀ x r', (out₁.2.seenEmpty x = some r' ∨ out₁.2.seenOccupied x = some r') →
            (s.seenEmpty x = some r' ∨ s.seenOccupied x = some r') ∨ r' = c := by
          intro x r' h
          rcases h with h | h
          · rcases hstep₁.roots_new_empty x r' h with h | h
            · exact Or.inl (Or.inl h)
            · right
              have : (setRoot s c).root = some r' := h
              simp [setRoot] at this
              exact this.symm
          · rcases hstep₁.roots_new_occupied x r' h with h | h
            · exact Or.inl (Or.inr h)
            · right
              have : (setRoot s c).root = some r' := h
              simp [setRoot] at this
              exact this.symm
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
            roots := by
              intro x r' h
              rcases hnew_root x r' h with h | rfl
              · obtain ⟨hP, hc⟩ := hloop.roots x r' h
                exact ⟨List.mem_append_left _ hP, hc⟩
              · refine ⟨List.mem_append_right _ (List.mem_singleton_self _), ?_⟩
                intro hm
                -- A marked cell is an anchor, and a marked anchor makes no
                -- records: the record was already there.
                have hf : (s.sheet r').formula?.isSome := hformula_of_order r' (hcirc r' hm)
                have hcm : r' ∈ s₀.circular := hloop.circ hm
                obtain ⟨hse, hso, -⟩ := evalCell_marked_top U s₀ r' hinv₀ hstack₀ hcm hf
                rcases h with h | h
                · rw [hse] at h
                  exact (hloop.roots x r' (Or.inl h)).2 hm
                · rw [hso] at h
                  exact (hloop.roots x r' (Or.inr h)).2 hm }
        exact passBody_facts order horder circ₀ hcirc l (P ++ [c]) out₁.2 hloop₁
          (by rw [hsplit, List.append_assoc]; rfl) r hr

/-- What the driver learns from a restart. -/
theorem runPass_facts (S S' : Sheet Pos Value) (order : List Pos) (circ₀ : Finset Pos)
    (r : Restart Pos) (hwf : WellFormed S) (horder : OrderOf S order)
    (hcirc : ∀ x ∈ circ₀, x ∈ order) (h : runPass U S order circ₀ = (S', some r)) :
    RestartFacts S order circ₀ r := by
  rw [runPass_eq] at h
  simp only [Prod.mk.injEq] at h
  obtain ⟨_, hrest⟩ := h
  have hloop : LoopInv S order circ₀ [] (PassState.initial S circ₀) :=
    { inv := PassInv.initial S circ₀ hwf
      stack := rfl
      circ := fun x hx => hx
      done := by simp
      roots := fun x r h => by simp [PassState.initial] at h }
  exact passBody_facts S U order horder circ₀ hcirc (order ++ U.positions) []
    (PassState.initial S circ₀) hloop rfl r hrest

end loop

end IronCalcEval

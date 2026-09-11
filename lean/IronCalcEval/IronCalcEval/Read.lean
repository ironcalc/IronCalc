import IronCalcEval.Commit

/-!
# Reads

The specification of `evaluate_cell` (`ReadSpec`): from a state satisfying
the invariant, with enough fuel, a read either abandons the pass or leaves
the invariant and the stack as they were and returns the pass view of the
position it read, unless the cell on whose behalf it read (the top of the
stack) ends up marked circular. The last clause is what a read that closes a
cycle does: it returns `#CIRC!` where the view says empty, and marks every
cell from the one it met down to the reader.

`RunSpec` is the same statement for a whole formula. The mutual recursion of
the Rust becomes an induction on the fuel: `evalCell_spec f` is proved from
the specs of `evalSpillCell` and `evalFormulaCell` with `rec := evalCell U f`,
which assume `evalCell_spec f` in the form `RecSpec`.
-/

namespace IronCalcEval

variable {Pos Value : Type} [DecidableEq Pos] [ValueSort Value]

/-! ## Monotone steps (for abandoned paths) -/

/-- The unconditional half of `PassStep`. -/
structure MonoStep (a b : PassState Pos Value) : Prop where
  restart_mono : a.restart.isSome → b.restart.isSome
  cells_mono : ∀ q, a.cells q = some .evaluated → b.cells q = some .evaluated
  circular_mono : a.circular ⊆ b.circular
  seenEmpty_mono : ∀ q r, a.seenEmpty q = some r → b.seenEmpty q = some r
  seenOccupied_mono : ∀ q r, a.seenOccupied q = some r → b.seenOccupied q = some r

def monoRel (Pos Value : Type) [DecidableEq Pos] : StateRel (PassState Pos Value) where
  R := MonoStep
  refl _ := ⟨id, fun _ h => h, Finset.Subset.refl _, fun _ _ h => h, fun _ _ h => h⟩
  trans h₁ h₂ := ⟨fun h => h₂.1 (h₁.1 h), fun q h => h₂.2 q (h₁.2 q h),
    Finset.Subset.trans h₁.3 h₂.3, fun q r h => h₂.4 q r (h₁.4 q r h),
    fun q r h => h₂.5 q r (h₁.5 q r h)⟩

theorem PassStep.of_abandoned {S₀ : Sheet Pos Value} {a b : PassState Pos Value}
    (m : MonoStep a b) (hb : b.restart.isSome) : PassStep S₀ a b :=
  ⟨m.1, m.2, m.3, m.4, m.5, fun _ => Or.inl hb, fun _ hbn => by simp [hbn] at hb⟩

theorem PassStep.toMono {S₀ : Sheet Pos Value} {a b : PassState Pos Value} (h : PassStep S₀ a b) :
    MonoStep a b :=
  ⟨h.1, h.2, h.3, h.4, h.5⟩

section mono
local notation "MR" => monoRel Pos Value

omit [DecidableEq Pos] [ValueSort Value] in
/-- A step that leaves cells, circular set and records alone. -/
theorem MonoStep.of_same {a b : PassState Pos Value} (hc : b.cells = a.cells)
    (hcirc : b.circular = a.circular) (hse : b.seenEmpty = a.seenEmpty)
    (hso : b.seenOccupied = a.seenOccupied) (hr : a.restart.isSome → b.restart.isSome) :
    MonoStep a b :=
  ⟨hr, fun q h => hc ▸ h, hcirc ▸ Finset.Subset.refl _, fun q r h => hse ▸ h, fun q r h => hso ▸ h⟩

theorem storedValue_mono (p : Pos) : Preserves MR (storedValue (Value := Value) p) :=
  Preserves.getBind fun st => (monoRel Pos Value).refl st

omit [ValueSort Value] in
theorem recordSeen_mono (q : Pos) (s : Seen) : Preserves MR (recordSeen (Value := Value) q s) :=
  Preserves.getBind fun st => by
    split
    · exact (monoRel Pos Value).refl st
    · rename_i r _
      split
      · split
        · exact (monoRel Pos Value).refl st
        · rename_i hnone
          refine ⟨id, fun _ h => h, Finset.Subset.refl _, fun x r' hx => ?_, fun _ _ h => h⟩
          by_cases hxq : x = q
          · subst hxq
            rw [hnone] at hx
            cases hx
          · show Function.update st.seenEmpty q (some r) x = some r'
            rw [Function.update_of_ne hxq]
            exact hx
      · split
        · exact (monoRel Pos Value).refl st
        · rename_i hnone
          refine ⟨id, fun _ h => h, Finset.Subset.refl _, fun _ _ h => h, fun x r' hx => ?_⟩
          by_cases hxq : x = q
          · subst hxq
            rw [hnone] at hx
            cases hx
          · show Function.update st.seenOccupied q (some r) x = some r'
            rw [Function.update_of_ne hxq]
            exact hx

omit [ValueSort Value] in
theorem markCycle_mono (o : Pos) : Preserves MR (markCycle (Value := Value) o) :=
  Preserves.modifyM fun st => by
    split
    · exact ⟨id, fun _ h => h, Finset.subset_union_left, fun _ _ h => h, fun _ _ h => h⟩
    · exact (monoRel Pos Value).refl st

omit [ValueSort Value] in
theorem spillContradictsARead_mono (anchor : Pos) (writes clears : List Pos) :
    Preserves MR (spillContradictsARead (Value := Value) anchor writes clears) :=
  Preserves.getBind fun st => by
    dsimp only
    split
    · exact (monoRel Pos Value).refl st
    · exact MonoStep.of_same rfl rfl rfl rfl fun _ => rfl

theorem storeScalar_mono (U : Universe Pos) (p : Pos) (t : Formula Pos Value) (v : Value) :
    Preserves MR (storeScalar U p t v) :=
  Preserves.getBind fun st => by
    dsimp only
    refine Preserves.bindM (spillContradictsARead_mono p [] _) (fun b =>
      Preserves.iteM (Preserves.pureM _) (Preserves.modifyM ?_)) st
    exact fun _ => MonoStep.of_same rfl rfl rfl rfl id

omit [ValueSort Value] in
theorem recordBlockers_mono (st : PassState Pos Value) (p : Pos) :
    ∀ l : List Pos, Preserves MR (recordBlockers st p l)
  | [] => Preserves.pureM _
  | q :: l => by
      simp only [recordBlockers]
      refine Preserves.bindM ?_ fun _ => recordBlockers_mono st p l
      split
      · exact Preserves.iteM (recordSeen_mono q .occupied) (Preserves.pureM _)
      · exact Preserves.pureM _

theorem spillDynamicArray_mono (U : Universe Pos) (p : Pos) (t : Formula Pos Value)
    (area : List Pos) (vals : Pos → Value) :
    Preserves MR (spillDynamicArray U p t area vals) :=
  Preserves.getBind fun st => by
    dsimp only
    refine Preserves.bindM (recordBlockers_mono st p _) (fun _ =>
      Preserves.iteM (Preserves.bindM (storeScalar_mono U p t _) fun _ => Preserves.pureM _)
        (Preserves.bindM (spillContradictsARead_mono p _ _) fun b =>
          Preserves.iteM (Preserves.pureM _) (Preserves.modifyM ?mod))) st
    case mod => exact fun _ => MonoStep.of_same rfl rfl rfl rfl id

theorem commit_mono (U : Universe Pos) (p : Pos) (r : Result Pos Value) :
    Preserves MR (commit U p r) :=
  Preserves.getBind fun st => by
    split
    · exact MonoStep.of_same rfl rfl rfl rfl id
    · exact MonoStep.of_same rfl rfl rfl rfl id
    · split
      · exact storeScalar_mono U p _ _ st
      · exact spillDynamicArray_mono U p _ _ _ st
    · exact (monoRel Pos Value).refl st

end mono

/-! ## Abandoned states -/

section abandoned
variable (U : Universe Pos)

/-- Once the pass is abandoned, a read does nothing. -/
theorem evalCell_abandoned :
    ∀ (f : Nat) (q : Pos) (s : PassState Pos Value), s.restart.isSome →
      (evalCell U f q).run s = (emptyValue, s)
  | 0, _, _, _ => rfl
  | f + 1, q, s, h => by
      simp only [evalCell, StateM.run_getBind, h, ↓reduceIte, StateT.run_pure]
      rfl

omit [DecidableEq Pos] in
/-- Nor does a formula run. -/
theorem Formula.run_abandoned {rec : Pos → PassM Pos Value Value}
    (hab : ∀ q s, s.restart.isSome → (rec q).run s = (emptyValue, s)) :
    ∀ (t : Formula Pos Value) (s : PassState Pos Value), s.restart.isSome →
      ∃ r, (t.run rec).run s = (r, s)
  | .done r, s, _ => ⟨r, rfl⟩
  | .read q k, s, h => by
      rw [Formula.run, StateT.run_bind, hab q s h, Id.bind_apply]
      exact Formula.run_abandoned hab (k emptyValue) s h

/-- Nor does the tail of a formula cell. -/
theorem finishFormulaCell_abandoned (p : Pos) (r : Result Pos Value) (s : PassState Pos Value)
    (h : s.restart.isSome) : (finishFormulaCell U p r).run s = (emptyValue, s) := by
  have hn : s.restart.isNone = false := by
    cases hr : s.restart <;> simp_all
  simp only [finishFormulaCell, StateT.run_bind, StateT.run_get, Id.pure_apply, Id.bind_apply, hn,
    Bool.false_eq_true, ↓reduceIte, StateT.run_pure, h]

end abandoned

/-! ## Pushing a cell -/

/-- The state after `p` is put in `Evaluating` and pushed. -/
def pushState (st : PassState Pos Value) (p : Pos) : PassState Pos Value :=
  { st with cells := Function.update st.cells p (some .evaluating), stack := p :: st.stack }

theorem push_step {S₀ : Sheet Pos Value} {st : PassState Pos Value} (hinv : PassInv S₀ st)
    {p : Pos} (hp : st.cells p = none) :
    PassStep S₀ st (pushState st p) ∧ PassInv S₀ (pushState st p) := by
  have hnot : p ∉ st.stack := by
    intro h
    rw [hinv.stack_evaluating] at h
    rw [hp] at h
    cases h
  have hcells : ∀ a, Function.update st.cells p (some .evaluating) a = some .evaluated ↔
      st.cells a = some .evaluated := by
    intro a
    by_cases hap : a = p
    · subst hap
      simp [hp]
    · simp [Function.update_of_ne hap]
  have hview : passView (pushState st p) = passView st := by
    funext q
    simp only [passView, pushState]
    cases hc : st.sheet q <;> simp only []
    case spill a v => simp only [hcells a]
  have hprot : ∀ q, Protected (pushState st p) q ↔ Protected st q := by
    intro q
    simp only [Protected, pushState, hcells]
  have hsb : StableBlocked (pushState st p) = StableBlocked st := by
    funext q area
    simp [StableBlocked, pushState]
  have hinv' : PassInv S₀ (pushState st p) :=
    { not_abandoned := hinv.not_abandoned
      orig_spill := fun q a v hq hne => hinv.orig_spill q a v hq fun h => hne ((hcells a).mpr h)
      shape := hinv.shape
      no_orphans := hinv.no_orphans
      cse_areas := hinv.cse_areas
      cse_spills := hinv.cse_spills
      stack_nodup := List.nodup_cons.mpr ⟨hnot, hinv.stack_nodup⟩
      stack_evaluating := by
        intro q
        by_cases hq : q = p
        · subst hq
          simp [pushState]
        · simp only [pushState, List.mem_cons, hq, false_or, Function.update_of_ne hq]
          exact hinv.stack_evaluating q
      seen_empty := by
        intro q r hq
        rcases hinv.seen_empty q r hq with h | ⟨a, w, h1, h2, h3⟩
        · exact Or.inl h
        · right
          have hap : a ≠ p := by
            rintro rfl
            rw [hp] at h3
            cases h3
          exact ⟨a, w, h1, h2, by simp [pushState, Function.update_of_ne hap, h3]⟩
      seen_occupied := hinv.seen_occupied
      evaluated_consistent := by
        intro q hq
        rw [hview, hsb]
        exact hinv.evaluated_consistent q ((hcells q).mp hq)
      reads_protected := by
        intro q t hq ht hne x hx
        rw [hview] at hx
        exact (hprot x).mpr (hinv.reads_protected q t ((hcells q).mp hq) ht hne x hx) }
  refine ⟨⟨id, fun q h => (hcells q).mpr h, Finset.Subset.refl _, fun _ _ h => h, fun _ _ h => h,
    fun _ => Or.inr hinv', fun _ _ q hq => ⟨(hprot q).mpr hq, by rw [hview]⟩⟩, hinv'⟩

/-! ## Kinds are stable -/

omit [DecidableEq Pos] [ValueSort Value] in
/-- Two sheets of the same shape as `S₀` agree on the kind and formula of
every formula cell and anchor. -/
theorem SameShape.kind_stable {S₀ S₁ S₂ : Sheet Pos Value} (h₁ : SameShape S₀ S₁)
    (h₂ : SameShape S₀ S₂) (a : Pos) : SameKind (S₁ a) (S₂ a) ∨ (S₁ a).formula? = none := by
  have g₁ := h₁ a
  have g₂ := h₂ a
  revert g₁ g₂
  cases S₀ a with
  | const v =>
      intro g₁ g₂
      rw [g₁]
      simp [Content.formula?]
  | formula t _ =>
      rintro ⟨v₁, hv₁⟩ ⟨v₂, hv₂⟩
      rw [hv₁, hv₂]
      exact Or.inl ⟨v₂, rfl⟩
  | cseAnchor t area _ =>
      rintro ⟨v₁, hv₁⟩ ⟨v₂, hv₂⟩
      rw [hv₁, hv₂]
      exact Or.inl ⟨v₂, rfl⟩
  | dynAnchor t _ =>
      rintro ⟨v₁, hv₁⟩ ⟨v₂, hv₂⟩
      rw [hv₁, hv₂]
      exact Or.inl ⟨v₂, rfl⟩
  | empty =>
      intro g₁ _
      right
      cases hs : S₁ a <;> simp_all [Content.isEmpty, Content.spillAnchor?, Content.formula?]
  | spill _ _ =>
      intro g₁ _
      right
      cases hs : S₁ a <;> simp_all [Content.isEmpty, Content.spillAnchor?, Content.formula?]

omit [DecidableEq Pos] [ValueSort Value] in
theorem SameShape.cse_stable {S₀ S₁ S₂ : Sheet Pos Value} (h₁ : SameShape S₀ S₁)
    (h₂ : SameShape S₀ S₂) {a : Pos} {t : Formula Pos Value} {area : List Pos} {v : Value}
    (ha : S₁ a = .cseAnchor t area v) : ∃ v', S₂ a = .cseAnchor t area v' := by
  rcases SameShape.kind_stable h₁ h₂ a with h | h
  · rw [ha] at h
    exact h
  · rw [ha] at h
    simp [Content.formula?] at h

omit [ValueSort Value] in
/-- `recordSeen q .empty` at one state. -/
theorem recordSeen_empty_run (q : Pos) (s : PassState Pos Value) :
    ∃ s', (recordSeen (Value := Value) q .empty).run s = ((), s') ∧
      s'.sheet = s.sheet ∧ s'.cells = s.cells ∧ s'.stack = s.stack ∧ s'.circular = s.circular ∧
      s'.seenOccupied = s.seenOccupied ∧ s'.restart = s.restart ∧
      (s.root.isSome → (s'.seenEmpty q).isSome) := by
  simp only [recordSeen, StateM.run_getBind]
  split
  · rename_i hroot
    exact ⟨s, rfl, rfl, rfl, rfl, rfl, rfl, rfl, fun h => by simp [hroot] at h⟩
  · rename_i r hroot
    try dsimp only
    split
    · rename_i hse
      exact ⟨s, rfl, rfl, rfl, rfl, rfl, rfl, rfl, fun _ => by simp [hse]⟩
    · exact ⟨{ s with seenEmpty := Function.update s.seenEmpty q (some r) }, rfl, rfl, rfl, rfl, rfl,
        rfl, rfl, fun _ => by simp⟩

/-! ## Cycles -/

omit [ValueSort Value] in
/-- Marking a cycle marks the reader: the top of the stack lies between the
cell met and the top. -/
theorem markCycle_marks_head (st : PassState Pos Value) {origin c : Pos} (ho : origin ∈ st.stack)
    (hc : st.stack.head? = some c) : c ∈ (((markCycle (Value := Value) origin).run st).2).circular := by
  simp only [markCycle, StateT.run_modify, ho, ↓reduceIte]
  apply Finset.mem_union_right
  rw [Finset.mem_insert]
  by_cases hco : c = origin
  · exact Or.inl hco
  · right
    rw [List.mem_toFinset]
    obtain ⟨tl, htl⟩ := List.head?_eq_some_iff.mp hc
    rw [htl, List.takeWhile_cons]
    simp [hco]

/-! ## Fuel -/

/-- The fuel a read of `q` needs: two per cell that can still go on the stack,
plus one more if `q` is a spill cell, which forwards to its anchor first. -/
def FuelOk (U : Universe Pos) (st : PassState Pos Value) (q : Pos) (f : Nat) : Prop :=
  2 * (U.positions.length - st.stack.length) +
    (if (st.sheet q).spillAnchor?.isSome then 2 else 1) ≤ f

theorem stack_length_le {S₀ : Sheet Pos Value} (U : Universe Pos) {st : PassState Pos Value}
    (hinv : PassInv S₀ st) : st.stack.length ≤ U.positions.length :=
  (List.subperm_of_subset hinv.stack_nodup fun x _ => U.complete x).length_le

/-! ## The specifications -/

/-- What is known of a restart, relative to the state a step started from:
it names a dynamic anchor that had not been evaluated, and if it drops stale
cells, one of them is an original spill cell of that anchor. -/
def RestartOk (S₀ : Sheet Pos Value) (st : PassState Pos Value) (r : Restart Pos) : Prop :=
  (st.sheet r.anchor).isDynAnchor = true ∧ st.cells r.anchor ≠ some .evaluated ∧ r.StaleOk S₀

/-- `RestartOk` moves back along a step. -/
theorem RestartOk.transport {S₀ : Sheet Pos Value} {st st' : PassState Pos Value}
    (hinv : PassInv S₀ st) (hinv' : PassInv S₀ st') (hstep : PassStep S₀ st st') {r : Restart Pos}
    (h : RestartOk S₀ st' r) : RestartOk S₀ st r := by
  obtain ⟨hdyn, hev, hstale⟩ := h
  refine ⟨?_, ?_, hstale⟩
  · rcases SameShape.kind_stable hinv'.shape hinv.shape r.anchor with hk | hk
    · rw [hk.formula_eq.2.2.1]
      exact hdyn
    · cases hs : st'.sheet r.anchor <;> simp_all [Content.isDynAnchor, Content.formula?]
  · intro h
    exact hev (hstep.cells_mono _ h)

/-- A state that is not abandoned restarts nothing. -/
theorem RestartOk.of_none {S₀ : Sheet Pos Value} {st st' : PassState Pos Value}
    (h : st'.restart = none) :
    ∀ r, st'.restart = some r → RestartOk S₀ st r := by
  intro r hr
  rw [h] at hr
  cases hr

/-- What a read of `q` does. -/
def ReadSpec (S₀ : Sheet Pos Value) (st : PassState Pos Value) (q : Pos)
    (out : Value × PassState Pos Value) : Prop :=
  PassStep S₀ st out.2 ∧
  (out.2.restart.isSome ∨
    (PassInv S₀ out.2 ∧ out.2.stack = st.stack ∧
      (∀ c, st.stack.head? = some c →
        (out.1 = passView out.2 q ∧ Protected out.2 q) ∨ c ∈ out.2.circular) ∧
      ((st.sheet q).formula?.isSome → st.cells q ≠ some .evaluating →
        out.2.cells q = some .evaluated))) ∧
  (∀ r, out.2.restart = some r → RestartOk S₀ st r)

/-- What a formula run does. -/
def RunSpec (S₀ : Sheet Pos Value) (st : PassState Pos Value) (t : Formula Pos Value)
    (out : Result Pos Value × PassState Pos Value) : Prop :=
  PassStep S₀ st out.2 ∧
  (out.2.restart.isSome ∨
    (PassInv S₀ out.2 ∧ out.2.stack = st.stack ∧
      ∀ c, st.stack.head? = some c →
        (out.1 = t.runPure (passView out.2) ∧ ∀ q ∈ t.reads (passView out.2), Protected out.2 q) ∨
          c ∈ out.2.circular)) ∧
  (∀ r, out.2.restart = some r → RestartOk S₀ st r)

/-- What is assumed of the recursive read while the stack is `stack`. -/
structure RecSpec (S₀ : Sheet Pos Value) (rec : Pos → PassM Pos Value Value) (stack : List Pos) :
    Prop where
  spec : ∀ q s, PassInv S₀ s → s.stack = stack → ReadSpec S₀ s q ((rec q).run s)
  abandoned : ∀ q s, s.restart.isSome → (rec q).run s = (emptyValue, s)


/-! ## A formula run -/

theorem Formula.run_spec {S₀ : Sheet Pos Value} {rec : Pos → PassM Pos Value Value}
    {stack : List Pos} (hrec : RecSpec S₀ rec stack) :
    ∀ (t : Formula Pos Value) (s : PassState Pos Value), PassInv S₀ s → s.stack = stack →
      RunSpec S₀ s t ((t.run rec).run s)
  | .done r, s, hinv, _ =>
      ⟨PassStep.refl S₀ s, Or.inr ⟨hinv, rfl, fun _ _ =>
        Or.inl ⟨rfl, fun q hq => absurd hq List.not_mem_nil⟩⟩, RestartOk.of_none hinv.not_abandoned⟩
  | .read q k, s, hinv, hs => by
      rw [Formula.run, StateT.run_bind]
      obtain ⟨hstep₁, h₁, hr₁⟩ := hrec.spec q s hinv hs
      set out₁ := (rec q).run s with hout₁
      rw [Id.bind_apply]
      rcases h₁ with hab | ⟨hinv₁, hstack₁, hval, _⟩
      · obtain ⟨r, hr⟩ := Formula.run_abandoned hrec.abandoned (k out₁.1) out₁.2 hab
        rw [hr]
        exact ⟨hstep₁, Or.inl hab, hr₁⟩
      · obtain ⟨hstep₂, h₂, hr₂⟩ :=
          Formula.run_spec hrec (k out₁.1) out₁.2 hinv₁ (hstack₁.trans hs)
        set out₂ := (Formula.run rec (k out₁.1)).run out₁.2 with hout₂
        refine ⟨hstep₁.trans hstep₂, ?_, fun r hr => (hr₂ r hr).transport hinv hinv₁ hstep₁⟩
        rcases h₂ with hab | ⟨hinv₂, hstack₂, hval₂⟩
        · exact Or.inl hab
        · right
          refine ⟨hinv₂, hstack₂.trans hstack₁, fun c hc => ?_⟩
          rcases hval c hc with ⟨hv, hp⟩ | hcirc
          · obtain ⟨hp₂, hview⟩ := hstep₂.protect hinv₁ hinv₂.not_abandoned q hp
            rcases hval₂ c (by rw [hstack₁]; exact hc) with ⟨hr, hreads⟩ | hcirc₂
            · left
              constructor
              · simp only [Formula.runPure]
                rw [hview, ← hv]
                exact hr
              · intro x hx
                simp only [Formula.reads, List.mem_cons] at hx
                rcases hx with rfl | hx
                · exact hp₂
                · rw [hview, ← hv] at hx
                  exact hreads x hx
            · exact Or.inr hcirc₂
          · exact Or.inr (hstep₂.circular_mono hcirc)

/-! ## A spill cell -/

/-- `evaluate_spill_cell`. -/
theorem evalSpillCell_spec {S₀ : Sheet Pos Value} (U : Universe Pos)
    {rec : Pos → PassM Pos Value Value} {f : Nat}
    (hrec : ∀ q s, PassInv S₀ s → FuelOk U s q f → ReadSpec S₀ s q ((rec q).run s))
    (st : PassState Pos Value) (hinv : PassInv S₀ st) (p a : Pos) (w : Value)
    (hp : st.sheet p = .spill a w)
    (hfuel : 2 * (U.positions.length - st.stack.length) + 1 ≤ f) :
    ReadSpec S₀ st p ((evalSpillCell rec p a).run st) := by
  simp only [evalSpillCell, StateM.run_getBind]
  have hanchor := hinv.no_orphans p a w hp
  have hpa : p ≠ a := by
    rintro rfl
    rw [hp] at hanchor
    simp [Content.isAnchor] at hanchor
  cases hsa : st.sheet a with
  | cseAnchor t area v =>
      try simp only []
      have hfa : FuelOk U st a f := by
        simp only [FuelOk, hsa, Content.spillAnchor?, Option.isSome_none, Bool.false_eq_true,
          ↓reduceIte]
        exact hfuel
      rcases hca : st.cells a with _ | (_ | _)
      · -- Not evaluated yet: evaluate the anchor, then read what it wrote.
        try simp only []
        rw [StateT.run_bind]
        obtain ⟨hstep, h, hr⟩ := hrec a st hinv hfa
        set out := (rec a).run st with hout
        rw [Id.bind_apply]
        show ReadSpec S₀ st p (valueAt out.2.sheet p, out.2)
        rcases h with hab' | ⟨hinv', hstack', hval, _⟩
        · exact ⟨hstep, Or.inl hab', hr⟩
        · refine ⟨hstep, Or.inr ⟨hinv', hstack', fun c hc => ?_,
            fun h => absurd h (by simp [hp, Content.formula?])⟩, hr⟩
          rcases hval c hc with ⟨_, hprot⟩ | hcirc
          · left
            obtain ⟨v', hv'⟩ := SameShape.cse_stable hinv.shape hinv'.shape hsa
            have hev : out.2.cells a = some .evaluated := by
              rcases hprot with ⟨x, hx⟩ | ⟨_, h⟩ | ⟨a', w', hs, _⟩ | hse
              · rw [hx] at hv'
                cases hv'
              · exact h
              · rw [hs] at hv'
                cases hv'
              · obtain ⟨r, hr⟩ := Option.isSome_iff_exists.mp hse
                rcases hinv'.seen_empty a r hr with he | ⟨_, _, hs, _⟩
                · rw [hv'] at he
                  simp [Content.isEmpty] at he
                · rw [hs] at hv'
                  cases hv'
            have hpa' : p ∈ area := hinv.cse_spills p a w hp t area v hsa
            obtain ⟨w', hw'⟩ := hinv'.cse_areas a t area v' hv' p hpa' hpa
            refine ⟨?_, Or.inr (Or.inr (Or.inl ⟨a, w', hw', hev⟩))⟩
            simp [passView, hw', hev]
          · exact Or.inr hcirc
      · -- Running: the read closes a cycle at the anchor.
        try simp only []
        obtain ⟨hstep, h, hr⟩ := hrec a st hinv hfa
        refine ⟨hstep, ?_, hr⟩
        rcases h with hab' | ⟨hinv', hstack', hval, _⟩
        · exact Or.inl hab'
        · right
          refine ⟨hinv', hstack', fun c hc => ?_, fun h => absurd h (by simp [hp, Content.formula?])⟩
          rcases hval c hc with ⟨_, hprot⟩ | hcirc
          · exfalso
            have ha_stack : a ∈ st.stack := (hinv.stack_evaluating a).mpr hca
            have ha_ev : ((rec a).run st).2.cells a = some .evaluating :=
              (hinv'.stack_evaluating a).mp (hstack' ▸ ha_stack)
            obtain ⟨v', hv'⟩ := SameShape.cse_stable hinv.shape hinv'.shape hsa
            exact not_protected_of_evaluating hinv' (by simp [hv', Content.formula?]) ha_ev hprot
          · exact Or.inr hcirc
      · -- Evaluated: what the anchor wrote.
        try simp only []
        refine ⟨PassStep.refl S₀ st, Or.inr ⟨hinv, rfl, fun _ _ => Or.inl ⟨?_, ?_⟩,
          fun h => absurd h (by simp [hp, Content.formula?])⟩, RestartOk.of_none hinv.not_abandoned⟩
        · show valueAt st.sheet p = passView st p
          simp [passView, hp, hca]
        · exact Or.inr (Or.inr (Or.inl ⟨a, w, hp, hca⟩))
  | dynAnchor t v =>
      try simp only []
      rcases hca : st.cells a with _ | (_ | _)
      · -- Left over from a previous evaluation: restart with the anchor first.
        try simp only []
        refine ⟨PassStep.of_restart_only rfl rfl rfl rfl rfl rfl (Or.inr rfl), Or.inl rfl, ?_⟩
        intro r hr
        cases hr
        exact ⟨by simp [Restart.anchor, hsa, Content.isDynAnchor], by simp [Restart.anchor, hca],
          trivial⟩
      · -- The anchor's own area, before it commits: empty, on record.
        try simp only []
        rw [StateT.run_bind]
        obtain ⟨s', hrun, g1, g2, g3, g4, g5, g6, g7⟩ := recordSeen_empty_run p st
        have hstep : PassStep S₀ st s' := by
          have := recordSeen_empty_step S₀ p st
            (fun _ => Or.inr ⟨a, w, hp, by simp [hsa, Content.isDynAnchor], hca⟩)
          rw [hrun] at this
          exact this
        rw [hrun, Id.bind_apply]
        show ReadSpec S₀ st p (emptyValue, s')
        have hinv' : PassInv S₀ s' := by
          rcases hstep.inv hinv with h | h
          · rw [g6, hinv.not_abandoned] at h
            cases h
          · exact h
        refine ⟨hstep, Or.inr ⟨hinv', g3, fun c hc => Or.inl ⟨?_, ?_⟩,
          fun h => absurd h (by simp [hp, Content.formula?])⟩, RestartOk.of_none hinv'.not_abandoned⟩
        · simp [passView, g1, g2, hp, hca]
        · refine Or.inr (Or.inr (Or.inr (g7 ?_)))
          unfold PassState.root
          rw [(List.cons_head?_tail hc).symm]
          simp
      · -- Written in this pass.
        try simp only []
        refine ⟨PassStep.refl S₀ st, Or.inr ⟨hinv, rfl, fun _ _ => Or.inl ⟨?_, ?_⟩,
          fun h => absurd h (by simp [hp, Content.formula?])⟩, RestartOk.of_none hinv.not_abandoned⟩
        · show valueAt st.sheet p = passView st p
          simp [passView, hp, hca]
        · exact Or.inr (Or.inr (Or.inl ⟨a, w, hp, hca⟩))
  | empty => simp [hsa, Content.isAnchor] at hanchor
  | const _ => simp [hsa, Content.isAnchor] at hanchor
  | formula _ _ => simp [hsa, Content.isAnchor] at hanchor
  | spill _ _ => simp [hsa, Content.isAnchor] at hanchor

/-! ## A formula cell -/

/-- A position holding a formula cell or anchor reads as its stored value. -/
theorem passView_of_formula (st : PassState Pos Value) {q : Pos} (h : (st.sheet q).formula?.isSome) :
    passView st q = valueAt st.sheet q := by
  unfold passView
  cases hs : st.sheet q <;> simp_all [Content.formula?]

omit [ValueSort Value] in
theorem markCycle_run_fields (o : Pos) (st : PassState Pos Value) :
    (((markCycle (Value := Value) o).run st).2).restart = st.restart ∧
      (((markCycle (Value := Value) o).run st).2).stack = st.stack := by
  simp only [markCycle, StateT.run_modify]
  split <;> exact ⟨rfl, rfl⟩

/-- The tail of `evaluate_formula_cell`: commit, mark, return what was stored. -/
theorem finishFormulaCell_spec {S₀ : Sheet Pos Value} (U : Universe Pos) (st : PassState Pos Value)
    (hinv : PassInv S₀ st) (p : Pos) (htop : st.stack.head? = some p) (t : Formula Pos Value)
    (ht : (st.sheet p).formula? = some t) (r : Result Pos Value)
    (hr : (r = t.runPure (passView st) ∧ ∀ q ∈ t.reads (passView st), Protected st q) ∨
      p ∈ st.circular) :
    PassStep S₀ st ((finishFormulaCell U p r).run st).2 ∧
      ((((finishFormulaCell U p r).run st).2).restart.isSome ∨
        (PassInv S₀ ((finishFormulaCell U p r).run st).2 ∧
          (((finishFormulaCell U p r).run st).2).stack = st.stack.tail ∧
          (((finishFormulaCell U p r).run st).2).cells p = some .evaluated ∧
          ((finishFormulaCell U p r).run st).1 =
            valueAt (((finishFormulaCell U p r).run st).2).sheet p)) ∧
      (∀ r', (((finishFormulaCell U p r).run st).2).restart = some r' → RestartOk S₀ st r') := by
  have hnone : st.restart.isNone = true := by simp [hinv.not_abandoned]
  simp only [finishFormulaCell, StateM.run_getBind, hnone, ↓reduceIte]
  try dsimp only
  set r' := if p ∈ st.circular then Result.scalar circ else r with hr'
  have hres : ResultOf st p r' := by
    by_cases hm : p ∈ st.circular
    · left
      simp [hr', hm]
    · rcases hr with ⟨h1, h2⟩ | h
      · right
        exact ⟨t, ht, by simp [hr', hm, h1], h2⟩
      · exact absurd h hm
  rw [StateT.run_bind]
  rcases commit_spec S₀ U st p r' hinv htop hres (by simp [ht])
    with ⟨r'', hab, hrp, hdyn, hstale⟩ | hok
  · have hab' : (((commit U p r').run st).2).restart.isSome := by simp [hab]
    rw [Id.bind_apply, StateM.run_getBind]
    simp only [hab', ↓reduceIte, StateT.run_pure, Id.pure_apply]
    refine ⟨PassStep.of_abandoned (commit_mono U p r' st) hab', Or.inl (by simpa using hab'), ?_⟩
    intro r₃ hr₃
    rw [hab] at hr₃
    cases hr₃
    unfold RestartOk
    rw [hrp]
    exact ⟨hdyn, by rw [cells_of_head hinv htop]; simp, hstale⟩
  · rw [Id.bind_apply, StateM.run_getBind]
    simp only [hok.not_abandoned, Option.isSome_none, Bool.false_eq_true, ↓reduceIte]
    rw [StateT.run_bind, StateT.run_modify, Id.pure_apply, Id.bind_apply]
    show PassStep S₀ st (markEvaluated ((commit U p r').run st).2 p) ∧ _
    refine ⟨hok.step, Or.inr ⟨hok.inv, ?_, ?_, rfl⟩, ?_⟩
    · show (((commit U p r').run st).2).stack.tail = st.stack.tail
      rw [hok.stack]
    · exact Function.update_self _ _ _
    · exact RestartOk.of_none hok.not_abandoned

/-- `evaluate_formula_cell`. -/
theorem evalFormulaCell_spec {S₀ : Sheet Pos Value} (U : Universe Pos)
    {rec : Pos → PassM Pos Value Value} {f : Nat}
    (hrec : ∀ q s, PassInv S₀ s → FuelOk U s q f → ReadSpec S₀ s q ((rec q).run s))
    (hab : ∀ q s, s.restart.isSome → (rec q).run s = (emptyValue, s))
    (st : PassState Pos Value) (hinv : PassInv S₀ st) (p : Pos) (t : Formula Pos Value)
    (ht : (st.sheet p).formula? = some t)
    (hfuel : 2 * (U.positions.length - st.stack.length) ≤ f) :
    ReadSpec S₀ st p ((evalFormulaCell U rec p t).run st) := by
  simp only [evalFormulaCell, StateM.run_getBind]
  rcases hcp : st.cells p with _ | (_ | _)
  · -- Not evaluated yet.
    try simp only []
    obtain ⟨hstep₁, hinv₁⟩ := push_step hinv hcp
    rw [StateT.run_bind, StateT.run_modify, Id.pure_apply, Id.bind_apply, StateM.run_getBind]
    show ReadSpec S₀ st p (((if p ∈ (pushState st p).circular then pure (Result.scalar circ)
      else t.run rec : PassM Pos Value (Result Pos Value)) >>= fun r => finishFormulaCell U p r).run
        (pushState st p))
    set st₁ := pushState st p with hst₁
    have hstack₁ : st₁.stack = p :: st.stack := rfl
    have hlen : st.stack.length + 1 ≤ U.positions.length := by
      have := stack_length_le U hinv₁
      rw [hstack₁] at this
      simpa using this
    have hkind₁ := SameShape.kind_stable hinv.shape hinv₁.shape p
    -- The reads of the formula happen with `p` on top of the stack.
    have hrecspec : RecSpec S₀ rec st₁.stack :=
      { spec := fun q s hs hstack => hrec q s hs (by
          unfold FuelOk
          rw [hstack, hstack₁]
          simp only [List.length_cons]
          split <;> omega)
        abandoned := hab }
    -- Once the formula has run, finish.
    have hfinish : ∀ (out : Result Pos Value × PassState Pos Value),
        PassStep S₀ st₁ out.2 →
        (out.2.restart.isSome ∨
          (PassInv S₀ out.2 ∧ out.2.stack = st₁.stack ∧
            ((out.1 = t.runPure (passView out.2) ∧ ∀ q ∈ t.reads (passView out.2), Protected out.2 q) ∨
              p ∈ out.2.circular))) →
        (∀ r, out.2.restart = some r → RestartOk S₀ st₁ r) →
        ReadSpec S₀ st p ((finishFormulaCell U p out.1).run out.2) := by
      intro out hstep₂ h₂ hr₂
      rcases h₂ with hab₂ | ⟨hinv₂, hstack₂, hval₂⟩
      · rw [finishFormulaCell_abandoned U p out.1 out.2 hab₂]
        exact ⟨hstep₁.trans hstep₂, Or.inl hab₂, fun r hr => (hr₂ r hr).transport hinv hinv₁ hstep₁⟩
      · have htop₂ : out.2.stack.head? = some p := by rw [hstack₂, hstack₁]; rfl
        have ht₂ : (out.2.sheet p).formula? = some t := by
          rcases SameShape.kind_stable hinv.shape hinv₂.shape p with h | h
          · rw [h.formula_eq.1, ht]
          · rw [h] at ht
            cases ht
        obtain ⟨hstep₃, h₃, hr₃⟩ := finishFormulaCell_spec U out.2 hinv₂ p htop₂ t ht₂ out.1 hval₂
        refine ⟨hstep₁.trans (hstep₂.trans hstep₃), ?_, fun r hr =>
          ((hr₃ r hr).transport hinv₂ hinv₂ (PassStep.refl S₀ _)).transport hinv hinv₂
            (hstep₁.trans hstep₂)⟩
        rcases h₃ with hab₃ | ⟨hinv₃, hstack₃, hcells₃, hv₃⟩
        · exact Or.inl hab₃
        · right
          refine ⟨hinv₃, by rw [hstack₃, hstack₂, hstack₁]; rfl, fun c _ => Or.inl ⟨?_, ?_⟩,
            fun _ _ => hcells₃⟩
          · have ht₃ : ((((finishFormulaCell U p out.1).run out.2).2).sheet p).formula? = some t := by
              rcases SameShape.kind_stable hinv.shape hinv₃.shape p with h | h
              · rw [h.formula_eq.1, ht]
              · rw [h] at ht
                cases ht
            rw [hv₃, passView_of_formula _ (by simp [ht₃])]
          · have ht₃ : ((((finishFormulaCell U p out.1).run out.2).2).sheet p).formula? = some t := by
              rcases SameShape.kind_stable hinv.shape hinv₃.shape p with h | h
              · rw [h.formula_eq.1, ht]
              · rw [h] at ht
                cases ht
            exact Or.inr (Or.inl ⟨by simp [ht₃], hcells₃⟩)
    by_cases hmark : p ∈ st₁.circular
    · simp only [hmark, ↓reduceIte, StateT.run_bind, StateT.run_pure, Id.bind_apply]
      exact hfinish (Result.scalar circ, st₁) (PassStep.refl S₀ st₁)
        (Or.inr ⟨hinv₁, rfl, Or.inr hmark⟩) (RestartOk.of_none hinv₁.not_abandoned)
    · simp only [hmark, ↓reduceIte, StateT.run_bind]
      obtain ⟨hstep₂, h₂, hr₂⟩ := Formula.run_spec hrecspec t st₁ hinv₁ rfl
      rw [Id.bind_apply]
      refine hfinish ((t.run rec).run st₁) hstep₂ ?_ hr₂
      rcases h₂ with hab₂ | ⟨hinv₂, hstack₂, hval₂⟩
      · exact Or.inl hab₂
      · exact Or.inr ⟨hinv₂, hstack₂, hval₂ p (by rw [hstack₁]; rfl)⟩
  · -- Running: the read closes a cycle.
    try simp only []
    rw [StateT.run_bind]
    have hstep := markCycle_step S₀ p st
    obtain ⟨hrest, hstack⟩ := markCycle_run_fields p st
    set st' := ((markCycle p).run st).2 with hst'
    rw [Id.bind_apply]
    show ReadSpec S₀ st p (circ, st')
    have hinv' : PassInv S₀ st' := by
      rcases hstep.inv hinv with h | h
      · rw [hrest, hinv.not_abandoned] at h
        cases h
      · exact h
    refine ⟨hstep, Or.inr ⟨hinv', hstack, fun c hc => Or.inr ?_, fun _ hne => absurd hcp hne⟩,
      RestartOk.of_none hinv'.not_abandoned⟩
    exact markCycle_marks_head st ((hinv.stack_evaluating p).mpr hcp) hc
  · -- Evaluated: the stored value.
    show ReadSpec S₀ st p (valueAt st.sheet p, st)
    refine ⟨PassStep.refl S₀ st, Or.inr ⟨hinv, rfl, fun _ _ => Or.inl ⟨?_, ?_⟩, fun _ _ => hcp⟩,
      RestartOk.of_none hinv.not_abandoned⟩
    · show valueAt st.sheet p = passView st p
      rw [passView_of_formula st (q := p) (by simp [ht])]
    · exact Or.inr (Or.inl ⟨by simp [ht], hcp⟩)

/-! ## A read -/

/-- `evaluate_cell`, by induction on the fuel. -/
theorem evalCell_spec {S₀ : Sheet Pos Value} (U : Universe Pos) :
    ∀ (f : Nat) (q : Pos) (st : PassState Pos Value), PassInv S₀ st → FuelOk U st q f →
      ReadSpec S₀ st q ((evalCell U f q).run st)
  | 0, q, st, _, hfuel => by
      exfalso
      unfold FuelOk at hfuel
      split at hfuel <;> omega
  | f + 1, q, st, hinv, hfuel => by
      simp only [evalCell, StateM.run_getBind, hinv.not_abandoned, Option.isSome_none,
        Bool.false_eq_true, ↓reduceIte]
      cases hsq : st.sheet q with
      | empty =>
          try simp only []
          rw [StateT.run_bind]
          obtain ⟨s', hrun, g1, g2, g3, g4, g5, g6, g7⟩ := recordSeen_empty_run q st
          have hstep : PassStep S₀ st s' := by
            have := recordSeen_empty_step S₀ q st (fun _ => Or.inl (by simp [hsq, Content.isEmpty]))
            rw [hrun] at this
            exact this
          rw [hrun, Id.bind_apply]
          show ReadSpec S₀ st q (emptyValue, s')
          have hinv' : PassInv S₀ s' := by
            rcases hstep.inv hinv with h | h
            · rw [g6, hinv.not_abandoned] at h
              cases h
            · exact h
          refine ⟨hstep, Or.inr ⟨hinv', g3, fun c hc => Or.inl ⟨?_, ?_⟩,
            fun h => absurd h (by simp [hsq, Content.formula?])⟩, RestartOk.of_none hinv'.not_abandoned⟩
          · simp [passView, g1, hsq, valueAt]
          · refine Or.inr (Or.inr (Or.inr (g7 ?_)))
            unfold PassState.root
            rw [(List.cons_head?_tail hc).symm]
            simp
      | const v =>
          show ReadSpec S₀ st q (v, st)
          refine ⟨PassStep.refl S₀ st, Or.inr ⟨hinv, rfl, fun _ _ => Or.inl ⟨?_, Or.inl ⟨v, hsq⟩⟩,
            fun h => absurd h (by simp [hsq, Content.formula?])⟩, RestartOk.of_none hinv.not_abandoned⟩
          simp [passView, hsq, valueAt]
      | spill a w =>
          try simp only []
          refine evalSpillCell_spec U (evalCell_spec U f) st hinv q a w hsq ?_
          unfold FuelOk at hfuel
          rw [hsq] at hfuel
          simp only [Content.spillAnchor?, Option.isSome_some, ↓reduceIte] at hfuel
          omega
      | formula t v =>
          try simp only []
          refine evalFormulaCell_spec U (evalCell_spec U f) (evalCell_abandoned U f) st hinv q t
            (by simp [hsq, Content.formula?]) ?_
          unfold FuelOk at hfuel
          rw [hsq] at hfuel
          simp only [Content.spillAnchor?, Option.isSome_none, Bool.false_eq_true, ↓reduceIte] at hfuel
          omega
      | cseAnchor t area v =>
          try simp only []
          refine evalFormulaCell_spec U (evalCell_spec U f) (evalCell_abandoned U f) st hinv q t
            (by simp [hsq, Content.formula?]) ?_
          unfold FuelOk at hfuel
          rw [hsq] at hfuel
          simp only [Content.spillAnchor?, Option.isSome_none, Bool.false_eq_true, ↓reduceIte] at hfuel
          omega
      | dynAnchor t v =>
          try simp only []
          refine evalFormulaCell_spec U (evalCell_spec U f) (evalCell_abandoned U f) st hinv q t
            (by simp [hsq, Content.formula?]) ?_
          unfold FuelOk at hfuel
          rw [hsq] at hfuel
          simp only [Content.spillAnchor?, Option.isSome_none, Bool.false_eq_true, ↓reduceIte] at hfuel
          omega

end IronCalcEval

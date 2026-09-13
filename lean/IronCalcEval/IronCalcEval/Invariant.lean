import IronCalcEval.StateLemmas

/-!
# The invariant of a pass

`cold-evaluation.md`, section 5.1, as a predicate on `PassState` that holds
after every step of a pass that has not been abandoned, together with the
relation `PassRel` that every step of a pass satisfies between its start
and end states. `PassRel` is what `Preserves` carries through `bind`; the
invariant alone is not enough, because a computation in the middle of a
formula's run must not disturb what earlier reads of that formula saw.

The three invariants of §5.1 appear as follows.

* Exact records: `seen_empty` and `seen_occupied`.
* No stale value is ever read: every evaluated cell is consistent with the
  pass's view of the current sheet (`evaluated_consistent`), and every
  position it read is `Protected`: no later step changes its view
  (`reads_protected`).
* A completed pass contradicts none of its records: this is not a state
  invariant but the effect of `spillContradictsARead`, used in the proof of
  every commit.
-/

namespace IronCalcEval

variable {Pos Value : Type} [DecidableEq Pos] [ValueSort Value]

/-- Constants, formula cells and anchors keep their kind and their formula
during a pass; only stored values change, and positions that are empty or
spill cells stay in that class. -/
def SameShape (S₀ S : Sheet Pos Value) : Prop :=
  ∀ p, match S₀ p with
  | .const v => S p = .const v
  | .formula t _ => ∃ v, S p = .formula t v
  | .cseAnchor t area _ => ∃ v, S p = .cseAnchor t area v
  | .dynAnchor t _ => ∃ v, S p = .dynAnchor t v
  | .empty => (S p).isEmpty = true ∨ (S p).spillAnchor?.isSome
  | .spill _ _ => (S p).isEmpty = true ∨ (S p).spillAnchor?.isSome

/-- How the pass reads a position: like the sheet, except that a spill cell
of an anchor that has not committed in this pass counts as empty (its value
is a leftover, and `evaluate_spill_cell` never returns it). At the end of a
completed pass every anchor has committed and this is `valueAt`. -/
def passView (st : PassState Pos Value) (q : Pos) : Value :=
  match st.sheet q with
  | .spill a _ => if st.cells a = some .evaluated then valueAt st.sheet q else emptyValue
  | _ => valueAt st.sheet q

/-- A position whose pass view no later step of the pass changes: a
constant; a formula cell or anchor that has committed; a spill cell of an
anchor that has committed; or a position on record as read empty, which no
commit may write without abandoning the pass. -/
def Protected (st : PassState Pos Value) (q : Pos) : Prop :=
  (∃ v, st.sheet q = .const v) ∨
  ((st.sheet q).formula?.isSome ∧ st.cells q = some .evaluated) ∨
  (∃ a v, st.sheet q = .spill a v ∧ st.cells a = some .evaluated) ∨
  (st.seenEmpty q).isSome

/-- A blocker no later step of the pass removes: content that never changes
kind, or another array's spill cell whose removal is on record. -/
def StableBlocked (st : PassState Pos Value) (p : Pos) (area : List Pos) : Prop :=
  ∃ q ∈ area, q ≠ p ∧
    (((st.sheet q).isEmpty = false ∧ (st.sheet q).spillAnchor? = none) ∨
      ∃ a v, st.sheet q = .spill a v ∧ a ≠ p ∧ (st.seenOccupied q).isSome)

omit [ValueSort Value] in
theorem StableBlocked.blocked {st : PassState Pos Value} {p : Pos} {area : List Pos}
    (h : StableBlocked st p area) : blocked st.sheet p area := by
  obtain ⟨q, hq, hqp, h⟩ := h
  refine ⟨q, hq, hqp, ?_⟩
  rcases h with ⟨hne, hsp⟩ | ⟨a, v, hs, hap, _⟩
  · cases hc : st.sheet q <;> simp_all [Content.freeFor, Content.isEmpty, Content.spillAnchor?]
  · simp [Content.freeFor, hs, hap]

/-- The invariant of a pass that has not been abandoned. -/
structure PassInv (S₀ : Sheet Pos Value) (st : PassState Pos Value) : Prop where
  not_abandoned : st.restart = none
  shape : SameShape S₀ st.sheet
  no_orphans : NoOrphans st.sheet
  cse_areas : CseAreasFixed st.sheet
  cse_spills : CseSpillsInArea st.sheet
  /-- The stack holds exactly the cells in state `Evaluating`, once each. -/
  stack_nodup : st.stack.Nodup
  stack_evaluating : ∀ p, p ∈ st.stack ↔ st.cells p = some .evaluating
  /-- Exact records, empty: a position read as empty is still empty, or holds
  a leftover spill cell of an anchor that is still running (it counted as
  empty when read, and the anchor's commit will clear it or contradict). -/
  seen_empty : ∀ q r, st.seenEmpty q = some r →
    (st.sheet q).isEmpty = true ∨
      ∃ a v, st.sheet q = .spill a v ∧ (st.sheet a).isDynAnchor = true ∧
        st.cells a = some .evaluating
  /-- Exact records, occupied: a position that blocked an anchor still holds
  a spill cell. -/
  seen_occupied : ∀ q r, st.seenOccupied q = some r → ∃ a v, st.sheet q = .spill a v
  /-- A spill cell of an anchor that has not committed in this pass is a
  leftover of a previous evaluation: it is in the original sheet. Only the
  anchor writes its own spill cells, and it is marked evaluated as it does. -/
  orig_spill : ∀ q a v, st.sheet q = .spill a v → st.cells a ≠ some .evaluated →
    S₀ q = .spill a v
  /-- Every evaluated cell is consistent with the pass's view of the current
  sheet, with blockers that stay. -/
  evaluated_consistent : ∀ p, st.cells p = some .evaluated →
    ConsistentAtWith (passView st) (StableBlocked st) st.sheet p
  /-- What an evaluated cell read is protected. -/
  reads_protected : ∀ p t, st.cells p = some .evaluated → (st.sheet p).formula? = some t →
    valueAt st.sheet p ≠ circ → ∀ q ∈ t.reads (passView st), Protected st q
  /-- Once a cell is on the stack, the root (the cell the driver is
  evaluating) has been started, unless it is not a formula cell at all (a
  spill cell position, whose anchor is evaluated through it). -/
  root_cell : ∀ c, st.root = some c → st.stack ≠ [] →
    st.cells c ≠ none ∨ (st.sheet c).formula? = none

/-- What the driver guarantees of the root when a read of `p` starts: the
root is set, and if nothing is on the stack yet, the root is `p` itself or a
position that is not a formula cell. -/
def RootOk (st : PassState Pos Value) (p : Pos) : Prop :=
  st.root.isSome ∧ (st.stack = [] → ∀ c, st.root = some c → c = p ∨ (st.sheet c).formula? = none)

/-- A stale-cells restart names at least one original spill cell of its
anchor; the driver drops it, so a restart of this kind cannot repeat forever. -/
def Restart.StaleOk (S₀ : Sheet Pos Value) : Restart Pos → Prop
  | .staleCells a cells => ∃ q ∈ cells, ∃ v, S₀ q = .spill a v
  | _ => True

/-- Whether a restart teaches the driver a fact about the order. -/
def Restart.learns : Restart Pos → Bool
  | .staleRead .. => true
  | .conflict .. => true
  | _ => false

theorem PassInv.consistentAt {S₀ : Sheet Pos Value} {st : PassState Pos Value}
    (hinv : PassInv S₀ st) {p : Pos} (hp : st.cells p = some .evaluated) :
    ConsistentAt (passView st) st.sheet p :=
  ConsistentAtWith.mono (fun _ _ h => h.blocked) (hinv.evaluated_consistent p hp)

/-- What one step of a pass does to the state. -/
structure PassStep (S₀ : Sheet Pos Value) (a b : PassState Pos Value) : Prop where
  restart_mono : a.restart.isSome → b.restart.isSome
  cells_mono : ∀ q, a.cells q = some .evaluated → b.cells q = some .evaluated
  circular_mono : a.circular ⊆ b.circular
  seenEmpty_mono : ∀ q r, a.seenEmpty q = some r → b.seenEmpty q = some r
  seenOccupied_mono : ∀ q r, a.seenOccupied q = some r → b.seenOccupied q = some r
  inv : PassInv S₀ a → b.restart.isSome ∨ PassInv S₀ b
  protect : PassInv S₀ a → b.restart = none → ∀ q, Protected a q →
    Protected b q ∧ passView b q = passView a q
  /-- Only the driver sets the root. -/
  root_eq : b.root = a.root
  /-- Every new record is made on behalf of the root. -/
  roots_new_empty : ∀ q r, b.seenEmpty q = some r → a.seenEmpty q = some r ∨ a.root = some r
  roots_new_occupied : ∀ q r, b.seenOccupied q = some r →
    a.seenOccupied q = some r ∨ a.root = some r

theorem PassStep.refl (S₀ : Sheet Pos Value) (a : PassState Pos Value) : PassStep S₀ a a where
  restart_mono h := h
  cells_mono _ h := h
  circular_mono := Finset.Subset.refl _
  seenEmpty_mono _ _ h := h
  seenOccupied_mono _ _ h := h
  inv h := Or.inr h
  protect _ _ _ h := ⟨h, rfl⟩
  root_eq := rfl
  roots_new_empty _ _ h := Or.inl h
  roots_new_occupied _ _ h := Or.inl h

theorem PassStep.trans {S₀ : Sheet Pos Value} {a b c : PassState Pos Value}
    (h₁ : PassStep S₀ a b) (h₂ : PassStep S₀ b c) : PassStep S₀ a c where
  restart_mono h := h₂.restart_mono (h₁.restart_mono h)
  cells_mono q h := h₂.cells_mono q (h₁.cells_mono q h)
  circular_mono := Finset.Subset.trans h₁.circular_mono h₂.circular_mono
  seenEmpty_mono q r h := h₂.seenEmpty_mono q r (h₁.seenEmpty_mono q r h)
  seenOccupied_mono q r h := h₂.seenOccupied_mono q r (h₁.seenOccupied_mono q r h)
  inv ha := by
    rcases h₁.inv ha with hb | hb
    · exact Or.inl (h₂.restart_mono hb)
    · exact h₂.inv hb
  protect ha hc q hq := by
    have hb : b.restart = none := by
      cases hb : b.restart with
      | none => rfl
      | some r =>
        have := h₂.restart_mono (by simp [hb])
        simp [hc] at this
    have hinvb : PassInv S₀ b := by
      rcases h₁.inv ha with h | h
      · simp [hb] at h
      · exact h
    obtain ⟨hpb, hvb⟩ := h₁.protect ha hb q hq
    obtain ⟨hpc, hvc⟩ := h₂.protect hinvb hc q hpb
    exact ⟨hpc, hvc.trans hvb⟩
  root_eq := h₂.root_eq.trans h₁.root_eq
  roots_new_empty q r h := by
    rcases h₂.roots_new_empty q r h with h | h
    · exact h₁.roots_new_empty q r h
    · exact Or.inr (h₁.root_eq ▸ h)
  roots_new_occupied q r h := by
    rcases h₂.roots_new_occupied q r h with h | h
    · exact h₁.roots_new_occupied q r h
    · exact Or.inr (h₁.root_eq ▸ h)

/-- The relation every step of a pass satisfies. -/
def PassRel (S₀ : Sheet Pos Value) : StateRel (PassState Pos Value) where
  R := PassStep S₀
  refl := PassStep.refl S₀
  trans := PassStep.trans

/-- A step that only sets `restart`, or changes nothing. -/
theorem PassStep.of_restart_only {S₀ : Sheet Pos Value} {a b : PassState Pos Value}
    (hsheet : b.sheet = a.sheet) (hcells : b.cells = a.cells) (hstack : b.stack = a.stack)
    (hroot : b.root = a.root)
    (hcirc : b.circular = a.circular) (hse : b.seenEmpty = a.seenEmpty)
    (hso : b.seenOccupied = a.seenOccupied) (hr : b.restart = a.restart ∨ b.restart.isSome) :
    PassStep S₀ a b := by
  rcases hr with hr | hr
  · have : b = a := by
      cases a; cases b
      simp_all
    subst this
    exact PassStep.refl S₀ b
  · refine ⟨fun _ => hr, fun q h => hcells ▸ h, hcirc ▸ Finset.Subset.refl _,
      fun q r h => hse ▸ h, fun q r h => hso ▸ h, fun _ => Or.inl hr, fun _ hb => ?_, hroot,
      fun q r h => Or.inl (hse ▸ h), fun q r h => Or.inl (hso ▸ h)⟩
    simp [hb] at hr

/-- A step that only grows the circular set. -/
theorem PassStep.of_circular_only {S₀ : Sheet Pos Value} {a b : PassState Pos Value}
    (hsheet : b.sheet = a.sheet) (hcells : b.cells = a.cells) (hstack : b.stack = a.stack)
    (hroot : b.root = a.root)
    (hcirc : a.circular ⊆ b.circular) (hse : b.seenEmpty = a.seenEmpty)
    (hso : b.seenOccupied = a.seenOccupied) (hr : b.restart = a.restart) :
    PassStep S₀ a b := by
  have hview : passView b = passView a := by
    funext q
    simp [passView, hsheet, hcells]
  have hprot : ∀ q, Protected b q ↔ Protected a q := by
    intro q
    simp [Protected, hsheet, hcells, hse]
  refine ⟨fun h => hr ▸ h, fun q h => hcells ▸ h, hcirc, fun q r h => hse ▸ h,
    fun q r h => hso ▸ h, fun ha => Or.inr ?_, fun _ _ q hq => ⟨(hprot q).mpr hq, by rw [hview]⟩,
    hroot, fun q r h => Or.inl (hse ▸ h), fun q r h => Or.inl (hso ▸ h)⟩
  refine ⟨hr ▸ ha.not_abandoned, hsheet ▸ ha.shape, hsheet ▸ ha.no_orphans,
    hsheet ▸ ha.cse_areas, hsheet ▸ ha.cse_spills, hstack ▸ ha.stack_nodup, ?_, ?_, ?_, ?_, ?_, ?_,
    ?_⟩
  · intro p
    rw [hstack, hcells]
    exact ha.stack_evaluating p
  · intro q r h
    rw [hse] at h
    rw [hsheet, hcells]
    exact ha.seen_empty q r h
  · intro q r h
    rw [hso] at h
    rw [hsheet]
    exact ha.seen_occupied q r h
  · intro q a v hq hne
    rw [hsheet] at hq
    rw [hcells] at hne
    exact ha.orig_spill q a v hq hne
  · intro p hp
    rw [hcells] at hp
    have := ha.evaluated_consistent p hp
    rw [hview, hsheet]
    have hsb : StableBlocked b = StableBlocked a := by
      funext p area
      simp [StableBlocked, hsheet, hso]
    rw [hsb]
    exact this
  · intro p t hp ht hne q hq
    rw [hcells] at hp
    rw [hsheet] at ht hne
    rw [hview] at hq
    exact (hprot q).mpr (ha.reads_protected p t hp ht hne q hq)
  · intro c hc hne
    rw [hroot] at hc
    rw [hstack] at hne
    rw [hcells, hsheet]
    exact ha.root_cell c hc hne

section primitives
variable (S₀ : Sheet Pos Value) (U : Universe Pos)

local notation "PR" => PassRel S₀

theorem storedValue_step (p : Pos) : Preserves PR (storedValue (Value := Value) p) :=
  Preserves.getBind fun st => PassStep.refl S₀ st

theorem markCycle_step (o : Pos) : Preserves PR (markCycle (Value := Value) o) :=
  Preserves.modifyM fun st => by
    split
    · exact PassStep.of_circular_only rfl rfl rfl rfl Finset.subset_union_left rfl rfl rfl
    · exact PassStep.refl S₀ st

theorem spillContradictsARead_step (anchor : Pos) (writes clears : List Pos) :
    Preserves PR (spillContradictsARead (Value := Value) anchor writes clears) :=
  Preserves.getBind fun st => by
    dsimp only
    split
    · exact PassStep.refl S₀ st
    · exact PassStep.of_restart_only rfl rfl rfl rfl rfl rfl rfl (Or.inr rfl)

/-- `recordSeen q .empty` is a step when `q` is empty or a leftover cell of a
running anchor, which is when the pass records it. -/
theorem recordSeen_empty_step (q : Pos) (st : PassState Pos Value)
    (hq : PassInv S₀ st → (st.sheet q).isEmpty = true ∨
      ∃ a v, st.sheet q = .spill a v ∧ (st.sheet a).isDynAnchor = true ∧
        st.cells a = some .evaluating) :
    PassStep S₀ st ((recordSeen (Value := Value) q .empty).run st).2 :=
  Preserves.getBind_run (I := PassRel S₀) st <| by
  split
  · exact PassStep.refl S₀ st
  · rename_i r hroot
    dsimp only
    split
    · exact PassStep.refl S₀ st
    · rename_i hnone
      have hse : ∀ x r', st.seenEmpty x = some r' →
          Function.update st.seenEmpty q (some r) x = some r' := by
        intro x r' hx
        by_cases hxq : x = q
        · subst hxq
          rw [hnone] at hx
          cases hx
        · rw [Function.update_of_ne hxq]
          exact hx
      have hprot : ∀ x, Protected st x →
          Protected { st with seenEmpty := Function.update st.seenEmpty q (some r) } x := by
        intro x hx
        rcases hx with h | h | h | h
        · exact Or.inl h
        · exact Or.inr (Or.inl h)
        · exact Or.inr (Or.inr (Or.inl h))
        · refine Or.inr (Or.inr (Or.inr ?_))
          obtain ⟨r', hr'⟩ := Option.isSome_iff_exists.mp h
          simp [hse x r' hr']
      refine ⟨fun h => h, fun _ h => h, Finset.Subset.refl _, hse, fun _ _ h => h,
        fun ha => Or.inr ?_, fun _ _ x hx => ⟨hprot x hx, rfl⟩, rfl, ?_, fun _ _ h => Or.inl h⟩
      refine ⟨ha.not_abandoned, ha.shape, ha.no_orphans, ha.cse_areas, ha.cse_spills,
        ha.stack_nodup, ha.stack_evaluating, ?_, ha.seen_occupied, ha.orig_spill, ?_, ?_,
        ha.root_cell⟩
      · intro x r' hx
        by_cases hxq : x = q
        · subst hxq
          exact hq ha
        · have hx' : Function.update st.seenEmpty q (some r) x = some r' := hx
          rw [Function.update_of_ne hxq] at hx'
          exact ha.seen_empty x r' hx'
      · intro p hp
        have := ha.evaluated_consistent p hp
        exact this
      · intro p t hp ht hne x hx
        exact hprot x (ha.reads_protected p t hp ht hne x hx)
      · intro x r' hx
        by_cases hxq : x = q
        · subst hxq
          have hx' : Function.update st.seenEmpty x (some r) x = some r' := hx
          rw [Function.update_self] at hx'
          cases hx'
          exact Or.inr hroot
        · have hx' : Function.update st.seenEmpty q (some r) x = some r' := hx
          rw [Function.update_of_ne hxq] at hx'
          exact Or.inl hx'

/-- `recordSeen q .occupied` is a step when `q` holds a spill cell, which is
when the blocking scan records it. -/
theorem recordSeen_occupied_step (q : Pos) (st : PassState Pos Value)
    (hq : ∃ a v, st.sheet q = .spill a v) :
    PassStep S₀ st ((recordSeen (Value := Value) q .occupied).run st).2 :=
  Preserves.getBind_run (I := PassRel S₀) st <| by
  split
  · exact PassStep.refl S₀ st
  · rename_i r hroot
    dsimp only
    split
    · exact PassStep.refl S₀ st
    · rename_i hnone
      have hso : ∀ x r', st.seenOccupied x = some r' →
          Function.update st.seenOccupied q (some r) x = some r' := by
        intro x r' hx
        by_cases hxq : x = q
        · subst hxq
          rw [hnone] at hx
          cases hx
        · rw [Function.update_of_ne hxq]
          exact hx
      have hsb : ∀ p area, StableBlocked st p area →
          StableBlocked { st with seenOccupied := Function.update st.seenOccupied q (some r) }
            p area := by
        intro p area ⟨x, hx, hxp, h⟩
        refine ⟨x, hx, hxp, ?_⟩
        rcases h with h | ⟨a, v, hs, hap, hso'⟩
        · exact Or.inl h
        · refine Or.inr ⟨a, v, hs, hap, ?_⟩
          obtain ⟨r', hr'⟩ := Option.isSome_iff_exists.mp hso'
          simp [hso x r' hr']
      refine ⟨fun h => h, fun _ h => h, Finset.Subset.refl _, fun _ _ h => h, hso,
        fun ha => Or.inr ?_, fun _ _ x hx => ⟨hx, rfl⟩, rfl, fun _ _ h => Or.inl h, ?_⟩
      refine ⟨ha.not_abandoned, ha.shape, ha.no_orphans, ha.cse_areas, ha.cse_spills,
        ha.stack_nodup, ha.stack_evaluating, ha.seen_empty, ?_, ha.orig_spill, ?_,
        ha.reads_protected, ha.root_cell⟩
      · intro x r' hx
        by_cases hxq : x = q
        · subst hxq
          exact hq
        · have hx' : Function.update st.seenOccupied q (some r) x = some r' := hx
          rw [Function.update_of_ne hxq] at hx'
          exact ha.seen_occupied x r' hx'
      · intro p hp
        exact ConsistentAtWith.mono (hsb) (ha.evaluated_consistent p hp)
      · intro x r' hx
        by_cases hxq : x = q
        · subst hxq
          have hx' : Function.update st.seenOccupied x (some r) x = some r' := hx
          rw [Function.update_self] at hx'
          cases hx'
          exact Or.inr hroot
        · have hx' : Function.update st.seenOccupied q (some r) x = some r' := hx
          rw [Function.update_of_ne hxq] at hx'
          exact Or.inl hx'

end primitives

end IronCalcEval

import IronCalcEval.Invariant

/-!
# Commit

What `set_cells_with_result` does to a state satisfying the invariant, when
the cell on top of the stack stores its result. The specification
`commit_spec` is stated for the state *after* the cell is marked evaluated
(`markEvaluated`), because the invariant does not hold in between: the fresh
spill cells of an anchor are on record nowhere, which the invariant alone
does not say, and that is what lets the mark step keep every protected
position's view.
-/

namespace IronCalcEval

variable {Pos Value : Type} [DecidableEq Pos] [ValueSort Value]

/-- What is known of the result a cell is about to store: `#CIRC!` because
the cell is marked, or what its formula gives against the current view, with
every read protected. -/
def ResultOf (st : PassState Pos Value) (p : Pos) (r : Result Pos Value) : Prop :=
  r = .scalar circ ∨
  ∃ t, (st.sheet p).formula? = some t ∧ r = t.runPure (passView st) ∧
    ∀ q ∈ t.reads (passView st), Protected st q

/-- A formula cell or anchor being evaluated is not protected: its view is
about to change. -/
theorem not_protected_of_evaluating {S₀ : Sheet Pos Value} {st : PassState Pos Value}
    (hinv : PassInv S₀ st) {p : Pos} (hf : (st.sheet p).formula?.isSome)
    (hp : st.cells p = some .evaluating) : ¬ Protected st p := by
  rintro (⟨v, hv⟩ | ⟨_, hev⟩ | ⟨a, v, hs, _⟩ | hse)
  · rw [hv] at hf
    simp [Content.formula?] at hf
  · rw [hp] at hev
    cases hev
  · rw [hs] at hf
    simp [Content.formula?] at hf
  · obtain ⟨r, hr⟩ := Option.isSome_iff_exists.mp hse
    rcases hinv.seen_empty p r hr with h | ⟨a, v, hs, _⟩
    · cases hc : st.sheet p <;> simp_all [Content.isEmpty, Content.formula?]
    · rw [hs] at hf
      simp [Content.formula?] at hf

/-- The cell on top of the stack is being evaluated. -/
theorem cells_of_head {S₀ : Sheet Pos Value} {st : PassState Pos Value} (hinv : PassInv S₀ st)
    {p : Pos} (htop : st.stack.head? = some p) : st.cells p = some .evaluating :=
  (hinv.stack_evaluating p).mp (List.mem_of_mem_head? htop)

/-- Consistency at a position transported to another reader, sheet and
notion of blocked, when nothing that clause looks at has changed. -/
theorem ConsistentAtWith.transport {r₁ r₂ : Pos → Value} {b₁ b₂ : Pos → List Pos → Prop}
    {S₁ S₂ : Sheet Pos Value} {q : Pos} (hc : ConsistentAtWith r₁ b₁ S₁ q)
    (hq : S₂ q = S₁ q)
    (hreads : valueAt S₁ q ≠ circ → ∀ t, (S₁ q).formula? = some t →
      ∀ x ∈ t.reads r₁, r₁ x = r₂ x)
    (hspill : ∀ x w, S₂ x = .spill q w ↔ S₁ x = .spill q w)
    (hanchor : ∀ x, (S₂ x).spillAnchor? = some q ↔ (S₁ x).spillAnchor? = some q)
    (hb : ∀ area, b₁ q area → b₂ q area) :
    ConsistentAtWith r₂ b₂ S₂ q := by
  unfold ConsistentAtWith at hc ⊢
  rw [hq]
  cases hS : S₁ q with
  | formula t stored =>
      rw [hS] at hc
      intro hne
      have hv : valueAt S₁ q = stored := by simp [valueAt, hS]
      rw [← Formula.runPure_congr t (hreads (hv ▸ hne) t (by simp [hS, Content.formula?]))]
      exact hc hne
  | cseAnchor t area stored =>
      rw [hS] at hc
      intro hne
      have hv : valueAt S₁ q = stored := by simp [valueAt, hS]
      rw [← Formula.runPure_congr t (hreads (hv ▸ hne) t (by simp [hS, Content.formula?]))]
      obtain ⟨h1, h2⟩ := hc hne
      exact ⟨h1, fun x hx hxq => (hspill x _).mpr (h2 x hx hxq)⟩
  | dynAnchor t stored =>
      rw [hS] at hc
      intro hne
      have hv : valueAt S₁ q = stored := by simp [valueAt, hS]
      rw [← Formula.runPure_congr t (hreads (hv ▸ hne) t (by simp [hS, Content.formula?]))]
      specialize hc hne
      revert hc
      cases t.runPure r₁ with
      | scalar v =>
          rintro ⟨h1, h2⟩
          exact ⟨h1, fun x hx => h2 x ((hanchor x).mp hx)⟩
      | array area vals =>
          rintro (⟨h1, h2⟩ | ⟨hbl, h2, h3⟩)
          · refine Or.inl ⟨h1, fun x hxq => ⟨fun hx => (hspill x _).mpr ((h2 x hxq).1 hx),
              fun hx hs => (h2 x hxq).2 hx ((hanchor x).mp hs)⟩⟩
          · exact Or.inr ⟨hb area hbl, h2, fun x hx => h3 x ((hanchor x).mp hx)⟩
  | empty => trivial
  | const v => trivial
  | spill a v => trivial

/-- `p` keeps its formula and its kind; only its stored value changes. -/
def SameKind : Content Pos Value → Content Pos Value → Prop
  | .formula t _, c => ∃ v, c = .formula t v
  | .cseAnchor t area _, c => ∃ v, c = .cseAnchor t area v
  | .dynAnchor t _, c => ∃ v, c = .dynAnchor t v
  | _, _ => False

/-- The state after a commit at `p` with the new sheet and occupied records,
once `p` is marked evaluated. -/
def commitState (st : PassState Pos Value) (p : Pos) (S' : Sheet Pos Value)
    (SO' : Pos → Option Pos) : PassState Pos Value :=
  markEvaluated { st with sheet := S', seenOccupied := SO' } p

/-- What a commit at `p` did: it wrote its spill cells at `W` (free cells,
none on record as read empty), removed its own leftover cells at `C` (none on
record as blocking), changed nothing else but its own stored value, and may
have put foreign spill cells on record as blocking. -/
structure SheetChange (st : PassState Pos Value) (p : Pos) (S' : Sheet Pos Value)
    (SO' : Pos → Option Pos) (W C : List Pos) : Prop where
  p_kind : SameKind (st.sheet p) (S' p)
  untouched : ∀ x, x ≠ p → x ∉ W → x ∉ C → S' x = st.sheet x
  written : ∀ x ∈ W, x ≠ p ∧ (∃ w, S' x = .spill p w) ∧ (st.sheet x).freeFor p = true ∧
    st.seenEmpty x = none
  cleared : ∀ x ∈ C, x ≠ p ∧ S' x = .empty ∧ (∃ w, st.sheet x = .spill p w) ∧
    st.seenOccupied x = none
  /-- Every spill cell of `p` left after the commit is fresh. -/
  fresh_unrecorded : ∀ x w, S' x = .spill p w → st.seenEmpty x = none
  p_anchor_of_written : W ≠ [] → (S' p).isAnchor = true
  /-- A CSE anchor writes only inside its area. -/
  written_in_cse : ∀ x ∈ W, ∀ t area v, S' p = .cseAnchor t area v → x ∈ area
  cleared_dyn : C ≠ [] → (st.sheet p).isDynAnchor = true
  so_mono : ∀ x r, st.seenOccupied x = some r → SO' x = some r
  so_spill : ∀ x r, SO' x = some r → ∃ a v, S' x = .spill a v
  /-- New records were made on behalf of the root. -/
  so_new : ∀ x r, SO' x = some r → st.seenOccupied x = some r ∨ st.root = some r

section generic
variable {S₀ : Sheet Pos Value} {st : PassState Pos Value} {p : Pos} {S' : Sheet Pos Value}
  {SO' : Pos → Option Pos} {W C : List Pos}

omit [DecidableEq Pos] [ValueSort Value] in
theorem SameKind.formula_eq {c₁ c₂ : Content Pos Value} (h : SameKind c₁ c₂) :
    c₂.formula? = c₁.formula? ∧ c₂.isAnchor = c₁.isAnchor ∧ c₂.isDynAnchor = c₁.isDynAnchor ∧
      c₂.isEmpty = false ∧ c₂.spillAnchor? = none ∧ c₁.isEmpty = false ∧ c₁.spillAnchor? = none := by
  cases c₁ <;> simp [SameKind] at h <;> obtain ⟨v, rfl⟩ := h <;>
    simp [Content.formula?, Content.isAnchor, Content.isDynAnchor, Content.isEmpty,
      Content.spillAnchor?]

omit [DecidableEq Pos] [ValueSort Value] in
theorem SameKind.isSome {c₁ c₂ : Content Pos Value} (h : SameKind c₁ c₂) : c₁.formula?.isSome := by
  cases c₁ <;> simp [SameKind, Content.formula?] at h ⊢

/-- The facts every case of the commit uses. -/
structure ChangeFacts (S₀ : Sheet Pos Value) (st : PassState Pos Value) (p : Pos)
    (S' : Sheet Pos Value) (SO' : Pos → Option Pos) : Prop where
  pev : st.cells p = some .evaluating
  not_prot : ¬ Protected st p
  ne_of_prot : ∀ q, Protected st q → q ≠ p
  view : ∀ q, Protected st q → passView (commitState st p S' SO') q = passView st q
  prot : ∀ q, Protected st q → Protected (commitState st p S' SO') q

theorem SheetChange.facts (hinv : PassInv S₀ st) (htop : st.stack.head? = some p)
    (hc : SheetChange st p S' SO' W C) : ChangeFacts S₀ st p S' SO' := by
  have hpev : st.cells p = some .evaluating := cells_of_head hinv htop
  have hkind := hc.p_kind.formula_eq
  have hnp : ¬ Protected st p := not_protected_of_evaluating hinv hc.p_kind.isSome hpev
  have hne_of_prot : ∀ q, Protected st q → q ≠ p := fun q hq hqp => hnp (hqp ▸ hq)
  -- A position that is not free for `p` and not `p`'s own spill cell is untouched.
  have hunt : ∀ q, q ≠ p → (st.sheet q).freeFor p = false → S' q = st.sheet q := by
    intro q hqp hfree
    apply hc.untouched q hqp
    · intro hW
      rw [(hc.written q hW).2.2.1] at hfree
      cases hfree
    · intro hC
      obtain ⟨w, hw⟩ := (hc.cleared q hC).2.2.1
      simp [Content.freeFor, hw] at hfree
  have hcells : ∀ a, a ≠ p → (commitState st p S' SO').cells a = st.cells a :=
    fun a ha => Function.update_of_ne ha _ _
  have hsheet : (commitState st p S' SO').sheet = S' := rfl
  refine ⟨hpev, hnp, hne_of_prot, ?_, ?_⟩
  · intro q hq
    have hqp := hne_of_prot q hq
    rcases hq with ⟨w, h⟩ | ⟨h1, h2⟩ | ⟨a, w, h1, h2⟩ | h
    · have hu := hunt q hqp (by simp [Content.freeFor, h])
      simp only [passView, hsheet, hu, h, valueAt]
    · have hnf : (st.sheet q).freeFor p = false := by
        cases hs : st.sheet q <;> simp_all [Content.formula?, Content.freeFor]
      have hu := hunt q hqp hnf
      unfold passView
      rw [hsheet, hu]
      cases hs : st.sheet q <;> simp_all [Content.formula?, valueAt]
    · have hap : a ≠ p := by
        rintro rfl
        rw [hpev] at h2
        cases h2
      have hu := hunt q hqp (by simp [Content.freeFor, h1, hap])
      simp only [passView, hsheet, hu, h1, hcells a hap, valueAt]
    · obtain ⟨r, hr⟩ := Option.isSome_iff_exists.mp h
      rcases hinv.seen_empty q r hr with he | ⟨a, w, h1, hdyn, h2⟩
      · -- Empty when read: not written (on record), not cleared (not a spill cell).
        have hu : S' q = st.sheet q := by
          apply hc.untouched q hqp
          · intro hW
            rw [(hc.written q hW).2.2.2] at hr
            cases hr
          · intro hC
            obtain ⟨w, hw⟩ := (hc.cleared q hC).2.2.1
            rw [hw] at he
            simp [Content.isEmpty] at he
        cases hs : st.sheet q <;> simp_all [Content.isEmpty, passView, valueAt]
      · by_cases hap : a = p
        · -- A leftover of `p` read as empty: `p` must have cleared it.
          subst hap
          have hnw : ∀ w', S' q ≠ .spill a w' := by
            intro w' hw'
            rw [hc.fresh_unrecorded q w' hw'] at hr
            cases hr
          have hC : q ∈ C := by
            by_contra hnC
            by_cases hW : q ∈ W
            · obtain ⟨w', hw'⟩ := (hc.written q hW).2.1
              exact hnw w' hw'
            · exact hnw w (hc.untouched q hqp hW hnC ▸ h1)
          obtain ⟨_, he, _, _⟩ := hc.cleared q hC
          simp [passView, hsheet, he, h1, h2, valueAt]
        · have hu := hunt q hqp (by simp [Content.freeFor, h1, hap])
          simp [passView, hsheet, hu, h1, hcells a hap, h2]
  · intro q hq
    have hqp := hne_of_prot q hq
    rcases hq with ⟨w, h⟩ | ⟨h1, h2⟩ | ⟨a, w, h1, h2⟩ | h
    · exact Or.inl ⟨w, by rw [hsheet, hunt q hqp (by simp [Content.freeFor, h]), h]⟩
    · have hnf : (st.sheet q).freeFor p = false := by
        cases hs : st.sheet q <;> simp_all [Content.formula?, Content.freeFor]
      exact Or.inr (Or.inl ⟨by rw [hsheet, hunt q hqp hnf]; exact h1, by rw [hcells q hqp]; exact h2⟩)
    · have hap : a ≠ p := by
        rintro rfl
        rw [hpev] at h2
        cases h2
      exact Or.inr (Or.inr (Or.inl ⟨a, w,
        by rw [hsheet, hunt q hqp (by simp [Content.freeFor, h1, hap])]; exact h1,
        by rw [hcells a hap]; exact h2⟩))
    · exact Or.inr (Or.inr (Or.inr h))

/-- Consistency holds trivially at a position that is not a formula cell. -/
theorem ConsistentAtWith.of_not_formula {r : Pos → Value} {b : Pos → List Pos → Prop}
    {S : Sheet Pos Value} {q : Pos} (h : (S q).formula? = none) : ConsistentAtWith r b S q := by
  unfold ConsistentAtWith
  cases hS : S q <;> simp_all [Content.formula?]

/-- The whole invariant from a `SheetChange`, given the anchor's own clause. -/
theorem SheetChange.commitOk (hinv : PassInv S₀ st) (htop : st.stack.head? = some p)
    (hc : SheetChange st p S' SO' W C)
    (hcons : ConsistentAtWith (passView (commitState st p S' SO'))
      (StableBlocked (commitState st p S' SO')) S' p)
    (hreads : ∀ t, (S' p).formula? = some t → valueAt S' p ≠ circ →
      ∀ x ∈ t.reads (passView (commitState st p S' SO')), Protected (commitState st p S' SO') x) :
    PassStep S₀ st (commitState st p S' SO') ∧ PassInv S₀ (commitState st p S' SO') := by
  have hf := hc.facts hinv htop
  set m := commitState st p S' SO' with hm
  have hpev := hf.pev
  have hstack : st.stack = p :: st.stack.tail := (List.cons_head?_tail htop).symm
  have hkind := hc.p_kind.formula_eq
  have hm_sheet : m.sheet = S' := rfl
  have hm_cells : ∀ q, q ≠ p → m.cells q = st.cells q := fun q hq => Function.update_of_ne hq _ _
  have hm_cellsp : m.cells p = some .evaluated := Function.update_self _ _ _
  have hm_stack : m.stack = st.stack.tail := rfl
  have hm_rest : m.restart = st.restart := rfl
  have hm_se : m.seenEmpty = st.seenEmpty := rfl
  have hm_so : m.seenOccupied = SO' := rfl
  have hm_circ : m.circular = st.circular := rfl
  -- A formula cell or anchor other than `p` is untouched: it is not free.
  have hunt_formula : ∀ q, q ≠ p → (st.sheet q).formula?.isSome → S' q = st.sheet q := by
    intro q hqp hq
    apply hc.untouched q hqp
    · intro hW
      have := (hc.written q hW).2.2.1
      cases hs : st.sheet q <;> simp [hs, Content.formula?, Content.freeFor] at this hq
    · intro hC
      obtain ⟨w, hw⟩ := (hc.cleared q hC).2.2.1
      rw [hw] at hq
      simp [Content.formula?] at hq
  -- A spill cell of another anchor is untouched: it is not free either.
  have hunt_spill : ∀ q a w, a ≠ p → st.sheet q = .spill a w → S' q = st.sheet q := by
    intro q a w hap hq
    have hqp : q ≠ p := by
      rintro rfl
      rw [hq] at hkind
      simp [Content.spillAnchor?] at hkind
    apply hc.untouched q hqp
    · intro hW
      have := (hc.written q hW).2.2.1
      simp [Content.freeFor, hq, hap] at this
    · intro hC
      obtain ⟨w', hw'⟩ := (hc.cleared q hC).2.2.1
      rw [hq] at hw'
      cases hw'
      exact hap rfl
  -- Anything that is not a formula cell stays that way.
  have hnf_stable : ∀ q, (st.sheet q).formula? = none → (S' q).formula? = none := by
    intro q hq
    by_cases hqp : q = p
    · subst hqp
      exact absurd hq (Option.isSome_iff_ne_none.mp hc.p_kind.isSome)
    by_cases hW : q ∈ W
    · obtain ⟨_, ⟨w, hw⟩, _, _⟩ := hc.written q hW
      simp [hw, Content.formula?]
    by_cases hC : q ∈ C
    · obtain ⟨_, he, _, _⟩ := hc.cleared q hC
      simp [he, Content.formula?]
    rw [hc.untouched q hqp hW hC]
    exact hq
  -- `p` is not a spill cell before or after.
  have hpne : ∀ w a, st.sheet p ≠ .spill a w ∧ S' p ≠ .spill a w := by
    intro w a
    constructor
    · intro h
      rw [h] at hkind
      simp [Content.spillAnchor?] at hkind
    · intro h
      rw [h] at hkind
      simp [Content.spillAnchor?] at hkind
  have hshape : SameShape S₀ S' := by
    intro q
    by_cases hqp : q = p
    · subst hqp
      have := hinv.shape q
      have hk := hc.p_kind
      revert this hk
      cases S₀ q <;> cases hs : st.sheet q <;>
        simp [SameKind, Content.isEmpty, Content.spillAnchor?] <;>
        first
        | (rintro rfl x hx; exact ⟨x, hx⟩)
        | (rintro rfl rfl x hx; exact ⟨x, hx⟩)
    · by_cases hu : S' q = st.sheet q
      · rw [hu]
        exact hinv.shape q
      · have hfree : ((st.sheet q).isEmpty = true ∨ (st.sheet q).spillAnchor?.isSome) ∧
            ((S' q).isEmpty = true ∨ (S' q).spillAnchor?.isSome) := by
          by_cases hW : q ∈ W
          · obtain ⟨_, ⟨w, hw⟩, hfr, _⟩ := hc.written q hW
            refine ⟨?_, Or.inr (by simp [hw, Content.spillAnchor?])⟩
            cases hs : st.sheet q <;>
              simp [hs, Content.freeFor, Content.isEmpty, Content.spillAnchor?] at hfr ⊢
          · by_cases hC : q ∈ C
            · obtain ⟨_, he, ⟨w, hw⟩, _⟩ := hc.cleared q hC
              exact ⟨Or.inr (by simp [hw, Content.spillAnchor?]),
                Or.inl (by simp [he, Content.isEmpty])⟩
            · exact absurd (hc.untouched q hqp hW hC) hu
        have := hinv.shape q
        revert this
        cases S₀ q with
        | const v =>
            intro h
            rw [h] at hfree
            simp [Content.isEmpty, Content.spillAnchor?] at hfree
        | formula t _ =>
            rintro ⟨v, hv⟩
            rw [hv] at hfree
            simp [Content.isEmpty, Content.spillAnchor?] at hfree
        | cseAnchor t area _ =>
            rintro ⟨v, hv⟩
            rw [hv] at hfree
            simp [Content.isEmpty, Content.spillAnchor?] at hfree
        | dynAnchor t _ =>
            rintro ⟨v, hv⟩
            rw [hv] at hfree
            simp [Content.isEmpty, Content.spillAnchor?] at hfree
        | empty => exact fun _ => hfree.2
        | spill _ _ => exact fun _ => hfree.2
  have hm_root : m.root = st.root := rfl
  have hinv' : PassInv S₀ m :=
    { not_abandoned := hm_rest ▸ hinv.not_abandoned
      root_cell := by
        intro c hc hne
        rw [hm_root] at hc
        have hne' : st.stack ≠ [] := fun h => hne (by rw [hm_stack, h]; rfl)
        rcases hinv.root_cell c hc hne' with h | h
        · left
          by_cases hcp : c = p
          · subst hcp
            rw [hm_cellsp]
            simp
          · rw [hm_cells c hcp]
            exact h
        · right
          rw [hm_sheet]
          exact hnf_stable c h
      orig_spill := by
        intro q a v hq hne
        rw [hm_sheet] at hq
        have hap : a ≠ p := by
          rintro rfl
          exact hne hm_cellsp
        rw [hm_cells a hap] at hne
        by_cases hqp : q = p
        · subst hqp
          rw [hq] at hkind
          simp [Content.spillAnchor?] at hkind
        by_cases hW : q ∈ W
        · obtain ⟨_, ⟨w, hw⟩, _, _⟩ := hc.written q hW
          rw [hw] at hq
          cases hq
          exact absurd rfl hap
        by_cases hC : q ∈ C
        · obtain ⟨_, he, _, _⟩ := hc.cleared q hC
          rw [he] at hq
          cases hq
        rw [hc.untouched q hqp hW hC] at hq
        exact hinv.orig_spill q a v hq hne
      shape := hshape
      no_orphans := by
        intro q a w hq
        rw [hm_sheet] at hq ⊢
        by_cases hqp : q = p
        · subst hqp
          exact absurd hq (hpne w a).2
        by_cases hW : q ∈ W
        · obtain ⟨_, ⟨w', hw'⟩, _, _⟩ := hc.written q hW
          rw [hw'] at hq
          cases hq
          exact hc.p_anchor_of_written (List.ne_nil_of_mem hW)
        by_cases hC : q ∈ C
        · obtain ⟨_, he, _, _⟩ := hc.cleared q hC
          rw [he] at hq
          cases hq
        rw [hc.untouched q hqp hW hC] at hq
        have ha := hinv.no_orphans q a w hq
        by_cases hap : a = p
        · subst hap
          rw [hkind.2.1]
          exact ha
        · rw [hunt_formula a hap (by cases hs : st.sheet a <;>
            simp [hs, Content.isAnchor, Content.formula?] at ha ⊢)]
          exact ha
      cse_areas := by
        intro q t area v hq x hx hxq
        rw [hm_sheet] at hq ⊢
        by_cases hqp : q = p
        · subst hqp
          have hk := hc.p_kind
          rw [hq] at hk
          obtain ⟨v', hv'⟩ : ∃ v', st.sheet q = .cseAnchor t area v' := by
            cases hs : st.sheet q <;> simp [hs, SameKind] at hk
            obtain ⟨rfl, rfl⟩ := hk
            exact ⟨_, rfl⟩
          obtain ⟨w, hw⟩ := hinv.cse_areas q t area v' hv' x hx hxq
          have hC : C = [] := by
            by_contra hne
            have := hc.cleared_dyn hne
            rw [hv'] at this
            simp [Content.isDynAnchor] at this
          by_cases hW : x ∈ W
          · exact (hc.written x hW).2.1
          · rw [hc.untouched x hxq hW (by simp [hC])]
            exact ⟨w, hw⟩
        · have hq' : st.sheet q = .cseAnchor t area v := by
            by_cases hW : q ∈ W
            · obtain ⟨_, ⟨w', hw'⟩, _, _⟩ := hc.written q hW
              rw [hw'] at hq
              cases hq
            by_cases hC : q ∈ C
            · obtain ⟨_, he, _, _⟩ := hc.cleared q hC
              rw [he] at hq
              cases hq
            rw [hc.untouched q hqp hW hC] at hq
            exact hq
          obtain ⟨w, hw⟩ := hinv.cse_areas q t area v hq' x hx hxq
          exact ⟨w, by rw [hunt_spill x q w hqp hw]; exact hw⟩
      cse_spills := by
        intro q a w hq t area v ha
        rw [hm_sheet] at hq ha
        by_cases hqp : q = p
        · subst hqp
          exact absurd hq (hpne w a).2
        by_cases hW : q ∈ W
        · obtain ⟨_, ⟨w', hw'⟩, _, _⟩ := hc.written q hW
          rw [hw'] at hq
          cases hq
          exact hc.written_in_cse q hW t area v ha
        by_cases hC : q ∈ C
        · obtain ⟨_, he, _, _⟩ := hc.cleared q hC
          rw [he] at hq
          cases hq
        rw [hc.untouched q hqp hW hC] at hq
        by_cases hap : a = p
        · subst hap
          have hk := hc.p_kind
          rw [ha] at hk
          obtain ⟨v', hv'⟩ : ∃ v', st.sheet a = .cseAnchor t area v' := by
            cases hs : st.sheet a <;> simp [hs, SameKind] at hk
            obtain ⟨rfl, rfl⟩ := hk
            exact ⟨_, rfl⟩
          exact hinv.cse_spills q a w hq t area v' hv'
        · have hf : (st.sheet a).formula?.isSome := by
            have := hinv.no_orphans q a w hq
            cases hs : st.sheet a <;> simp [hs, Content.isAnchor, Content.formula?] at this ⊢
          rw [hunt_formula a hap hf] at ha
          exact hinv.cse_spills q a w hq t area v ha
      stack_nodup := by
        rw [hm_stack]
        exact hinv.stack_nodup.sublist (List.tail_sublist _)
      stack_evaluating := by
        intro q
        rw [hm_stack]
        by_cases hq : q = p
        · subst hq
          rw [hm_cellsp]
          have hnd := hinv.stack_nodup
          rw [hstack] at hnd
          simp [List.nodup_cons.mp hnd |>.1]
        · rw [hm_cells q hq]
          rw [← hinv.stack_evaluating q]
          conv_rhs => rw [hstack]
          simp [hq]
      seen_empty := by
        intro q r hq
        rw [hm_se] at hq
        rw [hm_sheet]
        rcases hinv.seen_empty q r hq with he | ⟨a, w, h1, hdyn, h2⟩
        · left
          have hqp : q ≠ p := by
            rintro rfl
            rw [hkind.2.2.2.2.2.1] at he
            cases he
          have hu : S' q = st.sheet q := by
            apply hc.untouched q hqp
            · intro hW
              rw [(hc.written q hW).2.2.2] at hq
              cases hq
            · intro hC
              obtain ⟨w, hw⟩ := (hc.cleared q hC).2.2.1
              rw [hw] at he
              simp [Content.isEmpty] at he
          rw [hu]
          exact he
        · have hqp : q ≠ p := by
            rintro rfl
            exact (hpne w a).1 h1
          by_cases hap : a = p
          · subst hap
            left
            have hnw : ∀ w', S' q ≠ .spill a w' := by
              intro w' hw'
              rw [hc.fresh_unrecorded q w' hw'] at hq
              cases hq
            have hC : q ∈ C := by
              by_contra hnC
              by_cases hW : q ∈ W
              · obtain ⟨w', hw'⟩ := (hc.written q hW).2.1
                exact hnw w' hw'
              · exact hnw w (hc.untouched q hqp hW hnC ▸ h1)
            obtain ⟨_, he, _, _⟩ := hc.cleared q hC
            simp [he, Content.isEmpty]
          · right
            have hadyn : (st.sheet a).formula?.isSome := by
              cases hs : st.sheet a <;> simp [hs, Content.isDynAnchor, Content.formula?] at hdyn ⊢
            refine ⟨a, w, by rw [hunt_spill q a w hap h1]; exact h1, ?_, ?_⟩
            · rw [hunt_formula a hap hadyn]
              exact hdyn
            · rw [hm_cells a hap]
              exact h2
      seen_occupied := by
        intro q r hq
        rw [hm_so] at hq
        rw [hm_sheet]
        exact hc.so_spill q r hq
      evaluated_consistent := by
        intro q hq
        by_cases hqp : q = p
        · subst hqp
          exact hcons
        · rw [hm_cells q hqp] at hq
          by_cases hfq : (st.sheet q).formula?.isSome
          · have hu := hunt_formula q hqp hfq
            refine ConsistentAtWith.transport (hinv.evaluated_consistent q hq)
              (by rw [hm_sheet, hu]) ?_ ?_ ?_ ?_
            · intro hne t ht x hx
              exact (hf.view x (hinv.reads_protected q t hq ht hne x hx)).symm
            · intro x w
              rw [hm_sheet]
              by_cases hxp : x = p
              · subst hxp
                simp [(hpne w q).1, (hpne w q).2]
              by_cases hW : x ∈ W
              · obtain ⟨_, ⟨w', hw'⟩, hfr, _⟩ := hc.written x hW
                rw [hw']
                constructor
                · intro h
                  cases h
                  exact absurd rfl hqp
                · intro h
                  rw [h] at hfr
                  simp [Content.freeFor, hqp] at hfr
              by_cases hC : x ∈ C
              · obtain ⟨_, he, ⟨w', hw'⟩, _⟩ := hc.cleared x hC
                rw [he, hw']
                constructor
                · intro h
                  cases h
                · intro h
                  cases h
                  exact absurd rfl hqp
              rw [hc.untouched x hxp hW hC]
            · intro x
              rw [hm_sheet]
              by_cases hxp : x = p
              · subst hxp
                rw [hkind.2.2.2.2.1, hkind.2.2.2.2.2.2]
              by_cases hW : x ∈ W
              · obtain ⟨_, ⟨w', hw'⟩, hfr, _⟩ := hc.written x hW
                rw [hw']
                cases hs : st.sheet x with
                | spill a w'' =>
                    have hap : a = p := by simpa [hs, Content.freeFor] using hfr
                    subst hap
                    simp [Content.spillAnchor?, Ne.symm hqp]
                | _ => simp [hs, Content.freeFor, Content.spillAnchor?, Ne.symm hqp] at hfr ⊢
              by_cases hC : x ∈ C
              · obtain ⟨_, he, ⟨w', hw'⟩, _⟩ := hc.cleared x hC
                rw [he, hw']
                simp [Content.spillAnchor?, Ne.symm hqp]
              rw [hc.untouched x hxp hW hC]
            · intro area ⟨x, hx, hxq, h⟩
              refine ⟨x, hx, hxq, ?_⟩
              rw [hm_sheet, hm_so]
              rcases h with ⟨hne, hsp⟩ | ⟨a, w, h1, haq, hso⟩
              · left
                by_cases hxp : x = p
                · subst hxp
                  exact ⟨hkind.2.2.2.1, hkind.2.2.2.2.1⟩
                have hu : S' x = st.sheet x := by
                  apply hc.untouched x hxp
                  · intro hW
                    have := (hc.written x hW).2.2.1
                    cases hs : st.sheet x <;>
                      simp [hs, Content.freeFor, Content.isEmpty, Content.spillAnchor?] at this hne hsp
                  · intro hC
                    obtain ⟨w, hw⟩ := (hc.cleared x hC).2.2.1
                    rw [hw] at hsp
                    simp [Content.spillAnchor?] at hsp
                rw [hu]
                exact ⟨hne, hsp⟩
              · right
                obtain ⟨r, hr⟩ := Option.isSome_iff_exists.mp hso
                have hso' : (SO' x).isSome := by simp [hc.so_mono x r hr]
                by_cases hW : x ∈ W
                · obtain ⟨_, ⟨w', hw'⟩, _, _⟩ := hc.written x hW
                  exact ⟨p, w', hw', Ne.symm hqp, hso'⟩
                · have hnC : x ∉ C := by
                    intro hC
                    rw [(hc.cleared x hC).2.2.2] at hr
                    cases hr
                  have hxp : x ≠ p := by
                    rintro rfl
                    exact (hpne w a).1 h1
                  rw [hc.untouched x hxp hW hnC]
                  exact ⟨a, w, h1, haq, hso'⟩
          · apply ConsistentAtWith.of_not_formula
            rw [hm_sheet]
            exact hnf_stable q (Option.not_isSome_iff_eq_none.mp hfq)
      reads_protected := by
        intro q t hq ht hne x hx
        by_cases hqp : q = p
        · subst hqp
          exact hreads t ht hne x hx
        · rw [hm_cells q hqp] at hq
          have hfq : (st.sheet q).formula?.isSome := by
            by_contra hn
            have := hnf_stable q (Option.not_isSome_iff_eq_none.mp hn)
            rw [hm_sheet, this] at ht
            cases ht
          have hu := hunt_formula q hqp hfq
          rw [hm_sheet, hu] at ht
          have hval : valueAt S' q = valueAt st.sheet q := by simp only [valueAt, hu]
          rw [hm_sheet, hval] at hne
          have hag : ∀ y ∈ t.reads (passView st), passView st y = passView m y :=
            fun y hy => (hf.view y (hinv.reads_protected q t hq ht hne y hy)).symm
          rw [← Formula.reads_congr t hag] at hx
          exact hf.prot x (hinv.reads_protected q t hq ht hne x hx) }
  refine ⟨?_, hinv'⟩
  exact
    { restart_mono := fun h => hm_rest ▸ h
      cells_mono := fun q hq => by
        have hqp : q ≠ p := by
          rintro rfl
          rw [hpev] at hq
          cases hq
        rw [hm_cells q hqp]
        exact hq
      circular_mono := hm_circ ▸ Finset.Subset.refl _
      seenEmpty_mono := fun q r h => hm_se ▸ h
      seenOccupied_mono := fun q r h => hm_so ▸ hc.so_mono q r h
      inv := fun _ => Or.inr hinv'
      protect := fun _ _ q hq => ⟨hf.prot q hq, hf.view q hq⟩
      root_eq := rfl
      roots_new_empty := fun q r h => Or.inl (hm_se ▸ h)
      roots_new_occupied := fun q r h => hc.so_new q r (hm_so ▸ h) }

end generic

/-- After a successful commit of `p`, once `p` is marked evaluated. -/
structure CommitOk (S₀ : Sheet Pos Value) (st st' : PassState Pos Value) (p : Pos) : Prop where
  not_abandoned : st'.restart = none
  step : PassStep S₀ st (markEvaluated st' p)
  inv : PassInv S₀ (markEvaluated st' p)
  cells : st'.cells = st.cells
  stack : st'.stack = st.stack

/-- What the restart raised by a commit of `p` with result `r` says, relative
to the state the commit started from. -/
structure Abandoned (S₀ : Sheet Pos Value) (st : PassState Pos Value) (p : Pos)
    (r : Result Pos Value) (r' : Restart Pos) : Prop where
  anchor : r'.anchor = p
  stale : r'.StaleOk S₀
  /-- Every reader is another cell, on whose behalf something was read. -/
  readers : ∀ x ∈ r'.readers, x ≠ p ∧
    ((∃ q, st.seenEmpty q = some x ∨ st.seenOccupied q = some x) ∨ st.root = some x)
  nonempty : r'.learns = true → r'.readers ≠ []
  nodup : r'.readers.Nodup
  /-- A scalar commit restarts only over stale cells of `p`, which are in
  the original sheet. -/
  spill : (∃ v, r = .scalar v) → ∃ q v, S₀ q = .spill p v

section
variable (S₀ : Sheet Pos Value) (U : Universe Pos)

theorem commit_run_formula (st : PassState Pos Value) (p : Pos) (r : Result Pos Value)
    (t : Formula Pos Value) (v : Value) (h : st.sheet p = .formula t v) :
    ((commit U p r).run st).2 =
      { st with sheet := Function.update st.sheet p (.formula t (r.valueAt p)) } := by
  simp only [commit, StateM.run_getBind]
  rw [h]
  rfl

/-- The formula-cell case: only the stored value at `p` changes. -/
theorem commit_spec_formula (st : PassState Pos Value) (p : Pos) (r : Result Pos Value)
    (hinv : PassInv S₀ st) (htop : st.stack.head? = some p) (hr : ResultOf st p r)
    (t : Formula Pos Value) (v : Value) (hp : st.sheet p = .formula t v) :
    CommitOk S₀ st ((commit U p r).run st).2 p := by
  rw [commit_run_formula U st p r t v hp]
  have hpev : st.cells p = some .evaluating := cells_of_head hinv htop
  have hstack : st.stack = p :: st.stack.tail := (List.cons_head?_tail htop).symm
  have hnp : ¬ Protected st p :=
    not_protected_of_evaluating hinv (by simp [hp, Content.formula?]) hpev
  -- `p` is not an anchor: no spill cell points at it.
  have hnospill : ∀ q a w, st.sheet q = .spill a w → a ≠ p := by
    intro q a w hq hap
    have := hinv.no_orphans q a w hq
    rw [hap, hp] at this
    simp [Content.isAnchor] at this
  set S' := Function.update st.sheet p (.formula t (r.valueAt p)) with hS'
  have hS'ne : ∀ q, q ≠ p → S' q = st.sheet q := fun q hq => Function.update_of_ne hq _ _
  have hS'p : S' p = .formula t (r.valueAt p) := Function.update_self _ _ _
  set m := markEvaluated { st with sheet := S' } p with hm
  have hm_sheet : m.sheet = S' := rfl
  have hm_cells : ∀ q, q ≠ p → m.cells q = st.cells q := fun q hq => Function.update_of_ne hq _ _
  have hm_cellsp : m.cells p = some .evaluated := Function.update_self _ _ _
  have hm_stack : m.stack = st.stack.tail := rfl
  have hm_rest : m.restart = st.restart := rfl
  have hm_se : m.seenEmpty = st.seenEmpty := rfl
  have hm_so : m.seenOccupied = st.seenOccupied := rfl
  have hm_circ : m.circular = st.circular := rfl
  -- Views agree away from `p`.
  have hview : ∀ q, q ≠ p → passView m q = passView st q := by
    intro q hq
    unfold passView
    rw [hm_sheet, hS'ne q hq]
    cases hc : st.sheet q with
    | spill a w => simp only [hm_cells a (hnospill q a w hc), valueAt, hS'ne q hq]
    | _ => simp only [valueAt, hS'ne q hq]
  have hne_of_prot : ∀ q, Protected st q → q ≠ p := fun q hq hqp => hnp (hqp ▸ hq)
  have hprot : ∀ q, Protected st q → Protected m q := by
    intro q hq
    have hqp := hne_of_prot q hq
    rcases hq with ⟨w, h⟩ | ⟨h1, h2⟩ | ⟨a, w, h1, h2⟩ | h
    · exact Or.inl ⟨w, by rw [hm_sheet, hS'ne q hqp, h]⟩
    · exact Or.inr (Or.inl ⟨by rw [hm_sheet, hS'ne q hqp]; exact h1, by rw [hm_cells q hqp]; exact h2⟩)
    · exact Or.inr (Or.inr (Or.inl ⟨a, w, by rw [hm_sheet, hS'ne q hqp]; exact h1,
        by rw [hm_cells a (hnospill q a w h1)]; exact h2⟩))
    · exact Or.inr (Or.inr (Or.inr h))
  -- What the formula at `p` gives.
  have hrun : (r = .scalar circ) ∨
      (r = t.runPure (passView m) ∧ ∀ q ∈ t.reads (passView m), Protected st q) := by
    rcases hr with h | ⟨t', ht', hrt, hreads⟩
    · exact Or.inl h
    · right
      rw [hp] at ht'
      simp only [Content.formula?, Option.some.injEq] at ht'
      rw [← ht'] at hrt hreads
      have hag : ∀ q ∈ t.reads (passView st), passView st q = passView m q :=
        fun q hq => (hview q (hne_of_prot q (hreads q hq))).symm
      refine ⟨hrt.trans (Formula.runPure_congr t hag), ?_⟩
      rw [← Formula.reads_congr t hag]
      exact hreads
  have hshape : SameShape S₀ S' := by
    intro q
    by_cases hq : q = p
    · subst hq
      have := hinv.shape q
      rw [hS'p]
      revert this
      cases S₀ q <;> simp [hp, Content.isEmpty, Content.spillAnchor?]
    · rw [hS'ne q hq]
      exact hinv.shape q
  have hm_root : m.root = st.root := rfl
  have hinv' : PassInv S₀ m :=
    { not_abandoned := hm_rest ▸ hinv.not_abandoned
      root_cell := by
        intro c hc hne
        rw [hm_root] at hc
        have hne' : st.stack ≠ [] := fun h => hne (by rw [hm_stack, h]; rfl)
        by_cases hcp : c = p
        · subst hcp
          left
          rw [hm_cellsp]
          simp
        · rcases hinv.root_cell c hc hne' with h | h
          · left
            rw [hm_cells c hcp]
            exact h
          · right
            rw [hm_sheet, hS'ne c hcp]
            exact h
      orig_spill := by
        intro q a v hq hne
        rw [hm_sheet] at hq
        have hqp : q ≠ p := by
          rintro rfl
          rw [hS'p] at hq
          cases hq
        rw [hS'ne q hqp] at hq
        rw [hm_cells a (hnospill q a v hq)] at hne
        exact hinv.orig_spill q a v hq hne
      shape := hshape
      no_orphans := by
        intro q a w hq
        rw [hm_sheet] at hq ⊢
        have hqp : q ≠ p := by
          rintro rfl
          rw [hS'p] at hq
          cases hq
        rw [hS'ne q hqp] at hq
        rw [hS'ne a (hnospill q a w hq)]
        exact hinv.no_orphans q a w hq
      cse_areas := by
        intro q t' area v' hq x hx hxq
        rw [hm_sheet] at hq ⊢
        have hqp : q ≠ p := by
          rintro rfl
          rw [hS'p] at hq
          cases hq
        rw [hS'ne q hqp] at hq
        obtain ⟨w, hw⟩ := hinv.cse_areas q t' area v' hq x hx hxq
        have hxp : x ≠ p := by
          rintro rfl
          rw [hp] at hw
          cases hw
        exact ⟨w, by rw [hS'ne x hxp]; exact hw⟩
      cse_spills := by
        intro q a w hq t' area v' ha
        rw [hm_sheet] at hq ha
        have hqp : q ≠ p := by
          rintro rfl
          rw [hS'p] at hq
          cases hq
        rw [hS'ne q hqp] at hq
        have hap := hnospill q a w hq
        rw [hS'ne a hap] at ha
        exact hinv.cse_spills q a w hq t' area v' ha
      stack_nodup := by
        rw [hm_stack]
        exact hinv.stack_nodup.sublist (List.tail_sublist _)
      stack_evaluating := by
        intro q
        rw [hm_stack]
        by_cases hq : q = p
        · subst hq
          rw [hm_cellsp]
          have hnd := hinv.stack_nodup
          rw [hstack] at hnd
          simp [List.nodup_cons.mp hnd |>.1]
        · rw [hm_cells q hq]
          rw [← hinv.stack_evaluating q]
          conv_rhs => rw [hstack]
          simp [hq]
      seen_empty := by
        intro q r' hq
        rw [hm_se] at hq
        rcases hinv.seen_empty q r' hq with h | ⟨a, w, h1, h2, h3⟩
        · left
          have hqp : q ≠ p := by
            rintro rfl
            rw [hp] at h
            simp [Content.isEmpty] at h
          rw [hm_sheet, hS'ne q hqp]
          exact h
        · right
          have hqp : q ≠ p := by
            rintro rfl
            rw [hp] at h1
            cases h1
          have hap := hnospill q a w h1
          exact ⟨a, w, by rw [hm_sheet, hS'ne q hqp]; exact h1,
            by rw [hm_sheet, hS'ne a hap]; exact h2, by rw [hm_cells a hap]; exact h3⟩
      seen_occupied := by
        intro q r' hq
        rw [hm_so] at hq
        obtain ⟨a, w, h⟩ := hinv.seen_occupied q r' hq
        have hqp : q ≠ p := by
          rintro rfl
          rw [hp] at h
          cases h
        exact ⟨a, w, by rw [hm_sheet, hS'ne q hqp]; exact h⟩
      evaluated_consistent := by
        intro q hq
        by_cases hqp : q = p
        · subst hqp
          rw [hm_sheet]
          unfold ConsistentAtWith
          rw [hS'p]
          intro hne
          rcases hrun with h | ⟨h, _⟩
          · rw [h] at hne
            exact absurd rfl hne
          · rw [h]
        · rw [hm_cells q hqp] at hq
          refine ConsistentAtWith.transport (hinv.evaluated_consistent q hq)
            (by rw [hm_sheet, hS'ne q hqp]) ?_ ?_ ?_ ?_
          · intro hne t' ht' x hx
            exact (hview x (hne_of_prot x (hinv.reads_protected q t' hq ht' hne x hx))).symm
          · intro x w
            rw [hm_sheet]
            by_cases hxp : x = p
            · subst hxp
              rw [hS'p, hp]
              simp
            · rw [hS'ne x hxp]
          · intro x
            rw [hm_sheet]
            by_cases hxp : x = p
            · subst hxp
              rw [hS'p, hp]
              simp [Content.spillAnchor?]
            · rw [hS'ne x hxp]
          · intro area ⟨x, hx, hxq, h⟩
            refine ⟨x, hx, hxq, ?_⟩
            rw [hm_sheet, hm_so]
            by_cases hxp : x = p
            · subst hxp
              rw [hS'p]
              rcases h with ⟨_, _⟩ | ⟨a, w, h1, _⟩
              · exact Or.inl ⟨rfl, rfl⟩
              · rw [hp] at h1
                cases h1
            · rw [hS'ne x hxp]
              exact h
      reads_protected := by
        intro q t' hq ht' hne x hx
        by_cases hqp : q = p
        · subst hqp
          rw [hm_sheet] at ht' hne
          rw [hS'p] at ht'
          have hne' : r.valueAt q ≠ circ := by simpa [valueAt, hS'p] using hne
          simp only [Content.formula?, Option.some.injEq] at ht'
          rw [← ht'] at hx
          rcases hrun with h | ⟨_, hreads⟩
          · rw [h] at hne'
            exact absurd rfl hne'
          · exact hprot x (hreads x hx)
        · rw [hm_cells q hqp] at hq
          rw [hm_sheet] at ht' hne
          rw [hS'ne q hqp] at ht'
          have hval : valueAt S' q = valueAt st.sheet q := by simp only [valueAt, hS'ne q hqp]
          rw [hval] at hne
          have hag : ∀ y ∈ t'.reads (passView st), passView st y = passView m y :=
            fun y hy => (hview y (hne_of_prot y (hinv.reads_protected q t' hq ht' hne y hy))).symm
          rw [← Formula.reads_congr t' hag] at hx
          exact hprot x (hinv.reads_protected q t' hq ht' hne x hx) }
  refine ⟨hinv.not_abandoned, ?_, hinv', rfl, rfl⟩
  exact
    { restart_mono := fun h => hm_rest ▸ h
      cells_mono := fun q hq => by
        have hqp : q ≠ p := by
          rintro rfl
          rw [hpev] at hq
          cases hq
        rw [hm_cells q hqp]
        exact hq
      circular_mono := hm_circ ▸ Finset.Subset.refl _
      seenEmpty_mono := fun q r' h => hm_se ▸ h
      seenOccupied_mono := fun q r h => hm_so ▸ h
      root_eq := rfl
      roots_new_empty := fun q r h => Or.inl (hm_se ▸ h)
      roots_new_occupied := fun q r h => Or.inl (hm_so ▸ h)
      inv := fun _ => Or.inr hinv'
      protect := fun _ _ q hq => ⟨hprot q hq, hview q (hne_of_prot q hq)⟩ }

end

/-- The result at `p`, seen from the marked state: what the formula gives
against the new view, with every read protected, unless the cell is marked. -/
theorem ResultOf.transfer {S₀ : Sheet Pos Value} {st : PassState Pos Value} {p : Pos}
    {S' : Sheet Pos Value} {SO' : Pos → Option Pos} (hf : ChangeFacts S₀ st p S' SO')
    {r : Result Pos Value} (hr : ResultOf st p r) {t : Formula Pos Value}
    (ht : (st.sheet p).formula? = some t) :
    r = .scalar circ ∨
      (r = t.runPure (passView (commitState st p S' SO')) ∧
        ∀ x ∈ t.reads (passView (commitState st p S' SO')), Protected (commitState st p S' SO') x) := by
  rcases hr with h | ⟨t', ht', hrt, hreads⟩
  · exact Or.inl h
  · right
    rw [ht] at ht'
    simp only [Option.some.injEq] at ht'
    subst ht'
    have hag : ∀ q ∈ t.reads (passView st), passView st q = passView (commitState st p S' SO') q :=
      fun q hq => (hf.view q (hreads q hq)).symm
    refine ⟨hrt.trans (Formula.runPure_congr t hag), ?_⟩
    rw [← Formula.reads_congr t hag]
    exact fun x hx => hf.prot x (hreads x hx)

/-- A spill cell of a CSE anchor is never on record as read empty. -/
theorem seenEmpty_none_of_cse {S₀ : Sheet Pos Value} {st : PassState Pos Value}
    (hinv : PassInv S₀ st) {p : Pos} {t : Formula Pos Value} {area : List Pos} {v : Value}
    (hp : st.sheet p = .cseAnchor t area v) {x : Pos} {w : Value} (hx : st.sheet x = .spill p w) :
    st.seenEmpty x = none := by
  cases hse : st.seenEmpty x with
  | none => rfl
  | some r =>
    rcases hinv.seen_empty x r hse with he | ⟨a, w', h1, hdyn, _⟩
    · rw [hx] at he
      simp [Content.isEmpty] at he
    · rw [hx] at h1
      cases h1
      rw [hp] at hdyn
      simp [Content.isDynAnchor] at hdyn

section
variable (S₀ : Sheet Pos Value) (U : Universe Pos)

theorem commit_run_cse (st : PassState Pos Value) (p : Pos) (r : Result Pos Value)
    (t : Formula Pos Value) (area : List Pos) (v : Value) (h : st.sheet p = .cseAnchor t area v) :
    ((commit U p r).run st).2 =
      { st with sheet := (Sheet.writeAll (Function.update st.sheet p (.cseAnchor t area (r.valueAt p)))
          ((area.filter fun q => decide (q ≠ p)).map fun q => (q, .spill p (r.valueAt q)))) } := by
  simp only [commit, StateM.run_getBind]
  rw [h]
  rfl

/-- The CSE case: the anchor and its fixed area are rewritten. -/
theorem commit_spec_cse (st : PassState Pos Value) (p : Pos) (r : Result Pos Value)
    (hinv : PassInv S₀ st) (htop : st.stack.head? = some p) (hr : ResultOf st p r)
    (t : Formula Pos Value) (area : List Pos) (v : Value) (hp : st.sheet p = .cseAnchor t area v) :
    CommitOk S₀ st ((commit U p r).run st).2 p := by
  rw [commit_run_cse U st p r t area v hp]
  set W := area.filter fun q => decide (q ≠ p) with hW
  set S' := Sheet.writeAll (Function.update st.sheet p (.cseAnchor t area (r.valueAt p)))
    (W.map fun q => (q, .spill p (r.valueAt q))) with hS'
  have hpW : p ∉ W := by simp [hW]
  have hS'W : ∀ x ∈ W, S' x = .spill p (r.valueAt x) := by
    intro x hx
    rw [hS', Sheet.writeAll_map]
    simp [hx]
  have hS'p : S' p = .cseAnchor t area (r.valueAt p) := by
    rw [hS', Sheet.writeAll_map]
    simp [hpW]
  have hS'ne : ∀ x, x ≠ p → x ∉ W → S' x = st.sheet x := by
    intro x hxp hxW
    rw [hS', Sheet.writeAll_map]
    simp [hxW, Function.update_of_ne hxp]
  have hspill_none : ∀ x w, st.sheet x = .spill p w → st.seenEmpty x = none :=
    fun x w hx => seenEmpty_none_of_cse hinv hp hx
  have hchange : SheetChange st p S' st.seenOccupied W [] :=
    { so_new := fun _ _ h => Or.inl h
      p_kind := by
        rw [hp, hS'p]
        exact ⟨_, rfl⟩
      untouched := fun x hxp hxW _ => hS'ne x hxp hxW
      written := by
        intro x hx
        have hxp : x ≠ p := by simpa [hW] using (List.mem_filter.mp hx).2
        have hxa : x ∈ area := (List.mem_filter.mp hx).1
        obtain ⟨w, hw⟩ := hinv.cse_areas p t area v hp x hxa hxp
        exact ⟨hxp, ⟨_, hS'W x hx⟩, by simp [Content.freeFor, hw], hspill_none x w hw⟩
      cleared := by simp
      fresh_unrecorded := by
        intro x w hx
        by_cases hxp : x = p
        · subst hxp
          rw [hS'p] at hx
          cases hx
        by_cases hxW : x ∈ W
        · have hxa : x ∈ area := (List.mem_filter.mp hxW).1
          obtain ⟨w', hw'⟩ := hinv.cse_areas p t area v hp x hxa hxp
          exact hspill_none x w' hw'
        · rw [hS'ne x hxp hxW] at hx
          exact hspill_none x w hx
      p_anchor_of_written := fun _ => by simp [hS'p, Content.isAnchor]
      written_in_cse := by
        intro x hx t' area' v' h
        rw [hS'p] at h
        cases h
        exact (List.mem_filter.mp hx).1
      cleared_dyn := fun h => absurd rfl h
      so_mono := fun _ _ h => h
      so_spill := by
        intro x r' hx
        obtain ⟨a, w, hw⟩ := hinv.seen_occupied x r' hx
        by_cases hxW : x ∈ W
        · exact ⟨p, _, hS'W x hxW⟩
        · have hxp : x ≠ p := by
            rintro rfl
            rw [hp] at hw
            cases hw
          exact ⟨a, w, by rw [hS'ne x hxp hxW]; exact hw⟩ }
  have hf := hchange.facts hinv htop
  have ht : (st.sheet p).formula? = some t := by simp [hp, Content.formula?]
  have hres := ResultOf.transfer hf hr ht
  have hcons : ConsistentAtWith (passView (commitState st p S' st.seenOccupied))
      (StableBlocked (commitState st p S' st.seenOccupied)) S' p := by
    unfold ConsistentAtWith
    rw [hS'p]
    intro hne
    rcases hres with h | ⟨h, _⟩
    · rw [h] at hne
      exact absurd rfl hne
    · rw [← h]
      refine ⟨rfl, fun q hq hqp => ?_⟩
      exact hS'W q (List.mem_filter.mpr ⟨hq, by simp [hqp]⟩)
  have hreads : ∀ t', (S' p).formula? = some t' → valueAt S' p ≠ circ →
      ∀ x ∈ t'.reads (passView (commitState st p S' st.seenOccupied)),
        Protected (commitState st p S' st.seenOccupied) x := by
    intro t' ht' hne x hx
    rw [hS'p] at ht'
    simp only [Content.formula?, Option.some.injEq] at ht'
    rw [← ht'] at hx
    have hne' : r.valueAt p ≠ circ := by simpa [valueAt, hS'p] using hne
    rcases hres with h | ⟨_, hreads⟩
    · rw [h] at hne'
      exact absurd rfl hne'
    · exact hreads x hx
  obtain ⟨hstep, hinv'⟩ := hchange.commitOk hinv htop hcons hreads
  exact ⟨hinv.not_abandoned, hstep, hinv', rfl, rfl⟩

/-! ## Dynamic anchors -/

omit [ValueSort Value] in
theorem spillContradictsARead_run (anchor : Pos) (writes clears : List Pos)
    (st : PassState Pos Value) :
    (spillContradictsARead (Value := Value) anchor writes clears).run st =
      if (writes.filterMap st.seenEmpty ++ clears.filterMap st.seenOccupied).isEmpty then (false, st)
      else (true, { st with restart := (some
        (if (((writes.filterMap st.seenEmpty ++ clears.filterMap st.seenOccupied).filter
            fun r => decide (r ≠ anchor)).dedup).isEmpty
          then (if (writes.filterMap st.seenEmpty).isEmpty then .staleCells anchor clears
            else .selfContradiction anchor)
          else .conflict anchor (((writes.filterMap st.seenEmpty ++
            clears.filterMap st.seenOccupied).filter fun r => decide (r ≠ anchor)).dedup))) }) := by
  simp only [spillContradictsARead, StateM.run_getBind]
  try dsimp only
  split <;> rfl

omit [ValueSort Value] in
theorem storeScalar_run (st : PassState Pos Value) (p : Pos) (t : Formula Pos Value) (v : Value) :
    ((storeScalar U p t v).run st).2 =
      let clears := (ownSpillCells U st.sheet p).filter fun q => decide (q ≠ p)
      if (clears.filterMap st.seenOccupied).isEmpty then
        { st with sheet := (Sheet.writeAll (Function.update st.sheet p (.dynAnchor t v))
            (clears.map fun q => (q, .empty))) }
      else { st with restart := (some
        (if (((clears.filterMap st.seenOccupied).filter fun r => decide (r ≠ p)).dedup).isEmpty
          then .staleCells p clears
          else .conflict p (((clears.filterMap st.seenOccupied).filter
            fun r => decide (r ≠ p)).dedup))) } := by
  simp only [storeScalar, StateM.run_getBind]
  try dsimp only
  rw [StateT.run_bind, spillContradictsARead_run]
  simp only [List.filterMap_nil, List.nil_append, List.isEmpty_nil, ↓reduceIte]
  split <;> rfl

/-- `storeScalar` either abandons the pass, naming `p`, or is a `SheetChange`
that clears every spill cell of `p`. -/
theorem storeScalar_spec (st : PassState Pos Value) (p : Pos) (t : Formula Pos Value) (v v₀ : Value)
    (hinv : PassInv S₀ st) (htop : st.stack.head? = some p) (hp : st.sheet p = .dynAnchor t v₀) :
    (∃ r, (((storeScalar U p t v).run st).2).restart = some r ∧
        Abandoned S₀ st p (.scalar v) r) ∨
      ∃ S' C, ((storeScalar U p t v).run st).2 = { st with sheet := S' } ∧
        SheetChange st p S' st.seenOccupied [] C ∧ S' p = .dynAnchor t v ∧
        ∀ x, (S' x).spillAnchor? ≠ some p := by
  rw [storeScalar_run]
  try dsimp only
  set clears := (ownSpillCells U st.sheet p).filter fun q => decide (q ≠ p) with hclears
  have hmem : ∀ x, x ∈ clears ↔ x ≠ p ∧ ∃ w, st.sheet x = .spill p w := by
    intro x
    rw [hclears, List.mem_filter, ownSpillCells, List.mem_filter]
    constructor
    · rintro ⟨⟨_, h1⟩, h2⟩
      refine ⟨by simpa using h2, ?_⟩
      cases hs : st.sheet x <;> simp_all [Content.spillAnchor?]
    · rintro ⟨h1, w, hw⟩
      exact ⟨⟨U.complete x, by simp [hw, Content.spillAnchor?]⟩, by simp [h1]⟩
  split
  · rename_i hroots
    right
    have hnone : ∀ x ∈ clears, st.seenOccupied x = none := by
      intro x hx
      rw [List.isEmpty_iff, List.filterMap_eq_nil_iff] at hroots
      exact hroots x hx
    have hpC : p ∉ clears := fun h => ((hmem p).mp h).1 rfl
    set S' := Sheet.writeAll (Function.update st.sheet p (.dynAnchor t v))
      (clears.map fun q => (q, .empty)) with hS'
    have hS'C : ∀ x ∈ clears, S' x = .empty := by
      intro x hx
      rw [hS', Sheet.writeAll_map]
      simp [hx]
    have hS'p : S' p = .dynAnchor t v := by
      rw [hS', Sheet.writeAll_map]
      simp [hpC]
    have hS'ne : ∀ x, x ≠ p → x ∉ clears → S' x = st.sheet x := by
      intro x hxp hxC
      rw [hS', Sheet.writeAll_map]
      simp [hxC, Function.update_of_ne hxp]
    have hnospill : ∀ x, (S' x).spillAnchor? ≠ some p := by
      intro x hx
      by_cases hxp : x = p
      · subst hxp
        rw [hS'p] at hx
        simp [Content.spillAnchor?] at hx
      by_cases hxC : x ∈ clears
      · rw [hS'C x hxC] at hx
        simp [Content.spillAnchor?] at hx
      rw [hS'ne x hxp hxC] at hx
      apply hxC
      rw [hmem]
      refine ⟨hxp, ?_⟩
      cases hs : st.sheet x <;> simp_all [Content.spillAnchor?]
    refine ⟨S', clears, rfl, ?_, hS'p, hnospill⟩
    exact
      { p_kind := by
          rw [hp, hS'p]
          exact ⟨_, rfl⟩
        untouched := fun x hxp _ hxC => hS'ne x hxp hxC
        written := by simp
        cleared := fun x hx => ⟨((hmem x).mp hx).1, hS'C x hx, ((hmem x).mp hx).2, hnone x hx⟩
        fresh_unrecorded := fun x w hx => absurd (by simp [hx, Content.spillAnchor?]) (hnospill x)
        p_anchor_of_written := fun h => absurd rfl h
        written_in_cse := by simp
        cleared_dyn := fun _ => by simp [hp, Content.isDynAnchor]
        so_mono := fun _ _ h => h
        so_spill := by
          intro x r hx
          obtain ⟨a, w, hw⟩ := hinv.seen_occupied x r hx
          have hxp : x ≠ p := by
            rintro rfl
            rw [hp] at hw
            cases hw
          have hxC : x ∉ clears := fun h => by
            rw [hnone x h] at hx
            cases hx
          exact ⟨a, w, by rw [hS'ne x hxp hxC]; exact hw⟩
        so_new := fun _ _ h => Or.inl h }
  · left
    rename_i hroots
    -- One of the cleared cells was on record as blocking, so it is a spill
    -- cell of `p`, which has not committed: it is in the original sheet.
    have hstale : ∃ q ∈ clears, ∃ v, S₀ q = .spill p v := by
      rw [Bool.not_eq_true, List.isEmpty_eq_false_iff_exists_mem] at hroots
      obtain ⟨r', hr'⟩ := hroots
      obtain ⟨x, hx, _⟩ := List.mem_filterMap.mp hr'
      obtain ⟨_, w, hw⟩ := (hmem x).mp hx
      exact ⟨x, hx, w, hinv.orig_spill x p w hw (by rw [cells_of_head hinv htop]; simp)⟩
    have hspill : ∃ q v, S₀ q = .spill p v := by
      obtain ⟨q, _, v, hv⟩ := hstale
      exact ⟨q, v, hv⟩
    have hreaders : ∀ x ∈ ((clears.filterMap st.seenOccupied).filter
        fun r => decide (r ≠ p)).dedup, x ≠ p ∧ ∃ q, st.seenOccupied q = some x := by
      intro x hx
      rw [List.mem_dedup, List.mem_filter] at hx
      obtain ⟨hx, hne⟩ := hx
      obtain ⟨q, _, hqx⟩ := List.mem_filterMap.mp hx
      exact ⟨by simpa using hne, q, hqx⟩
    refine ⟨_, rfl, ?_⟩
    split
    · rename_i hempty
      exact
        { anchor := rfl
          stale := hstale
          readers := by simp [Restart.readers]
          nonempty := by simp [Restart.learns]
          nodup := by simp [Restart.readers]
          spill := fun _ => hspill }
    · rename_i hne
      exact
        { anchor := rfl
          stale := trivial
          readers := fun x hx => ⟨(hreaders x hx).1,
            Or.inl ⟨(hreaders x hx).2.choose, Or.inr (hreaders x hx).2.choose_spec⟩⟩
          nonempty := fun _ h => hne (List.isEmpty_iff.mpr h)
          nodup := List.nodup_dedup _
          spill := fun _ => hspill }

theorem commit_run_dyn (st : PassState Pos Value) (p : Pos) (r : Result Pos Value)
    (t : Formula Pos Value) (v : Value) (h : st.sheet p = .dynAnchor t v) :
    (commit U p r).run st =
      (match r with
        | .scalar v => storeScalar U p t v
        | .array area vals => spillDynamicArray U p t area vals).run st := by
  simp only [commit, StateM.run_getBind]
  rw [h]
  rfl

/-- The dynamic-anchor case with a scalar result. -/
theorem commit_spec_dyn_scalar (st : PassState Pos Value) (p : Pos) (v : Value)
    (hinv : PassInv S₀ st) (htop : st.stack.head? = some p) (hr : ResultOf st p (.scalar v))
    (t : Formula Pos Value) (v₀ : Value) (hp : st.sheet p = .dynAnchor t v₀) :
    (∃ r, (((commit U p (.scalar v)).run st).2).restart = some r ∧
        Abandoned S₀ st p (.scalar v) r) ∨
      CommitOk S₀ st ((commit U p (.scalar v)).run st).2 p := by
  rw [commit_run_dyn U st p _ t v₀ hp]
  try dsimp only
  rcases storeScalar_spec S₀ U st p t v v₀ hinv htop hp with h | ⟨S', C, hst, hchange, hS'p, hnospill⟩
  · exact Or.inl h
  · right
    rw [hst]
    have hf := hchange.facts hinv htop
    have ht : (st.sheet p).formula? = some t := by simp [hp, Content.formula?]
    have hres := ResultOf.transfer hf hr ht
    have hcons : ConsistentAtWith (passView (commitState st p S' st.seenOccupied))
        (StableBlocked (commitState st p S' st.seenOccupied)) S' p := by
      unfold ConsistentAtWith
      rw [hS'p]
      intro hne
      rcases hres with h | ⟨h, _⟩
      · simp only [Result.scalar.injEq] at h
        exact absurd h hne
      · rw [← h]
        exact ⟨rfl, hnospill⟩
    have hreads : ∀ t', (S' p).formula? = some t' → valueAt S' p ≠ circ →
        ∀ x ∈ t'.reads (passView (commitState st p S' st.seenOccupied)),
          Protected (commitState st p S' st.seenOccupied) x := by
      intro t' ht' hne x hx
      rw [hS'p] at ht'
      simp only [Content.formula?, Option.some.injEq] at ht'
      rw [← ht'] at hx
      have hne' : v ≠ circ := by simpa [valueAt, hS'p] using hne
      rcases hres with h | ⟨_, hreads⟩
      · simp only [Result.scalar.injEq] at h
        exact absurd h hne'
      · exact hreads x hx
    obtain ⟨hstep, hinv'⟩ := hchange.commitOk hinv htop hcons hreads
    exact ⟨hinv.not_abandoned, hstep, hinv', rfl, rfl⟩

end

/-! ## The array case -/

omit [ValueSort Value] in
/-- `recordSeen q .occupied` at one state. -/
theorem recordSeen_occupied_run (q : Pos) (s : PassState Pos Value) :
    ∃ s', (recordSeen (Value := Value) q .occupied).run s = ((), s') ∧
      s'.sheet = s.sheet ∧ s'.cells = s.cells ∧ s'.stack = s.stack ∧ s'.circular = s.circular ∧
      s'.seenEmpty = s.seenEmpty ∧ s'.restart = s.restart ∧
      (∀ x r, s.seenOccupied x = some r → s'.seenOccupied x = some r) ∧
      (∀ x r, s'.seenOccupied x = some r →
        s.seenOccupied x = some r ∨ (x = q ∧ s.root = some r)) ∧
      (s.root.isSome → (s'.seenOccupied q).isSome) ∧ s'.root = s.root := by
  simp only [recordSeen, StateM.run_getBind]
  split
  · rename_i hroot
    exact ⟨s, rfl, rfl, rfl, rfl, rfl, rfl, rfl, fun _ _ h => h, fun _ _ h => Or.inl h,
      fun h => by simp [hroot] at h, rfl⟩
  · rename_i r hroot
    try dsimp only
    split
    · rename_i hso
      exact ⟨s, rfl, rfl, rfl, rfl, rfl, rfl, rfl, fun _ _ h => h, fun _ _ h => Or.inl h,
        fun _ => by simp [hso], rfl⟩
    · rename_i hso
      refine ⟨{ s with seenOccupied := Function.update s.seenOccupied q (some r) }, rfl, rfl, rfl,
        rfl, rfl, rfl, rfl, ?_, ?_, ?_, rfl⟩
      · intro x r' h
        by_cases hxq : x = q
        · subst hxq
          rw [hso] at h
          cases h
        · simp [Function.update_of_ne hxq, h]
      · intro x r' h
        by_cases hxq : x = q
        · subst hxq
          have h' : Function.update s.seenOccupied x (some r) x = some r' := h
          rw [Function.update_self] at h'
          cases h'
          exact Or.inr ⟨rfl, hroot⟩
        · left
          simpa [Function.update_of_ne hxq] using h
      · intro _
        simp

omit [ValueSort Value] in
/-- The blocking scan at one state: only occupied records change, and only
at targets holding another array's spill cell. -/
theorem recordBlockers_run (st : PassState Pos Value) (p : Pos) :
    ∀ (l : List Pos) (s : PassState Pos Value), ∃ s',
      (recordBlockers st p l).run s = ((), s') ∧
      s'.sheet = s.sheet ∧ s'.cells = s.cells ∧ s'.stack = s.stack ∧ s'.circular = s.circular ∧
      s'.seenEmpty = s.seenEmpty ∧ s'.restart = s.restart ∧
      (∀ x r, s.seenOccupied x = some r → s'.seenOccupied x = some r) ∧
      (∀ x r, s'.seenOccupied x = some r →
        s.seenOccupied x = some r ∨
          (x ∈ l ∧ (∃ a w, st.sheet x = .spill a w ∧ a ≠ p) ∧ s.root = some r)) ∧
      (s.root.isSome → ∀ x ∈ l, (∃ a w, st.sheet x = .spill a w ∧ a ≠ p) →
        (s'.seenOccupied x).isSome) ∧ s'.root = s.root
  | [], s => ⟨s, rfl, rfl, rfl, rfl, rfl, rfl, rfl, fun _ _ h => h, fun _ _ h => Or.inl h,
      fun _ x hx => absurd hx List.not_mem_nil, rfl⟩
  | q :: l, s => by
      simp only [recordBlockers, StateT.run_bind]
      cases hsa : (st.sheet q).spillAnchor? with
      | none =>
          obtain ⟨s', hrun, h1, h2, h3, h4, h5, h6, h7, h8, h9, h10⟩ := recordBlockers_run st p l s
          refine ⟨s', ?_, h1, h2, h3, h4, h5, h6, h7, ?_, ?_, h10⟩
          · rw [StateT.run_pure, Id.bind_apply]
            exact hrun
          · intro x r h
            rcases h8 x r h with h | ⟨hx, hf, hr⟩
            · exact Or.inl h
            · exact Or.inr ⟨List.mem_cons_of_mem q hx, hf, hr⟩
          · intro hroot x hx hf
            rcases List.mem_cons.mp hx with rfl | hx
            · obtain ⟨a, w, hw, _⟩ := hf
              rw [hw] at hsa
              cases hsa
            · exact h9 hroot x hx hf
      | some a =>
          try dsimp only
          by_cases hap : a ≠ p
          · simp only [hap, ne_eq, not_false_eq_true, ↓reduceIte]
            obtain ⟨s₁, hrun₁, g1, g2, g3, g4, g5, g6, g7, g8, g9, g10⟩ :=
              recordSeen_occupied_run q s
            obtain ⟨s', hrun, h1, h2, h3, h4, h5, h6, h7, h8, h9, h10⟩ :=
              recordBlockers_run st p l s₁
            refine ⟨s', ?_, h1.trans g1, h2.trans g2, h3.trans g3, h4.trans g4, h5.trans g5,
              h6.trans g6, fun x r h => h7 x r (g7 x r h), ?_, ?_, h10.trans g10⟩
            · rw [hrun₁, Id.bind_apply]
              exact hrun
            · intro x r h
              rcases h8 x r h with h | ⟨hx, hf, hr⟩
              · rcases g8 x r h with h | ⟨rfl, hr⟩
                · exact Or.inl h
                · right
                  refine ⟨List.mem_cons_self, ?_, hr⟩
                  cases hs : st.sheet x <;> simp_all [Content.spillAnchor?]
              · exact Or.inr ⟨List.mem_cons_of_mem q hx, hf, g10 ▸ hr⟩
            · intro hroot x hx hf
              have hroot₁ : s₁.root.isSome := by
                rw [g10]
                exact hroot
              rcases List.mem_cons.mp hx with rfl | hx
              · obtain ⟨r, hr⟩ := Option.isSome_iff_exists.mp (g9 hroot)
                simp [h7 x r hr]
              · exact h9 hroot₁ x hx hf
          · have hap' : a = p := not_not.mp hap
            subst hap'
            simp only [ne_eq, not_true_eq_false, ↓reduceIte]
            obtain ⟨s', hrun, h1, h2, h3, h4, h5, h6, h7, h8, h9, h10⟩ :=
              recordBlockers_run st a l s
            refine ⟨s', ?_, h1, h2, h3, h4, h5, h6, h7, ?_, ?_, h10⟩
            · rw [StateT.run_pure, Id.bind_apply]
              exact hrun
            · intro x r h
              rcases h8 x r h with h | ⟨hx, hf, hr⟩
              · exact Or.inl h
              · exact Or.inr ⟨List.mem_cons_of_mem q hx, hf, hr⟩
            · intro hroot x hx hf
              rcases List.mem_cons.mp hx with rfl | hx
              · obtain ⟨a', w, hw, hap'⟩ := hf
                rw [hw] at hsa
                simp only [Content.spillAnchor?, Option.some.injEq] at hsa
                exact absurd hsa hap'
              · exact h9 hroot x hx hf

section
variable (S₀ : Sheet Pos Value)

/-- The blocking scan is a step of the pass. -/
theorem recordBlockers_step (st : PassState Pos Value) (p : Pos) :
    ∀ (l : List Pos) (s : PassState Pos Value), s.sheet = st.sheet →
      PassStep S₀ s ((recordBlockers st p l).run s).2
  | [], s, _ => PassStep.refl S₀ s
  | q :: l, s, hs => by
      simp only [recordBlockers, StateT.run_bind]
      cases hsa : (st.sheet q).spillAnchor? with
      | none =>
          rw [StateT.run_pure, Id.bind_apply]
          exact recordBlockers_step st p l s hs
      | some a =>
          try dsimp only
          by_cases hap : a ≠ p
          · simp only [hap, ne_eq, not_false_eq_true, ↓reduceIte]
            obtain ⟨s₁, hrun₁, g1, -⟩ := recordSeen_occupied_run q s
            have hstep₁ : PassStep S₀ s s₁ := by
              have := recordSeen_occupied_step S₀ q s (by
                rw [hs]
                cases hc : st.sheet q <;> simp_all [Content.spillAnchor?])
              rw [hrun₁] at this
              exact this
            rw [hrun₁, Id.bind_apply]
            exact hstep₁.trans (recordBlockers_step st p l s₁ (g1.trans hs))
          · have hap' : a = p := not_not.mp hap
            subst hap'
            simp only [ne_eq, not_true_eq_false, ↓reduceIte, StateT.run_pure, Id.bind_apply]
            exact recordBlockers_step st a l s hs

end

omit [DecidableEq Pos] in
/-- `ResultOf` only looks at the sheet, the cells and the empty records. -/
theorem ResultOf.of_same {st st' : PassState Pos Value} (hsheet : st'.sheet = st.sheet)
    (hcells : st'.cells = st.cells) (hse : st'.seenEmpty = st.seenEmpty) {p : Pos}
    {r : Result Pos Value} (h : ResultOf st p r) : ResultOf st' p r := by
  have hview : passView st' = passView st := by
    funext q
    simp [passView, hsheet, hcells]
  unfold ResultOf at h ⊢
  rw [hsheet, hview]
  rcases h with h | ⟨t, ht, hr, hprot⟩
  · exact Or.inl h
  · refine Or.inr ⟨t, ht, hr, fun q hq => ?_⟩
    have := hprot q hq
    unfold Protected at this ⊢
    rw [hsheet, hcells, hse]
    exact this

section
variable (S₀ : Sheet Pos Value) (U : Universe Pos)

/-- The dynamic-anchor case with an array result. -/
theorem commit_spec_dyn_array (st : PassState Pos Value) (p : Pos) (area : List Pos)
    (vals : Pos → Value) (hinv : PassInv S₀ st) (htop : st.stack.head? = some p)
    (hroot : st.root.isSome)
    (hr : ResultOf st p (.array area vals)) (t : Formula Pos Value) (v₀ : Value)
    (hp : st.sheet p = .dynAnchor t v₀) :
    (∃ r, (((commit U p (.array area vals)).run st).2).restart = some r ∧
        Abandoned S₀ st p (.array area vals) r) ∨
      CommitOk S₀ st ((commit U p (.array area vals)).run st).2 p := by
  rw [commit_run_dyn U st p _ t v₀ hp]
  try dsimp only
  simp only [spillDynamicArray, StateM.run_getBind]
  try dsimp only
  set targets := area.filter fun q => decide (q ≠ p) with htargets
  have hmem_t : ∀ x, x ∈ targets ↔ x ∈ area ∧ x ≠ p := by
    intro x
    simp [htargets, List.mem_filter]
  -- The scan.
  obtain ⟨st₁, hrun₁, h1sheet, h1cells, h1stack, h1circ, h1se, h1rest, h1so_mono, h1so_src,
    h1so_rec, h1root⟩ := recordBlockers_run st p targets st
  -- A restart raised from `st₁` says the same relative to `st`: the scan
  -- only added occupied records, on behalf of the root.
  have hback : ∀ (r : Result Pos Value) (r' : Restart Pos), Abandoned S₀ st₁ p r r' →
      Abandoned S₀ st p (.array area vals) r' := by
    intro r r' h
    refine ⟨h.anchor, h.stale, ?_, h.nonempty, h.nodup, fun ⟨v, hv⟩ => by cases hv⟩
    intro x hx
    obtain ⟨hne, hrec⟩ := h.readers x hx
    refine ⟨hne, ?_⟩
    rcases hrec with ⟨q, hq | hq⟩ | hq
    · rw [h1se] at hq
      exact Or.inl ⟨q, Or.inl hq⟩
    · rcases h1so_src q x hq with hq | ⟨_, _, hq⟩
      · exact Or.inl ⟨q, Or.inr hq⟩
      · exact Or.inr hq
    · rw [h1root] at hq
      exact Or.inr hq
  have hstep₀ : PassStep S₀ st st₁ := by
    have := recordBlockers_step S₀ st p targets st rfl
    rw [hrun₁] at this
    exact this
  have hinv₁ : PassInv S₀ st₁ := by
    rcases hstep₀.inv hinv with h | h
    · rw [h1rest, hinv.not_abandoned] at h
      cases h
    · exact h
  have htop₁ : st₁.stack.head? = some p := by rw [h1stack]; exact htop
  have hp₁ : st₁.sheet p = .dynAnchor t v₀ := by rw [h1sheet]; exact hp
  have hr₁ : ResultOf st₁ p (.array area vals) := ResultOf.of_same h1sheet h1cells h1se hr
  have ht₁ : (st₁.sheet p).formula? = some t := by simp [hp₁, Content.formula?]
  have hrest₁ : st₁.restart = none := by rw [h1rest]; exact hinv.not_abandoned
  rw [StateT.run_bind, hrun₁, Id.bind_apply]
  try dsimp only
  split
  · -- Blocked: `#SPILL!`, own leftovers removed.
    rename_i hany
    show (∃ r, (((storeScalar U p t spillError).run st₁).2).restart = some r ∧
        Abandoned S₀ st p (.array area vals) r) ∨
      CommitOk S₀ st (((storeScalar U p t spillError).run st₁).2) p
    rcases storeScalar_spec S₀ U st₁ p t spillError v₀ hinv₁ htop₁ hp₁
      with ⟨r', hr', hab⟩ | ⟨S', C, hst', hchange, hS'p, hnospill⟩
    · exact Or.inl ⟨r', hr', hback _ r' hab⟩
    right
    rw [hst']
    have hf := hchange.facts hinv₁ htop₁
    have hres := ResultOf.transfer hf hr₁ ht₁
    have hblocked : StableBlocked (commitState st₁ p S' st₁.seenOccupied) p area := by
      obtain ⟨q, hq, hnf⟩ := List.any_eq_true.mp hany
      obtain ⟨hqa, hqp⟩ := (hmem_t q).mp hq
      simp only [Bool.not_eq_eq_eq_not, Bool.not_true] at hnf
      refine ⟨q, hqa, hqp, ?_⟩
      show ((S' q).isEmpty = false ∧ (S' q).spillAnchor? = none) ∨
        ∃ a v, S' q = .spill a v ∧ a ≠ p ∧ (st₁.seenOccupied q).isSome
      cases hc : st.sheet q with
      | empty => simp [hc, Content.freeFor] at hnf
      | spill a w =>
          have hap : a ≠ p := by
            intro h
            simp [hc, Content.freeFor, h] at hnf
          have hu : S' q = st₁.sheet q := by
            apply hchange.untouched q hqp (by simp)
            intro hC
            obtain ⟨w', hw'⟩ := (hchange.cleared q hC).2.2.1
            rw [h1sheet, hc] at hw'
            cases hw'
            exact hap rfl
          rw [hu, h1sheet, hc]
          exact Or.inr ⟨a, w, rfl, hap, h1so_rec hroot q hq ⟨a, w, hc, hap⟩⟩
      | _ =>
          left
          have hu : S' q = st₁.sheet q := by
            apply hchange.untouched q hqp (by simp)
            intro hC
            obtain ⟨w', hw'⟩ := (hchange.cleared q hC).2.2.1
            rw [h1sheet, hc] at hw'
            cases hw'
          rw [hu, h1sheet, hc]
          simp [Content.isEmpty, Content.spillAnchor?]
    have hcons : ConsistentAtWith (passView (commitState st₁ p S' st₁.seenOccupied))
        (StableBlocked (commitState st₁ p S' st₁.seenOccupied)) S' p := by
      unfold ConsistentAtWith
      rw [hS'p]
      intro _
      rcases hres with h | ⟨h, _⟩
      · cases h
      · rw [← h]
        exact Or.inr ⟨hblocked, rfl, hnospill⟩
    have hreads : ∀ t', (S' p).formula? = some t' → valueAt S' p ≠ circ →
        ∀ x ∈ t'.reads (passView (commitState st₁ p S' st₁.seenOccupied)),
          Protected (commitState st₁ p S' st₁.seenOccupied) x := by
      intro t' ht' _ x hx
      rw [hS'p] at ht'
      simp only [Content.formula?, Option.some.injEq] at ht'
      rw [← ht'] at hx
      rcases hres with h | ⟨_, hreads⟩
      · cases h
      · exact hreads x hx
    obtain ⟨hstep, hinv'⟩ := hchange.commitOk hinv₁ htop₁ hcons hreads
    exact ⟨hrest₁, hstep₀.trans hstep, hinv', h1cells, h1stack⟩
  · -- Free: the check, then the write.
    rename_i hany
    have hfree : ∀ x ∈ targets, (st.sheet x).freeFor p = true := by
      intro x hx
      have := List.any_eq_false.mp (Bool.eq_false_iff.mpr hany) x hx
      simpa using this
    rw [StateT.run_bind, spillContradictsARead_run, Id.bind_apply]
    try dsimp only
    set clears := (ownSpillCells U st.sheet p).filter fun q => decide (q ∉ area) with hclears
    have hmem_c : ∀ x, x ∈ clears ↔ x ∉ area ∧ ∃ w, st.sheet x = .spill p w := by
      intro x
      rw [hclears, List.mem_filter, ownSpillCells, List.mem_filter]
      constructor
      · rintro ⟨⟨_, h1⟩, h2⟩
        refine ⟨by simpa using h2, ?_⟩
        cases hs : st.sheet x <;> simp_all [Content.spillAnchor?]
      · rintro ⟨h1, w, hw⟩
        exact ⟨⟨U.complete x, by simp [hw, Content.spillAnchor?]⟩, by simp [h1]⟩
    split
    · -- No contradiction: write.
      rename_i hroots
      rw [List.isEmpty_iff, List.append_eq_nil_iff] at hroots
      obtain ⟨hse_none, hso_none⟩ := hroots
      rw [List.filterMap_eq_nil_iff] at hse_none hso_none
      simp only [Bool.false_eq_true, ↓reduceIte, StateT.run_modify]
      try dsimp only
      right
      set S' := Sheet.writeAll (Sheet.writeAll (Function.update st₁.sheet p (.dynAnchor t (vals p)))
        (targets.map fun q => (q, .spill p (vals q)))) (clears.map fun q => (q, .empty)) with hS'
      have hdisj : ∀ x, x ∈ targets → x ∉ clears :=
        fun x hx hc => ((hmem_c x).mp hc).1 ((hmem_t x).mp hx).1
      have hpT : p ∉ targets := fun h => ((hmem_t p).mp h).2 rfl
      have hpC : p ∉ clears := fun h => by
        obtain ⟨w, hw⟩ := ((hmem_c p).mp h).2
        rw [hp] at hw
        cases hw
      have hS'C : ∀ x ∈ clears, S' x = .empty := by
        intro x hx
        rw [hS', Sheet.writeAll_map]
        simp [hx]
      have hS'T : ∀ x ∈ targets, S' x = .spill p (vals x) := by
        intro x hx
        rw [hS', Sheet.writeAll_map, Sheet.writeAll_map]
        simp [hx, hdisj x hx]
      have hS'p : S' p = .dynAnchor t (vals p) := by
        rw [hS', Sheet.writeAll_map, Sheet.writeAll_map]
        simp [hpT, hpC]
      have hS'ne : ∀ x, x ≠ p → x ∉ targets → x ∉ clears → S' x = st.sheet x := by
        intro x hxp hxT hxC
        rw [hS', Sheet.writeAll_map, Sheet.writeAll_map]
        simp [hxT, hxC, Function.update_of_ne hxp, h1sheet]
      have hnospill_out : ∀ x, x ∉ area → (S' x).spillAnchor? ≠ some p := by
        intro x hxa hx
        by_cases hxp : x = p
        · subst hxp
          rw [hS'p] at hx
          simp [Content.spillAnchor?] at hx
        by_cases hxC : x ∈ clears
        · rw [hS'C x hxC] at hx
          simp [Content.spillAnchor?] at hx
        have hxT : x ∉ targets := fun h => hxa ((hmem_t x).mp h).1
        rw [hS'ne x hxp hxT hxC] at hx
        apply hxC
        rw [hmem_c]
        refine ⟨hxa, ?_⟩
        cases hs : st.sheet x <;> simp_all [Content.spillAnchor?]
      have hchange : SheetChange st₁ p S' st₁.seenOccupied targets clears :=
        { so_new := fun _ _ h => Or.inl h
          p_kind := by
            rw [hp₁, hS'p]
            exact ⟨_, rfl⟩
          untouched := fun x hxp hxT hxC => by rw [hS'ne x hxp hxT hxC, h1sheet]
          written := fun x hx => ⟨((hmem_t x).mp hx).2, ⟨_, hS'T x hx⟩,
            by rw [h1sheet]; exact hfree x hx, hse_none x hx⟩
          cleared := fun x hx => ⟨fun h => hpC (h ▸ hx), hS'C x hx,
            by rw [h1sheet]; exact ((hmem_c x).mp hx).2, hso_none x hx⟩
          fresh_unrecorded := by
            intro x w hx
            by_cases hxT : x ∈ targets
            · exact hse_none x hxT
            by_cases hxp : x = p
            · subst hxp
              rw [hS'p] at hx
              cases hx
            by_cases hxC : x ∈ clears
            · rw [hS'C x hxC] at hx
              cases hx
            rw [hS'ne x hxp hxT hxC] at hx
            exfalso
            apply hxT
            rw [hmem_t]
            refine ⟨?_, hxp⟩
            by_contra hxa
            exact hxC ((hmem_c x).mpr ⟨hxa, w, hx⟩)
          p_anchor_of_written := fun _ => by simp [hS'p, Content.isAnchor]
          written_in_cse := fun x _ t' area' v' h => by
            rw [hS'p] at h
            cases h
          cleared_dyn := fun _ => by simp [hp₁, Content.isDynAnchor]
          so_mono := fun _ _ h => h
          so_spill := by
            intro x r hx
            obtain ⟨a, w, hw⟩ := hinv₁.seen_occupied x r hx
            by_cases hxT : x ∈ targets
            · exact ⟨p, _, hS'T x hxT⟩
            have hxp : x ≠ p := by
              rintro rfl
              rw [hp₁] at hw
              cases hw
            have hxC : x ∉ clears := fun h => by
              rw [hso_none x h] at hx
              cases hx
            exact ⟨a, w, by rw [hS'ne x hxp hxT hxC, ← h1sheet]; exact hw⟩ }
      have hf := hchange.facts hinv₁ htop₁
      have hres := ResultOf.transfer hf hr₁ ht₁
      have hcons : ConsistentAtWith (passView (commitState st₁ p S' st₁.seenOccupied))
          (StableBlocked (commitState st₁ p S' st₁.seenOccupied)) S' p := by
        unfold ConsistentAtWith
        rw [hS'p]
        intro _
        rcases hres with h | ⟨h, _⟩
        · cases h
        · rw [← h]
          left
          exact ⟨rfl, fun q hqp => ⟨fun hq => hS'T q ((hmem_t q).mpr ⟨hq, hqp⟩),
            fun hq => hnospill_out q hq⟩⟩
      have hreads : ∀ t', (S' p).formula? = some t' → valueAt S' p ≠ circ →
          ∀ x ∈ t'.reads (passView (commitState st₁ p S' st₁.seenOccupied)),
            Protected (commitState st₁ p S' st₁.seenOccupied) x := by
        intro t' ht' _ x hx
        rw [hS'p] at ht'
        simp only [Content.formula?, Option.some.injEq] at ht'
        rw [← ht'] at hx
        rcases hres with h | ⟨_, hreads⟩
        · cases h
        · exact hreads x hx
      obtain ⟨hstep, hinv'⟩ := hchange.commitOk hinv₁ htop₁ hcons hreads
      exact ⟨hrest₁, hstep₀.trans hstep, hinv', h1cells, h1stack⟩
    · -- Contradiction: abandoned.
      left
      rename_i hroots
      have hreaders : ∀ x ∈ (((targets.filterMap st₁.seenEmpty ++
          clears.filterMap st₁.seenOccupied).filter fun r => decide (r ≠ p)).dedup),
          x ≠ p ∧ ((∃ q, st.seenEmpty q = some x ∨ st.seenOccupied q = some x) ∨
            st.root = some x) := by
        intro x hx
        rw [List.mem_dedup, List.mem_filter] at hx
        obtain ⟨hx, hne⟩ := hx
        refine ⟨by simpa using hne, ?_⟩
        rcases List.mem_append.mp hx with hx | hx
        · obtain ⟨q, _, hq⟩ := List.mem_filterMap.mp hx
          rw [h1se] at hq
          exact Or.inl ⟨q, Or.inl hq⟩
        · obtain ⟨q, _, hq⟩ := List.mem_filterMap.mp hx
          rcases h1so_src q x hq with hq | ⟨_, _, hq⟩
          · exact Or.inl ⟨q, Or.inr hq⟩
          · exact Or.inr hq
      refine ⟨_, rfl, ?_⟩
      split
      · rename_i hempty
        split
        · -- Stale cells: no write contradicts, so a removal does, of a spill
          -- cell of `p` on record as blocking; `p` has not committed.
          rename_i hwe
          refine ⟨rfl, ?_, by simp [Restart.readers], by simp [Restart.learns],
            by simp [Restart.readers], fun ⟨v, hv⟩ => by cases hv⟩
          show ∃ q ∈ clears, ∃ v, S₀ q = .spill p v
          rw [Bool.not_eq_true, List.isEmpty_eq_false_iff_exists_mem] at hroots
          obtain ⟨r', hr'⟩ := hroots
          rcases List.mem_append.mp hr' with hw | hcl
          · rw [List.isEmpty_iff] at hwe
            rw [hwe] at hw
            exact absurd hw List.not_mem_nil
          · obtain ⟨x, hx, _⟩ := List.mem_filterMap.mp hcl
            obtain ⟨_, w, hw⟩ := (hmem_c x).mp hx
            exact ⟨x, hx, w, hinv.orig_spill x p w hw (by rw [cells_of_head hinv htop]; simp)⟩
        · exact ⟨rfl, trivial, by simp [Restart.readers], by simp [Restart.learns],
            by simp [Restart.readers], fun ⟨v, hv⟩ => by cases hv⟩
      · rename_i hne
        exact ⟨rfl, trivial, hreaders, fun _ h => hne (List.isEmpty_iff.mpr h),
          List.nodup_dedup _, fun ⟨v, hv⟩ => by cases hv⟩


/-- `set_cells_with_result` on the cell on top of the stack: either the pass
is abandoned by a restart naming `p`, which is then a dynamic anchor, or the
cell is committed and, once marked evaluated, the invariant holds again. -/
theorem commit_spec (st : PassState Pos Value) (p : Pos) (r : Result Pos Value)
    (hinv : PassInv S₀ st) (htop : st.stack.head? = some p) (hroot : st.root.isSome)
    (hr : ResultOf st p r) (hf : (st.sheet p).formula?.isSome) :
    (∃ r', (((commit U p r).run st).2).restart = some r' ∧
        (st.sheet p).isDynAnchor = true ∧ Abandoned S₀ st p r r') ∨
      CommitOk S₀ st ((commit U p r).run st).2 p := by
  cases hp : st.sheet p with
  | formula t v => exact Or.inr (commit_spec_formula S₀ U st p r hinv htop hr t v hp)
  | cseAnchor t area v => exact Or.inr (commit_spec_cse S₀ U st p r hinv htop hr t area v hp)
  | dynAnchor t v =>
      cases r with
      | scalar v' =>
          rcases commit_spec_dyn_scalar S₀ U st p v' hinv htop hr t v hp with ⟨r', h1, h2⟩ | h
          · exact Or.inl ⟨r', h1, by simp [hp, Content.isDynAnchor], h2⟩
          · exact Or.inr h
      | array area vals =>
          rcases commit_spec_dyn_array S₀ U st p area vals hinv htop hroot hr t v hp
            with ⟨r', h1, h2⟩ | h
          · exact Or.inl ⟨r', h1, by simp [hp, Content.isDynAnchor], h2⟩
          · exact Or.inr h
  | _ => rw [hp] at hf; simp [Content.formula?] at hf

end

end IronCalcEval

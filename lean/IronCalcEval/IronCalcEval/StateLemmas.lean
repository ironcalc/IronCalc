import IronCalcEval.Pass

/-!
# Reasoning about the pass monad

`Preserves I m` says that the computation `m` relates every start state to
its end state by the relation `I.R`, a preorder. The combinators below follow
the shape of `do` blocks: `pure`, `bind`, `get` with its continuation, `set`,
`modify`, `if`, and `for` over a list. With them, a property of `evalCell` is
proved by walking the definition once.

The first relation proved is `cellsMono`: a cell that is `Evaluated` stays
`Evaluated`. Every primitive of `Pass.lean` gets a lemma; `evalCell` follows
by induction on the fuel.
-/

namespace IronCalcEval

/-- A preorder on states: what a computation may do to the state. -/
structure StateRel (σ : Type) where
  R : σ → σ → Prop
  refl : ∀ s, R s s
  trans : ∀ {a b c}, R a b → R b c → R a c

/-- `m` relates every start state to its end state by `I.R`. -/
def Preserves {σ α : Type} (I : StateRel σ) (m : StateM σ α) : Prop :=
  ∀ st, I.R st (m.run st).2

namespace Preserves

variable {σ α β : Type} {I : StateRel σ}

theorem pureM (a : α) : Preserves I (pure a : StateM σ α) := fun st => I.refl st

theorem bindM {m : StateM σ α} {f : α → StateM σ β} (hm : Preserves I m)
    (hf : ∀ a, Preserves I (f a)) : Preserves I (m >>= f) :=
  fun st => I.trans (hm st) (hf _ _)

/-- `get >>= f`: the continuation runs on the very state it was given. -/
theorem getBind {f : σ → StateM σ β} (hf : ∀ st, I.R st ((f st).run st).2) :
    Preserves I (get >>= f) :=
  fun st => hf st

/-- Running `get >>= f` is running `f` on the state it was given. -/
theorem _root_.StateM.run_getBind {σ β : Type} {f : σ → StateM σ β} (st : σ) :
    (get >>= f).run st = (f st).run st :=
  rfl

/-- `pure` in `Id` is the identity. -/
theorem _root_.Id.pure_apply {α : Type} (x : α) : (pure x : Id α) = x :=
  rfl

/-- `bind` in `Id` is application. -/
theorem _root_.Id.bind_apply {α β : Type} (x : α) (f : α → Id β) : (x >>= f : Id β) = f x :=
  rfl

/-- `get >>= f` at one state, for lemmas with a precondition on that state. -/
theorem getBind_run {f : σ → StateM σ β} (st : σ) (hf : I.R st ((f st).run st).2) :
    I.R st ((get >>= f).run st).2 :=
  hf

theorem modifyM {g : σ → σ} (h : ∀ st, I.R st (g st)) :
    Preserves I (modify g : StateM σ PUnit) :=
  fun st => h st

/-- The `Decidable` instance is a plain implicit so that it is read off the
goal rather than searched for. -/
theorem iteM {c : Prop} {inst : Decidable c} {m₁ m₂ : StateM σ α} (h₁ : Preserves I m₁)
    (h₂ : Preserves I m₂) : Preserves I (@ite _ c inst m₁ m₂) := by
  split <;> assumption

/-- `modify g >>= k`, at one state: the step to `g st` and then `k` from there. -/
theorem modifyBind_run {g : σ → σ} {k : PUnit → StateM σ β} (st : σ) (h : I.R st (g st))
    (hk : Preserves I (k ())) : I.R st ((modify g >>= k).run st).2 :=
  I.trans h (hk (g st))

theorem forInList {f : α → β → StateM σ (ForInStep β)} (hf : ∀ a b, Preserves I (f a b)) :
    ∀ (l : List α) (b : β), Preserves I (forIn l b f)
  | [], b => by
      rw [List.forIn_nil]
      exact pureM b
  | a :: as, b => by
      rw [List.forIn_cons]
      exact bindM (hf a b) fun
        | .done b => pureM b
        | .yield b => forInList hf as b

theorem mono {I' : StateRel σ} (h : ∀ a b, I.R a b → I'.R a b) {m : StateM σ α}
    (hm : Preserves I m) : Preserves I' m :=
  fun st => h _ _ (hm st)

end Preserves

variable {Pos Value : Type}

/-- An `Evaluated` cell stays `Evaluated`. -/
def cellsMono (Pos Value : Type) : StateRel (PassState Pos Value) where
  R a b := ∀ q, a.cells q = some .evaluated → b.cells q = some .evaluated
  refl _ _ h := h
  trans h₁ h₂ q h := h₂ q (h₁ q h)

/-- Running a formula preserves whatever its reads preserve. -/
theorem Formula.run_preserves {I : StateRel (PassState Pos Value)}
    {rec : Pos → PassM Pos Value Value} (hrec : ∀ q, Preserves I (rec q)) :
    ∀ t : Formula Pos Value, Preserves I (Formula.run rec t)
  | .done r => by
      rw [Formula.run]
      exact Preserves.pureM r
  | .read p k => by
      rw [Formula.run]
      exact Preserves.bindM (hrec p) fun v => Formula.run_preserves hrec (k v)

section
variable [DecidableEq Pos] [ValueSort Value]

local notation "CM" => cellsMono Pos Value

omit [DecidableEq Pos] in
theorem storedValue_cells (p : Pos) : Preserves CM (storedValue (Value := Value) p) :=
  Preserves.getBind fun _ _ h => h

omit [ValueSort Value] in
theorem recordSeen_cells (q : Pos) (s : Seen) : Preserves CM (recordSeen (Value := Value) q s) :=
  Preserves.getBind fun st => by
    split
    · exact fun _ h => h
    · split <;> split <;> exact fun _ h => h

omit [ValueSort Value] in
theorem markCycle_cells (o : Pos) : Preserves CM (markCycle (Value := Value) o) :=
  Preserves.modifyM fun st => by
    split <;> exact fun _ h => h

omit [ValueSort Value] in
theorem spillContradictsARead_cells (anchor : Pos) (writes clears : List Pos) :
    Preserves CM (spillContradictsARead (Value := Value) anchor writes clears) :=
  Preserves.getBind fun st => by
    dsimp only
    split
    · exact fun _ h => h
    · exact fun _ h => h

omit [ValueSort Value] in
theorem storeScalar_cells (U : Universe Pos) (p : Pos) (t : Formula Pos Value) (v : Value) :
    Preserves CM (storeScalar U p t v) :=
  Preserves.getBind fun st => by
    dsimp only
    refine Preserves.bindM (spillContradictsARead_cells p [] _) (fun b =>
      Preserves.iteM (Preserves.pureM _) (Preserves.modifyM ?_)) st
    exact fun _ _ h => h

theorem recordBlockers_cells (st : PassState Pos Value) (p : Pos) :
    ∀ l : List Pos, Preserves CM (recordBlockers st p l)
  | [] => Preserves.pureM _
  | q :: l => by
      simp only [recordBlockers]
      refine Preserves.bindM ?_ fun _ => recordBlockers_cells st p l
      split
      · exact Preserves.iteM (recordSeen_cells q .occupied) (Preserves.pureM _)
      · exact Preserves.pureM _

theorem spillDynamicArray_cells (U : Universe Pos) (p : Pos) (t : Formula Pos Value)
    (area : List Pos) (vals : Pos → Value) :
    Preserves CM (spillDynamicArray U p t area vals) :=
  Preserves.getBind fun st => by
    dsimp only
    refine Preserves.bindM (recordBlockers_cells st p _) (fun _ =>
      Preserves.iteM (Preserves.bindM (storeScalar_cells U p t _) fun _ => Preserves.pureM _)
        (Preserves.bindM (spillContradictsARead_cells p _ _) fun b =>
          Preserves.iteM (Preserves.pureM _) (Preserves.modifyM ?mod))) st
    case mod => exact fun _ _ h => h

theorem commit_cells (U : Universe Pos) (p : Pos) (r : Result Pos Value) :
    Preserves CM (commit U p r) :=
  Preserves.getBind fun st => by
    dsimp only
    split
    · exact fun _ h => h
    · exact fun _ h => h
    · split
      · exact storeScalar_cells U p _ _ st
      · exact spillDynamicArray_cells U p _ _ _ st
    · exact fun _ h => h

theorem evalSpillCell_cells {rec : Pos → PassM Pos Value Value} (hrec : ∀ q, Preserves CM (rec q))
    (p a : Pos) : Preserves CM (evalSpillCell rec p a) :=
  Preserves.getBind fun st => by
    split
    · split
      · exact storedValue_cells p st
      · exact hrec a st
      · exact Preserves.bindM (hrec a) (fun _ => storedValue_cells p) st
    · split
      · exact storedValue_cells p st
      · exact Preserves.bindM (recordSeen_cells p .empty) (fun _ => Preserves.pureM _) st
      · exact fun _ h => h
    · exact Preserves.bindM (recordSeen_cells p .empty) (fun _ => Preserves.pureM _) st

theorem finishFormulaCell_cells (U : Universe Pos) (p : Pos) (r : Result Pos Value) :
    Preserves CM (finishFormulaCell U p r) :=
  Preserves.getBind fun st => by
    dsimp only
    refine Preserves.bindM (Preserves.iteM (commit_cells U p _) (Preserves.pureM _)) (fun _ =>
      Preserves.getBind fun st' => ?_) st
    refine Preserves.iteM (Preserves.pureM _) ?_ st'
    refine Preserves.bindM (Preserves.modifyM ?_) fun _ => storedValue_cells p
    intro s q h
    unfold markEvaluated
    dsimp only
    by_cases hqp : q = p
    · subst hqp
      simp
    · rw [Function.update_of_ne hqp]
      exact h

theorem evalFormulaCell_cells (U : Universe Pos) {rec : Pos → PassM Pos Value Value}
    (hrec : ∀ q, Preserves CM (rec q)) (p : Pos) (t : Formula Pos Value) :
    Preserves CM (evalFormulaCell U rec p t) :=
  Preserves.getBind fun st => by
    split
    · exact Preserves.bindM (markCycle_cells p) (fun _ => Preserves.pureM _) st
    · exact storedValue_cells p st
    · rename_i hnone
      -- The first `modify` marks `p` as `Evaluating`; `p` was not evaluated, so
      -- nothing evaluated is touched.
      refine Preserves.modifyBind_run st (fun q hq => ?_) ?_
      · dsimp only
        rw [Function.update_of_ne]
        · exact hq
        · rintro rfl
          rw [hnone] at hq
          cases hq
      · exact Preserves.getBind fun st₁ =>
          Preserves.bindM (Preserves.iteM (Preserves.pureM _) (Formula.run_preserves hrec t))
            (fun r => finishFormulaCell_cells U p r) st₁

theorem evalCell_cells (U : Universe Pos) :
    ∀ (fuel : Nat) (p : Pos), Preserves CM (evalCell (Value := Value) U fuel p)
  | 0, p => Preserves.pureM _
  | fuel + 1, p =>
    Preserves.getBind fun st => by
      refine Preserves.iteM (Preserves.pureM _) ?_ st
      split
      · exact Preserves.bindM (recordSeen_cells p .empty) (fun _ => Preserves.pureM _)
      · exact Preserves.pureM _
      · exact evalSpillCell_cells (evalCell_cells U fuel) p _
      · exact evalFormulaCell_cells U (evalCell_cells U fuel) p _
      · exact evalFormulaCell_cells U (evalCell_cells U fuel) p _
      · exact evalFormulaCell_cells U (evalCell_cells U fuel) p _

end

end IronCalcEval

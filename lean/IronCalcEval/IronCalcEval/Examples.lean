import IronCalcEval.Driver

/-!
# Executable examples

The model is computable, so small sheets can be evaluated with `#eval` and
compared with what the Rust tests expect. This is a sanity check of the
model, not a proof.
-/

namespace IronCalcEval.Examples

open IronCalcEval

/-- A concrete value type: numbers plus the three distinguished values. -/
inductive V where
  | blank
  | circ
  | spill
  | num (n : Int)
  deriving Repr, DecidableEq

instance : ValueSort V := ⟨.blank, .circ, .spill⟩

/-- Four positions in a column: A1, A2, A3, A4. -/
abbrev P := Fin 4

def U : Universe P :=
  ⟨List.finRange 4, fun p => List.mem_finRange p, List.nodup_finRange 4⟩

/-- A formula that reads one position and returns it. -/
def readCell (p : P) : Formula P V :=
  .read p fun v => .done (.scalar v)

/-- A formula that spills the array `vals` over `area` without reading. -/
def constArray (area : List P) (vals : P → V) : Formula P V :=
  .done (.array area vals)

/-- The values of a sheet, in natural order, the new order and the restart count. -/
def run (S : Sheet P V) (order : List P) : Option (List V × List P × Nat) :=
  (evaluate U S order 20).map fun (S', order', log) =>
    (U.positions.map (valueAt S'), order', log.restarts)

/-- A1 spills `1, 2` into A1:A2; A3 is a dynamic anchor reading A2. With the
order `[A3, A1]`, A3 reads A2 as empty, then A1's spill contradicts that read:
one restart, then A3 reads 2. -/
def conflict : Sheet P V := fun p =>
  match p with
  | 0 => .dynAnchor (constArray [0, 1] fun q => .num (q.val + 1)) .blank
  | 2 => .dynAnchor (.read 1 fun v => .done (.array [2] fun _ => v)) .blank
  | _ => .empty

#eval run conflict [2, 0]
-- expected: some ([num 1, num 2, num 2, blank], [0, 2], 1)

/-- A1 = A2 and A2 = A1: a value cycle. Both are marked. -/
def valueCycle : Sheet P V := fun p =>
  match p with
  | 0 => .formula (readCell 1) .blank
  | 1 => .formula (readCell 0) .blank
  | _ => .empty

#eval run valueCycle []
-- expected: some ([circ, circ, blank, blank], [], 0)

/-- A1 reads A2 and spills over A1:A2: its inputs read its own area. The
anchor is circular; nothing else is affected. -/
def selfContradiction : Sheet P V := fun p =>
  match p with
  | 0 => .dynAnchor (.read 1 fun _ => .done (.array [0, 1] fun _ => .num 7)) .blank
  | _ => .empty

#eval run selfContradiction []
-- expected: some ([circ, blank, blank, blank], [0], 1)

/-- A leftover spill cell of a previous evaluation at A2, pointing at A1,
which now produces a scalar. A3 reads A2 first (order `[A3, A1]`): a stale
read restarts with A1 first, which then clears A2. -/
def staleRead : Sheet P V := fun p =>
  match p with
  | 0 => .dynAnchor (.done (.scalar (.num 5))) .blank
  | 1 => .spill 0 (.num 99)
  | 2 => .dynAnchor (.read 1 fun v => .done (.array [2] fun _ => v)) .blank
  | _ => .empty

#eval run staleRead [2, 0]
-- expected: some ([num 5, blank, blank, blank], [0, 2], 1)

end IronCalcEval.Examples

namespace IronCalcEval.Examples

/-- The situation of `a_blocking_cell_read_as_empty_before_the_scan_keeps_both_records`
in the Rust tests, with lists for areas. A1 is an anchor with leftover spill
cells A2 and A3 from a previous evaluation. Its formula reads A2 and A3 (its
own leftovers, seen empty), then reads A4, an anchor whose array covers
A4, A2 and A3 and is blocked by those leftovers. A1 then stores a scalar and
removes A2 and A3, which contradicts the record of A4 being blocked by them,
made on A1's own behalf. With one record per position that record was lost
and A4 kept `#SPILL!` over a free area. -/
def staleCells : Sheet P V := fun p =>
  match p with
  | 0 => .dynAnchor (.read 1 fun _ => .read 2 fun _ => .read 3 fun _ => .done (.scalar (.num 5)))
      .blank
  | 1 => .spill 0 (.num 9)
  | 2 => .spill 0 (.num 9)
  | 3 => .dynAnchor (constArray [3, 1, 2] fun _ => .num 1) .blank
  | _ => .empty

#eval run staleCells [0, 3]
-- expected: some ([num 5, num 1, num 1, num 1], [3, 0], 2)
-- The leftovers were blocking only A1's own input: they are dropped (first
-- restart). A1 then reads A2 and A3 as empty while A4, evaluated on its
-- behalf, spills into them: a conflict moves A4 first (second restart), and
-- the third pass is the fresh sheet's. Before this rule A1 was marked
-- circular, and the same sheet without the leftovers gave `5` with A4 spilled.

/-- The same history, but A1 wants its cells back once A4 spills: it reads
A4 and spills over A1:A3 unless A4 is `#SPILL!`, in which case it stores 5.
There is no sheet in which A1 keeps the cells. A1's leftovers go, A4 takes
them, and A1 is blocked, as on a fresh sheet. Nothing is circular. -/
def shrunkAnchor : Sheet P V := fun p =>
  match p with
  | 0 => .dynAnchor (.read 3 fun v =>
      if v = spillError then .done (.scalar (.num 5)) else .done (.array [0, 1, 2] fun _ => .num 7))
      .blank
  | 1 => .spill 0 (.num 9)
  | 2 => .spill 0 (.num 9)
  | 3 => .dynAnchor (constArray [3, 1, 2] fun _ => .num 1) .blank
  | _ => .empty

#eval run shrunkAnchor [0, 3]
-- expected: some ([spill, num 1, num 1, num 1], [0, 3], 1)

end IronCalcEval.Examples

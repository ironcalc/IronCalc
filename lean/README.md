# Lean model of the evaluation algorithm

A Lean 4 model of the cold evaluation algorithm of `cold-evaluation.md`
(section 3) and `base/src/evaluation.rs`, with the specification of section 1
and the theorems of sections 5.2 and 5.3 stated. **Both theorems are fully
proved, with no `sorry` left.** `evaluate_consistent` in `Correctness.lean`
says that whatever `evaluate` returns is a consistent sheet, for any
well-formed input; `evaluate_terminates` in `Termination.lean` says that some
amount of fuel always makes it return. Together: for every well-formed sheet
and duplicate-free anchor order, `evaluate` returns, and what it returns is
consistent.

Lean cannot verify the Rust. What it can verify is that the algorithm as
specified satisfies the contract. The link between the Lean definitions and
the Rust is by inspection: the definitions mirror the Rust function by
function, under the same names, and `Examples.lean` runs the model on the
situations the Rust tests cover.

## Layout

The project is `IronCalcEval/` (a `lake` project depending on Mathlib).

| File | Contents | Rust |
|---|---|---|
| `Sheet.lean` | positions, values, formulas as strategy trees, contents, `valueAt`, `Consistent` | `Cell`, `cold-evaluation.md` §1 |
| `Pass.lean` | `PassState`, `recordSeen`, `markCycle`, `spillContradictsARead`, `commit`, `evalCell`, `runPass` | `evaluation.rs` and `spill_dynamic_array` in `model.rs` |
| `Driver.lean` | `RestartLog` (facts and marks), `closure`, `learn`, `record`, `dropStale`, `dropMarked`, `syncAnchorOrder`, `evaluate` | `RestartLog`, `Model::evaluate` |
| `StateLemmas.lean` | `Preserves`: reasoning about the pass monad; one lemma per primitive; `evalCell` keeps evaluated cells evaluated | |
| `Invariant.lean` | `PassInv` (§5.1 as a state predicate), `Protected`, `StableBlocked`, the step relation `PassStep`/`PassRel`, the simple primitives | |
| `Commit.lean` | `SheetChange` (what a commit did), the generic `SheetChange.commitOk`, and `commit_spec`: a commit either abandons the pass or re-establishes the invariant once the cell is marked evaluated | `set_cells_with_result`, `spill_dynamic_array` |
| `Read.lean` | `ReadSpec`/`RunSpec`, the specs of `evalSpillCell`, `evalFormulaCell`, `finishFormulaCell`, `Formula.run` and `evalCell` (by induction on fuel) | `evaluate_cell`, `evaluate_spill_cell`, `evaluate_formula_cell` |
| `Loop.lean` | the pass seen from the driver (`RestartFacts`): a restart names an unmarked anchor of the order, and its readers are unmarked anchors placed before it | `run_pass` |
| `Correctness.lean` | the pass invariant (§5.1) and partial correctness (§5.2) | |
| `Termination.lean` | closure lemmas, the two-element sublist as "before", `learn_spec`, `record_spec`, the measure, termination (§5.3) | `RestartLog`, `Model::evaluate` |
| `Examples.lean` | `#eval` of the model on six small sheets | `test_ordered_restart.rs` |

Build with `lake build` inside `IronCalcEval/`. The `#eval` results appear in
the build output and in the editor. `grep -c sorry IronCalcEval/*.lean` shows
what is left.

## Abstractions

* A formula is a tree: either it has its result, or it reads a position and
  continues with what it found. The result depends only on what was read, by
  construction. This is the "function of the sheet together with the set of
  positions it reads" of §5.5.
* Positions are an abstract type; the workbook is a `Universe`, the list of
  all positions in natural order. Values are an abstract type with the three
  distinguished elements the algorithm produces: empty, `#CIRC!`, `#SPILL!`.
* Array areas are lists of positions, not rectangles. Worksheet bounds and
  merged cells are not modelled.
* `evaluate_formula_cell` is split in two (`evalFormulaCell`, `finishFormulaCell`)
  and binds its `if`s as terms, so that the `do` elaborator does not duplicate
  the continuation and the proofs can follow the code line by line.
* The recursion of `evaluate_cell` is written with fuel; `Correctness.lean`
  states that the fuel provided is never exhausted.
* The driver loop is written with fuel; `Termination.lean` states that some
  fuel is always enough. This is the theorem, so the definition may not
  assume it.
* A restarted pass starts again from the original sheet, minus the stale
  cells a `staleCells` restart dropped (`dropStale`) and the spill cells of
  the anchors the restart marked (`dropMarked`). The Rust restores only
  dynamic anchors and spill cells, and leaves the values an abandoned pass
  stored in formula cells, which a pass never reads before recomputing.
* The root of a record, the cell the driver is evaluating, is a field of the
  state set by `passBody` before each cell, as `Evaluation.root` in the Rust.
* The reachability the driver needs (`closure`) is computed by saturating a
  list a fixed number of times; the Rust walks the graph. Both compute the
  same set, and `Termination.lean` proves the saturated list is closed.
* A dynamic anchor's own spill cells are found by scanning the whole sheet,
  not the remembered old area.
* `in_pass = false` (evaluation outside a pass) is out of scope.

## What writing the model surfaced

1. **A marked anchor could restart.** §5.3 said "a marked anchor never
   restarts again: it stores `#CIRC!` without spilling and keeps no spill
   cells". But the spill cells it left from a *previous* evaluation were
   restored at every restart and removed only when its turn came, so an
   unmarked anchor placed before it could be blocked by them or read one,
   and restart it. Resolved with finding 5: the driver now drops a marked
   anchor's spill cells when it marks it (`dropMarked`), which makes the
   sentence true; `RestartFacts.marked_spill` and the driver invariant
   `no_stale_marked` are the two halves of the argument that a restart's
   anchor is unmarked.
2. **The budget was reachable, and it fired on a cycle-free sheet.** The
   pigeonhole gives at most `n!` orders between two changes of the circular
   set; the budget was `n² + 2`; the document's cycle-free bound of `n`
   restarts was false, because moving an anchor to the front puts a reader
   ahead of what it reads. Three tiers of ten anchors, each tier reading
   every spill cell of the next, take about `k·m²` restarts, hit the budget,
   and got innocent `=SEQUENCE(2)` cells marked `#CIRC!`
   (`a_cycle_free_cascade_is_not_marked_circular`). Resolved by replacing
   the driver: a restart is a fact, "the anchor runs before its readers"; the
   driver keeps the facts and repairs the order to respect them, moving the
   anchor and what must precede it to just before the reader
   (`RestartLog.learn`); a fact that would close a loop marks the anchors on
   it. Every restart adds a fact the order broke, hence a new one (at most
   `n²`), or marks an anchor (at most `n`), or drops a stale cell. There is
   no budget any more, and `evaluate_terminates` is proved from that
   measure. Proving it needed the readers of a restart to be known: the root
   became an explicit field, every new record is on record as made for the
   root (`PassStep.roots_new_*`), and the loop invariant of `Loop.lean`
   tracks that every record's root is a processed, unmarked cell. The same
   sheet takes nine restarts now, one per link of the chain.

3. **`record_seen` drops the `Occupied` record of a position already read as
   empty.** `record_seen` uses `or_insert`, one record per position. A cell
   can be read as empty (a leftover spill cell of the anchor being evaluated)
   and later found blocking another array's spill; the second record is lost,
   so removing the cell contradicts nothing. Reproduced in the model
   (`lostOccupied` in `Examples.lean`) and in the engine:
   `probe_blocking_cell_read_as_empty_before_the_scan` in
   `test_ordered_restart.rs` fails with three oracle violations and no
   restart, leaving `C1 = #SPILL!` over a free area. Fixed: `record_seen` now keeps
   one record per kind (`SeenRecord`), which is what §3.5 and §3.6 describe,
   and the probe is the regression test
   `a_blocking_cell_read_as_empty_before_the_scan_keeps_both_records`. The
   model does the same (`seenEmpty`, `seenOccupied`). The invariant
   `seen_occupied` is unprovable without it.
4. **A false `#CIRC!` the fixed example raised, now resolved.** With both
   records kept, the same sheet ended with `B2 = #CIRC!` and `C1` spilled,
   while `B2 = 7, C1 spilled` is also consistent and has no circular
   reference; a second `evaluate()`, with the leftovers gone, gave 7. The
   self-contradiction rule attributed the blocked scan of `C1`, evaluated on
   demand under `B2`, to `B2`'s own inputs, when `C1` was blocked by cells
   `B2` was about to remove: history, not output. The rule now
   distinguishes the two (`Restart.staleCells`, `Restart::StaleCells`):
   when only removals contradict, and only on the anchor's own behalf, the
   cells are dropped from the starting sheet and the pass restarts, marking
   nothing. Termination needed a new leading component in the measure (the
   spill cells of the starting sheet, `staleCount`), which needed a new
   clause of the pass invariant (`orig_spill`: a spill cell of an anchor that
   has not committed is in the original sheet) so that a dropped cell is
   known to exist. `staleCells` and `shrunkAnchor` in `Examples.lean` are
   the two Rust tests.

## Suggested order of work

1. Done: `LogInv.new`, `LogInv.record`, `record_circular_mono`, `evaluateLoop_mono`.
2. Done: `PassInv.initial`, `evalCell_cells_mono`, `passView_eq_valueAt`, with the
   `Preserves` framework of `StateLemmas.lean` as the tool for walking `evalCell`.
3. Done: `evalCell_preserves`, the heart of §5.2, as a corollary of
   `evalCell_spec` in `Read.lean`: a read from a state satisfying the
   invariant, with enough fuel (`FuelOk`), either abandons the pass or
   leaves the invariant and the stack as they were and returns the pass view
   of the position, unless the reader ends up marked circular. The commit
   side is `Commit.lean`. What the proof needed beyond the skeleton:
   * the relation carried through `Preserves` must include: restart, cells,
     circular and `seenEmpty` monotone; `PassInv` in → abandoned or `PassInv`
     out; and "a protected position keeps its `passView`";
   * `reads_protected` option (ii) must require the formula cell read to be
     `Evaluated` (its stored value changes at its commit);
   * `evalCell q` returns `passView st' q`, or the cell on top of the stack
     when the read was made ends up marked circular (a read that closes a
     cycle returns `#CIRC!` where the view says empty);
   * `commit` followed by the `Evaluated` mark must be treated as one step:
     after a commit every remaining spill cell of the anchor is fresh and has
     no `seenEmpty` record, which the invariant alone does not say;
   * `finishFormulaCell` now marks the cell and pops the stack in one
     `modify` (`markEvaluated`), so the stack/`Evaluating` clause holds at
     every step;
   * the blocking scan is the recursive `recordBlockers` rather than a
     `for` loop, with the same semantics, so that it has clean equations;
   * the fuel bound is per read: two units per cell that can still go on
     the stack, plus one if the position is a spill cell (it forwards to its
     anchor first); `evalCell_fuel_stable` was dropped in favour of this;
   * three more clauses were needed: the cells of a CSE area are the anchor's
     spill cells (`CseAreasFixed`, part of `WellFormed`, which the editing
     paths guarantee), and an empty-record can only sit on a *dynamic*
     anchor's leftover cell, and a CSE anchor has no spill cell outside its
     area (`CseSpillsInArea`, part of `WellFormed`).
4. Done: `passBody_spec`, `runPass_consistent`, `evaluate_consistent`,
   `syncAnchorOrder_orderOf`. The pass body is the recursive `passBody`
   (a `for` with `break` in the Rust), and `ReadSpec` gained the clause that
   a formula cell read while not on the stack ends up evaluated.
5. Done, then redone with finding 2: `runPass_facts` (`Loop.lean`), then
   `evaluate_terminates`. The measure is: spill cells of the sheet the
   passes start from, then unmarked anchors, then facts still to learn
   (`measure` in `Termination.lean`). `learn_spec` says one reader either
   adds a fact and repairs the order (`repair_ok`, `repair_perm`) or marks
   the reader and what the facts place between it and the anchor;
   `fold_spec` folds that over the readers; `record_spec` adds the
   self-contradiction mark; `restart_step` adds the sheet. The "before"
   relation of an order is the two-element sublist `[a, b] <+ order`, with
   `pair_sublist_append_iff` and `pair_sublist_cons_iff` doing the work of
   the repair proof.
6. Gone: `ordersSeen_le_factorial` and the budget, with the old driver.

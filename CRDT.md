# CRDTs in IronCalc — Design Document

**Status:** draft for discussion · 2026-07-16 (updated 2026-07-17: phase 9
done — see §11 "as implemented" and §16.9) · **Phases 0–2 implemented** in
`base/src/crdt/` (ids, fractional order, flat doc schema, session with
outbound/inbound translation, update-wins keep-sets, undo journal + model
repair). 33 tests incl. a seeded convergence fuzzer
(`base/src/test/user_model/test_crdt_sync.rs`, `CRDT_FUZZ_SEEDS=n` to stress).
Interim deviation from §8 until the id-ref formula phase: formulas are
replicated as text and structural ops fan out displaced formula text
(alternative (a) of §13.4). Implementation notes: the schema uses flat root
maps with composite keys (no nested Y-maps — concurrent subtree creation is
whole-map LWW in Yjs and silently drops writes); concurrent same-gap inserts
need client/counter-unique position suffixes (`unique_position`); undo of a
deletion is only model-authoritative when nothing structural intervened —
otherwise the model is rebuilt from the document (`repair_sheet_from_shadow`),
including the case where update-wins had already resurrected the deleted line.
**Companion docs:** `Annette.pdf` (Yanakieva/Bird/Bieniusa, PaPoC '23),
`AegisSheet.pdf` (Pfeil/Scandurra/Haas, PaPoC '26), and the repo's own conflict
analysis `CRDTs-in-IronCalc.md`.

---

## 1. Goals and non-goals

**Goals**

- Genuine concurrent and offline collaboration: any set of replicas that exchange
  all updates converge to the same workbook, regardless of delivery order.
- A new module `base/src/crdt/` containing the replicated data structure and the
  communication layer, talking to the existing `user_model` API.
- **Compact** stable identifiers for rows, columns, sheets and everything else that
  needs identity — a 1M-row sheet with 200 touched rows must store ~200 ids, not 1M.
- Practical conflict semantics: last-write-wins for concurrent edits of the same
  cell; update-wins (no silent data loss) when a row/column deletion races with an
  edit inside it.
- Formula **results are never replicated**. The doc carries user input only; every
  replica re-evaluates after convergence. Spill cells, `shared_strings` interning,
  and `parsed_formulas` never cross the wire.
- Local-first / P2P-capable: two replicas can sync directly via state-vector
  exchange; a server relay is just one transport, never an ordering authority.

**Non-goals (v1)**

- Character-level merging of text inside a cell (LWW on the whole input string).
- Multi-value conflict surfacing in the UI (AegisSheet's MV registers) — the format
  leaves room to add it later.
- Replacing `.ic`/xlsx file formats. The CRDT doc is a collaboration artifact.

## 2. Background: why the current sync is not enough

`UserModel` records every mutation as a `Diff` (`base/src/user_model/history.rs`)
and exposes `flush_send_queue()` / `apply_external_diffs()` (`common.rs`). Each
`Diff` is an inverse-carrying record keyed on **absolute coordinates**
(`SetCellValue{sheet, row, column, …, old_value}`). As `CRDTs-in-IronCalc.md` §0
spells out, this stream is neither commutative nor idempotent, has no causal
metadata, and its `old_value`-based inverses are wrong the moment peers diverge.
It converges only under serialized, conflict-free delivery.

The Annette paper shows the way out: **stable identifiers** for rows/columns plus a
map from `(ColumnId, RowId)` to content dissolves most structural conflicts, and
**keep-maps** give update-wins deletion semantics. AegisSheet adds the missing
pieces for a real product: non-duplicating **moves**, id-anchored **ranges**, and
**undo** as explicit inverse operations. This design adopts both, simplified where
we can afford it (LWW instead of multi-value registers).

## 3. Architecture: model-first mirror over a yrs doc

We keep `UserModel`/`Model` as the local engine and add a **`crdt::Session`** that
mirrors all shared state into a [yrs](https://github.com/y-crdt/y-crdt) `Doc`. The
doc is the *convergence point*; each replica's `Model` is a deterministic
projection of it.

```
            local edit                        remote update (bytes)
                │                                     │
                ▼                                     ▼
   UserModel::set_user_input …            Session::apply_update
                │                                     │
        DiffList (existing seam:              yrs Doc ── observe deltas
        push_diff_list, common.rs)                    │
                │                                     ▼
                ▼                          inbound translation:
   outbound translation:                   doc delta → Model mutations
   Diff → doc transaction                  (bypasses history, like
   (indices → ids at this moment)          apply_external_diffs today)
                │                                     │
                ▼                                     ▼
        yrs update (bytes) ──► transport        evaluate() → UI refresh
```

Why mirror instead of making the doc the primary store: it is incremental — the
engine, evaluator, undo history, and all bindings keep working unchanged; the CRDT
layer is additive. The cost is a proof obligation: **the outbound and inbound
paths must produce identical model state on every replica.** That obligation is
discharged by the property-based fuzzer in §12, which asserts byte-identical
workbooks and identical evaluation results after arbitrary concurrent histories.

yrs gives us, for free and battle-tested: per-key LWW maps with deterministic
tiebreaks (Lamport clock + client id), compact binary update encoding, state-vector
diff sync, causal ordering of updates, idempotent application, awareness
(presence), and wasm compatibility (`ywasm`) for the webapp.

## 4. Identifiers

Every replicated entity gets a stable id that is never reused:

| Entity | Id |
|---|---|
| Sheet | `SheetId` |
| Row / Column | `RowId` / `ColId` |
| CF rule, merge range | `RuleId`, `MergeId` |
| Operation (for keep-sets) | `OpId` |

```rust
enum EntityId {
    /// The k-th row/column of the sheet as created. Virtual: needs NO storage,
    /// its position in the order is implicitly k.
    Original(u32),
    /// Created by an insert. clientId is the yrs doc's client id.
    Inserted { client: u64, counter: u32 },
}
```

**The compactness trick is `Original(k)`.** An untouched grid stores nothing. When
a cell `B7` is first written on a pristine sheet, its key is derived
deterministically — `Original(2) : Original(7)` — so two replicas concurrently
writing `B7` produce the *same* key and converge by LWW, with no coordination and
no pre-materialized id arrays. Only rows/columns created by inserts allocate real
ids. Storage is O(touched), never O(grid).

String encoding for map keys (yrs map keys are strings): `Original(k)` → the
decimal number (`"7"`); `Inserted` → `"x" + base36(client) + "." + base36(counter)`
(~8–14 bytes). Cell key: `"<colId>:<rowId>"`. If key size ever matters for very
large sheets, a doc-local client-registry (u64 → small int) can shrink inserted
ids; deferred as an optimization.

## 5. Ordering: fractional position registers

Row/column/sheet order and CF priority all use the same mechanism: each element
carries a **position register** — a lexicographically ordered string (Figma-style
fractional index). Display order = visible elements sorted by `(pos, id)`.

- `Original(k)` has the implicit position `fixed_width(k)` (a 4-byte big-endian
  prefix covering 1..1,048,576) — again, no storage.
- **Insert** at display slot *s*: find the positions of the visible neighbors,
  write a midpoint string. Concurrent inserts into the same gap get distinct
  positions (each appends its own disambiguator) and order deterministically by
  the `(pos, id)` tiebreak — no duplication, no interleaving anomaly (the Annette
  Figure-4 failure).
- **Move = LWW overwrite of the position register.** This is the punchline of the
  fractional-position choice: AegisSheet needed a custom `ReplicatedUniqueList`
  because sequence-CRDT moves implemented as delete+reinsert duplicate under
  concurrency. Here a row's identity is a map key — it *cannot* duplicate — and
  concurrent moves of the same row resolve last-writer-wins, matching AegisSheet's
  chosen semantics (Table 3: MoveR/C ‖ MoveR/C → last move wins). Concurrent
  move ‖ edit trivially commute (different registers).
- **Delete** does not remove the entry; visibility is governed by keep-sets (§7).
- **Rebalancing:** repeated inserts at the same spot grow position strings. We cap
  growth by (a) allocating with jitter, and (b) a local *rebalance op* that
  rewrites positions of a crowded region — safe because positions are LWW
  registers and rebalancing preserves relative order; a concurrent insert using an
  old neighbor position still lands adjacent (worst case: off by the rebalanced
  neighborhood, never lost). Bounds to be validated by the fuzzer.

A note on `move_rows`: today `common.rs` pre-adjusts the move delta to skip
*locally hidden* rows. The doc operation must carry the **resolved target
position**, not the raw delta — otherwise two replicas with different hidden sets
compute different moves from the "same" op.

## 6. Doc schema

```
Doc
├─ "wb":     YMap { name, theme, locale, tz, language }             // LWW registers
├─ "sheets": YMap<SheetId → YMap>
│    ├─ meta: name, color, state, frozen_rows, frozen_cols,
│    │        grid_lines, pos                                       // LWW registers
│    ├─ "rows":  YMap<RowId → {pos?, height?, custom_height?, hidden?, style?}>
│    ├─ "cols":  YMap<ColId → {pos?, width?,  custom_width?,  hidden?, style?}>
│    ├─ "cells": YMap<"colId:rowId" → {i: input, s?: styleHash}>    // non-empty only
│    ├─ "keep_rows": YMap<RowId → YMap<OpId → ()>>                  // update-wins (§7)
│    ├─ "keep_cols": YMap<ColId → YMap<OpId → ()>>
│    ├─ "v_edges": YMap<"colId:rowId" → BorderReg>   // line left of col, at row (§10.2)
│    ├─ "h_edges": YMap<"colId:rowId" → BorderReg>   // line above row, at col
│    ├─ "cf":     YMap<RuleId → {pos, body, ranges: id-ranges}>     // priority = pos
│    └─ "merges": YMap<MergeId → {tl: (colId,rowId), br: (colId,rowId)}>
├─ "names":  YMap<"scopeSheetId|name" → formula (id refs)>          // defined names
├─ "named_styles": YMap<name → styleHash>
└─ "styles": YMap<styleHash → style body (JSON)>                    // content-addressed
```

Entries marked `?` are optional — a row entry exists only if the row was inserted,
resized, hidden, styled, or moved. `cells` values are small structs (yrs `Any`),
whole-value LWW: concurrent writes to the same cell resolve by Lamport clock with
client-id tiebreak — the required same-cell LWW costs nothing.

Sheet-level: `SheetId`s are materialized (sheets are few). `sheets` is keyed by id
with a `pos` register for tab order; `deleteSheet` uses a sheet-level keep-set
(same mechanism as rows) so a whole sheet of concurrent work is not silently lost.
`newSheet`/`duplicateSheet` create fresh `SheetId`s; concurrent same-name creation
yields two sheets, and a deterministic render-time fixup suffixes the later one
(by id tiebreak) with `" (2)"`, mirroring what the engine does locally.
`moveSheet` is a pure rewrite of the `pos` register (the sheet keeps its id, so
content and references are untouched); like row/column moves it counts as a
positive op, preempting a concurrent deletion of the moved sheet.

## 7. Deletion: update-wins keep-sets

Per the Annette paper's *Fixed Elements (Remove-keep)* model:

- Every **positive** operation touching row *R* (cell edit in *R*, row-prop write,
  the insert that created *R*) adds its `OpId` as a key of `keep_rows[R]`.
- **Delete row *R*** = create `keep_rows[R]` if absent, then remove every entry
  *currently visible to the deleter*. Cells of *R* are **not** deleted — they are
  masked.
- **Visibility rule:** a row is visible iff its keep-set is *absent* (pristine,
  e.g. an untouched `Original(k)`) or *non-empty*. An existing-but-empty keep-set
  is a tombstone.

yrs YMap semantics implement the concurrent case exactly: a `remove(key)` only
deletes entries the remover has seen, so an `OpId` added concurrently to the
deletion survives the clear → the keep-set is non-empty → the row stays visible
*with all its cells* (they were only masked). That is precisely update-wins, and
it is why deletion must mask rather than erase.

Columns and sheets work identically. Downgrading any of these to remove-wins later
is a policy switch in the visibility rule, not a format change.

**Purge/GC** (AegisSheet §"Purging"): tombstoned rows/columns whose deletion is
older than a cutoff can have their masked cells and keep-set entries physically
removed by any replica (deterministic criterion) or a coordinating peer. A replica
offline past the cutoff may resurrect the row itself but not its purged contents —
the same tradeoff AegisSheet accepts. Cutoff policy is an open item (§13).

## 8. Cell content and formulas

The doc stores exactly what the user typed — except that formula references are
translated to **stable ids at the doc boundary**. The engine keeps its A1-text /
RC-relative `shared_formulas` world completely unchanged.

**Doc formula format.** A compact serialization of the parsed formula where every
reference/range node is `(sheetId, colId, rowId, abs_col, abs_row)` (ids, not
offsets); everything else (functions, operators, literals) is carried structurally
or as canonical text with reference placeholders.

- **Outbound** (`crdt/formula.rs`): when a `SetCellValue` diff carries a formula,
  parse it with the existing parser (`base/src/expressions/parser`), resolve each
  reference's row/column index to an id using the session's current order maps,
  emit the id-form.
- **Inbound**: render id-refs back to A1 text using the *local current* order
  (id → display index), hand the text to the model. A reference to a tombstoned id
  renders as `#REF!` (the parser's `WrongReferenceKind` already models this).

Consequences:

- **Structural operations never rewrite formulas in the doc.** `displace_cells` /
  `DisplaceData` (`actions.rs`) remain a purely local-model mechanism; the
  edit-vs-insert formula conflicts of `CRDTs-in-IronCalc.md` §3.1/§3.5 vanish.
  Concurrent "write `=A5`" and "insert row 3" converge with the formula pointing
  at the original logical cell, which now renders as `=A6`.
- **Ranges** (`A1:B5`, CF `sqref`s, defined names) carry id endpoints — the
  AegisSheet "anchored markers" idea. Insertions strictly inside a range are
  included automatically (they sort between the endpoints); interior deletions
  shrink it. A tombstoned endpoint renders as `#REF!` for that endpoint
  (`=SUM(A1:#REF!)`) — **matching the engine's own `displace_cells` semantics**,
  which the codec must reproduce exactly for render-equivalence (the design
  originally called for Excel-style inward clamping; the engine does not clamp
  endpoints — if that ever changes engine-side, the codec follows). The dead
  endpoint id stays in the doc, so a resurrected row can heal the range.
- Relative vs absolute refs: the id-form stores the *resolved target* plus the
  original abs flags, so rendering reproduces the exact `$`-style the user typed.
  Copy/paste and autofill run locally in the engine as today; their per-cell
  result diffs are translated outbound like any other edit.
- Cross-sheet refs carry `SheetId`, so `renameSheet` never breaks doc formulas.

**Row 1,048,576 edge:** with ids, "insert shifts the last row off the grid" becomes
a render-time truncation — order all visible rows, render the first 1,048,576. The
local engine already refuses inserts that would push content off; concurrent
merges that overflow simply leave the tail unrendered (and restorable). 

## 9. Styles

Style content is **content-addressed**: the doc-level `"styles"` pool maps
`hash(style JSON)` → style body; cells, rows, columns and named styles reference
the hash. Two replicas defining the same style converge on the same key by
construction — this eliminates the `xf_id` allocation collision
(`CRDTs-in-IronCalc.md` §3.8) without shipping the interning tables. The engine's
`cell_xfs` interning remains a local optimization rebuilt at import time.

- Cell style: `s: styleHash` inside the cell entry — LWW with the content write
  (an edit and a restyle of the same cell race whole-cell LWW; acceptable v1,
  splittable into a second register later if it annoys).
- Row/column style, width/height/hidden: LWW fields on the row/col entry. Because
  cells inherit row/column style by lookup at render time, the "full-column style
  misses a concurrently inserted cell" divergence (§3.3) disappears.
- Named styles: `"named_styles"` YMap name → hash; apply-to-cell writes the hash
  into the cell (dereferenced at apply time, as the engine does today).
- The auto-grow row height side effect of `set_user_input` arrives as a separate
  `SetRowHeight` diff and becomes an ordinary LWW write to the row entry — two
  users editing different cells in the same row race on height and converge LWW.

## 10. The remaining dimensions

### 10.1 Conditional formatting

Rules are `RuleId → {pos, body, ranges}`. Priority is the fractional `pos`
(raise/lower = position write — no more index-keyed swaps that break under
concurrent add/delete, §3.7). The rule body is whole-value LWW; ranges use id
endpoints (§8). Evaluated appearance is recomputed downstream from the converged
rule order, like formulas.

### 10.2 Borders: per-edge registers

Borders live on **grid lines, not cells** — the line between `A1` and `B1` has one
identity (`v_edges["colId(B):rowId(1)"]` = "left of B at row 1"; `h_edges`
analogous for "top of"). This dissolves the shared-edge conflict by construction:
`setAreaWithBorder` on adjacent ranges writes the same register instead of
fighting over two cells' styles. Each edge register is LWW (a practical
simplification of the heaviest-wins lattice; `None` = an ordinary LWW write, so
set-vs-clear is deterministic). Outbound translation maps the engine's
write-into-neighbour behavior (`border.rs`) onto edge writes; inbound renders the
four surrounding edges into the cell styles the engine expects.

### 10.3 Merged cells

`MergeId → {tl, br}` with id-anchored corners. Structural inserts inside a merge
grow it naturally; deleting a corner clamps like ranges. Concurrent overlapping
merges converge in the doc (both entries exist) and are resolved by a
**deterministic render-time fixup**: keep the winner by (Lamport, id) tiebreak,
ignore overlapped losers (CKEditor's "post-fixer" pattern). The loser entry
remains and re-emerges if the winner is unmerged.

### 10.4 Defined names

`"names"` YMap keyed `scopeSheetId|name` → formula in id-ref form. Concurrent
same-key creation = LWW; rename = remove + add (the remove masks, so a concurrent
edit of the old name resolves LWW on the key). Structural edits never displace
name ranges (id endpoints).

### 10.5 Workbook settings

`theme, locale, tz, language, name` — plain LWW registers. Trivially convergent.

## 11. Communication layer

`crdt/sync.rs` implements the y-sync protocol over a transport abstraction:

```rust
trait Transport {                       // any ordered, lossy-tolerant pipe
    fn send(&self, msg: Vec<u8>);
    fn on_message(&self, cb: impl Fn(&[u8]));
}
```

- **Sync**: on connect, exchange state vectors → each side sends
  `diff_update(their_sv)` → apply. Steady state: broadcast incremental updates on
  every local transaction. Updates are idempotent and may arrive duplicated or
  out of order across reconnects — yrs handles both (pending-update queue for
  causal gaps).
- **Local-first**: nothing above assumes a server. A websocket relay room is the
  first transport (fan-out + persistence convenience); replicas can equally sync
  peer-to-peer or via files. The server never inspects or orders content.
- **Awareness** (presence, cursors, selections): the ephemeral y-awareness
  channel, never persisted. This fits the existing design — view state is already
  excluded from the diff stream (the `SetViewDiffs` FIXME in `history.rs`).
- **Persistence**: doc snapshot (`encode_state_as_update`) plus append-only update
  log, compacted on save. Relationship to `.ic` files is an open item (§13).

**As implemented (phase 9):** there is no `Transport` trait — `crdt/sync.rs`
exposes `SyncPeer`, which speaks in opaque *byte frames* (each one or more
y-sync `Message`s, lib0 v1 — y-websocket-compatible), and the caller owns the
pipe: `start_sync()` on (re)connect, `handle_frame()` for every incoming
message (returns replies plus re-render flags), `flush_local()` after local
edits, `set/clear_presence()` + `presence()` for awareness. Outbound updates
are **incremental**, collected by a doc update-observer that filters out
remote-origin transactions (a `REMOTE_ORIGIN` tag on `apply_remote`'s
transaction): `encode_state_as_update(sent_sv)` was abandoned because it
re-ships the **full delete set** on every flush and makes "nothing to send"
undetectable. The relay server (`collab-server/`) holds one doc per room and
broadcasts every *integrated* update (observer-driven) to the whole room,
source included — receivers deduplicate, updates are idempotent.

## 12. Undo

Undo stays **local and selective** (undo your own ops only), reusing the existing
`History`. An undo pops the `DiffList` and applies inverses locally; the session
translates those inverse diffs outbound like any other local change — i.e. undo is
an explicit new operation, exactly AegisSheet's model, and their Table 4 semantics
follow: undoing an edit concurrent with a remote edit of the same cell resolves
LWW; undoing a structural op whose target ids were concurrently removed is a no-op.

One requirement: undoing `DeleteRows` must resurrect the **same** `RowId`s (and
their keep-entries and masked cells), not mint fresh ids — otherwise concurrent
references to the deleted rows stay broken. The session therefore journals, per
`DiffList`, the ids each op touched, so inverses are id-precise. (This is also why
`Diff::DeleteRows.old_data` alone is not enough: it stores contents, not ids.)

yrs's own `UndoManager` is not used: undo semantics must operate at the
user-intention level (a `DiffList` = one user action, possibly several doc writes),
and the engine already owns that grouping.

## 13. Alternatives considered

1. **yrs-native sequences for order** (Y-Arrays of row ids, `move_range_to`).
   Rejected: a 1M-row grid forces materializing the used range up front; pristine
   rows between touched ones need "gap" entries whose concurrent splitting does
   not converge; array items are awkward to reference from cell keys. The
   fractional-position map keeps yrs for what it is great at (LWW maps, sync) and
   makes order data, not structure.
2. **No yrs — extend the existing `QueueDiffs` channel** with Lamport stamps and
   ids (bitcode wire format, no new dependency). Rejected: we would hand-roll
   causal delivery, state-vector sync, idempotence, pending-gap buffering, and
   persistence — exactly the machinery where subtle bugs live, and exactly what
   yrs has already hardened. The dependency is wasm-clean.
3. **Doc-first architecture** (yrs doc as primary store, model rebuilt from it;
   single apply path for local and remote). Cleaner in principle — no
   two-paths-must-agree obligation — but it rewires `UserModel` internals, undo,
   and every binding. The mirror gets us collaborative IronCalc incrementally;
   doc-first remains a possible end-state once the schema is proven.
4. **Formula alternatives**: text + replicated rewrite (zero translation work, but
   structural ops fan out O(formulas) writes and LWW-eat concurrent formula
   edits); id refs inside the engine (kills `displace_cells` entirely, but rewires
   parser/evaluator/xlsx). Decision: **id refs at the doc boundary only** — the
   sweet spot; engine-wide ids stay a possible v2.
5. **Multi-value registers** for same-cell conflicts (AegisSheet §4.2). Deferred
   by decision: LWW. The cell entry being a small struct means an MV upgrade is a
   value-schema change, not a topology change.
6. **Heaviest-wins lattice for borders** — elegant (a true state-lattice join) but
   set-vs-clear still needs causal tiebreaking; plain LWW per edge is simpler and
   deterministic. Revisit if LWW feels wrong in practice.

## 14. Test design

**Harness**: two (later N) `UserModel` + `crdt::Session` pairs. Script concurrent
operations, exchange updates in controlled orders, then assert (a) doc state
vectors converge, (b) workbooks are cell-by-cell identical — including styles,
props, CF, merges — and (c) **evaluation results are identical** (results are
never shipped, so this checks deterministic recomputation). Template: the existing
`base/src/test/user_model/test_diff_queue.rs` round-trip pattern.

**Pairwise convergence cases** (each in both delivery orders):

| # | Scenario | Expected |
|---|---|---|
| 1 | edit `B2` ‖ edit `B2` | LWW, same winner both sides |
| 2 | edit `B5` ‖ insert row 2 | edit lands at `B6` |
| 3 | insert row 3 ‖ insert row 3 | both rows, deterministic order, no duplication |
| 4 | delete row 4 ‖ delete row 4 | idempotent single delete |
| 5 | delete row 4 ‖ edit `A4` | **update-wins**: row survives with all cells |
| 6 | delete col B ‖ set col B width | update-wins: column survives |
| 7 | `=A5` written ‖ insert row 3 | formula renders `=A6`, targets same logical cell |
| 8 | delete row 5 ‖ cell elsewhere has `=A5` | `#REF!` on both replicas |
| 9 | `=SUM(A1:A10)` ‖ insert row 5 / delete row 10 | range grows / clamps identically |
| 10 | move rows 2–3 → 7 ‖ edit `A2` | edit travels with the moved row |
| 11 | move row 2 → 5 ‖ move row 2 → 8 | last move wins, single row |
| 12 | move rows ‖ hide rows (the hidden-delta quirk) | resolved positions, not deltas |
| 13 | rename sheet ‖ rename sheet | LWW |
| 14 | rename sheet ‖ delete sheet | keep-set: rename resurrects (update-wins) |
| 15 | new sheet "X" ‖ new sheet "X" | two sheets, deterministic "X (2)" fixup |
| 16 | style `B2` ‖ edit `B2` | whole-cell LWW, documented |
| 17 | edit `A1` ‖ edit `C1` (both auto-grow row 1) | row height converges LWW |
| 18 | border around `A1:B2` ‖ border around `C1:D2` | shared edge B/C converges (one register) |
| 19 | CF raise priority ‖ CF add rule | order converges, no index skew |
| 20 | merge `A1:B2` ‖ merge `B2:C3` | deterministic winner, loser masked not lost |
| 21 | create name "foo" ‖ create name "foo" | LWW on key |
| 22 | 100 mixed ops offline each side, single merge | convergence + identical eval |
| 23 | duplicate delivery / out-of-order delivery / late joiner from state vector | idempotent, converges |
| 24 | undo own edit after remote interleave | only own op reverted, LWW vs remote |
| 25 | undo `DeleteRows` after remote edits elsewhere | same `RowId`s resurrected, refs intact |

**Property-based fuzzer** (the backbone, and the guard on the mirror
architecture's two-apply-paths obligation): 2–4 replicas, random op sequences
drawn from the full in-scope `Diff` vocabulary, random partition/sync schedules,
seeded and shrinkable, run in CI. Invariants: state-vector convergence ⇒
byte-identical workbook projections ⇒ identical evaluated values; plus schema
invariants (no orphan cells for never-existing ids, keep-set/visibility
consistency, position strings bounded).

**Unit layers below**: id codec round-trips; fractional-index generation
(midpoints, tiebreaks, rebalancing bounds); formula A1↔id translation round-trips
including `$` flags, cross-sheet, ranges, `#REF!`; keep-set semantics against a
model checker of the paper's rules.

## 15. Open items

1. **Doc persistence & lifecycle vs `.ic` files** — snapshot+log format, when a
   workbook "becomes" collaborative, export back to plain `.ic`/xlsx.
2. **Purge/GC cutoff policy** for tombstoned rows/cols and masked cells (P2P makes
   "everyone has seen it" undecidable; probably wall-clock cutoff à la AegisSheet).
3. **Fractional-index rebalancing** bounds and the concurrent-insert-during-
   rebalance edge; validate with the fuzzer.
4. **Very large pastes** (100k cells) — one yrs transaction per `DiffList` should
   batch fine, but measure update size and apply cost.
5. **Key-size optimization** (client registry) if profiling shows map-key overhead.
6. **Tables** (`workbook.tables`) and comments — not yet mapped; same patterns
   apply (id-anchored ranges, LWW bodies).
7. **Multi-value upgrade path** for same-cell conflicts if LWW proves too lossy.

## 16. Implementation phases

Each phase lands with its convergence tests; the fuzzer grows with the vocabulary.

- **0. Scaffolding** — `crdt/` module, `yrs` dep, id codec, fractional index,
  schema, session skeleton, two-replica test harness.
- **1. Cell content core** — cells map, outbound/inbound for
  `SetCellValue`/`RangeClearContents`, values only (formulas passed as text
  temporarily). Tests 1, 22, 23.
- **2. Structure** — rows/cols entries, insert/delete, keep-sets, visibility,
  props (height/width/hidden). Tests 2–6, 17.
- **3. Formulas** — id-ref translation both ways, ranges, `#REF!`, defined names.
  Tests 7–9, 21. Split into:
  - **3.1 Reference extraction** — DONE: `crdt/formula.rs::extract_reference_spans`
    over the lexer's positioned tokens (char spans; leading whitespace
    trimmed). Structured/table references and illegal tokens make the formula
    *unsupported* → the caller stores plain text (fallback keeps the fan-out).
  - **3.2 Id-form codec** — DONE: `encode_formula`/`render_formula` with
    `\u{1F}`-delimited id tokens (`[s<sheet>;]<a|r><colId>;<a|r><rowId>[:…]`),
    doubling-escape for the delimiter in string literals. Tombstoned ref →
    `#REF!`; range endpoints clamp inward via `AxisOrder::resolve` (`Gone{rank}`);
    crossed → `#REF!`; deleted sheet → `#REF!`; renames render the new (quoted)
    name. Canonical source text = `Model::get_english_cell_formula` (English
    functions; number-locale canonicalization still open). Note: `D:D` renders
    back as the explicit `D$1:D$1048576` form (semantically identical).
  - **3.3 Session wiring** — DONE: `DocResolver` (sheet display names + all
    sheets' axis orders) with outbound (`from_ctx` + model names), inbound
    (`from_projection` + deduped display names) and bootstrap views; encode in
    `read_cell_for_doc`, render in `set_projected_cell`; fan-out kept on.
    Findings: (a) with id-form the fan-out no longer transmits displacement,
    so **reconcile re-renders id-form cells on every sheet whenever any
    sheet's order changed** (cross-sheet refs; delta-path sheets included);
    (b) the codec had to adopt the engine's endpoint-`#REF!` range semantics —
    the cross-check caught the clamping mismatch immediately (see §8);
    (c) the traced fuzzer now verifies engine-displacement ≡ codec-render at
    every sync. Unsupported formulas (structured refs …) stay plain text.
  - **3.4 Remove the fan-out** — DONE: `touch_all_formulas` →
    `touch_at_risk_formulas`, marking only (a) plain-text fallback formulas
    and (b) id-form formulas with an *overflowed* reference (demoted to plain
    text, see below). Equivalence proven by a 72-case matrix test
    (9 formulas × 8 structural ops, model↔render checked on both replicas)
    plus the traced fuzzer. Two engine behaviors the codec had to replicate:
    **full ranges are pinned** (`D:D`/`5:9` skip displacement on their
    spanning axis iff both endpoints are absolute and span 1..LAST — codec
    encodes pinned literal endpoints, renders the short form) and **near-edge
    refs displace out of the grid** (`=A1048577`, rendered literally, then
    *frozen* as an identifier — codec renders `Overflow` ranks literally and
    demotes such formulas to plain text so they freeze identically).
    Bonus from making undo-of-delete always doc-authoritative (repair): id
    tokens pointing at a resurrected row **heal** (`#REF!` → `=A5`), which the
    engine's own undo cannot do; the drift-detection journal machinery
    (cross-axis snapshot) became unnecessary and was removed.
  - **3.5 Deletion semantics end-to-end** — DONE (absorbed by 3.4): tests 7–9
    live in the equivalence matrix and the dedicated `#REF!`/range tests;
    healing after undo-resurrect has its own test
    (`resurrected_row_heals_references_to_it`).
  - **3.6 Defined names** — DONE: `names` root map (`<scope>|<name>` → id-form
    or plain), no per-diff translation — pass 2 re-syncs the whole (small)
    name map from the post-batch model, which handles create/update/delete/
    rename and undo uniformly; renames also fan out the dependent cells the
    engine rewrote. Inbound applies by delete+recreate against the rendered
    list; concurrent same-name creation is key-level LWW (test 21).
    Engine finding: **defined names are NOT displaced by structural edits**
    (`Sheet1!$A$5` stays `$A$5` after inserting a row — Excel displaces;
    possible engine issue). The collab layer replicates the engine
    faithfully; if the engine gains displacement, `sync_names` re-encoding
    picks it up automatically.
  - **3.7 Fuzzer extension** — DONE: vocabulary now includes range formulas,
    `$`-anchored refs, `D:D`, name-referencing formulas, and defined-name
    create/update/delete. `SetArrayValue` (CSE arrays) remains approximate:
    replicated per-cell as dynamic formulas (documented TODO).
- **4. Moves** — DONE: `move_axis` translates `MoveRows`/`MoveColumns` into
  LWW position-register writes (remove from the cached order, re-insert
  before the element at `from + delta` in the block-less order — the engine's
  block semantics). Cells, properties, keep-sets and id-form references
  travel with the ids untouched; inbound needs nothing (the structural
  rebuild path covers position changes). The hidden-rows delta quirk resolves
  itself: `move_rows_action` adjusts the delta *before* recording the diff.
  A move keep-adds the moved ids, so it preempts a concurrent deletion
  (update-wins, AegisSheet semantics — tested). Undo is an inverse move at
  current indices (exactly how the model replays it) — no journal needed.
  Engine finding: crossed ranges are **normalized per axis with the absolute
  flag traveling with its coordinate** (`$D$5:B9` → `B$5:$D9`) — a move that
  drags one range endpoint past the other must render normalized; the matrix
  test caught the mismatch and the codec now replicates it. Tests 10–12 plus
  move‖delete, move‖move, formula-follows-move, undo-of-move; moves are in
  the fuzz vocabulary and the equivalence matrix.
- **5. Sheets & workbook** — DONE: sheet keep-sets (`keep_sheets` root map;
  every positive op on a sheet — cell/prop/keep/meta write — keep-adds it, so
  edits and renames resurrect a concurrently deleted sheet, update-wins;
  tests 13–15 plus edit‖delete-sheet). Deterministic zero-sheets fixup: if
  concurrent deletions tombstone everything, the min-`(pos,id)` sheet stays
  visible on every replica. Sheet meta now covers tab color / state
  (hide/unhide) / grid lines (session codecs, no serde_json) and workbook
  locale + timezone as LWW registers. Reconcile's sheet alignment was
  rebuilt: working-list with deferred last-sheet deletions (insert the
  survivor first), placeholder names for insertions, and a **two-phase display
  -name rename** (fuzz-found: name swaps and concurrent renames of different
  sheets to the same name collide transiently; and the dedupe fixup must also
  run after *local* translation, since a local deletion can dissolve a name
  collision). Undo of delete-sheet resurrects the same id only when coherent
  (id still invisible and positional slot unchanged), else registers the
  model's restored sheet as a fresh one. Fuzz vocabulary: sheet create /
  delete / rename. Fuzz-found formula corner: engine range normalization is
  *stateful* (endpoints physically swap in the node), so crossed ranges are
  re-encoded from the model text like overflow (`needs_reencode`).
- **6. Styles** — DONE: content-addressed pool (`styles` root map, fnv1a-128
  over the style's bitcode bytes — no serde_json dependency, no shared xf_id
  allocation); per-cell styles live in their own root map (`cell_styles`), an
  **independent LWW register per cell**, so concurrent style and content edits
  of the same cell both survive (upgrade over the whole-cell-LWW design,
  test 16+). Row/column styles as `sty` fields on axis entries; named-style
  *definitions* replicate (name → bitcode `(Style, StyleIncludes)`), while
  `ApplyNamedStyle` replicates the flattened result — the cell↔named-style
  *link* stays local (documented corner: updating a definition re-resolves
  only linked cells; conservative re-marking of all styled locations keeps
  the originator consistent). Styles travel with moved rows, survive
  resurrects (masked, never erased), and structural rebuild/repair handle
  them like content. Fuzz-found: **content ops can change styles** — an undo
  of `SetCellValue` restores/removes the whole old cell including its style —
  so content marks imply style marks, with a changed-vs-shadow guard so plain
  content edits never stomp the independent style register. Themes remain
  unreplicated (TODO).
- **7. CF, borders, merges** — Tests 18–20.
  - **7.1 Conditional formatting** — DONE: `cf` root map,
    `<sid>!<ruleId>.p` (fractional position = priority order; raise/lower is
    a position write) + `<sid>!<ruleId>.v` (bitcode `(range, CfRule,
    Option<Dxf>)`, range and rule formulas in id-form, the rule's `dxf_id`
    zeroed and the dxf *content* inlined — replica-local dxf ids never cross
    the wire; receivers intern into their own pool). Pass 1 only mirrors the
    engine's index/priority bookkeeping per sheet (`(RuleId, priority)` vec,
    including the engine's undo-by-priority-match and swap semantics); a new
    pass 2 (`sync_cf`) compare-and-writes registers from the post-batch
    model, so structural displacement that the ids already capture costs
    zero writes and concurrent CF body edits survive structural ops.
    **Canonical form invariant**: after attach, after every reconcile and
    after every CF-touched local batch, the model's CF vector follows the
    document — `(pos, id)` order, priorities renumbered 1..n, text
    normalized to the codec render — so replicas store byte-identical CF
    state (dxf ids excepted, resolved by content). Engine findings: CF sqref
    displacement *differs* from formula-range displacement (a deleted range
    corner leaves the part untouched instead of clamping; a move can leave
    crossed corners textually swapped where the codec render normalizes) —
    both heal through re-encode/canonicalize, doc render wins. Tests: test
    19 (raise ‖ add), add‖add, update‖delete both orders, range follows
    remote insert, undo/redo of add, duplicate-sheet CF; fuzz vocabulary
    now includes CF add (CellIs/Formula/DataBar), delete, raise/lower,
    update (200-seed traced + 500-seed release green).
  - **7.2 Borders (per-edge registers)** — DONE: `edges` root map,
    `<sid>!v.<cid>:<rid>` (line left of column `cid` at row `rid`) /
    `<sid>!h.<cid>:<rid>` (line top of row `rid`), value = session-encoded
    `BorderItem`, plain LWW per line — the shared-edge conflict is dissolved
    by construction (test 18). Replicated **cell** styles are stripped of
    left/right/top/bottom (diagonals stay; row/column styles keep their
    borders in the axis channel — documented limitation). Key engine
    insight: after any `set_area_with_border` the *visible* edge always
    equals `max(two adjacent sides)` under `is_max_border` (the primary side
    is absolute, the neighbour is demoted only when heavier — and
    `UserModel::get_cell_style` applies exactly this max at render time), so
    the edge register faithfully carries the max and canonicalizing both
    sides to it never changes what the user sees. **Composition invariant**
    (fuzz-found, the hard part): the set of cells carrying composed border
    sides must be a *pure function of the document* — edges compose ONLY
    into cells that have a `cell_styles` register (a border-only explicit
    style now writes a default-hash register to mark edge ownership), and
    bordered model cells without a register are stripped. Anything keyed to
    replica-local state (explicit-style presence differs between originator
    and receivers) eventually diverges. Canonicalization runs after
    CF/border-touched batches and — wholesale, register cells plus bordered
    model cells — after every structural batch/reconcile (deleted
    rows/columns orphan edge registers while the engine keeps the sides).
    This also makes undo of a border op converge back to no border. Style
    pool format break (stripped styles hash differently). Tests: test 18
    (shared edge coherent), border ‖ same-cell style edit (edge survives —
    the register-independence payoff), border follows remote row insert,
    clear, undo; fuzz vocabulary: `set_area_with_border`
    All/Outer/Inner/Top/Left/CenterH/None × thin/medium/thick.
  - **7.3 Merged cells** — test 20, pending (engine support itself is
    limited).
- **8. Undo integration** — id-precise journal. Tests 24–25.
- **9. Transport & awareness** — DONE: split as 9.1 protocol peer /
  9.2 relay server / 9.3 persistence / 9.4 integration tests / 9.5 wasm
  bindings.
  - **9.1 `crdt/sync.rs::SyncPeer`** — y-sync over byte frames (see §11 "as
    implemented"): handshake (`SyncStep1/2`), incremental `Update`s via a
    remote-origin-filtered doc observer (no echo, no full-delete-set
    re-shipping), awareness presence (opaque JSON states; constant clock on
    wasm — `SystemTime` panics on wasm32-unknown-unknown). **yrs 0.27.3 bug
    found** (confirmed on bare yrs, no engine code): an update applied with a
    causal gap parks its map-key-*overwrite* items (origin = previous item
    for the key — exactly our keep-set writes) in the pending queue and
    **never re-integrates them when the gap fills** — fresh-key items
    integrate behind a skip and the docs silently diverge. Worth reporting
    upstream. Mitigation everywhere (client and server): `classify_update`
    walks the update's insertion ranges against the local state vector; a
    gapped update is never handed to yrs — it goes to a bounded stash,
    triggers a `SyncStep1` resync request, and is retried to a fixpoint
    after every successful apply (duplicates classify as `Empty` and are
    dropped). On an ordered websocket pipe gaps only arise across
    reconnects, where the handshake heals anyway — the guard makes it
    deterministic. Star-topology echo suppression: `apply_remote` advances
    `sent_sv` for clients the remote update extended; `handshake_diff` marks
    the full state sent. 10 peer tests + a peer-level fuzzer (random ops,
    duplicated frames, partitions, final re-handshake; green at 200 seeds).
  - **9.2 `collab-server/`** — workspace crate, yrs + tokio + tungstenite
    only (never sees cell content). One room per URL path (names validated:
    they double as file names). Fan-out is observer-driven off the room doc,
    so handshake-carried and steady-state content broadcast uniformly; the
    same gap guard protects the room doc; awareness states are kept for late
    joiners and pruned (with a broadcast) when a connection drops without a
    goodbye. Slow consumers that overflow the broadcast buffer are
    disconnected and heal by reconnect handshake.
  - **9.3 Persistence** — per room: snapshot (one full-state update) +
    length-prefixed append-only log, appended from the update observer,
    replayed *before* the observer subscribes (no re-log/re-broadcast).
    Torn tails from crashes are truncated on load. Compaction (log > 1 MiB)
    rewrites the snapshot via tmp-file + rename and truncates the log — and
    runs strictly *outside* the observer, whose transaction is still open
    (reading the full state there deadlocks). Awareness is never persisted.
  - **9.4 Integration tests** — in-process server, real websockets, real
    `SyncPeer` clients: convergence + room isolation, presence relay/late
    joiner/disconnect pruning, and restart recovery (edits → kill server →
    fresh server on the same data dir serves a new client the workbook,
    formulas evaluating). The server logs before broadcasting, so the echo
    of your own update is a deterministic "persisted" barrier — no sleeps.
  - **9.5 wasm bindings** — `collabAttach/collabStartSync/collabHandleFrame/
    collabFlushLocal/collabSetPresence/collabClearPresence/collabPresence`
    on `Model`; JS owns the websocket and shuttles opaque `Uint8Array`
    frames (multi-message frames are legal, so reply lists pack into one
    buffer); `CollabFrameOutcome.appliedUpdate/presenceChanged` drive
    re-rendering. Node tests run the real wasm build peer-to-peer.
  - Webapp UI wiring (websocket provider, remote cursors) is **phase 10**.
- **10. Webapp UI wiring** — split as 10.1 websocket provider / 10.2 remote
  cursors from `collabPresence` / 10.3 session UI (share/join flow, status
  indicator, collaborator list).
  - **10.1 `CollabProvider`** — DONE:
    `webapp/IronCalc/src/collab/CollabProvider.ts` (exported from
    `@ironcalc/workbook`) owns the websocket and shuttles the opaque frames:
    handshake via `collabStartSync` on every (re)open, incoming messages
    through `collabHandleFrame` (replies sent back, `remoteUpdate` /
    `presenceChange` / `statusChange` events fan out to subscribers), local
    edits shipped on a 200ms interval via `collabFlushLocal` (called even
    while offline so edits fold into the doc and the reconnect handshake
    heals them), reconnect with doubling backoff, presence passthrough
    (`setPresence` serializes to JSON; awareness re-publishes itself through
    the handshake, so presence set while offline survives). The websocket is
    injectable (`createWebSocket`) for tests. UI: `IronCalc`/`Workbook` take
    an optional `collabProvider` prop; a `remoteUpdate` subscription bumps
    `setRedrawId` *inside* `Workbook` (an app-level re-render would recreate
    `WorkbookState` and kill an in-progress cell edit). App wiring
    (`app.ironcalc.com`): `?room=<name>` starts from a blank workbook (the
    room doc is authoritative), connects to
    `VITE_COLLAB_SERVER_URL ?? ws://<host>:9000`, skips the localStorage
    autosave, and destroys the provider (goodbye frame) on unload. Tests:
    provider-level peer-to-peer over fake sockets (the y-sync protocol is
    symmetric) covering both-ways sync, presence, destroy and
    offline-edit healing across a reconnect
    (`webapp/IronCalc/tests/collabProvider.test.ts`), plus an integration
    test through the real relay binary — late joiner receives the workbook —
    that skips itself when the binary is not built
    (`tests/collabServer.integration.test.ts`).
  - **10.2 Remote cursors** — DONE: `src/collab/presence.ts` is the only
    place that knows the presence JSON shape
    (`{name, sheet, row, column, range}`); `decodeCursor` validates
    defensively (states come from other clients), `decodeCursors` filters
    our own client id, and colors derive from the client id
    (`colorForClient`, 8-color palette) so replicas agree without
    transmitting them. Publishing: a `Workbook` effect (no deps) publishes
    the selection from `getSelectedView` after every render, deduped
    against the last published state **per provider** — StrictMode-found
    bug: the app's double-run start effect creates a provider that is
    immediately replaced, and a dedupe keyed only on content suppressed the
    re-publish on the surviving provider, so a client that never moved its
    cursor was invisible. Rendering: `WorksheetCanvas` takes an optional
    `remoteCursors` getter (threaded `Workbook` → `Worksheet` → canvas
    options) and `drawRemoteCursors` paints, per cursor on the selected
    sheet, the viewport-clamped selected range (1px stroke + 10% fill, the
    `drawActiveRanges` idiom) and the active cell (2px stroke) with a name
    flag above it; `presenceChange` bumps the same redraw id as
    `remoteUpdate`. `?name=<user>` sets the display name (provider
    `userName` option, default "Guest"). Verified by unit tests
    (`tests/collabPresence.test.ts`, plus a provider-level decode test) and
    a two-browser playwright run (pixel-checking the palette colors on the
    peer's canvas; cursor appears both ways and disappears when the peer
    leaves). Dev-loop gotcha: the app consumes `@ironcalc/workbook` from
    its built `dist/`, so library edits need `npm run build` before a vite
    dev server shows them.
  - **10.3 Session UI** — DONE (app-level, `frontend/src/components/Collab/`):
    a "Collaborate" button in the file bar turns the *current* workbook into
    a live session — `CollabSession::attach` bootstraps the full content
    into the doc (deterministic `Original` ids), so the app just creates a
    provider on the existing model, connects to a fresh room
    (`crypto.randomUUID()` sans dashes; server room names allow
    `[A-Za-z0-9._-]{1,64}`), rewrites the URL to `?room=<id>` via
    `history.replaceState` (dropping `model`/`example`), and the flush
    interval pushes the workbook up; joiners receive values, formulas and
    styles. Once live the button becomes a status chip: colored dot
    (green/amber/red from `onStatusChange`) plus stacked collaborator
    avatars (initial + `colorForClient` color, "+N" overflow) fed by a
    `useCollaborators` hook decoding the presence map on every
    `presenceChange` — self sorted first. Clicking opens `CollabDialog`
    (share-dialog styling): QR code, invite URL (`origin/?room=…`, each
    joiner appends their own `&name=`), copy button and the live
    collaborator list. The static "Share a copy" button stays separate.
    i18n keys under `file_bar.collab` in all five app locales. Verified by
    a playwright run: local workbook with content → Collaborate → joiner
    gets the content, both bars show 2 avatars, dialog lists "Bo (you)|Ana",
    live edits flow, avatar count drops when a tab closes.  - **10.4 Large workbooks** — DONE (harness `base/examples/collab_bench.rs`:
    host attach + in-process relay room + joiner + one edit each way,
    `cargo run --release --example collab_bench -- <rows>`; measured on the
    `forward_chain` example). Three fixes: (1) `reinherit_cell_styles` scanned
    the whole sheet for a single-cell scope and the delta path calls it once
    per written cell — O(n²) joins (100k rows: 170 s → 0.8 s). (2) The shadow
    is no longer rebuilt with `Projection::from_doc` on every remote update
    and every local flush (O(doc) per keystroke): every root map has an
    observer collecting changed keys, `refresh_shadow` re-reads just those
    (`Projection::patch`, which `from_doc` itself is built on; a `cfg(test)`
    assert keeps them equal) and produces a `Delta` with the old values of
    the changed keys, which drives `reconcile`/`reconcile_sheet`. Structural
    sheets still take the full rebuild with the old sheet snapshotted into
    the delta. `evaluate()` is skipped for style/link/border/size-only
    deltas. At 1M cells: flush 1.6 s → 10 ms, receive 1.9 s → 0.3 s (the
    engine's full evaluate). (3) UX/transport: relay message cap raised from
    tungstenite's 64 MiB default (a 1M-cell full state is 31 MB) to 1 GiB;
    the provider queues frames and reports a `syncing` status, deferring a
    large frame past a paint so the joiner shows a loading overlay; the
    Collaborate click paints the dialog/URL before attaching. (4) Wire
    compression: a frame ≥ 64 KB travels as `Message::Custom(0x10,
    gzip(frame))` (`collab-server/src/compress.rs`; the relay wraps replies
    and, once per update, the fan-out, and unwraps what clients send; the
    provider does the same with `CompressionStream`/`DecompressionStream`,
    keeping both directions ordered through queues since the browser
    codecs are asynchronous). yrs v1 state gzips 6-7x (300k rows: 9.8 MB →
    1.5 MB), so the join download of a 1M-cell workbook drops from ~31 MB
    to ~5 MB. Verified by `large_workbooks_travel_compressed` (relay
    integration test) and a Node run of the real provider over the relay.
    (5) Join apply: cells landing on empty locations (every cell on a join,
    every cell of a structural rebuild) take `write_fresh_cells` →
    `Model::set_cell_input_with_style`: the interactive pipeline's spill
    preparation, quote-prefix/number-format/units restyling and auto-linking
    are skipped because the document's registers already carry the exact
    style and link — the style index is derived from the register (composed
    with the edge registers like `set_projected_cell_style`) or the
    inherited row/column style, memoized per pool hash, and the link and
    restyle passes skip those cells. The map observers now record the key
    *and the new value* straight from the event (`DirtyEntry`) instead of
    copying keys into a sorted set and looking values up again, and the
    peer decodes an update once. 1M-cell join: 11.8 s → 8.8 s native, of
    which yrs decode+apply 1.3 s, yrs change events 1.0 s, shadow patch
    ~0.8 s, formula render 0.6 s, engine parse+insert 2.6 s, evaluate 0.4 s.
    (6) Measured in V8 (Node with the web wasm build, `bindings/wasm/pkg`
    loaded via `init({module_or_path: bytes})` — same engine as Chrome), the
    join was 23 s, not the native 8 s: allocation-heavy code runs 3-7x
    slower under wasm's allocator, and the engine's A1 parse alone was
    5.6 s for a million `=A<n>+1` cells. Fix: `render_formula_rc` renders
    an id-form formula straight into the engine's internal R1C1 form
    relative to its own cell (`=R[-1]C[0]+1` for every cell of the column),
    `Model::set_cell_with_rc_formula` parses that text once (lexer in R1C1
    mode, no leading `=`, context-independent) and returns the shared
    formula index, and `Model::set_cell_with_formula_index` writes the other
    cells with no parse at all (`write_fresh_cells` keeps the text → index
    cache per call). Shapes the fast path does not cover (pinned/full
    ranges, crossed ranges, dead endpoints, missing sheets) fall back to the
    A1 path. V8 join apply: 23 s → 11 s (engine write 13.4 s → 1.4 s).
    (7) A `talc` global allocator was tried and made no measurable
    difference (10.5-12.5 s vs 10.8-11.0 s), so allocation count is not the
    wasm penalty. A V8 CPU profile (node `--cpu-prof` with the wasm name
    section kept: `wasm-opt = ["-O", "-g"]` in the wasm-pack profile) put
    the rest in our own code: id→index resolution allocating a position
    string per lookup, `format!` chains in the R1C1 renderer, a second
    million-entry BTreeMap for the join delta, and the derived `EntityId`
    ordering. Fixes: `AxisOrder::resolve` fast path for a pristine order,
    allocation-free `render_payload_rc`, `SheetDelta.cells` as a Vec, and
    `EntityId::Ord` on a packed u128 key. V8 join apply: 23 s → ~10 s.
    What is left (V8): yrs decode+apply ≈ 2.5 s, yrs change events ≈ 1.1 s,
    shadow patch ≈ 1.2 s, evaluate ≈ 1.1 s, R1C1 render ≈ 1 s, cell inserts
    ≈ 0.6 s. Next step is the snapshot/local cache so joins and reloads
    skip the rebuild entirely.

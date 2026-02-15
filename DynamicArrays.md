# Dynamic Arrays and array formulas

How are they stored in Excel?

## Importer

* Importing dynamic arrays
* Importing CSE formulas (array formulas)
* Maybe test LibreOffice and Google Sheets

## Exporter

* Exporting dynamic arrays
* Exporting CSE formulas (array formulas)
* Maybe test LibreOffice and Google Sheets

## Base

### Parsing new rules

`=SUM(A1#)`

Sums the set of spilled cells in A1 (if any)

### New Cell types

Dynamic arrays have an "anchor cell" and spill cells or spill range
The spill range of a dynamic array is computed dynamically (at "runtime") as opposed the spill range of an array formula.
Sometimes the spill range of a dynamic array is known at runtime and in this sense the formula is equivalent to an array formula.

For the anchor cells, should we have new cell types?

For instance `DynamicCellFormulaBoolean` for `CellFormulaBoolean`?

```rust
DynamicCellFormulaBoolean {
    f: i32,
    v: bool,
    s: i32,
    // range of the formula (width, height)
    r: (i32, i32),
    // true if the formula is a CSE formula
    cse: bool,
},
```

Note that this cell has a formula that evaluates to an array the first of which objects is a boolean. a cell in the spill range would be:

```rust
SpillNumberCell {
    v: f64,
    s: i32,
    // anchor cell (row, column)
    a: (i32, i32),
},
```


Or we have a single type for all `CellFormulaBoolean`.

Whether a formula is dynamic or not can be decided at compile time. Maybe the range is always `(1, 1)` for non dynamic?
How many of those we expect in the future?



### New algorithm

At parse time classify formulas:

* Do not spill
* Spill a controlled amount (TAKE(A:A, 7))
* Spill an uncontrolled but bounded amount (TAKE(A:A, B1))
* Spill an uncontrolled and unbounded amount (UNIQUE(A:A))

Some formulas 

We start of a list of cells that spill an uncontrolled with any order:

main_list: [cell_1, cell_2, ..., cell_n]

support_1: []
support_2: []
...
support_n: []

We start evaluating cell_1:

* During the course of evaluation we keep track of all empty cells accessed, that includes uncontrolled spill cells (support_i)
* If during the course of evaluation we hit one of the cells in main_list, we bail, switch order and start again
* If everything ends we write the result
* If when writing the result we write write on a cell previously accessed (support_k), we switch orders and start all over again

Note that this algorithm doesn't need to succeed, as there might be cyclic dependencies. Also dependencies might be random, so it might succeed some times


### Examples

=A1:A10  => It is range tof 10 cells
=SIN(A1:A10) => Same as above
=UNIQUE(A1:A10) => At most 10 cells
=TAKE(A1:A10, B1) => At most 10 cells


## UI

* Visual indication it is an array formula {=whatever} or =ArrayFormula()
* Can't modify part of an array formula. but no issue in modifying part of a dynamic array
* Visual indication it is a dynamic array


## Blog post

### Working title:

Support graphs, dynamic arrays and all that

#### Section 1: the general algorithm

Description of the recursive top down algorithm

#### Section 2: the general algorithm works with array formulas

Mother cells and all that

### Section 3; support graph and the simplest generalization


## Referencences:


* Peyton-Jones S, et al, Microsoft Research POPL 2021: https://www.youtube.com/watch?v=tfz4jdwsEaQ
* Bricklin D, “Meet the inventor of the electronic spreadsheet”, https://youtu.be/YDvbDiJZpy0

In September 2018 Microsoft announced the impending release of dynamic arrays within
Excel. Prior to that, the value property of a single cell was limited to a single value, be it a
String, a Double, a Boolean or an Error. At the Florida Ignite meeting Joe McDaid of
Microsoft described how the newest releases of Excel could associate an array object with
a single cell. Adjacent empty cells, known as a spill Range, would be used to display the
result. The size of the spill range depends solely on the formula references, rather than user
action, resulting in one element of risk being eliminated as a result

* [Preview of Dynamic Arrays in Excel](https://techcommunity.microsoft.com/blog/excelblog/preview-of-dynamic-arrays-in-excel/252944)
* [Write your own Excel in 100 lines of F#](https://tomasp.net/blog/2018/write-your-own-excel/)
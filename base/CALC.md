# Evaluation Strategy


We have a list of the spill cells:

```
// Checks if the array starting at cell will cover cells whose values
// has been requested
def CheckSpill(cell, array):
    for c in cell+array:
        support CellHasBeenRequested(c):
        if support is not empty:
            return support
    return []

// Fills cells with the result (an array)
def FillCells(cell, result):


def EvaluateNodeInContext(node, context):
    match node:
        case OP(left, right, op):
            l = EvaluateNodeInContext(left, context)?
            r = EvaluateNodeInContext(left, context)?
            return op(l, r)
        case FUNCTION(args, fn):
            ...
        case CELL(cell):
            EvaluateCell(cell)
        case RANGE(start, end):
            ...



def EvaluateCell(cell):
    if IsCellEvaluating(cell):
        return CIRC
    MarkEvaluating(cell)
    result = EvaluateNodeInContext(cell.formula, cell)
    if isSpill(result):
        CheckSpill(cell, array)?
        FillCells(result)


def EvaluateWorkbook():
    spill_cells = [cell_1, ...., cell_n];


    for cell in spill_cells:
        result = evaluate(cell)
```   







# When updating a cell value

If it was a spill cell we nee
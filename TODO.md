# Implicit Intersection

1. The formula `=@A:A` must work
2. Formulas like `=A:A` or `=SUM(SIN(A:A))` must return `#N/IMPL!`
3. We are able to import functions from Excel
4. We are able to export functions to Excel

## Operator `=@A:A`

That is the easiest part and it is done!

## Automatic II `=A:A` => `#SPILL!`

This is a bit tricky, because things like `=SUM(SIN(A:A))` should work and it doesn't spill but it requieres arrays in place.

The other option is to return `#ERROR!` or even `#N/IMPL!`.

Decision: Let's go with `#N/IMPL!`

## Importing

This means some functions get added an "automatic" implicit intersection "@".

For instnace the formula `=A:A` gets imported as `=A:A`. Also reads proper implicit intersection.
Some formulas like `=SUM(@A:A)` will work normally.

## Exporting

This means removing the "automatic" implicit intersection operator.

`=@A:A` => `=A:A`
`=SUM(@A:A)` => `=SUM(@A:A)`, actually `=SUM(_xlfn.SINGLE(A:A))`


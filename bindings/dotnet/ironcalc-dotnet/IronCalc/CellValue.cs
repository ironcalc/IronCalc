namespace IronCalc;

/// <summary>
/// Represents the value of a cell in a spreadsheet.
/// This is a discriminated union type.
/// </summary>
public abstract class CellValue
{
    private CellValue()
    {
    }

    /// <summary>
    /// Represents an empty cell.
    /// </summary>
    public class None : CellValue;

    /// <summary>
    /// Represents a cell containing a number.
    /// </summary>
    public class Number : CellValue
    {
        /// <summary>
        /// The numeric value of the cell.
        /// </summary>
        public required double Value { get; init; }
    }

    /// <summary>
    /// Represents a cell containing a boolean value.
    /// </summary>
    public class Bool : CellValue
    {
        /// <summary>
        /// The boolean value of the cell.
        /// </summary>
        public required bool Value { get; init; }
    }

    /// <summary>
    /// Represents a cell containing a string.
    /// </summary>
    public class String : CellValue
    {
        /// <summary>
        /// The string value of the cell.
        /// </summary>
        public required string Value { get; init; }
    }
}
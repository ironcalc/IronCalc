namespace IronCalc;

public abstract class CellValue
{
    private CellValue()
    {
    }

    public class None : CellValue;

    public class Number : CellValue
    {
        public required double Value { get; init; }
    }

    public class Bool : CellValue
    {
        public required bool Value { get; init; }
    }

    public class String : CellValue
    {
        public required string Value { get; init; }
    }
}
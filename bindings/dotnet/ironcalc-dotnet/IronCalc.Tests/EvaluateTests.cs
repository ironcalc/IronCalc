using System.IO;
using Xunit;

namespace IronCalc.Tests;

public class EvaluateTests
{
    [Fact]
    public void SetUserInputInvalidSheetShouldThrow()
    {
        using var model = Model.NewEmpty("Book1", "en", "Europe/Oslo");
        var exception = Assert.Throws<IronCalcException>(() => model.SetUserInput(1, 0, 0, "1"));
        Assert.Equal(ErrorCode.SetUserInputError, exception.ErrorCode);
        Assert.Equal("Invalid sheet index", exception.Message);
    }

    [Fact]
    public void Sum()
    {
        var bytes = File.ReadAllBytes("SimpleSum.xlsx");
        using var model = Model.FromBytes(bytes, "en",  "Europe/Oslo");

        var value = model.GetValue(0, 3, 1);
        Assert.Equal(2, value);

        model.SetUserInput(0, 1, 1, "4");
        model.SetUserInput(0, 2, 1, "6");
        model.Evaluate();

        var updated= model.GetValue(0, 3, 1);
        Assert.Equal(10, updated);
    }
}
using System.IO;
using Xunit;

namespace IronCalc.Tests;

public class EvalutateTests
{
    [Fact]
    public void Sum()
    {
        var bytes = File.ReadAllBytes("SimpleSum.xlsx");
        using var model = Model.FromBytes(bytes, "en",  "Europe/Oslo");

        var value = model.GetValue(0, 3, 1);
        Assert.Equal(2, value);

        model.SetValue(0, 1, 1, 4);
        model.SetValue(0, 2, 1, 6);
        model.Evaluate();

        var updated= model.GetValue(0, 3, 1);
        Assert.Equal(10, updated);
    }
}
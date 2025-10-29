using System.IO;
using Xunit;

namespace IronCalc.Tests;

public class CreateModelTests
{
    [Fact]
    public void NewEmpty()
    {
        using var model = Model.NewEmpty("en", "Europe/Oslo");
    }

    [Fact]
    public void FromBytes()
    {
        var bytes = File.ReadAllBytes("SimpleSum.xlsx");
        using var model = Model.FromBytes(bytes, "en",  "Europe/Oslo");
    }
}

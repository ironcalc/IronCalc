using System.IO;
using Xunit;

namespace IronCalc.Tests;

public class CreateModelTests
{
    [Fact]
    public void NewEmpty()
    {
        using var model = Model.NewEmpty("Book1", "en", "Europe/Oslo");
    }

    [Fact]
    public void FromBytes()
    {
        var bytes = File.ReadAllBytes("SimpleSum.xlsx");
        using var model = Model.FromBytes(bytes, "en",  "Europe/Oslo");
    }

    [Fact]
    public void FromBytesInvalidLocale()
    {
        var bytes = File.ReadAllBytes("SimpleSum.xlsx");
        var exception = Assert.Throws<IronCalcException>(() =>
        {
            using var model = Model.FromBytes(bytes, "foobar", "Europe/Oslo");
        });

        Assert.Equal(ErrorCode.WorkbookError, exception.ErrorCode);
        Assert.Equal("Invalid locale", exception.Message);
    }
}

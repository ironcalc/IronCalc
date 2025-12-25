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
    public void LoadFromXlsxBytes()
    {
        var bytes = File.ReadAllBytes("SimpleSum.xlsx");
        using var model = Model.LoadFromXlsxBytes(bytes, "en", "Europe/Oslo");
    }

    [Fact]
    public void LoadFromXlsxBytesInvalidLocale()
    {
        var bytes = File.ReadAllBytes("SimpleSum.xlsx");
        var exception = Assert.Throws<IronCalcException>(() =>
        {
            using var model = Model.LoadFromXlsxBytes(bytes, "foobar", "Europe/Oslo");
        });

        Assert.Equal(ErrorCode.WorkbookError, exception.ErrorCode);
        Assert.Equal("Invalid locale", exception.Message);
    }
}

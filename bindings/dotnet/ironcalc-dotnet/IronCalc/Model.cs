using System;
using System.Text;

namespace IronCalc;

public class Model : IDisposable
{
    private readonly unsafe ModelContext* ctx;

    private unsafe Model(ModelContext* ctx)
    {
        this.ctx = ctx;
    }

    public static Model NewEmpty(string locale, string timezone)
    {
        unsafe
        {
            var localeBytes = Encoding.UTF8.GetBytes(locale);
            var timezoneBytes = Encoding.UTF8.GetBytes(timezone);
            fixed (byte* localeP = localeBytes)
            fixed (byte* timezoneP = timezoneBytes)
            {
                var ctx = NativeMethods.new_empty(localeP, timezoneP);
                return new Model(ctx);
            }
        }
    }

    public static Model FromBytes(byte[] bytes, string locale, string timezone, string? name = null)
    {
        unsafe
        {
            var localeBytes = Encoding.UTF8.GetBytes(locale);
            var timezoneBytes = Encoding.UTF8.GetBytes(timezone);
            var nameBytes = name is not null ? Encoding.UTF8.GetBytes(name) : null;
            fixed (byte* localeP = localeBytes)
            fixed (byte* timezoneP = timezoneBytes)
            fixed (byte* nameP = nameBytes)
            fixed (byte* byteP = bytes)
            {
                var ctx = NativeMethods.from_bytes(byteP, bytes.Length, localeP, timezoneP, nameP);
                return new Model(ctx);
            }
        }
    }

    public void Evaluate()
    {
        unsafe
        {
            NativeMethods.evaluate(ctx);
        }
    }

    public int GetValue(int sheet, int row, int column)
    {
        unsafe
        {
            return NativeMethods.get_value(ctx, sheet, row, column);
        }
    }

    public void SetValue(int sheet, int row, int col, int value)
    {
        unsafe
        {
            NativeMethods.set_value(ctx, sheet, row, col, value);
        }
    }

    public void Dispose()
    {
        unsafe
        {
            NativeMethods.dispose(ctx);
        }
    }
}

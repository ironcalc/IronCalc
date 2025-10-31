using System;
using System.Text;

namespace IronCalc;

public class IronCalcException : Exception
{
    public readonly ErrorCode ErrorCode;

    internal IronCalcException(string message, ErrorCode errorCode)
        : base(message)
    {
        ErrorCode = errorCode;
    }

    internal IronCalcException(string message, ModelContextErrorTag? tag)
        : base(message)
    {
        ErrorCode = tag switch {
            null => ErrorCode.Unknown,
            ModelContextErrorTag.XlsxError => ErrorCode.XslxError,
            ModelContextErrorTag.WorkbookError => ErrorCode.WorkbookError,
            _ => throw new ArgumentOutOfRangeException(nameof(tag), tag, null)
        };
    }
}

public enum ErrorCode
{
    Unknown = 1,
    XslxError = 2,
    WorkbookError = 3,
}

public class Model : IDisposable
{
    private readonly unsafe ModelContext* ctx;

    private unsafe Model(ModelContext* ctx)
    {
        this.ctx = ctx;
    }

    public static Model NewEmpty(string name, string locale, string timezone)
    {
        unsafe
        {
            var nameBytes = Encoding.UTF8.GetBytes(name);
            var localeBytes = Encoding.UTF8.GetBytes(locale);
            var timezoneBytes = Encoding.UTF8.GetBytes(timezone);
            fixed (byte* nameP = nameBytes)
            fixed (byte* localeP = localeBytes)
            fixed (byte* timezoneP = timezoneBytes)
            {
                var ctx = NativeMethods.new_empty(nameP, localeP, timezoneP);
                if (ctx.is_ok)
                {
                    return new Model(ctx.model);
                }

                string message;
                ModelContextErrorTag? errorTag = null;
                if (ctx.error->has_message)
                {
                    message = new String((sbyte*)ctx.error->message);
                    errorTag = ctx.error->tag;
                    NativeMethods.dispose_error(ctx.error);
                }
                else
                {
                    message = "Unknown error while create IronCalc model.";
                }

                throw new IronCalcException(message, errorTag);
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
                if (ctx.is_ok)
                {
                    return new Model(ctx.model);
                }

                string message;
                ModelContextErrorTag? errorTag = null;
                if (ctx.error->has_message)
                {
                    message = new String((sbyte*)ctx.error->message);
                    errorTag = ctx.error->tag;
                    NativeMethods.dispose_error(ctx.error);
                }
                else
                {
                    message = "Unknown error while create IronCalc model.";
                }

                throw new IronCalcException(message, errorTag);
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

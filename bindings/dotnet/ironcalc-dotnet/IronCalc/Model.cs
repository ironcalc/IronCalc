using System;
using System.Runtime.CompilerServices;
using System.Text;
using IronCalc.Native;

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
            ModelContextErrorTag.SetUserInputError => ErrorCode.SetUserInputError,
            ModelContextErrorTag.GetUserInputError => ErrorCode.GetUserInputError,
            _ => throw new ArgumentOutOfRangeException(nameof(tag), tag, null)
        };
    }
}

public enum ErrorCode
{
    Unknown = 1,
    XslxError = 2,
    WorkbookError = 3,
    SetUserInputError = 4,
    GetUserInputError = 5,
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

                throw CreateExceptionFromError(ctx.error);
            }
        }
    }

    public static Model LoadFromXlsxBytes(byte[] bytes, string locale, string timezone, string? name = null)
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
                var ctx = NativeMethods.load_from_xlsx_bytes(byteP, bytes.Length, localeP, timezoneP, nameP);
                if (ctx.is_ok)
                {
                    return new Model(ctx.model);
                }

                throw CreateExceptionFromError(ctx.error);
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

    public CellValue GetValue(uint sheet, int row, int column)
    {
        unsafe
        {
            var result = NativeMethods.get_cell_value_by_index(ctx, sheet, row, column);
            if (!result.is_ok)
            {
                throw CreateExceptionFromError(result.error);
            }

            try
            {
                switch (result.value->tag)
                {
                    case CellValueTag.None:
                        return new CellValue.None();
                    case CellValueTag.String:
                        var value = new String((sbyte*)result.value->string_value);
                        return new CellValue.String()
                        {
                            Value = value,
                        };
                    case CellValueTag.Number:
                        return new CellValue.Number()
                        {
                            Value = result.value->number_value,
                        };
                    case CellValueTag.Boolean:
                        return new CellValue.Bool()
                        {
                            Value = result.value->boolean_value,
                        };
                    default:
                        throw new ArgumentOutOfRangeException();
                }
            }
            finally
            {
                NativeMethods.dispose_cell_value(result.value);
            }
        }
    }

    public void SetUserInput(uint sheet, int row, int col, string value)
    {
        unsafe
        {
            var valueBytes = Encoding.UTF8.GetBytes(value);
            fixed (byte* valueP = valueBytes)
            {
                var error = NativeMethods.set_user_input(ctx, sheet, row, col, valueP);
                if (error != null)
                {
                    throw CreateExceptionFromError(error);
                }
            }
        }
    }

    private static unsafe IronCalcException CreateExceptionFromError(
        ModelContextError* error,
        [CallerMemberName] string? callerName = null)
    {
        string message;
        var errorTag = error->tag;
        if (error->has_message)
        {
            message = new String((sbyte*)error->message);
            NativeMethods.dispose_error(error);
        }
        else
        {
            message = $"Unknown error while calling {callerName ?? "UNKNOWN"} on IronCalc model.";
        }

        return new IronCalcException(message, errorTag);
    }

    public void Dispose()
    {
        unsafe
        {
            NativeMethods.dispose(ctx);
        }
    }
}

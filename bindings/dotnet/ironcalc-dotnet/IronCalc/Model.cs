using System;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading;
using IronCalc.Native;

namespace IronCalc;

/// <summary>
/// Represents an IronCalc spreadsheet model.
/// </summary>
public class Model : IDisposable
{
    private readonly unsafe ModelContext* _ctx;

    private unsafe Model(ModelContext* ctx)
    {
        _ctx = ctx;
    }

    /// <summary>
    /// Creates a new empty spreadsheet model.
    /// </summary>
    /// <param name="name">The name of the workbook.</param>
    /// <param name="locale">The locale to use for formula localization (e.g., "en_US").</param>
    /// <param name="timezone">The IANA timezone to use for date/time functions (e.g., "UTC", "America/New_York").</param>
    /// <returns>A new `Model` instance.</returns>
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

    /// <summary>
    /// Loads a spreadsheet model from a byte array containing XLSX data.
    /// </summary>
    /// <param name="bytes">The byte array containing the XLSX file content.</param>
    /// <param name="locale">The locale to use for formula localization (e.g., "en_US").</param>
    /// <param name="timezone">The IANA timezone to use for date/time functions (e.g., "UTC", "America/New_York").</param>
    /// <param name="name">An optional name for the workbook.</param>
    /// <returns>A new `Model` instance.</returns>
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

    /// <summary>
    /// Evaluates all formulas in the spreadsheet.
    /// </summary>
    public void Evaluate()
    {
        unsafe
        {
            NativeMethods.evaluate(_ctx);
        }
    }

    /// <summary>
    /// Gets the value of a cell by its sheet index, row, and column.
    /// </summary>
    /// <param name="sheet">The 0-based index of the sheet.</param>
    /// <param name="row">The 1-based index of the row.</param>
    /// <param name="column">The 1-based index of the column.</param>
    /// <returns>A `CellValue` representing the value of the cell.</returns>
    public CellValue GetValue(uint sheet, int row, int column)
    {
        unsafe
        {
            var result = NativeMethods.get_cell_value_by_index(_ctx, sheet, row, column);
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
                        throw new InvalidOperationException($"Unknown tag: {result.value->tag}.");
                }
            }
            finally
            {
                NativeMethods.dispose_cell_value(result.value);
            }
        }
    }

    /// <summary>
    /// Sets the value of a cell.
    /// </summary>
    /// <param name="sheet">The 0-based index of the sheet.</param>
    /// <param name="row">The 1-based index of the row.</param>
    /// <param name="col">The 1-based index of the column.</param>
    /// <param name="value">The value to set. If it starts with '=', it is treated as a formula.</param>
    public void SetUserInput(uint sheet, int row, int col, string value)
    {
        unsafe
        {
            var valueBytes = Encoding.UTF8.GetBytes(value);
            fixed (byte* valueP = valueBytes)
            {
                var error = NativeMethods.set_user_input(_ctx, sheet, row, col, valueP);
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
        var message = error->has_message
            ? new string((sbyte*)error->message)
            : $"Unknown error while calling {callerName ?? "UNKNOWN"} on IronCalc model.";

        var errorTag = error->tag;
        NativeMethods.dispose_error(error);

        return new IronCalcException(message, errorTag);
    }

    /// <summary>
    /// The finalizer for <see cref="Model"/>.
    /// This is called during garbage collection.
    /// </summary>
    ~Model()
    {
        Dispose(false);
    }

    /// <summary>
    /// Releases the resources used by the `Model` instance.
    /// </summary>
    public void Dispose()
    {
        Dispose(true);
        // Suppress finalization.
        GC.SuppressFinalize(this);
    }

    private int _isDisposed;

    /// <summary>
    /// Releases the resources used by the `Model` instance.
    /// </summary>
    /// <param name="disposing"></param>
    protected virtual void Dispose(bool disposing)
    {
        // In case _isDisposed is 0, atomically set it to 1.
        // Enter the branch only if the original value is 1.
        if (Interlocked.CompareExchange(ref _isDisposed, 1, 0) != 0)
        {
            return;
        }

        if (disposing)
        {
            // free managed resources.
        }

        unsafe
        {
            NativeMethods.dispose(_ctx);
        }
    }
}

using System;
using IronCalc.Native;

namespace IronCalc;

/// <summary>
/// Represents an error that occurred within the IronCalc engine.
/// </summary>
public class IronCalcException : Exception
{
    /// <summary>
    /// The error code indicating the type of error.
    /// </summary>
    public readonly ErrorCode ErrorCode;

    internal IronCalcException(string message, ErrorCode errorCode)
        : base(message)
    {
        ErrorCode = errorCode;
    }

    internal IronCalcException(string message, ModelContextErrorTag? tag)
        : base(message)
    {
        ErrorCode = tag switch
        {
            null => ErrorCode.Unknown,
            ModelContextErrorTag.XlsxError => ErrorCode.XslxError,
            ModelContextErrorTag.WorkbookError => ErrorCode.WorkbookError,
            ModelContextErrorTag.SetUserInputError => ErrorCode.SetUserInputError,
            ModelContextErrorTag.GetUserInputError => ErrorCode.GetUserInputError,
            _ => throw new ArgumentOutOfRangeException(nameof(tag), tag, null)
        };
    }
}

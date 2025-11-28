namespace IronCalc;

/// <summary>
/// Represents the type of error that occurred in the IronCalc engine.
/// </summary>
public enum ErrorCode
{
    /// <summary>
    /// An unknown or unspecified error occurred.
    /// </summary>
    Unknown = 1,

    /// <summary>
    /// An error occurred while parsing an XLSX file.
    /// </summary>
    XslxError = 2,

    /// <summary>
    /// An error occurred within the workbook model.
    /// </summary>
    WorkbookError = 3,

    /// <summary>
    /// An error occurred while setting a user input value.
    /// </summary>
    SetUserInputError = 4,

    /// <summary>
    /// An error occurred while retrieving a user input value.
    /// </summary>
    GetUserInputError = 5,
}
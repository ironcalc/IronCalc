/*
 * NOTE: this file is used to test the generated nuget package during CI.
 * That means that this Program.cs file is copied as-is into a new console application
 * and executing against the newly created nuget package. I.e. this file must not depend
 * on other files or special properties set in the csproj file, including other nuget dependencies.
 */

using System;
using System.Diagnostics;
using IronCalc;

using var model = Model.NewEmpty("Book1", "en", "Europe/Oslo");
model.SetUserInput(0, 1, 1, "21");
model.SetUserInput(0, 2, 1, "21");
model.SetUserInput(0, 3, 1, "=SUM(A1:A2)");
model.Evaluate();

var value = model.GetValue(0, 3, 1);
Debug.Assert(
    value is CellValue.Number number
    && Math.Abs(number.Value - 42) < 0.00001);

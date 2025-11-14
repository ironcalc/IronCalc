# IronCalc .NET Binding

This project provides a .NET binding for the [IronCalc](https://github.com/ironcalc/IronCalc) spreadsheet engine. It allows you to use the IronCalc engine from any .NET language (C#, F#, etc.).

## Requirements

To build and develop this project, you will need the following:

*   [.NET SDK](https://dotnet.microsoft.com/download) (versions 8, 9, and 10)
*   [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)

## Code Structure

The project is structured into two main parts:

*   `src/`: This directory contains the Rust source code for the native library. It uses `cbindgen` to generate a C-compatible header for the FFI (Foreign Function Interface). The Rust code is responsible for interacting with the core `ironcalc_base` engine and exposing a C-compatible API.
*   `ironcalc-dotnet/`: This directory contains the .NET solution with the following projects:
    *   `IronCalc/`: The main C# project that defines the public API for the .NET binding. It uses P/Invoke (`DllImport`) to call the native functions exposed by the Rust library.
    *   `IronCalc.Tests/`: Unit tests for the .NET binding.
    *   `IronCalc.Playground/`: A sample console application demonstrating how to use the library.

The native library (`.dll`, `.so`, or `.dylib` depending on the platform) is built by Cargo and then copied to the `ironcalc-dotnet/IronCalc/runtimes` directory, allowing the .NET runtime to find and load it.

## Building the Project

You can build the project using the provided `Makefile`:

```bash
# Build the native library and the .NET projects
make
```

The `make` command compiles the Rust code into a dynamic library and then builds the .NET solution. The resulting artifacts will be in the `bindings/dotnet/ironcalc-dotnet/IronCalc/bin/` directory.

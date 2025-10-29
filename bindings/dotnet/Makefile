# Define the name of your project (optional, but good practice)
PROJECT_NAME := ironcalc-dotnet

# Default target: builds the project
all: build

build: copy-native
	dotnet test ironcalc-dotnet/IronCalc.Tests
	dotnet build -c Release ironcalc-dotnet/IronCalc

# Build target: runs cargo build
build-native:
	cargo build --release

copy-native: build-native
	mkdir -p ironcalc-dotnet/IronCalc/runtimes/osx-arm64/native/
	cp ../../target/release/libironcalc_dotnet.dylib ironcalc-dotnet/IronCalc/runtimes/osx-arm64/native/libironcalc_dotnet.dylib

# Phony targets to prevent conflicts with files of the same name
.PHONY: all build-native copy-native

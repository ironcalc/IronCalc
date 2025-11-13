PROJECT_NAME := ironcalc-dotnet

# 1. ARCHITECTURE DETECTION
# Get the Operating System (e.g., Darwin, Linux)
OS := $(shell uname -s)
# Get the Machine Architecture (e.g., x86_64, aarch64)
ARCH := $(shell uname -m)

# 2. Determine Library Extension and Destination RID
# Initialize variables
NATIVE_LIB_NAME := libironcalc_dotnet
NATIVE_LIB_EXT :=
RID :=

ifeq ($(OS),Darwin) # macOS
    NATIVE_LIB_EXT := .dylib
    ifeq ($(ARCH),x86_64)
        RID := osx-x64
    else ifeq ($(ARCH),aarch64)
        RID := osx-arm64
    endif
else ifeq ($(OS),Linux) # Linux
    NATIVE_LIB_EXT := .so
    ifeq ($(ARCH),x86_64)
        RID := linux-x64
    else ifeq ($(ARCH),aarch64)
        RID := linux-arm64
    endif
else
    # Fallback or error for unsupported systems
    $(error Unsupported OS/ARCH combination: $(OS)/$(ARCH). Update Makefile.)
endif

# Construct the full file paths
SOURCE_LIB_PATH := ../../target/release/$(NATIVE_LIB_NAME)$(NATIVE_LIB_EXT)
DEST_DIR := ironcalc-dotnet/IronCalc/runtimes/$(RID)/native
DEST_LIB_PATH := $(DEST_DIR)/$(NATIVE_LIB_NAME)$(NATIVE_LIB_EXT)

# Default target: builds the project
all: build

build: copy-native
	dotnet test ironcalc-dotnet/IronCalc.Tests
	dotnet build -c Release ironcalc-dotnet/IronCalc

# Build target: runs cargo build
build-native:
	cargo build --release

copy-native: build-native
	@echo "Copying native library for $(RID) to $(DEST_DIR)/"
	mkdir -p $(DEST_DIR)
	cp $(SOURCE_LIB_PATH) $(DEST_LIB_PATH)

# Phony targets to prevent conflicts with files of the same name
.PHONY: all build-native copy-native

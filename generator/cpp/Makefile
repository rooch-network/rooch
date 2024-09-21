# Makefile for building the generator.cpp into WebAssembly using Emscripten

# Define the compiler to be the Emscripten C++ Compiler
CXX = emcc

# Define the source files
SRC = generator.cpp

# Define the output targets
TARGET = generator.wasm
TARGET_JS = generator.js

# Compiler flags
# -O3 for optimization, --no-entry to indicate no main() function
# EXPORTED_FUNCTIONS to list the functions you want to call from JavaScript
# ALLOW_MEMORY_GROWTH to allow the memory to grow at runtime if needed
CXXFLAGS = -g0 -Oz -flto -fno-rtti -fno-exceptions --no-entry \
	-s STANDALONE_WASM=1 \
	-s WASM=1 \
	-s WARN_ON_UNDEFINED_SYMBOLS=0 \
	-s EXPORTED_FUNCTIONS='["_inscribe_generate", "_inscribe_verify"]' \
	-s DEFAULT_LIBRARY_FUNCS_TO_INCLUDE='["stringToUTF8"]' \
	-s EXPORTED_RUNTIME_METHODS='["cwrap"]'

# Define the build rule
.PHONY: all build
all: build

build: $(TARGET)

$(TARGET): $(SRC)
	$(CXX) $(CXXFLAGS) $(SRC) -o $(TARGET_JS)

# Define the clean rule
.PHONY: clean
clean:
	rm -f $(TARGET) $(TARGET_JS)


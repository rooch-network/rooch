const fs = require('fs');
const path = require('path');
const cbor = require('cbor');

// load WebAssembly module
const wasmPath = path.join(__dirname, '../pkg/generator_bg.wasm');
const wasmBuffer = fs.readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBuffer);
const wasmInstance = new WebAssembly.Instance(wasmModule, {
  env: {
    js_log: (ptr, len) => {
      const message = new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));
      console.log(message);
    },
  },
});
const { stackAlloc, stackSave, stackRestore, inscribe_generate, memory } = wasmInstance.exports;

// Allocate memory and write string data
const encodeInputOnStack = (input, memory) => {
  const encodedBuffer = cbor.encodeOne(input)
  const len = encodedBuffer.length

  // Save the stack pointer before allocation
  const stackPointer = stackSave()

  // Allocate space on the stack
  const ptr = stackAlloc(len + 4)

  // write buffer length
  const dataView = new DataView(memory.buffer);
  dataView.setUint32(ptr, len, false)

  // Write the input to the stack
  const bytes = new Uint8Array(memory.buffer)
  bytes.set(encodedBuffer, ptr + 4)

  // Return a function that will restore the stack after use
  return {
    ptr,
    len: len + 5,
    free: () => stackRestore(stackPointer),
  }
}

// Read the output from WASM memory
const decodeOutputOnHeap = async (ptr, memory) => {
  const dataView = new DataView(memory.buffer)
  const length = dataView.getUint32(ptr, false)
  console.log("output length", length)

  const encodedResult = new Uint8Array(memory.buffer, ptr + 4, length)

  return await cbor.decodeFirst(encodedResult, {})
}

// Test inscribe_generate function
describe('inscribe_generate', () => {
  test('generates correct output for valid input', async () => {
    const deployArgs = [
      '{"level1":{"type":"range","data":{"min":1,"max":1000}}}',
      '{"level2":{"type":"range","data":{"min":1,"max":1000}}}',
    ]

    const argsBytes = new Uint8Array(cbor.encodeOne(deployArgs.map((json) => JSON.parse(json))))
    const argsArray = Array.from(argsBytes)
    console.log('argsArray:', JSON.stringify(argsArray))

    const input = {
      attrs: argsArray,
      seed: 'random-seed',
      user_input: 'user-input',
    };

    console.log('input:', input)

    // Encode seed and userInput and write them into WASM memory
    const inputEncoded = encodeInputOnStack(input, memory)

    // Call the WASM function
    const outputPtr = inscribe_generate(inputEncoded.ptr)
    console.log('outputPtr:', outputPtr)

    const output = await decodeOutputOnHeap(outputPtr, memory)

    expect(output).toEqual({
      amount: 1,
      attributes: {
        "id": "user-input",
        "level1": 27,
        "level2": 27,
      },
      content: {
        "content": Buffer.from([
          104,
          101,
          108,
          108,
          111,
          32,
          119,
          111,
          114,
          108,
          100,
          33,
        ]),
        "content_type": "text/plain"
      },
    });
  });
});

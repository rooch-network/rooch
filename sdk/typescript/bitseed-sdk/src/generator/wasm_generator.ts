import cbor from 'cbor'
import { IGenerator } from './interface'
import { SFTRecord } from '../types'
import { EmscriptenRuntime } from './emscripten_runtime'
import { InscribeSeed } from './seed'

export class WasmGenerator implements IGenerator {
  private wasmInstance: WebAssembly.Instance

  constructor(instance: WebAssembly.Instance) {
    this.wasmInstance = instance
  }

  public async inscribeGenerate(
    deployArgs: Array<string>,
    seed: InscribeSeed,
    userInput: string,
  ): Promise<SFTRecord> {
    // Convert deployArgs to a CBOR bytes
    const argsBytes = new Uint8Array(cbor.encodeOne(deployArgs.map((json)=>JSON.parse(json))))

    const input = {
      "seed": seed.seed(),
      "user_input": userInput,
      "attrs": Array.from(argsBytes),
    }

    // Get the memory of the WASM instance
    const memory = this.wasmInstance.exports.memory as WebAssembly.Memory

    // Allocate memory and write string data
    const encodeInputOnStack = (input: object, memory: WebAssembly.Memory) => {
      const encodedBuffer = cbor.encodeOne(input)
      const len = encodedBuffer.length

      const stackAllocFunction = this.wasmInstance.exports.stackAlloc as CallableFunction
      const stackSaveFunction = this.wasmInstance.exports.stackSave as CallableFunction
      const stackRestoreFunction = this.wasmInstance.exports.stackRestore as CallableFunction

      // Save the stack pointer before allocation
      const stackPointer = stackSaveFunction()

      // Allocate space on the stack
      const ptr = stackAllocFunction(len + 4)

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
        free: () => stackRestoreFunction(stackPointer),
      }
    }

    // Read the output from WASM memory
    const decodeOutputOnHeap = async (ptr: number, memory: WebAssembly.Memory) => {
      const dataView = new DataView(memory.buffer)
      const length = dataView.getUint32(ptr, false)
      const encodedResult = new Uint8Array(memory.buffer, ptr + 4, length)

      return await cbor.decodeFirst(encodedResult, {})
    }

    // Encode seed and userInput and write them into WASM memory
    const inputEncoded = encodeInputOnStack(input, memory)

    try {
      // Call the WASM function
      const inscribeGenerateFunction = this.wasmInstance.exports.inscribe_generate as CallableFunction
      const outputPtr = inscribeGenerateFunction(inputEncoded.ptr)

      const output = await decodeOutputOnHeap(outputPtr, memory)
      return output as SFTRecord;
    } catch(e: any) {
      console.log('call inscribe_generate error:', e)
      throw e
    } finally {
      inputEncoded.free()
    }
  }

  public static async loadWasmModule(wasmBytes: BufferSource): Promise<WasmGenerator> {
    const module = await WebAssembly.compile(wasmBytes)
    const runtime = new EmscriptenRuntime(module)
    const instance = await runtime.instantiate()
    return new WasmGenerator(instance)
  }
}

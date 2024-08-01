
var out = console.log.bind(console);
var err = console.error.bind(console);

var UTF8Decoder = typeof TextDecoder != 'undefined' ? new TextDecoder('utf8') : undefined;

/**
 * Given a pointer 'idx' to a null-terminated UTF8-encoded string in the given
 * array that contains uint8 values, returns a copy of that string as a
 * Javascript String object.
 * heapOrArray is either a regular array, or a JavaScript typed array view.
 * @param {number} idx
 * @param {number=} maxBytesToRead
 * @return {string}
 */
var UTF8ArrayToString = (heapOrArray: any, idx: any, maxBytesToRead: any) => {
  var endIdx = idx + maxBytesToRead;
  var endPtr = idx;
  // TextDecoder needs to know the byte length in advance, it doesn't stop on
  // null terminator by itself.  Also, use the length info to avoid running tiny
  // strings through TextDecoder, since .subarray() allocates garbage.
  // (As a tiny code save trick, compare endPtr against endIdx using a negation,
  // so that undefined means Infinity)
  while (heapOrArray[endPtr] && !(endPtr >= endIdx)) ++endPtr;

  if (endPtr - idx > 16 && heapOrArray.buffer && UTF8Decoder) {
    return UTF8Decoder.decode(heapOrArray.subarray(idx, endPtr));
  }
  var str = '';
  // If building with TextDecoder, we have already computed the string length
  // above, so test loop end condition against that
  while (idx < endPtr) {
    // For UTF8 byte structure, see:
    // http://en.wikipedia.org/wiki/UTF-8#Description
    // https://www.ietf.org/rfc/rfc2279.txt
    // https://tools.ietf.org/html/rfc3629
    var u0 = heapOrArray[idx++];
    if (!(u0 & 0x80)) { str += String.fromCharCode(u0); continue; }
    var u1 = heapOrArray[idx++] & 63;
    if ((u0 & 0xE0) == 0xC0) { str += String.fromCharCode(((u0 & 31) << 6) | u1); continue; }
    var u2 = heapOrArray[idx++] & 63;
    if ((u0 & 0xF0) == 0xE0) {
      u0 = ((u0 & 15) << 12) | (u1 << 6) | u2;
    } else {
      u0 = ((u0 & 7) << 18) | (u1 << 12) | (u2 << 6) | (heapOrArray[idx++] & 63);
    }

    if (u0 < 0x10000) {
      str += String.fromCharCode(u0);
    } else {
      var ch = u0 - 0x10000;
      str += String.fromCharCode(0xD800 | (ch >> 10), 0xDC00 | (ch & 0x3FF));
    }
  }
  return str;
};

var printCharBuffers = [null, [], []];

var printChar = (stream: any, curr: any) => {
  var buffer = printCharBuffers[stream];
  if (buffer) {
    if (curr === 0 || curr === 10) {
      (stream === 1 ? out : err)(UTF8ArrayToString(buffer, 0, undefined));
      buffer.length = 0;
    } else {
      buffer.push(curr as never);
    }
  }
};

class ExitStatus {
  public name: string;
  public message: string;
  public status: number;

  constructor(status: number) {
    this.name = 'ExitStatus';
    this.message = `Program terminated with exit(${status})`;
    this.status = status;
  }
}

export class EmscriptenRuntime {
  private module: WebAssembly.Module;
  private memory: WebAssembly.Memory | undefined;

  private HEAPU8: Uint8Array | undefined;
  private HEAPU32: Uint32Array | undefined;

  public EXITSTATUS: number = 0;

  constructor(module: WebAssembly.Module) {
    this.module = module;
  }

  async instantiate(): Promise<WebAssembly.Instance> {
    const wasmImports = {
      wasi_snapshot_preview1: {
        fd_write: (fd: any, iov: any, iovcnt: any, pnum: any) => {
          this.fd_write(fd, iov, iovcnt, pnum)
        },
        fd_seek: (fd: any, offset_low: any, offset_high: any, whence: any, new_offset: any) => {
          this.fd_seek(fd, offset_low, offset_high, whence, new_offset)
        },
        fd_close: (fd: any) => {
          this.fd_close(fd)
        },
        proc_exit: (code: any) => {
          this.proc_exit(code)
        },
      }
    }

    const instance = await WebAssembly.instantiate(this.module, wasmImports)

    this.memory = instance.exports['memory'] as WebAssembly.Memory;
    
    this.HEAPU8 = new Uint8Array(this.memory.buffer)
    this.HEAPU32 = new Uint32Array(this.memory.buffer)

    return instance
  }

  fd_write(fd: any, iov: any, iovcnt: any, pnum: any) {
    if (this.HEAPU8 && this.HEAPU32) {
      // hack to support printf in SYSCALLS_REQUIRE_FILESYSTEM=0
      var num = 0;
      for (var i = 0; i < iovcnt; i++) {
        var ptr = this.HEAPU32[((iov) >> 2)];
        var len = this.HEAPU32[(((iov) + (4)) >> 2)];
        iov += 8;
        for (var j = 0; j < len; j++) {
          printChar(fd, this.HEAPU8[ptr + j]);
        }
        num += len;
      }
      this.HEAPU32[((pnum) >> 2)] = num;
    }

    return 0;
  }

  fd_seek(_fd: any, _offset_low: any, _offset_high: any, _whence: any, _new_offset: any) {
    return 70;
  }

  fd_close(_fd: any) {
    return 52;
  }

  proc_exit(code: any) {
    this.EXITSTATUS = code;
    throw new ExitStatus(code);
  }

}
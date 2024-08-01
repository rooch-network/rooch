import cbor from 'cbor'
import * as bitcoin from 'bitcoinjs-lib';

const PROTOCOL_ID = Buffer.from("6f7264", 'hex');
const ENVELOPE_START_TAG = 0;
const ENVELOPE_END_TAG = 104;
const META_TAG = 85;

type EnvelopeData = number | Buffer;

function isBuffer(value: unknown): value is Buffer {
  return Buffer.isBuffer(value);
}

function getEnvelope(data: EnvelopeData[]): EnvelopeData[] | undefined {
  let startIndex = -1;
  let endIndex = -1;

  let index = 0;
  for (const op of data) {
    const started = startIndex !== -1;
    if (started === false && op === ENVELOPE_START_TAG) {
      startIndex = index;
      continue;
    }
    if (op === ENVELOPE_END_TAG) {
      if (started === false) {
        return [];
      }
      endIndex = index;
      break;
    }
    index += 1;
  }

  return data.slice(startIndex + 1, endIndex + 1);
}

function getEnvelopeMetadata(data: EnvelopeData[]) {
  const content: Buffer[] = [];
  
  for (var i=0; i<data.length; i++) {
    if (data[i] == META_TAG) {
      if (isBuffer(data[i + 1])) {
        content.push(data[i + 1] as Buffer);
      }
    }
  }

  try {
    return cbor.decodeFirstSync(Buffer.concat(content));
  } catch (err) {
    return undefined;
  }
}

export function decodeInscriptionMetadata(signedTxHex: string, index: number): any {
  const tx = bitcoin.Transaction.fromHex(signedTxHex)
  if (index < 0 || index >= tx.ins.length) {
    return undefined
  }

  const vin = tx.ins[index]

  for (let i = 0; i < vin.witness.length; i++) {
    const witness = vin.witness[i]
    if (witness.includes(PROTOCOL_ID)) {
      const data = bitcoin.script.decompile(witness);
      if (data) {
        const envelope = getEnvelope(data)

        if (envelope) {
          return getEnvelopeMetadata(envelope)
        }

        return undefined
      }
    }
  }

  return undefined
}

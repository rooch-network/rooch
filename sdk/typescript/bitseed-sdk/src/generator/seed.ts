import { sha3_256 } from 'js-sha3';
import { OutPoint } from "../types";

export class InscribeSeed {
  private utxo: OutPoint;
  private block_hash: string;

  constructor(block_hash: string, utxo: OutPoint) {
    this.block_hash = block_hash
    this.utxo = utxo;
  }

  seed(): string {
    const blockHashBuffer = Buffer.from(this.block_hash, 'hex');
    const txidBuffer = Buffer.from(this.utxo.txid, 'hex');
    const voutBuffer = Buffer.allocUnsafe(4);
    voutBuffer.writeUInt32LE(this.utxo.vout);

    const combinedBuffer = Buffer.concat([blockHashBuffer, txidBuffer, voutBuffer]);
    const hash = sha3_256(combinedBuffer);

    return hash;
  }
}

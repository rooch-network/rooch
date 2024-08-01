import * as bitcoin from 'bitcoinjs-lib';
import { UTXOLimited } from '@sadoprotocol/ordit-sdk';
export const ScriptTypeWitnessV1Taproot = "witness_v1_taproot"
export const ScriptTypeWitnessV0Scripthash = "witness_v0_scripthash"
export const ScriptTypeWitnessV0KeyHash = "witness_v0_keyhash"
export const ScriptTypeScriptHash = "scripthash"
export const ScriptTypePubkeyHash = "pubkeyhash"

export interface ScriptPubKey {
    asm: string;
    desc: string;
    hex: string;
    address: string;
    type: string;
}

const classifyOutputScript = (script: Buffer): string => {
  const isOutput = (paymentFn: (params: { output?: Buffer }) => bitcoin.payments.Payment) => {
    try { 
      return paymentFn({ output: script }) !== undefined;
    } catch (e) {
      return false;
    }
  }

  if (isOutput(bitcoin.payments.p2pk)) return 'P2PK';
  else if (isOutput(bitcoin.payments.p2pkh)) return ScriptTypePubkeyHash;
  else if (isOutput(bitcoin.payments.p2ms)) return 'P2MS';  
  else if (isOutput(bitcoin.payments.p2wpkh)) return ScriptTypeWitnessV0KeyHash;
  else if (isOutput(bitcoin.payments.p2sh)) return ScriptTypeScriptHash;
  else if (isOutput(bitcoin.payments.p2tr)) return ScriptTypeWitnessV1Taproot;
  
  return 'nonstandard';
}

export function decodeScriptPubKey(scriptPubKeyHex: string, network: bitcoin.Network): ScriptPubKey {
  const scriptPubKeyBuffer = Buffer.from(scriptPubKeyHex, 'hex');
  const decompiled = bitcoin.script.decompile(scriptPubKeyBuffer);
  if (!decompiled) {
      throw new Error('Invalid scriptPubKey: Unable to decompile');
  }
  const asm = bitcoin.script.toASM(decompiled);
  const type = classifyOutputScript(scriptPubKeyBuffer);

  let address: string = ""

  try {
    if ([ScriptTypePubkeyHash, 'P2PK', 'P2MS', ScriptTypeWitnessV0KeyHash, ScriptTypeScriptHash, ScriptTypeWitnessV1Taproot].includes(type)) {
      address = bitcoin.address.fromOutputScript(scriptPubKeyBuffer, network);
    }
  } catch (error) {
    // Log the error or handle it as needed
    console.error('Error getting address from output script:', error);
  }

  return {
      asm,
      desc: `Script ${type}`,
      hex: scriptPubKeyHex,
      address,
      type
  };
}

export function decodeUTXOs(signedTxHex: string, network: bitcoin.Network, filterAddress?: string): UTXOLimited[] {
  const tx = bitcoin.Transaction.fromHex(signedTxHex)
  const txid = tx.getId()

  return Array.from(tx.outs).filter((output)=>{
    if (!filterAddress) return true
    
    try {
      const address = bitcoin.address.fromOutputScript(output.script, network)
      return address && address == filterAddress
    } catch(e: any) {
      return false
    } 
  }).map((output, index)=>{
    const scriptPubKey = decodeScriptPubKey(output.script.toString('hex'), network)
    return {
      n: index,
      txid: txid,
      sats: output.value,
      scriptPubKey: scriptPubKey
    }
  })
}

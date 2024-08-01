import { InscriptionID } from '../types'

export function parseInscriptionID(id: string): InscriptionID {
  // Regular expression to match the hexadecimal txid and the index
  const match = id.match(/([a-fA-F0-9]+)(i)(\d+)$/)
  if (!match) {
    throw new Error('Invalid InscriptionID format')
  }

  // Extract the txid and index from the matched groups
  const txid = match[1]
  const index = parseInt(match[3], 10)

  return { txid, index }
}

export function inscriptionIDToString(inscriptionID: InscriptionID): string {
  return `${inscriptionID.txid}i${inscriptionID.index}`
}

export function extractInscription(generator: string): string | null {
  const match = generator.match(/\/inscription\/([a-f0-9]+i[0-9])/i);
  return match ? match[1] : null;
}

export function extractInscriptionID(generator: string): InscriptionID | null {
  const id = extractInscription(generator)
  if (!id) {
    return null
  }

  return parseInscriptionID(id)
}

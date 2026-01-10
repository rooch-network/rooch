
<a name="0x4_merkle_proof"></a>

# Module `0x4::merkle_proof`



-  [Constants](#@Constants_0)
-  [Function `verify_merkle_proof`](#0x4_merkle_proof_verify_merkle_proof)


<pre><code><b>use</b> <a href="bitcoin_hash.md#0x4_bitcoin_hash">0x4::bitcoin_hash</a>;
<b>use</b> <a href="types.md#0x4_types">0x4::types</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_merkle_proof_ErrorInvalidProof"></a>



<pre><code><b>const</b> <a href="merkle_proof.md#0x4_merkle_proof_ErrorInvalidProof">ErrorInvalidProof</a>: u64 = 1;
</code></pre>



<a name="0x4_merkle_proof_verify_merkle_proof"></a>

## Function `verify_merkle_proof`

Verify a Merkle proof against a known root


<pre><code><b>public</b> <b>fun</b> <a href="merkle_proof.md#0x4_merkle_proof_verify_merkle_proof">verify_merkle_proof</a>(tx_hash: <b>address</b>, merkle_root: <b>address</b>, proof: &<a href="types.md#0x4_types_MerkleProof">types::MerkleProof</a>): bool
</code></pre>

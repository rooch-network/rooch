
<a name="0x2_groth16"></a>

# Module `0x2::groth16`

Source from https://github.com/MystenLabs/sui/blob/924c294d9b4a98d5bc50cd6c830e7c0cdbc2a2b1/crates/sui-framework/packages/sui-framework/sources/crypto/groth16.move


-  [Struct `Curve`](#0x2_groth16_Curve)
-  [Struct `PreparedVerifyingKey`](#0x2_groth16_PreparedVerifyingKey)
-  [Struct `PublicProofInputs`](#0x2_groth16_PublicProofInputs)
-  [Struct `ProofPoints`](#0x2_groth16_ProofPoints)
-  [Constants](#@Constants_0)
-  [Function `bls12381`](#0x2_groth16_bls12381)
-  [Function `bn254`](#0x2_groth16_bn254)
-  [Function `pvk_from_bytes`](#0x2_groth16_pvk_from_bytes)
-  [Function `pvk_to_bytes`](#0x2_groth16_pvk_to_bytes)
-  [Function `public_proof_inputs_from_bytes`](#0x2_groth16_public_proof_inputs_from_bytes)
-  [Function `proof_points_from_bytes`](#0x2_groth16_proof_points_from_bytes)
-  [Function `prepare_verifying_key`](#0x2_groth16_prepare_verifying_key)
-  [Function `verify_groth16_proof`](#0x2_groth16_verify_groth16_proof)


<pre><code></code></pre>



<a name="0x2_groth16_Curve"></a>

## Struct `Curve`

Represents an elliptic curve construction to be used in the verifier. Currently we support BLS12-381 and BN254.
This should be given as the first parameter to <code>prepare_verifying_key</code> or <code>verify_groth16_proof</code>.


<pre><code><b>struct</b> <a href="groth16.md#0x2_groth16_Curve">Curve</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_groth16_PreparedVerifyingKey"></a>

## Struct `PreparedVerifyingKey`

A <code><a href="groth16.md#0x2_groth16_PreparedVerifyingKey">PreparedVerifyingKey</a></code> consisting of four components in serialized form.


<pre><code><b>struct</b> <a href="groth16.md#0x2_groth16_PreparedVerifyingKey">PreparedVerifyingKey</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_groth16_PublicProofInputs"></a>

## Struct `PublicProofInputs`

A <code><a href="groth16.md#0x2_groth16_PublicProofInputs">PublicProofInputs</a></code> wrapper around its serialized bytes.


<pre><code><b>struct</b> <a href="groth16.md#0x2_groth16_PublicProofInputs">PublicProofInputs</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_groth16_ProofPoints"></a>

## Struct `ProofPoints`

A <code><a href="groth16.md#0x2_groth16_ProofPoints">ProofPoints</a></code> wrapper around the serialized form of three proof points.


<pre><code><b>struct</b> <a href="groth16.md#0x2_groth16_ProofPoints">ProofPoints</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_groth16_E_INVALID_CURVE"></a>



<pre><code><b>const</b> <a href="groth16.md#0x2_groth16_E_INVALID_CURVE">E_INVALID_CURVE</a>: u64 = 0;
</code></pre>



<a name="0x2_groth16_E_INVALID_VERIFYING_KEY"></a>



<pre><code><b>const</b> <a href="groth16.md#0x2_groth16_E_INVALID_VERIFYING_KEY">E_INVALID_VERIFYING_KEY</a>: u64 = 1;
</code></pre>



<a name="0x2_groth16_E_TOO_MANY_PUBLIC_INPUTS"></a>



<pre><code><b>const</b> <a href="groth16.md#0x2_groth16_E_TOO_MANY_PUBLIC_INPUTS">E_TOO_MANY_PUBLIC_INPUTS</a>: u64 = 2;
</code></pre>



<a name="0x2_groth16_bls12381"></a>

## Function `bls12381`

Return the <code><a href="groth16.md#0x2_groth16_Curve">Curve</a></code> value indicating that the BLS12-381 construction should be used in a given function.


<pre><code><b>public</b> <b>fun</b> <a href="bls12381.md#0x2_bls12381">bls12381</a>(): <a href="groth16.md#0x2_groth16_Curve">groth16::Curve</a>
</code></pre>



<a name="0x2_groth16_bn254"></a>

## Function `bn254`

Return the <code><a href="groth16.md#0x2_groth16_Curve">Curve</a></code> value indicating that the BN254 construction should be used in a given function.


<pre><code><b>public</b> <b>fun</b> <a href="groth16.md#0x2_groth16_bn254">bn254</a>(): <a href="groth16.md#0x2_groth16_Curve">groth16::Curve</a>
</code></pre>



<a name="0x2_groth16_pvk_from_bytes"></a>

## Function `pvk_from_bytes`

Creates a <code><a href="groth16.md#0x2_groth16_PreparedVerifyingKey">PreparedVerifyingKey</a></code> from bytes.


<pre><code><b>public</b> <b>fun</b> <a href="groth16.md#0x2_groth16_pvk_from_bytes">pvk_from_bytes</a>(vk_gamma_abc_g1_bytes: <a href="">vector</a>&lt;u8&gt;, alpha_g1_beta_g2_bytes: <a href="">vector</a>&lt;u8&gt;, gamma_g2_neg_pc_bytes: <a href="">vector</a>&lt;u8&gt;, delta_g2_neg_pc_bytes: <a href="">vector</a>&lt;u8&gt;): <a href="groth16.md#0x2_groth16_PreparedVerifyingKey">groth16::PreparedVerifyingKey</a>
</code></pre>



<a name="0x2_groth16_pvk_to_bytes"></a>

## Function `pvk_to_bytes`

Returns bytes of the four components of the <code><a href="groth16.md#0x2_groth16_PreparedVerifyingKey">PreparedVerifyingKey</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="groth16.md#0x2_groth16_pvk_to_bytes">pvk_to_bytes</a>(pvk: <a href="groth16.md#0x2_groth16_PreparedVerifyingKey">groth16::PreparedVerifyingKey</a>): <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<a name="0x2_groth16_public_proof_inputs_from_bytes"></a>

## Function `public_proof_inputs_from_bytes`

Creates a <code><a href="groth16.md#0x2_groth16_PublicProofInputs">PublicProofInputs</a></code> wrapper from bytes.


<pre><code><b>public</b> <b>fun</b> <a href="groth16.md#0x2_groth16_public_proof_inputs_from_bytes">public_proof_inputs_from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="groth16.md#0x2_groth16_PublicProofInputs">groth16::PublicProofInputs</a>
</code></pre>



<a name="0x2_groth16_proof_points_from_bytes"></a>

## Function `proof_points_from_bytes`

Creates a Groth16 <code><a href="groth16.md#0x2_groth16_ProofPoints">ProofPoints</a></code> from bytes.


<pre><code><b>public</b> <b>fun</b> <a href="groth16.md#0x2_groth16_proof_points_from_bytes">proof_points_from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <a href="groth16.md#0x2_groth16_ProofPoints">groth16::ProofPoints</a>
</code></pre>



<a name="0x2_groth16_prepare_verifying_key"></a>

## Function `prepare_verifying_key`

@param curve: What elliptic curve construction to use. See <code><a href="bls12381.md#0x2_bls12381">bls12381</a></code> and <code>bn254</code>.
@param verifying_key: An Arkworks canonical compressed serialization of a verifying key.

Returns four vectors of bytes representing the four components of a prepared verifying key.
This step computes one pairing e(P, Q), and binds the verification to one particular proof statement.
This can be used as inputs for the <code>verify_groth16_proof</code> function.


<pre><code><b>public</b> <b>fun</b> <a href="groth16.md#0x2_groth16_prepare_verifying_key">prepare_verifying_key</a>(curve: &<a href="groth16.md#0x2_groth16_Curve">groth16::Curve</a>, verifying_key: &<a href="">vector</a>&lt;u8&gt;): <a href="groth16.md#0x2_groth16_PreparedVerifyingKey">groth16::PreparedVerifyingKey</a>
</code></pre>



<a name="0x2_groth16_verify_groth16_proof"></a>

## Function `verify_groth16_proof`

@param curve: What elliptic curve construction to use. See the <code><a href="bls12381.md#0x2_bls12381">bls12381</a></code> and <code>bn254</code> functions.
@param prepared_verifying_key: Consists of four vectors of bytes representing the four components of a prepared verifying key.
@param public_proof_inputs: Represent inputs that are public.
@param proof_points: Represent three proof points.

Returns a boolean indicating whether the proof is valid.


<pre><code><b>public</b> <b>fun</b> <a href="groth16.md#0x2_groth16_verify_groth16_proof">verify_groth16_proof</a>(curve: &<a href="groth16.md#0x2_groth16_Curve">groth16::Curve</a>, prepared_verifying_key: &<a href="groth16.md#0x2_groth16_PreparedVerifyingKey">groth16::PreparedVerifyingKey</a>, public_proof_inputs: &<a href="groth16.md#0x2_groth16_PublicProofInputs">groth16::PublicProofInputs</a>, proof_points: &<a href="groth16.md#0x2_groth16_ProofPoints">groth16::ProofPoints</a>): bool
</code></pre>

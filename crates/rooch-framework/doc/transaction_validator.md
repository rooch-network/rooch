
<a name="0x3_transaction_validator"></a>

# Module `0x3::transaction_validator`



-  [Constants](#@Constants_0)
-  [Function `validate`](#0x3_transaction_validator_validate)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="account.md#0x3_account">0x3::account</a>;
<b>use</b> <a href="address_mapping.md#0x3_address_mapping">0x3::address_mapping</a>;
<b>use</b> <a href="authenticator.md#0x3_authenticator">0x3::authenticator</a>;
<b>use</b> <a href="ecdsa_k1.md#0x3_ecdsa_k1">0x3::ecdsa_k1</a>;
<b>use</b> <a href="ed25519.md#0x3_ed25519">0x3::ed25519</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_transaction_validator_MAX_U64"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x3_transaction_validator_ED25519_SCHEME"></a>

Scheme identifier for Ed25519 signatures used to derive authentication keys for Ed25519 public keys.


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_ED25519_SCHEME">ED25519_SCHEME</a>: u64 = 0;
</code></pre>



<a name="0x3_transaction_validator_EInvalidAuthenticator"></a>

InvalidAuthenticator, incloude invalid signature


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EInvalidAuthenticator">EInvalidAuthenticator</a>: u64 = 1010;
</code></pre>



<a name="0x3_transaction_validator_EOUT_OF_GAS"></a>

Transaction exceeded its allocated max gas


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EOUT_OF_GAS">EOUT_OF_GAS</a>: u64 = 6;
</code></pre>



<a name="0x3_transaction_validator_EPrologueAccountDoesNotExist"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EPrologueAccountDoesNotExist">EPrologueAccountDoesNotExist</a>: u64 = 1004;
</code></pre>



<a name="0x3_transaction_validator_EPrologueBadChainId"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EPrologueBadChainId">EPrologueBadChainId</a>: u64 = 1007;
</code></pre>



<a name="0x3_transaction_validator_EPrologueCantPayGasDeposit"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EPrologueCantPayGasDeposit">EPrologueCantPayGasDeposit</a>: u64 = 1005;
</code></pre>



<a name="0x3_transaction_validator_EPrologueInvalidAccountAuthKey"></a>

Prologue errors. These are separated out from the other errors in this
module since they are mapped separately to major VM statuses, and are
important to the semantics of the system.


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EPrologueInvalidAccountAuthKey">EPrologueInvalidAccountAuthKey</a>: u64 = 1001;
</code></pre>



<a name="0x3_transaction_validator_EPrologueSecondaryKeysAddressesCountMismatch"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EPrologueSecondaryKeysAddressesCountMismatch">EPrologueSecondaryKeysAddressesCountMismatch</a>: u64 = 1009;
</code></pre>



<a name="0x3_transaction_validator_EPrologueSequenceNuberTooOld"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EPrologueSequenceNuberTooOld">EPrologueSequenceNuberTooOld</a>: u64 = 1002;
</code></pre>



<a name="0x3_transaction_validator_EPrologueSequenceNumberTooBig"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EPrologueSequenceNumberTooBig">EPrologueSequenceNumberTooBig</a>: u64 = 1008;
</code></pre>



<a name="0x3_transaction_validator_EPrologueSequenceNumberTooNew"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EPrologueSequenceNumberTooNew">EPrologueSequenceNumberTooNew</a>: u64 = 1003;
</code></pre>



<a name="0x3_transaction_validator_EPrologueTransactionExpired"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EPrologueTransactionExpired">EPrologueTransactionExpired</a>: u64 = 1006;
</code></pre>



<a name="0x3_transaction_validator_MULTI_ED25519_SCHEME"></a>

Scheme identifier for MultiEd25519 signatures used to derive authentication keys for MultiEd25519 public keys.


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>: u64 = 1;
</code></pre>



<a name="0x3_transaction_validator_SECP256K1_SCHEME"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_SECP256K1_SCHEME">SECP256K1_SCHEME</a>: u64 = 2;
</code></pre>



<a name="0x3_transaction_validator_validate"></a>

## Function `validate`

This function is for Rooch to validate the transaction sender's authenticator.
If the authenticator is invaid, abort this function.


<pre><code><b>public</b> <b>fun</b> <a href="transaction_validator.md#0x3_transaction_validator_validate">validate</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, authenticator_info_bytes: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_validator.md#0x3_transaction_validator_validate">validate</a>(ctx: &StorageContext, authenticator_info_bytes: <a href="">vector</a>&lt;u8&gt;){
    <b>let</b> (tx_sequence_number, <a href="authenticator.md#0x3_authenticator">authenticator</a>) = <a href="authenticator.md#0x3_authenticator_decode_authenticator_info">authenticator::decode_authenticator_info</a>(authenticator_info_bytes);
    <a href="authenticator.md#0x3_authenticator_check_authenticator">authenticator::check_authenticator</a>(&<a href="authenticator.md#0x3_authenticator">authenticator</a>);
    <b>let</b> scheme = <a href="authenticator.md#0x3_authenticator_scheme">authenticator::scheme</a>(&<a href="authenticator.md#0x3_authenticator">authenticator</a>);
    <b>if</b> (scheme == <a href="transaction_validator.md#0x3_transaction_validator_ED25519_SCHEME">ED25519_SCHEME</a>) {
        <b>let</b> ed25519_authenicator = <a href="authenticator.md#0x3_authenticator_decode_ed25519_authenticator">authenticator::decode_ed25519_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>);
        //FIXME we need <b>to</b> check the <b>public</b> key and <b>address</b> relationship
        //The <b>address</b> is the <b>public</b> key's <a href="">hash</a>
        //We also need <b>to</b> check the <b>public</b> key via <a href="account.md#0x3_account">account</a>'s auth key, <b>if</b> the user rotate the auth key.
        <b>assert</b>!(
        <a href="ed25519.md#0x3_ed25519_verify">ed25519::verify</a>(&<a href="authenticator.md#0x3_authenticator_ed25519_signature">authenticator::ed25519_signature</a>(&ed25519_authenicator),
            &<a href="authenticator.md#0x3_authenticator_ed25519_public">authenticator::ed25519_public</a>(&ed25519_authenicator),
            &<a href="_tx_hash">storage_context::tx_hash</a>(ctx)),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="transaction_validator.md#0x3_transaction_validator_EInvalidAuthenticator">EInvalidAuthenticator</a>));
    } <b>else</b> <b>if</b> (scheme == <a href="transaction_validator.md#0x3_transaction_validator_SECP256K1_SCHEME">SECP256K1_SCHEME</a>) {
        <b>let</b> ecdsa_k1_authenicator = <a href="authenticator.md#0x3_authenticator_decode_secp256k1_authenticator">authenticator::decode_secp256k1_authenticator</a>(<a href="authenticator.md#0x3_authenticator">authenticator</a>);
        <b>assert</b>!(
        <a href="ecdsa_k1.md#0x3_ecdsa_k1_verify">ecdsa_k1::verify</a>(
            &<a href="authenticator.md#0x3_authenticator_secp256k1_signature">authenticator::secp256k1_signature</a>(&ecdsa_k1_authenicator),
            &<a href="_tx_hash">storage_context::tx_hash</a>(ctx),
            0 // KECCAK256:0, SHA256:1, TODO: The <a href="">hash</a> type may need <b>to</b> be passed through the <a href="authenticator.md#0x3_authenticator">authenticator</a>
        ),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="transaction_validator.md#0x3_transaction_validator_EInvalidAuthenticator">EInvalidAuthenticator</a>));
    };

    <b>assert</b>!(
        (tx_sequence_number <b>as</b> u128) &lt; <a href="transaction_validator.md#0x3_transaction_validator_MAX_U64">MAX_U64</a>,
        <a href="_out_of_range">error::out_of_range</a>(<a href="transaction_validator.md#0x3_transaction_validator_EPrologueSequenceNumberTooBig">EPrologueSequenceNumberTooBig</a>)
    );

    <b>let</b> account_sequence_number = <a href="account.md#0x3_account_sequence_number_for_sender">account::sequence_number_for_sender</a>(ctx);
    <b>assert</b>!(
        tx_sequence_number &gt;= account_sequence_number,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="transaction_validator.md#0x3_transaction_validator_EPrologueSequenceNuberTooOld">EPrologueSequenceNuberTooOld</a>)
    );

    // [PCA12]: Check that the transaction's sequence number matches the
    // current sequence number. Otherwise sequence number is too new by [PCA11].
    <b>assert</b>!(
        tx_sequence_number == account_sequence_number,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="transaction_validator.md#0x3_transaction_validator_EPrologueSequenceNumberTooNew">EPrologueSequenceNumberTooNew</a>)
    );
}
</code></pre>



</details>

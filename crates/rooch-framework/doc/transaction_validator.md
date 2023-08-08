
<a name="0x3_transaction_validator"></a>

# Module `0x3::transaction_validator`



-  [Constants](#@Constants_0)
-  [Function `validate`](#0x3_transaction_validator_validate)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x2::storage_context</a>;
<b>use</b> <a href="account.md#0x3_account">0x3::account</a>;
<b>use</b> <a href="account_authentication.md#0x3_account_authentication">0x3::account_authentication</a>;
<b>use</b> <a href="address_mapping.md#0x3_address_mapping">0x3::address_mapping</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="auth_validator_registry.md#0x3_auth_validator_registry">0x3::auth_validator_registry</a>;
<b>use</b> <a href="builtin_validators.md#0x3_builtin_validators">0x3::builtin_validators</a>;
<b>use</b> <a href="session_key.md#0x3_session_key">0x3::session_key</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_transaction_validator_MAX_U64"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x3_transaction_validator_EOUT_OF_GAS"></a>

Transaction exceeded its allocated max gas


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EOUT_OF_GAS">EOUT_OF_GAS</a>: u64 = 6;
</code></pre>



<a name="0x3_transaction_validator_EValidateAccountDoesNotExist"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EValidateAccountDoesNotExist">EValidateAccountDoesNotExist</a>: u64 = 1003;
</code></pre>



<a name="0x3_transaction_validator_EValidateBadChainId"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EValidateBadChainId">EValidateBadChainId</a>: u64 = 1006;
</code></pre>



<a name="0x3_transaction_validator_EValidateCantPayGasDeposit"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EValidateCantPayGasDeposit">EValidateCantPayGasDeposit</a>: u64 = 1004;
</code></pre>



<a name="0x3_transaction_validator_EValidateNotInstalledAuthValidator"></a>

The authenticator's scheme is not installed to the sender's account


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EValidateNotInstalledAuthValidator">EValidateNotInstalledAuthValidator</a>: u64 = 1010;
</code></pre>



<a name="0x3_transaction_validator_EValidateSequenceNuberTooOld"></a>

Validate errors. These are separated out from the other errors in this
module since they are mapped separately to major VM statuses, and are
important to the semantics of the system.


<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EValidateSequenceNuberTooOld">EValidateSequenceNuberTooOld</a>: u64 = 1001;
</code></pre>



<a name="0x3_transaction_validator_EValidateSequenceNumberTooBig"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EValidateSequenceNumberTooBig">EValidateSequenceNumberTooBig</a>: u64 = 1007;
</code></pre>



<a name="0x3_transaction_validator_EValidateSequenceNumberTooNew"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EValidateSequenceNumberTooNew">EValidateSequenceNumberTooNew</a>: u64 = 1002;
</code></pre>



<a name="0x3_transaction_validator_EValidateTransactionExpired"></a>



<pre><code><b>const</b> <a href="transaction_validator.md#0x3_transaction_validator_EValidateTransactionExpired">EValidateTransactionExpired</a>: u64 = 1005;
</code></pre>



<a name="0x3_transaction_validator_validate"></a>

## Function `validate`

This function is for Rooch to validate the transaction sender's authenticator.
If the authenticator is invaid, abort this function.


<pre><code><b>public</b> <b>fun</b> <a href="transaction_validator.md#0x3_transaction_validator_validate">validate</a>(ctx: &<a href="_StorageContext">storage_context::StorageContext</a>, tx_sequence_number: u64, scheme: u64, authenticator_payload: <a href="">vector</a>&lt;u8&gt;): <a href="auth_validator.md#0x3_auth_validator_TxValidateResult">auth_validator::TxValidateResult</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_validator.md#0x3_transaction_validator_validate">validate</a>(ctx: &StorageContext, tx_sequence_number: u64, scheme: u64, authenticator_payload: <a href="">vector</a>&lt;u8&gt;): TxValidateResult {
    // === validate the sequence number ===

    <b>assert</b>!(
        (tx_sequence_number <b>as</b> u128) &lt; <a href="transaction_validator.md#0x3_transaction_validator_MAX_U64">MAX_U64</a>,
        <a href="_out_of_range">error::out_of_range</a>(<a href="transaction_validator.md#0x3_transaction_validator_EValidateSequenceNumberTooBig">EValidateSequenceNumberTooBig</a>)
    );

    <b>let</b> account_sequence_number = <a href="account.md#0x3_account_sequence_number_for_sender">account::sequence_number_for_sender</a>(ctx);
    <b>assert</b>!(
        tx_sequence_number &gt;= account_sequence_number,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="transaction_validator.md#0x3_transaction_validator_EValidateSequenceNuberTooOld">EValidateSequenceNuberTooOld</a>)
    );

    // [PCA12]: Check that the transaction's sequence number matches the
    // current sequence number. Otherwise sequence number is too new by [PCA11].
    <b>assert</b>!(
        tx_sequence_number == account_sequence_number,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="transaction_validator.md#0x3_transaction_validator_EValidateSequenceNumberTooNew">EValidateSequenceNumberTooNew</a>)
    );

    // === validate the authenticator ===

    // <b>if</b> the authenticator payload is session key, validate the session key
    // otherwise <b>return</b> the authentication validator via the scheme
    <b>let</b> session_key_option = <a href="session_key.md#0x3_session_key_validate">session_key::validate</a>(ctx, scheme, authenticator_payload);
    <b>if</b>(<a href="_is_some">option::is_some</a>(&session_key_option)){
        <a href="auth_validator.md#0x3_auth_validator_new_tx_validate_result">auth_validator::new_tx_validate_result</a>(<a href="_none">option::none</a>(), session_key_option)
    }<b>else</b>{
        <b>let</b> sender = <a href="_sender">storage_context::sender</a>(ctx);
        <b>let</b> <a href="auth_validator.md#0x3_auth_validator">auth_validator</a> = <a href="auth_validator_registry.md#0x3_auth_validator_registry_borrow_validator">auth_validator_registry::borrow_validator</a>(ctx, scheme);
        <b>let</b> validator_id = <a href="auth_validator.md#0x3_auth_validator_validator_id">auth_validator::validator_id</a>(<a href="auth_validator.md#0x3_auth_validator">auth_validator</a>);
        // builtin scheme do not need <b>to</b> install
        <b>if</b>(!rooch_framework::builtin_validators::is_builtin_scheme(scheme)){
            <b>assert</b>!(<a href="account_authentication.md#0x3_account_authentication_is_auth_validator_installed">account_authentication::is_auth_validator_installed</a>(ctx, sender, validator_id), <a href="_invalid_state">error::invalid_state</a>(<a href="transaction_validator.md#0x3_transaction_validator_EValidateNotInstalledAuthValidator">EValidateNotInstalledAuthValidator</a>));
        };
        <a href="auth_validator.md#0x3_auth_validator_new_tx_validate_result">auth_validator::new_tx_validate_result</a>(<a href="_some">option::some</a>(*<a href="auth_validator.md#0x3_auth_validator">auth_validator</a>), <a href="_none">option::none</a>())
    }
}
</code></pre>



</details>

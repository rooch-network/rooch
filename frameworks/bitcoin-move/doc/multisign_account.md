
<a name="0x4_multisign_account"></a>

# Module `0x4::multisign_account`

Bitcoin multisign account module


-  [Resource `MultisignAccountInfo`](#0x4_multisign_account_MultisignAccountInfo)
-  [Struct `ParticipantInfo`](#0x4_multisign_account_ParticipantInfo)
-  [Constants](#@Constants_0)
-  [Function `initialize_multisig_account_entry`](#0x4_multisign_account_initialize_multisig_account_entry)
-  [Function `initialize_multisig_account`](#0x4_multisign_account_initialize_multisig_account)
-  [Function `generate_multisign_address`](#0x4_multisign_account_generate_multisign_address)
-  [Function `is_participant`](#0x4_multisign_account_is_participant)
-  [Function `is_participant_via_public_key`](#0x4_multisign_account_is_participant_via_public_key)
-  [Function `is_multisign_account`](#0x4_multisign_account_is_multisign_account)
-  [Function `bitcoin_address`](#0x4_multisign_account_bitcoin_address)
-  [Function `threshold`](#0x4_multisign_account_threshold)
-  [Function `participant_public_key`](#0x4_multisign_account_participant_public_key)
-  [Function `participant_bitcoin_address`](#0x4_multisign_account_participant_bitcoin_address)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::compare</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::result</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x3::address_mapping</a>;
<b>use</b> <a href="">0x3::bitcoin_address</a>;
<b>use</b> <a href="">0x3::ecdsa_k1</a>;
<b>use</b> <a href="opcode.md#0x4_opcode">0x4::opcode</a>;
<b>use</b> <a href="script_buf.md#0x4_script_buf">0x4::script_buf</a>;
<b>use</b> <a href="taproot_builder.md#0x4_taproot_builder">0x4::taproot_builder</a>;
</code></pre>



<a name="0x4_multisign_account_MultisignAccountInfo"></a>

## Resource `MultisignAccountInfo`



<pre><code><b>struct</b> <a href="multisign_account.md#0x4_multisign_account_MultisignAccountInfo">MultisignAccountInfo</a> <b>has</b> store, key
</code></pre>



<a name="0x4_multisign_account_ParticipantInfo"></a>

## Struct `ParticipantInfo`



<pre><code><b>struct</b> <a href="multisign_account.md#0x4_multisign_account_ParticipantInfo">ParticipantInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_multisign_account_ErrorInvalidPublicKey"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorInvalidPublicKey">ErrorInvalidPublicKey</a>: u64 = 6;
</code></pre>



<a name="0x4_multisign_account_ErrorInvalidThreshold"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorInvalidThreshold">ErrorInvalidThreshold</a>: u64 = 1;
</code></pre>



<a name="0x4_multisign_account_ErrorInvalidSignature"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorInvalidSignature">ErrorInvalidSignature</a>: u64 = 10;
</code></pre>



<a name="0x4_multisign_account_BITCOIN_COMPRESSED_PUBLIC_KEY_LEN"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_BITCOIN_COMPRESSED_PUBLIC_KEY_LEN">BITCOIN_COMPRESSED_PUBLIC_KEY_LEN</a>: u64 = 33;
</code></pre>



<a name="0x4_multisign_account_ErrorInvalidParticipant"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorInvalidParticipant">ErrorInvalidParticipant</a>: u64 = 3;
</code></pre>



<a name="0x4_multisign_account_ErrorInvalidProposal"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorInvalidProposal">ErrorInvalidProposal</a>: u64 = 7;
</code></pre>



<a name="0x4_multisign_account_ErrorInvalidProposalStatus"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorInvalidProposalStatus">ErrorInvalidProposalStatus</a>: u64 = 9;
</code></pre>



<a name="0x4_multisign_account_ErrorMultisignAccountNotFound"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorMultisignAccountNotFound">ErrorMultisignAccountNotFound</a>: u64 = 2;
</code></pre>



<a name="0x4_multisign_account_ErrorParticipantAlreadyJoined"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorParticipantAlreadyJoined">ErrorParticipantAlreadyJoined</a>: u64 = 5;
</code></pre>



<a name="0x4_multisign_account_ErrorParticipantMustHasBitcoinAddress"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorParticipantMustHasBitcoinAddress">ErrorParticipantMustHasBitcoinAddress</a>: u64 = 4;
</code></pre>



<a name="0x4_multisign_account_ErrorProposalAlreadySigned"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_ErrorProposalAlreadySigned">ErrorProposalAlreadySigned</a>: u64 = 8;
</code></pre>



<a name="0x4_multisign_account_PROPOSAL_STATUS_APPROVED"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_PROPOSAL_STATUS_APPROVED">PROPOSAL_STATUS_APPROVED</a>: u8 = 1;
</code></pre>



<a name="0x4_multisign_account_PROPOSAL_STATUS_PENDING"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_PROPOSAL_STATUS_PENDING">PROPOSAL_STATUS_PENDING</a>: u8 = 0;
</code></pre>



<a name="0x4_multisign_account_PROPOSAL_STATUS_REJECTED"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_PROPOSAL_STATUS_REJECTED">PROPOSAL_STATUS_REJECTED</a>: u8 = 2;
</code></pre>



<a name="0x4_multisign_account_X_ONLY_PUBLIC_KEY_LEN"></a>



<pre><code><b>const</b> <a href="multisign_account.md#0x4_multisign_account_X_ONLY_PUBLIC_KEY_LEN">X_ONLY_PUBLIC_KEY_LEN</a>: u64 = 32;
</code></pre>



<a name="0x4_multisign_account_initialize_multisig_account_entry"></a>

## Function `initialize_multisig_account_entry`

Initialize a taproot multisign account
If the multisign account already exists, we will init the MultisignAccountInfo into the account


<pre><code><b>public</b> entry <b>fun</b> <a href="multisign_account.md#0x4_multisign_account_initialize_multisig_account_entry">initialize_multisig_account_entry</a>(threshold: u64, participant_public_keys: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<a name="0x4_multisign_account_initialize_multisig_account"></a>

## Function `initialize_multisig_account`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_account.md#0x4_multisign_account_initialize_multisig_account">initialize_multisig_account</a>(threshold: u64, participant_public_keys: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <b>address</b>
</code></pre>



<a name="0x4_multisign_account_generate_multisign_address"></a>

## Function `generate_multisign_address`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_account.md#0x4_multisign_account_generate_multisign_address">generate_multisign_address</a>(threshold: u64, public_keys: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;): <a href="_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>



<a name="0x4_multisign_account_is_participant"></a>

## Function `is_participant`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_account.md#0x4_multisign_account_is_participant">is_participant</a>(multisign_address: <b>address</b>, participant_address: <b>address</b>): bool
</code></pre>



<a name="0x4_multisign_account_is_participant_via_public_key"></a>

## Function `is_participant_via_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_account.md#0x4_multisign_account_is_participant_via_public_key">is_participant_via_public_key</a>(multisign_address: <b>address</b>, public_key: &<a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a name="0x4_multisign_account_is_multisign_account"></a>

## Function `is_multisign_account`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_account.md#0x4_multisign_account_is_multisign_account">is_multisign_account</a>(multisign_address: <b>address</b>): bool
</code></pre>



<a name="0x4_multisign_account_bitcoin_address"></a>

## Function `bitcoin_address`



<pre><code><b>public</b> <b>fun</b> <a href="">bitcoin_address</a>(multisign_address: <b>address</b>): <a href="_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>



<a name="0x4_multisign_account_threshold"></a>

## Function `threshold`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_account.md#0x4_multisign_account_threshold">threshold</a>(multisign_address: <b>address</b>): u64
</code></pre>



<a name="0x4_multisign_account_participant_public_key"></a>

## Function `participant_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_account.md#0x4_multisign_account_participant_public_key">participant_public_key</a>(multisign_address: <b>address</b>, participant_address: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x4_multisign_account_participant_bitcoin_address"></a>

## Function `participant_bitcoin_address`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_account.md#0x4_multisign_account_participant_bitcoin_address">participant_bitcoin_address</a>(multisign_address: <b>address</b>, participant_address: <b>address</b>): <a href="_BitcoinAddress">bitcoin_address::BitcoinAddress</a>
</code></pre>


<a name="0xa_multisign_wallet"></a>

# Module `0xa::multisign_wallet`

Bitcoin multisign account wallet to manage the multisign tx on Bitcoin and Rooch


-  [Resource `MultisignWallet`](#0xa_multisign_wallet_MultisignWallet)
-  [Struct `BitcoinProposal`](#0xa_multisign_wallet_BitcoinProposal)
-  [Struct `RoochProposal`](#0xa_multisign_wallet_RoochProposal)
-  [Constants](#@Constants_0)
-  [Function `submit_bitcoin_proposal`](#0xa_multisign_wallet_submit_bitcoin_proposal)
-  [Function `sign_bitcoin_proposal`](#0xa_multisign_wallet_sign_bitcoin_proposal)


<pre><code><b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::bcs</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::table_vec</a>;
<b>use</b> <a href="">0x3::ecdsa_k1</a>;
<b>use</b> <a href="">0x4::multisign_account</a>;
</code></pre>



<a name="0xa_multisign_wallet_MultisignWallet"></a>

## Resource `MultisignWallet`



<pre><code><b>struct</b> <a href="multisign_wallet.md#0xa_multisign_wallet_MultisignWallet">MultisignWallet</a> <b>has</b> key
</code></pre>



<a name="0xa_multisign_wallet_BitcoinProposal"></a>

## Struct `BitcoinProposal`



<pre><code><b>struct</b> <a href="multisign_wallet.md#0xa_multisign_wallet_BitcoinProposal">BitcoinProposal</a> <b>has</b> store
</code></pre>



<a name="0xa_multisign_wallet_RoochProposal"></a>

## Struct `RoochProposal`



<pre><code><b>struct</b> <a href="multisign_wallet.md#0xa_multisign_wallet_RoochProposal">RoochProposal</a> <b>has</b> store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0xa_multisign_wallet_ErrorInvalidPublicKey"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorInvalidPublicKey">ErrorInvalidPublicKey</a>: u64 = 6;
</code></pre>



<a name="0xa_multisign_wallet_ErrorInvalidThreshold"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorInvalidThreshold">ErrorInvalidThreshold</a>: u64 = 1;
</code></pre>



<a name="0xa_multisign_wallet_ErrorInvalidSignature"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorInvalidSignature">ErrorInvalidSignature</a>: u64 = 10;
</code></pre>



<a name="0xa_multisign_wallet_BITCOIN_COMPRESSED_PUBLIC_KEY_LEN"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_BITCOIN_COMPRESSED_PUBLIC_KEY_LEN">BITCOIN_COMPRESSED_PUBLIC_KEY_LEN</a>: u64 = 33;
</code></pre>



<a name="0xa_multisign_wallet_ErrorInvalidParticipant"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorInvalidParticipant">ErrorInvalidParticipant</a>: u64 = 3;
</code></pre>



<a name="0xa_multisign_wallet_ErrorInvalidProposal"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorInvalidProposal">ErrorInvalidProposal</a>: u64 = 7;
</code></pre>



<a name="0xa_multisign_wallet_ErrorInvalidProposalStatus"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorInvalidProposalStatus">ErrorInvalidProposalStatus</a>: u64 = 9;
</code></pre>



<a name="0xa_multisign_wallet_ErrorMultisignAccountNotFound"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorMultisignAccountNotFound">ErrorMultisignAccountNotFound</a>: u64 = 2;
</code></pre>



<a name="0xa_multisign_wallet_ErrorParticipantAlreadyJoined"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorParticipantAlreadyJoined">ErrorParticipantAlreadyJoined</a>: u64 = 5;
</code></pre>



<a name="0xa_multisign_wallet_ErrorParticipantMustHasBitcoinAddress"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorParticipantMustHasBitcoinAddress">ErrorParticipantMustHasBitcoinAddress</a>: u64 = 4;
</code></pre>



<a name="0xa_multisign_wallet_ErrorProposalAlreadySigned"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_ErrorProposalAlreadySigned">ErrorProposalAlreadySigned</a>: u64 = 8;
</code></pre>



<a name="0xa_multisign_wallet_PROPOSAL_STATUS_APPROVED"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_PROPOSAL_STATUS_APPROVED">PROPOSAL_STATUS_APPROVED</a>: u8 = 1;
</code></pre>



<a name="0xa_multisign_wallet_PROPOSAL_STATUS_PENDING"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_PROPOSAL_STATUS_PENDING">PROPOSAL_STATUS_PENDING</a>: u8 = 0;
</code></pre>



<a name="0xa_multisign_wallet_PROPOSAL_STATUS_REJECTED"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_PROPOSAL_STATUS_REJECTED">PROPOSAL_STATUS_REJECTED</a>: u8 = 2;
</code></pre>



<a name="0xa_multisign_wallet_X_ONLY_PUBLIC_KEY_LEN"></a>



<pre><code><b>const</b> <a href="multisign_wallet.md#0xa_multisign_wallet_X_ONLY_PUBLIC_KEY_LEN">X_ONLY_PUBLIC_KEY_LEN</a>: u64 = 32;
</code></pre>



<a name="0xa_multisign_wallet_submit_bitcoin_proposal"></a>

## Function `submit_bitcoin_proposal`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_wallet.md#0xa_multisign_wallet_submit_bitcoin_proposal">submit_bitcoin_proposal</a>(sender: &<a href="">signer</a>, multisign_address: <b>address</b>, tx_id: <b>address</b>, tx_data: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0xa_multisign_wallet_sign_bitcoin_proposal"></a>

## Function `sign_bitcoin_proposal`



<pre><code><b>public</b> <b>fun</b> <a href="multisign_wallet.md#0xa_multisign_wallet_sign_bitcoin_proposal">sign_bitcoin_proposal</a>(sender: &<a href="">signer</a>, multisign_address: <b>address</b>, proposal_id: u64, signature: <a href="">vector</a>&lt;u8&gt;)
</code></pre>

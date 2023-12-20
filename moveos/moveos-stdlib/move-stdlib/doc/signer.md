
<a name="0x1_signer"></a>

# Module `0x1::signer`



-  [Function `borrow_address`](#0x1_signer_borrow_address)
-  [Function `address_of`](#0x1_signer_address_of)
-  [Module Specification](#@Module_Specification_0)


<pre><code></code></pre>



<a name="0x1_signer_borrow_address"></a>

## Function `borrow_address`

Borrows the address of the signer
Conceptually, you can think of the <code><a href="signer.md#0x1_signer">signer</a></code> as being a struct wrapper around an
address
```
struct signer has drop { addr: address }
```
<code>borrow_address</code> borrows this inner field


<pre><code><b>public</b> <b>fun</b> <a href="signer.md#0x1_signer_borrow_address">borrow_address</a>(s: &<a href="signer.md#0x1_signer">signer</a>): &<b>address</b>
</code></pre>



<a name="0x1_signer_address_of"></a>

## Function `address_of`



<pre><code><b>public</b> <b>fun</b> <a href="signer.md#0x1_signer_address_of">address_of</a>(s: &<a href="signer.md#0x1_signer">signer</a>): <b>address</b>
</code></pre>



<a name="@Module_Specification_0"></a>

## Module Specification

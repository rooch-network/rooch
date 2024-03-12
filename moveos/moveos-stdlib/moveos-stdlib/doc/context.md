
<a name="0x2_context"></a>

# Module `0x2::context`

Context is part of the StorageAbstraction
It is used to provide a context for the storage operations, make the storage abstraction,
and let developers customize the storage


-  [Struct `Context`](#0x2_context_Context)
-  [Constants](#@Constants_0)


<pre><code><b>use</b> <a href="storage_context.md#0x2_storage_context">0x2::storage_context</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_context_Context"></a>

## Struct `Context`

Information about the global context include TxContext and StorageContext
We can not put the StorageContext to TxContext, because object module depends on tx_context module,
and storage_context module depends on object module.
We put both TxContext and StorageContext to Context, for convenience of developers.
The Context can not be <code>drop</code> or <code>store</code>, so developers need to pass the <code>&<a href="context.md#0x2_context_Context">Context</a></code> or <code>&<b>mut</b> <a href="context.md#0x2_context_Context">Context</a></code> to the <code>entry</code> function.


<pre><code><b>struct</b> <a href="context.md#0x2_context_Context">Context</a>
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_context_ErrorObjectIsBound"></a>

Can not take out the object which is bound to the account


<pre><code><b>const</b> <a href="context.md#0x2_context_ErrorObjectIsBound">ErrorObjectIsBound</a>: u64 = 3;
</code></pre>



<a name="0x2_context_ErrorObjectNotShared"></a>



<pre><code><b>const</b> <a href="context.md#0x2_context_ErrorObjectNotShared">ErrorObjectNotShared</a>: u64 = 2;
</code></pre>



<a name="0x2_context_ErrorObjectOwnerNotMatch"></a>



<pre><code><b>const</b> <a href="context.md#0x2_context_ErrorObjectOwnerNotMatch">ErrorObjectOwnerNotMatch</a>: u64 = 1;
</code></pre>


<a name="0x3_bitcoin_opcode"></a>

# Module `0x3::bitcoin_opcode`



-  [Constants](#@Constants_0)
-  [Function `pushbytes_0`](#0x3_bitcoin_opcode_pushbytes_0)
-  [Function `pushbytes_1`](#0x3_bitcoin_opcode_pushbytes_1)
-  [Function `pushbytes_2`](#0x3_bitcoin_opcode_pushbytes_2)
-  [Function `pushbytes_3`](#0x3_bitcoin_opcode_pushbytes_3)
-  [Function `pushbytes_4`](#0x3_bitcoin_opcode_pushbytes_4)
-  [Function `pushbytes_5`](#0x3_bitcoin_opcode_pushbytes_5)
-  [Function `pushbytes_6`](#0x3_bitcoin_opcode_pushbytes_6)
-  [Function `pushbytes_7`](#0x3_bitcoin_opcode_pushbytes_7)
-  [Function `pushbytes_8`](#0x3_bitcoin_opcode_pushbytes_8)
-  [Function `pushbytes_9`](#0x3_bitcoin_opcode_pushbytes_9)
-  [Function `pushbytes_10`](#0x3_bitcoin_opcode_pushbytes_10)
-  [Function `pushbytes_11`](#0x3_bitcoin_opcode_pushbytes_11)
-  [Function `pushbytes_12`](#0x3_bitcoin_opcode_pushbytes_12)
-  [Function `pushbytes_13`](#0x3_bitcoin_opcode_pushbytes_13)
-  [Function `pushbytes_14`](#0x3_bitcoin_opcode_pushbytes_14)
-  [Function `pushbytes_15`](#0x3_bitcoin_opcode_pushbytes_15)
-  [Function `pushbytes_16`](#0x3_bitcoin_opcode_pushbytes_16)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_0"></a>

Push an empty array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_0">OP_PUSHBYTES_0</a>: u8 = 0;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_1"></a>

Push the next byte as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_1">OP_PUSHBYTES_1</a>: u8 = 1;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_10"></a>

Push the next 10 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_10">OP_PUSHBYTES_10</a>: u8 = 10;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_11"></a>

Push the next 11 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_11">OP_PUSHBYTES_11</a>: u8 = 11;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_12"></a>

Push the next 12 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_12">OP_PUSHBYTES_12</a>: u8 = 12;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_13"></a>

Push the next 13 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_13">OP_PUSHBYTES_13</a>: u8 = 13;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_14"></a>

Push the next 14 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_14">OP_PUSHBYTES_14</a>: u8 = 14;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_15"></a>

Push the next 15 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_15">OP_PUSHBYTES_15</a>: u8 = 15;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_16"></a>

Push the next 16 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_16">OP_PUSHBYTES_16</a>: u8 = 16;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_2"></a>

Push the next 2 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_2">OP_PUSHBYTES_2</a>: u8 = 2;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_3"></a>

Push the next 3 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_3">OP_PUSHBYTES_3</a>: u8 = 3;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_4"></a>

Push the next 4 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_4">OP_PUSHBYTES_4</a>: u8 = 4;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_5"></a>

Push the next 5 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_5">OP_PUSHBYTES_5</a>: u8 = 5;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_6"></a>

Push the next 6 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_6">OP_PUSHBYTES_6</a>: u8 = 6;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_7"></a>

Push the next 7 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_7">OP_PUSHBYTES_7</a>: u8 = 7;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_8"></a>

Push the next 8 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_8">OP_PUSHBYTES_8</a>: u8 = 8;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_9"></a>

Push the next 9 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_9">OP_PUSHBYTES_9</a>: u8 = 9;
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_0"></a>

## Function `pushbytes_0`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_0">pushbytes_0</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_1"></a>

## Function `pushbytes_1`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_1">pushbytes_1</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_2"></a>

## Function `pushbytes_2`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_2">pushbytes_2</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_3"></a>

## Function `pushbytes_3`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_3">pushbytes_3</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_4"></a>

## Function `pushbytes_4`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_4">pushbytes_4</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_5"></a>

## Function `pushbytes_5`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_5">pushbytes_5</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_6"></a>

## Function `pushbytes_6`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_6">pushbytes_6</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_7"></a>

## Function `pushbytes_7`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_7">pushbytes_7</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_8"></a>

## Function `pushbytes_8`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_8">pushbytes_8</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_9"></a>

## Function `pushbytes_9`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_9">pushbytes_9</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_10"></a>

## Function `pushbytes_10`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_10">pushbytes_10</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_11"></a>

## Function `pushbytes_11`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_11">pushbytes_11</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_12"></a>

## Function `pushbytes_12`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_12">pushbytes_12</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_13"></a>

## Function `pushbytes_13`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_13">pushbytes_13</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_14"></a>

## Function `pushbytes_14`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_14">pushbytes_14</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_15"></a>

## Function `pushbytes_15`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_15">pushbytes_15</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_pushbytes_16"></a>

## Function `pushbytes_16`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_pushbytes_16">pushbytes_16</a>(): u8
</code></pre>

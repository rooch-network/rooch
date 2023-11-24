
<a name="0x3_bitcoin_opcode"></a>

# Module `0x3::bitcoin_opcode`

https://github.com/rust-bitcoin/rust-bitcoin/blob/71d92bdbb91693b7882f8cd4a7e874b4e6f9eb48/bitcoin/src/blockdata/opcodes.rs#L81


-  [Constants](#@Constants_0)
-  [Function `op_pushbytes_0`](#0x3_bitcoin_opcode_op_pushbytes_0)
-  [Function `op_pushbytes_1`](#0x3_bitcoin_opcode_op_pushbytes_1)
-  [Function `op_pushbytes_2`](#0x3_bitcoin_opcode_op_pushbytes_2)
-  [Function `op_pushbytes_3`](#0x3_bitcoin_opcode_op_pushbytes_3)
-  [Function `op_pushbytes_4`](#0x3_bitcoin_opcode_op_pushbytes_4)
-  [Function `op_pushbytes_5`](#0x3_bitcoin_opcode_op_pushbytes_5)
-  [Function `op_pushbytes_6`](#0x3_bitcoin_opcode_op_pushbytes_6)
-  [Function `op_pushbytes_7`](#0x3_bitcoin_opcode_op_pushbytes_7)
-  [Function `op_pushbytes_8`](#0x3_bitcoin_opcode_op_pushbytes_8)
-  [Function `op_pushbytes_9`](#0x3_bitcoin_opcode_op_pushbytes_9)
-  [Function `op_pushbytes_10`](#0x3_bitcoin_opcode_op_pushbytes_10)
-  [Function `op_pushbytes_11`](#0x3_bitcoin_opcode_op_pushbytes_11)
-  [Function `op_pushbytes_12`](#0x3_bitcoin_opcode_op_pushbytes_12)
-  [Function `op_pushbytes_13`](#0x3_bitcoin_opcode_op_pushbytes_13)
-  [Function `op_pushbytes_14`](#0x3_bitcoin_opcode_op_pushbytes_14)
-  [Function `op_pushbytes_15`](#0x3_bitcoin_opcode_op_pushbytes_15)
-  [Function `op_pushbytes_16`](#0x3_bitcoin_opcode_op_pushbytes_16)
-  [Function `op_pushbytes_17`](#0x3_bitcoin_opcode_op_pushbytes_17)
-  [Function `op_pushbytes_18`](#0x3_bitcoin_opcode_op_pushbytes_18)
-  [Function `op_pushbytes_19`](#0x3_bitcoin_opcode_op_pushbytes_19)
-  [Function `op_pushbytes_20`](#0x3_bitcoin_opcode_op_pushbytes_20)
-  [Function `op_pushbytes_21`](#0x3_bitcoin_opcode_op_pushbytes_21)
-  [Function `op_pushbytes_22`](#0x3_bitcoin_opcode_op_pushbytes_22)
-  [Function `op_pushbytes_23`](#0x3_bitcoin_opcode_op_pushbytes_23)
-  [Function `op_pushbytes_24`](#0x3_bitcoin_opcode_op_pushbytes_24)
-  [Function `op_pushbytes_25`](#0x3_bitcoin_opcode_op_pushbytes_25)
-  [Function `op_pushbytes_26`](#0x3_bitcoin_opcode_op_pushbytes_26)
-  [Function `op_pushbytes_27`](#0x3_bitcoin_opcode_op_pushbytes_27)
-  [Function `op_pushbytes_28`](#0x3_bitcoin_opcode_op_pushbytes_28)
-  [Function `op_pushbytes_29`](#0x3_bitcoin_opcode_op_pushbytes_29)
-  [Function `op_pushbytes_30`](#0x3_bitcoin_opcode_op_pushbytes_30)
-  [Function `op_pushbytes_31`](#0x3_bitcoin_opcode_op_pushbytes_31)
-  [Function `op_pushbytes_32`](#0x3_bitcoin_opcode_op_pushbytes_32)
-  [Function `op_pushbytes_33`](#0x3_bitcoin_opcode_op_pushbytes_33)
-  [Function `op_pushbytes_34`](#0x3_bitcoin_opcode_op_pushbytes_34)
-  [Function `op_pushbytes_35`](#0x3_bitcoin_opcode_op_pushbytes_35)
-  [Function `op_pushbytes_36`](#0x3_bitcoin_opcode_op_pushbytes_36)
-  [Function `op_pushbytes_37`](#0x3_bitcoin_opcode_op_pushbytes_37)
-  [Function `op_pushbytes_38`](#0x3_bitcoin_opcode_op_pushbytes_38)
-  [Function `op_pushbytes_39`](#0x3_bitcoin_opcode_op_pushbytes_39)
-  [Function `op_pushbytes_40`](#0x3_bitcoin_opcode_op_pushbytes_40)
-  [Function `op_pushbytes_41`](#0x3_bitcoin_opcode_op_pushbytes_41)
-  [Function `op_pushbytes_42`](#0x3_bitcoin_opcode_op_pushbytes_42)
-  [Function `op_pushbytes_43`](#0x3_bitcoin_opcode_op_pushbytes_43)
-  [Function `op_pushbytes_44`](#0x3_bitcoin_opcode_op_pushbytes_44)
-  [Function `op_pushbytes_45`](#0x3_bitcoin_opcode_op_pushbytes_45)
-  [Function `op_pushbytes_46`](#0x3_bitcoin_opcode_op_pushbytes_46)
-  [Function `op_pushbytes_47`](#0x3_bitcoin_opcode_op_pushbytes_47)
-  [Function `op_pushbytes_48`](#0x3_bitcoin_opcode_op_pushbytes_48)
-  [Function `op_pushbytes_49`](#0x3_bitcoin_opcode_op_pushbytes_49)
-  [Function `op_pushbytes_50`](#0x3_bitcoin_opcode_op_pushbytes_50)
-  [Function `op_pushbytes_51`](#0x3_bitcoin_opcode_op_pushbytes_51)
-  [Function `op_pushbytes_52`](#0x3_bitcoin_opcode_op_pushbytes_52)
-  [Function `op_pushbytes_53`](#0x3_bitcoin_opcode_op_pushbytes_53)
-  [Function `op_pushbytes_54`](#0x3_bitcoin_opcode_op_pushbytes_54)
-  [Function `op_pushbytes_55`](#0x3_bitcoin_opcode_op_pushbytes_55)
-  [Function `op_pushbytes_56`](#0x3_bitcoin_opcode_op_pushbytes_56)
-  [Function `op_pushbytes_57`](#0x3_bitcoin_opcode_op_pushbytes_57)
-  [Function `op_pushbytes_58`](#0x3_bitcoin_opcode_op_pushbytes_58)
-  [Function `op_pushbytes_59`](#0x3_bitcoin_opcode_op_pushbytes_59)
-  [Function `op_pushbytes_60`](#0x3_bitcoin_opcode_op_pushbytes_60)
-  [Function `op_pushbytes_61`](#0x3_bitcoin_opcode_op_pushbytes_61)
-  [Function `op_pushbytes_62`](#0x3_bitcoin_opcode_op_pushbytes_62)
-  [Function `op_pushbytes_63`](#0x3_bitcoin_opcode_op_pushbytes_63)
-  [Function `op_pushbytes_64`](#0x3_bitcoin_opcode_op_pushbytes_64)
-  [Function `op_pushbytes_65`](#0x3_bitcoin_opcode_op_pushbytes_65)
-  [Function `op_pushbytes_66`](#0x3_bitcoin_opcode_op_pushbytes_66)
-  [Function `op_pushbytes_67`](#0x3_bitcoin_opcode_op_pushbytes_67)
-  [Function `op_pushbytes_68`](#0x3_bitcoin_opcode_op_pushbytes_68)
-  [Function `op_pushbytes_69`](#0x3_bitcoin_opcode_op_pushbytes_69)
-  [Function `op_pushbytes_70`](#0x3_bitcoin_opcode_op_pushbytes_70)
-  [Function `op_pushbytes_71`](#0x3_bitcoin_opcode_op_pushbytes_71)
-  [Function `op_pushbytes_72`](#0x3_bitcoin_opcode_op_pushbytes_72)
-  [Function `op_pushbytes_73`](#0x3_bitcoin_opcode_op_pushbytes_73)
-  [Function `op_pushbytes_74`](#0x3_bitcoin_opcode_op_pushbytes_74)
-  [Function `op_pushbytes_75`](#0x3_bitcoin_opcode_op_pushbytes_75)
-  [Function `op_pushdata1`](#0x3_bitcoin_opcode_op_pushdata1)
-  [Function `op_pushdata2`](#0x3_bitcoin_opcode_op_pushdata2)
-  [Function `op_pushdata4`](#0x3_bitcoin_opcode_op_pushdata4)
-  [Function `op_pushnum_neg1`](#0x3_bitcoin_opcode_op_pushnum_neg1)
-  [Function `op_reserved`](#0x3_bitcoin_opcode_op_reserved)
-  [Function `op_pushnum_1`](#0x3_bitcoin_opcode_op_pushnum_1)
-  [Function `op_pushnum_2`](#0x3_bitcoin_opcode_op_pushnum_2)
-  [Function `op_pushnum_3`](#0x3_bitcoin_opcode_op_pushnum_3)
-  [Function `op_pushnum_4`](#0x3_bitcoin_opcode_op_pushnum_4)
-  [Function `op_pushnum_5`](#0x3_bitcoin_opcode_op_pushnum_5)
-  [Function `op_pushnum_6`](#0x3_bitcoin_opcode_op_pushnum_6)
-  [Function `op_pushnum_7`](#0x3_bitcoin_opcode_op_pushnum_7)
-  [Function `op_pushnum_8`](#0x3_bitcoin_opcode_op_pushnum_8)
-  [Function `op_pushnum_9`](#0x3_bitcoin_opcode_op_pushnum_9)
-  [Function `op_pushnum_10`](#0x3_bitcoin_opcode_op_pushnum_10)
-  [Function `op_pushnum_11`](#0x3_bitcoin_opcode_op_pushnum_11)
-  [Function `op_pushnum_12`](#0x3_bitcoin_opcode_op_pushnum_12)
-  [Function `op_pushnum_13`](#0x3_bitcoin_opcode_op_pushnum_13)
-  [Function `op_pushnum_14`](#0x3_bitcoin_opcode_op_pushnum_14)
-  [Function `op_pushnum_15`](#0x3_bitcoin_opcode_op_pushnum_15)
-  [Function `op_pushnum_16`](#0x3_bitcoin_opcode_op_pushnum_16)
-  [Function `op_nop`](#0x3_bitcoin_opcode_op_nop)
-  [Function `op_ver`](#0x3_bitcoin_opcode_op_ver)
-  [Function `op_if_op`](#0x3_bitcoin_opcode_op_if_op)
-  [Function `op_notif`](#0x3_bitcoin_opcode_op_notif)
-  [Function `op_verif`](#0x3_bitcoin_opcode_op_verif)
-  [Function `op_vernotif`](#0x3_bitcoin_opcode_op_vernotif)
-  [Function `op_else_op`](#0x3_bitcoin_opcode_op_else_op)
-  [Function `op_endif`](#0x3_bitcoin_opcode_op_endif)
-  [Function `op_verify`](#0x3_bitcoin_opcode_op_verify)
-  [Function `op_return`](#0x3_bitcoin_opcode_op_return)
-  [Function `op_toaltstack`](#0x3_bitcoin_opcode_op_toaltstack)
-  [Function `op_fromaltstack`](#0x3_bitcoin_opcode_op_fromaltstack)
-  [Function `op_2drop`](#0x3_bitcoin_opcode_op_2drop)
-  [Function `op_2dup`](#0x3_bitcoin_opcode_op_2dup)
-  [Function `op_3dup`](#0x3_bitcoin_opcode_op_3dup)
-  [Function `op_2over`](#0x3_bitcoin_opcode_op_2over)
-  [Function `op_2rot`](#0x3_bitcoin_opcode_op_2rot)
-  [Function `op_2swap`](#0x3_bitcoin_opcode_op_2swap)
-  [Function `op_ifdup`](#0x3_bitcoin_opcode_op_ifdup)
-  [Function `op_depth`](#0x3_bitcoin_opcode_op_depth)
-  [Function `op_drop`](#0x3_bitcoin_opcode_op_drop)
-  [Function `op_dup`](#0x3_bitcoin_opcode_op_dup)
-  [Function `op_nip`](#0x3_bitcoin_opcode_op_nip)
-  [Function `op_over`](#0x3_bitcoin_opcode_op_over)
-  [Function `op_pick`](#0x3_bitcoin_opcode_op_pick)
-  [Function `op_roll`](#0x3_bitcoin_opcode_op_roll)
-  [Function `op_rot`](#0x3_bitcoin_opcode_op_rot)
-  [Function `op_swap`](#0x3_bitcoin_opcode_op_swap)
-  [Function `op_tuck`](#0x3_bitcoin_opcode_op_tuck)
-  [Function `op_cat`](#0x3_bitcoin_opcode_op_cat)
-  [Function `op_substr`](#0x3_bitcoin_opcode_op_substr)
-  [Function `op_left`](#0x3_bitcoin_opcode_op_left)
-  [Function `op_right`](#0x3_bitcoin_opcode_op_right)
-  [Function `op_size`](#0x3_bitcoin_opcode_op_size)
-  [Function `op_invert`](#0x3_bitcoin_opcode_op_invert)
-  [Function `op_and_op`](#0x3_bitcoin_opcode_op_and_op)
-  [Function `op_or_op`](#0x3_bitcoin_opcode_op_or_op)
-  [Function `op_xor`](#0x3_bitcoin_opcode_op_xor)
-  [Function `op_equal`](#0x3_bitcoin_opcode_op_equal)
-  [Function `op_equalverify`](#0x3_bitcoin_opcode_op_equalverify)
-  [Function `op_reserved1`](#0x3_bitcoin_opcode_op_reserved1)
-  [Function `op_reserved2`](#0x3_bitcoin_opcode_op_reserved2)
-  [Function `op_1add`](#0x3_bitcoin_opcode_op_1add)
-  [Function `op_1sub`](#0x3_bitcoin_opcode_op_1sub)
-  [Function `op_2mul`](#0x3_bitcoin_opcode_op_2mul)
-  [Function `op_2div`](#0x3_bitcoin_opcode_op_2div)
-  [Function `op_negate`](#0x3_bitcoin_opcode_op_negate)
-  [Function `op_abs`](#0x3_bitcoin_opcode_op_abs)
-  [Function `op_not`](#0x3_bitcoin_opcode_op_not)
-  [Function `op_0notequal`](#0x3_bitcoin_opcode_op_0notequal)
-  [Function `op_add`](#0x3_bitcoin_opcode_op_add)
-  [Function `op_sub`](#0x3_bitcoin_opcode_op_sub)
-  [Function `op_mul`](#0x3_bitcoin_opcode_op_mul)
-  [Function `op_div`](#0x3_bitcoin_opcode_op_div)
-  [Function `op_mod`](#0x3_bitcoin_opcode_op_mod)
-  [Function `op_lshift`](#0x3_bitcoin_opcode_op_lshift)
-  [Function `op_rshift`](#0x3_bitcoin_opcode_op_rshift)
-  [Function `op_booland`](#0x3_bitcoin_opcode_op_booland)
-  [Function `op_boolor`](#0x3_bitcoin_opcode_op_boolor)
-  [Function `op_numequal`](#0x3_bitcoin_opcode_op_numequal)
-  [Function `op_numequalverify`](#0x3_bitcoin_opcode_op_numequalverify)
-  [Function `op_numnotequal`](#0x3_bitcoin_opcode_op_numnotequal)
-  [Function `op_lessthan`](#0x3_bitcoin_opcode_op_lessthan)
-  [Function `op_greaterthan`](#0x3_bitcoin_opcode_op_greaterthan)
-  [Function `op_lessthanorequal`](#0x3_bitcoin_opcode_op_lessthanorequal)
-  [Function `op_greaterthanorequal`](#0x3_bitcoin_opcode_op_greaterthanorequal)
-  [Function `op_min`](#0x3_bitcoin_opcode_op_min)
-  [Function `op_max`](#0x3_bitcoin_opcode_op_max)
-  [Function `op_within`](#0x3_bitcoin_opcode_op_within)
-  [Function `op_ripemd160`](#0x3_bitcoin_opcode_op_ripemd160)
-  [Function `op_sha1`](#0x3_bitcoin_opcode_op_sha1)
-  [Function `op_sha256`](#0x3_bitcoin_opcode_op_sha256)
-  [Function `op_hash160`](#0x3_bitcoin_opcode_op_hash160)
-  [Function `op_hash256`](#0x3_bitcoin_opcode_op_hash256)
-  [Function `op_codeseparator`](#0x3_bitcoin_opcode_op_codeseparator)
-  [Function `op_checksig`](#0x3_bitcoin_opcode_op_checksig)
-  [Function `op_checksigverify`](#0x3_bitcoin_opcode_op_checksigverify)
-  [Function `op_checkmultisig`](#0x3_bitcoin_opcode_op_checkmultisig)
-  [Function `op_checkmultisigverify`](#0x3_bitcoin_opcode_op_checkmultisigverify)
-  [Function `op_nop1`](#0x3_bitcoin_opcode_op_nop1)
-  [Function `op_cltv`](#0x3_bitcoin_opcode_op_cltv)
-  [Function `op_csv`](#0x3_bitcoin_opcode_op_csv)
-  [Function `op_nop4`](#0x3_bitcoin_opcode_op_nop4)
-  [Function `op_nop5`](#0x3_bitcoin_opcode_op_nop5)
-  [Function `op_nop6`](#0x3_bitcoin_opcode_op_nop6)
-  [Function `op_nop7`](#0x3_bitcoin_opcode_op_nop7)
-  [Function `op_nop8`](#0x3_bitcoin_opcode_op_nop8)
-  [Function `op_nop9`](#0x3_bitcoin_opcode_op_nop9)
-  [Function `op_nop10`](#0x3_bitcoin_opcode_op_nop10)
-  [Function `op_checksigadd`](#0x3_bitcoin_opcode_op_checksigadd)
-  [Function `op_return_187`](#0x3_bitcoin_opcode_op_return_187)
-  [Function `op_return_188`](#0x3_bitcoin_opcode_op_return_188)
-  [Function `op_return_189`](#0x3_bitcoin_opcode_op_return_189)
-  [Function `op_return_190`](#0x3_bitcoin_opcode_op_return_190)
-  [Function `op_return_191`](#0x3_bitcoin_opcode_op_return_191)
-  [Function `op_return_192`](#0x3_bitcoin_opcode_op_return_192)
-  [Function `op_return_193`](#0x3_bitcoin_opcode_op_return_193)
-  [Function `op_return_194`](#0x3_bitcoin_opcode_op_return_194)
-  [Function `op_return_195`](#0x3_bitcoin_opcode_op_return_195)
-  [Function `op_return_196`](#0x3_bitcoin_opcode_op_return_196)
-  [Function `op_return_197`](#0x3_bitcoin_opcode_op_return_197)
-  [Function `op_return_198`](#0x3_bitcoin_opcode_op_return_198)
-  [Function `op_return_199`](#0x3_bitcoin_opcode_op_return_199)
-  [Function `op_return_200`](#0x3_bitcoin_opcode_op_return_200)
-  [Function `op_return_201`](#0x3_bitcoin_opcode_op_return_201)
-  [Function `op_return_202`](#0x3_bitcoin_opcode_op_return_202)
-  [Function `op_return_203`](#0x3_bitcoin_opcode_op_return_203)
-  [Function `op_return_204`](#0x3_bitcoin_opcode_op_return_204)
-  [Function `op_return_205`](#0x3_bitcoin_opcode_op_return_205)
-  [Function `op_return_206`](#0x3_bitcoin_opcode_op_return_206)
-  [Function `op_return_207`](#0x3_bitcoin_opcode_op_return_207)
-  [Function `op_return_208`](#0x3_bitcoin_opcode_op_return_208)
-  [Function `op_return_209`](#0x3_bitcoin_opcode_op_return_209)
-  [Function `op_return_210`](#0x3_bitcoin_opcode_op_return_210)
-  [Function `op_return_211`](#0x3_bitcoin_opcode_op_return_211)
-  [Function `op_return_212`](#0x3_bitcoin_opcode_op_return_212)
-  [Function `op_return_213`](#0x3_bitcoin_opcode_op_return_213)
-  [Function `op_return_214`](#0x3_bitcoin_opcode_op_return_214)
-  [Function `op_return_215`](#0x3_bitcoin_opcode_op_return_215)
-  [Function `op_return_216`](#0x3_bitcoin_opcode_op_return_216)
-  [Function `op_return_217`](#0x3_bitcoin_opcode_op_return_217)
-  [Function `op_return_218`](#0x3_bitcoin_opcode_op_return_218)
-  [Function `op_return_219`](#0x3_bitcoin_opcode_op_return_219)
-  [Function `op_return_220`](#0x3_bitcoin_opcode_op_return_220)
-  [Function `op_return_221`](#0x3_bitcoin_opcode_op_return_221)
-  [Function `op_return_222`](#0x3_bitcoin_opcode_op_return_222)
-  [Function `op_return_223`](#0x3_bitcoin_opcode_op_return_223)
-  [Function `op_return_224`](#0x3_bitcoin_opcode_op_return_224)
-  [Function `op_return_225`](#0x3_bitcoin_opcode_op_return_225)
-  [Function `op_return_226`](#0x3_bitcoin_opcode_op_return_226)
-  [Function `op_return_227`](#0x3_bitcoin_opcode_op_return_227)
-  [Function `op_return_228`](#0x3_bitcoin_opcode_op_return_228)
-  [Function `op_return_229`](#0x3_bitcoin_opcode_op_return_229)
-  [Function `op_return_230`](#0x3_bitcoin_opcode_op_return_230)
-  [Function `op_return_231`](#0x3_bitcoin_opcode_op_return_231)
-  [Function `op_return_232`](#0x3_bitcoin_opcode_op_return_232)
-  [Function `op_return_233`](#0x3_bitcoin_opcode_op_return_233)
-  [Function `op_return_234`](#0x3_bitcoin_opcode_op_return_234)
-  [Function `op_return_235`](#0x3_bitcoin_opcode_op_return_235)
-  [Function `op_return_236`](#0x3_bitcoin_opcode_op_return_236)
-  [Function `op_return_237`](#0x3_bitcoin_opcode_op_return_237)
-  [Function `op_return_238`](#0x3_bitcoin_opcode_op_return_238)
-  [Function `op_return_239`](#0x3_bitcoin_opcode_op_return_239)
-  [Function `op_return_240`](#0x3_bitcoin_opcode_op_return_240)
-  [Function `op_return_241`](#0x3_bitcoin_opcode_op_return_241)
-  [Function `op_return_242`](#0x3_bitcoin_opcode_op_return_242)
-  [Function `op_return_243`](#0x3_bitcoin_opcode_op_return_243)
-  [Function `op_return_244`](#0x3_bitcoin_opcode_op_return_244)
-  [Function `op_return_245`](#0x3_bitcoin_opcode_op_return_245)
-  [Function `op_return_246`](#0x3_bitcoin_opcode_op_return_246)
-  [Function `op_return_247`](#0x3_bitcoin_opcode_op_return_247)
-  [Function `op_return_248`](#0x3_bitcoin_opcode_op_return_248)
-  [Function `op_return_249`](#0x3_bitcoin_opcode_op_return_249)
-  [Function `op_return_250`](#0x3_bitcoin_opcode_op_return_250)
-  [Function `op_return_251`](#0x3_bitcoin_opcode_op_return_251)
-  [Function `op_return_252`](#0x3_bitcoin_opcode_op_return_252)
-  [Function `op_return_253`](#0x3_bitcoin_opcode_op_return_253)
-  [Function `op_return_254`](#0x3_bitcoin_opcode_op_return_254)
-  [Function `op_invalidopcode`](#0x3_bitcoin_opcode_op_invalidopcode)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_bitcoin_opcode_OP_0NOTEQUAL"></a>

Map 0 to 0 and everything else to 1, in place.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_0NOTEQUAL">OP_0NOTEQUAL</a>: u8 = 146;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_1ADD"></a>

Increment the top stack element in place.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_1ADD">OP_1ADD</a>: u8 = 139;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_1SUB"></a>

Decrement the top stack element in place.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_1SUB">OP_1SUB</a>: u8 = 140;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_2DIV"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_2DIV">OP_2DIV</a>: u8 = 142;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_2DROP"></a>

Drops the top two stack items.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_2DROP">OP_2DROP</a>: u8 = 109;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_2DUP"></a>

Duplicates the top two stack items as AB -> ABAB.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_2DUP">OP_2DUP</a>: u8 = 110;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_2MUL"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_2MUL">OP_2MUL</a>: u8 = 141;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_2OVER"></a>

Copies the two stack items of items two spaces back to the front, as xxAB -> ABxxAB.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_2OVER">OP_2OVER</a>: u8 = 112;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_2ROT"></a>

Moves the two stack items four spaces back to the front, as xxxxAB -> ABxxxx.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_2ROT">OP_2ROT</a>: u8 = 113;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_2SWAP"></a>

Swaps the top two pairs, as ABCD -> CDAB.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_2SWAP">OP_2SWAP</a>: u8 = 114;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_3DUP"></a>

Duplicates the two three stack items as ABC -> ABCABC.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_3DUP">OP_3DUP</a>: u8 = 111;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_ABS"></a>

Absolute value the top stack item in place.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_ABS">OP_ABS</a>: u8 = 144;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_ADD"></a>

Pop two stack items and push their sum.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_ADD">OP_ADD</a>: u8 = 147;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_AND"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_AND">OP_AND</a>: u8 = 132;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_BOOLAND"></a>

Pop the top two stack items and push 1 if both are nonzero, else push 0.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_BOOLAND">OP_BOOLAND</a>: u8 = 154;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_BOOLOR"></a>

Pop the top two stack items and push 1 if either is nonzero, else push 0.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_BOOLOR">OP_BOOLOR</a>: u8 = 155;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_CAT"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_CAT">OP_CAT</a>: u8 = 126;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_CHECKMULTISIG"></a>

Pop N, N pubkeys, M, M signatures, a dummy (due to bug in reference code), and verify that all M signatures are valid. Push 1 for 'all valid', 0 otherwise.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_CHECKMULTISIG">OP_CHECKMULTISIG</a>: u8 = 174;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_CHECKMULTISIGVERIFY"></a>

Like the above but return success/failure.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_CHECKMULTISIGVERIFY">OP_CHECKMULTISIGVERIFY</a>: u8 = 175;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_CHECKSIG"></a>

<https://en.bitcoin.it/wiki/OP_CHECKSIG> pushing 1/0 for success/failure.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_CHECKSIG">OP_CHECKSIG</a>: u8 = 172;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_CHECKSIGADD"></a>

OP_CHECKSIGADD post tapscript.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_CHECKSIGADD">OP_CHECKSIGADD</a>: u8 = 186;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_CHECKSIGVERIFY"></a>

<https://en.bitcoin.it/wiki/OP_CHECKSIG> returning success/failure.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_CHECKSIGVERIFY">OP_CHECKSIGVERIFY</a>: u8 = 173;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_CLTV"></a>

<https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki>


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_CLTV">OP_CLTV</a>: u8 = 177;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_CODESEPARATOR"></a>

Ignore this and everything preceding when deciding what to sign when signature-checking.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_CODESEPARATOR">OP_CODESEPARATOR</a>: u8 = 171;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_CSV"></a>

<https://github.com/bitcoin/bips/blob/master/bip-0112.mediawiki>


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_CSV">OP_CSV</a>: u8 = 178;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_DEPTH"></a>

Push the current number of stack items onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_DEPTH">OP_DEPTH</a>: u8 = 116;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_DIV"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_DIV">OP_DIV</a>: u8 = 150;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_DROP"></a>

Drops the top stack item.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_DROP">OP_DROP</a>: u8 = 117;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_DUP"></a>

Duplicates the top stack item.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_DUP">OP_DUP</a>: u8 = 118;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_ELSE"></a>

Execute statements if those after the previous OP_IF were not, and vice-versa.
If there is no previous OP_IF, this acts as a RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_ELSE">OP_ELSE</a>: u8 = 103;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_ENDIF"></a>

Pop and execute the next statements if a zero element was popped.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_ENDIF">OP_ENDIF</a>: u8 = 104;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_EQUAL"></a>

Pushes 1 if the inputs are exactly equal, 0 otherwise.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_EQUAL">OP_EQUAL</a>: u8 = 135;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_EQUALVERIFY"></a>

Returns success if the inputs are exactly equal, failure otherwise.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_EQUALVERIFY">OP_EQUALVERIFY</a>: u8 = 136;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_FROMALTSTACK"></a>

Pop one element from the alt stack onto the main stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_FROMALTSTACK">OP_FROMALTSTACK</a>: u8 = 108;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_GREATERTHAN"></a>

Pop the top two items; push 1 if the second is greater than the top, 0 otherwise.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_GREATERTHAN">OP_GREATERTHAN</a>: u8 = 160;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_GREATERTHANOREQUAL"></a>

Pop the top two items; push 1 if the second is >= the top, 0 otherwise.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_GREATERTHANOREQUAL">OP_GREATERTHANOREQUAL</a>: u8 = 162;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_HASH160"></a>

Pop the top stack item and push its RIPEMD(SHA256) hash.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_HASH160">OP_HASH160</a>: u8 = 169;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_HASH256"></a>

Pop the top stack item and push its SHA256(SHA256) hash.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_HASH256">OP_HASH256</a>: u8 = 170;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_IF"></a>

Pop and execute the next statements if a nonzero element was popped.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_IF">OP_IF</a>: u8 = 99;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_IFDUP"></a>

Duplicate the top stack element unless it is zero.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_IFDUP">OP_IFDUP</a>: u8 = 115;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_INVALIDOPCODE"></a>

Invalid opcode.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_INVALIDOPCODE">OP_INVALIDOPCODE</a>: u8 = 255;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_INVERT"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_INVERT">OP_INVERT</a>: u8 = 131;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_LEFT"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_LEFT">OP_LEFT</a>: u8 = 128;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_LESSTHAN"></a>

Pop the top two items; push 1 if the second is less than the top, 0 otherwise.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_LESSTHAN">OP_LESSTHAN</a>: u8 = 159;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_LESSTHANOREQUAL"></a>

Pop the top two items; push 1 if the second is <= the top, 0 otherwise.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_LESSTHANOREQUAL">OP_LESSTHANOREQUAL</a>: u8 = 161;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_LSHIFT"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_LSHIFT">OP_LSHIFT</a>: u8 = 152;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_MAX"></a>

Pop the top two items; push the larger.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_MAX">OP_MAX</a>: u8 = 164;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_MIN"></a>

Pop the top two items; push the smaller.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_MIN">OP_MIN</a>: u8 = 163;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_MOD"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_MOD">OP_MOD</a>: u8 = 151;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_MUL"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_MUL">OP_MUL</a>: u8 = 149;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NEGATE"></a>

Multiply the top stack item by -1 in place.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NEGATE">OP_NEGATE</a>: u8 = 143;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NIP"></a>

Drops the second-to-top stack item.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NIP">OP_NIP</a>: u8 = 119;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOP"></a>

Does nothing.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOP">OP_NOP</a>: u8 = 97;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOP1"></a>

Does nothing.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOP1">OP_NOP1</a>: u8 = 176;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOP10"></a>

Does nothing.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOP10">OP_NOP10</a>: u8 = 185;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOP4"></a>

Does nothing.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOP4">OP_NOP4</a>: u8 = 179;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOP5"></a>

Does nothing.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOP5">OP_NOP5</a>: u8 = 180;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOP6"></a>

Does nothing.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOP6">OP_NOP6</a>: u8 = 181;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOP7"></a>

Does nothing.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOP7">OP_NOP7</a>: u8 = 182;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOP8"></a>

Does nothing.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOP8">OP_NOP8</a>: u8 = 183;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOP9"></a>

Does nothing.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOP9">OP_NOP9</a>: u8 = 184;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOT"></a>

Map 0 to 1 and everything else to 0, in place.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOT">OP_NOT</a>: u8 = 145;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NOTIF"></a>

Pop and execute the next statements if a zero element was popped.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NOTIF">OP_NOTIF</a>: u8 = 100;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NUMEQUAL"></a>

Pop the top two stack items and push 1 if both are numerically equal, else push 0.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NUMEQUAL">OP_NUMEQUAL</a>: u8 = 156;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NUMEQUALVERIFY"></a>

Pop the top two stack items and return success if both are numerically equal, else return failure.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NUMEQUALVERIFY">OP_NUMEQUALVERIFY</a>: u8 = 157;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_NUMNOTEQUAL"></a>

Pop the top two stack items and push 0 if both are numerically equal, else push 1.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_NUMNOTEQUAL">OP_NUMNOTEQUAL</a>: u8 = 158;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_OR"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_OR">OP_OR</a>: u8 = 133;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_OVER"></a>

Copies the second-to-top stack item, as xA -> AxA.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_OVER">OP_OVER</a>: u8 = 120;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PICK"></a>

Pop the top stack element as N. Copy the Nth stack element to the top.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PICK">OP_PICK</a>: u8 = 121;
</code></pre>



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



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_17"></a>

Push the next 17 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_17">OP_PUSHBYTES_17</a>: u8 = 17;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_18"></a>

Push the next 18 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_18">OP_PUSHBYTES_18</a>: u8 = 18;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_19"></a>

Push the next 19 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_19">OP_PUSHBYTES_19</a>: u8 = 19;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_2"></a>

Push the next 2 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_2">OP_PUSHBYTES_2</a>: u8 = 2;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_20"></a>

Push the next 20 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_20">OP_PUSHBYTES_20</a>: u8 = 20;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_21"></a>

Push the next 21 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_21">OP_PUSHBYTES_21</a>: u8 = 21;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_22"></a>

Push the next 22 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_22">OP_PUSHBYTES_22</a>: u8 = 22;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_23"></a>

Push the next 23 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_23">OP_PUSHBYTES_23</a>: u8 = 23;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_24"></a>

Push the next 24 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_24">OP_PUSHBYTES_24</a>: u8 = 24;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_25"></a>

Push the next 25 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_25">OP_PUSHBYTES_25</a>: u8 = 25;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_26"></a>

Push the next 26 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_26">OP_PUSHBYTES_26</a>: u8 = 26;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_27"></a>

Push the next 27 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_27">OP_PUSHBYTES_27</a>: u8 = 27;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_28"></a>

Push the next 28 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_28">OP_PUSHBYTES_28</a>: u8 = 28;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_29"></a>

Push the next 29 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_29">OP_PUSHBYTES_29</a>: u8 = 29;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_3"></a>

Push the next 3 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_3">OP_PUSHBYTES_3</a>: u8 = 3;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_30"></a>

Push the next 30 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_30">OP_PUSHBYTES_30</a>: u8 = 30;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_31"></a>

Push the next 31 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_31">OP_PUSHBYTES_31</a>: u8 = 31;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_32"></a>

Push the next 32 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_32">OP_PUSHBYTES_32</a>: u8 = 32;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_33"></a>

Push the next 33 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_33">OP_PUSHBYTES_33</a>: u8 = 33;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_34"></a>

Push the next 34 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_34">OP_PUSHBYTES_34</a>: u8 = 34;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_35"></a>

Push the next 35 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_35">OP_PUSHBYTES_35</a>: u8 = 35;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_36"></a>

Push the next 36 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_36">OP_PUSHBYTES_36</a>: u8 = 36;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_37"></a>

Push the next 37 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_37">OP_PUSHBYTES_37</a>: u8 = 37;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_38"></a>

Push the next 38 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_38">OP_PUSHBYTES_38</a>: u8 = 38;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_39"></a>

Push the next 39 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_39">OP_PUSHBYTES_39</a>: u8 = 39;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_4"></a>

Push the next 4 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_4">OP_PUSHBYTES_4</a>: u8 = 4;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_40"></a>

Push the next 40 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_40">OP_PUSHBYTES_40</a>: u8 = 40;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_41"></a>

Push the next 41 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_41">OP_PUSHBYTES_41</a>: u8 = 41;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_42"></a>

Push the next 42 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_42">OP_PUSHBYTES_42</a>: u8 = 42;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_43"></a>

Push the next 43 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_43">OP_PUSHBYTES_43</a>: u8 = 43;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_44"></a>

Push the next 44 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_44">OP_PUSHBYTES_44</a>: u8 = 44;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_45"></a>

Push the next 45 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_45">OP_PUSHBYTES_45</a>: u8 = 45;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_46"></a>

Push the next 46 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_46">OP_PUSHBYTES_46</a>: u8 = 46;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_47"></a>

Push the next 47 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_47">OP_PUSHBYTES_47</a>: u8 = 47;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_48"></a>

Push the next 48 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_48">OP_PUSHBYTES_48</a>: u8 = 48;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_49"></a>

Push the next 49 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_49">OP_PUSHBYTES_49</a>: u8 = 49;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_5"></a>

Push the next 5 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_5">OP_PUSHBYTES_5</a>: u8 = 5;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_50"></a>

Push the next 50 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_50">OP_PUSHBYTES_50</a>: u8 = 50;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_51"></a>

Push the next 51 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_51">OP_PUSHBYTES_51</a>: u8 = 51;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_52"></a>

Push the next 52 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_52">OP_PUSHBYTES_52</a>: u8 = 52;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_53"></a>

Push the next 53 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_53">OP_PUSHBYTES_53</a>: u8 = 53;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_54"></a>

Push the next 54 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_54">OP_PUSHBYTES_54</a>: u8 = 54;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_55"></a>

Push the next 55 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_55">OP_PUSHBYTES_55</a>: u8 = 55;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_56"></a>

Push the next 56 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_56">OP_PUSHBYTES_56</a>: u8 = 56;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_57"></a>

Push the next 57 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_57">OP_PUSHBYTES_57</a>: u8 = 57;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_58"></a>

Push the next 58 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_58">OP_PUSHBYTES_58</a>: u8 = 58;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_59"></a>

Push the next 59 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_59">OP_PUSHBYTES_59</a>: u8 = 59;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_6"></a>

Push the next 6 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_6">OP_PUSHBYTES_6</a>: u8 = 6;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_60"></a>

Push the next 60 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_60">OP_PUSHBYTES_60</a>: u8 = 60;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_61"></a>

Push the next 61 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_61">OP_PUSHBYTES_61</a>: u8 = 61;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_62"></a>

Push the next 62 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_62">OP_PUSHBYTES_62</a>: u8 = 62;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_63"></a>

Push the next 63 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_63">OP_PUSHBYTES_63</a>: u8 = 63;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_64"></a>

Push the next 64 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_64">OP_PUSHBYTES_64</a>: u8 = 64;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_65"></a>

Push the next 65 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_65">OP_PUSHBYTES_65</a>: u8 = 65;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_66"></a>

Push the next 66 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_66">OP_PUSHBYTES_66</a>: u8 = 66;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_67"></a>

Push the next 67 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_67">OP_PUSHBYTES_67</a>: u8 = 67;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_68"></a>

Push the next 68 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_68">OP_PUSHBYTES_68</a>: u8 = 68;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_69"></a>

Push the next 69 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_69">OP_PUSHBYTES_69</a>: u8 = 69;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_7"></a>

Push the next 7 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_7">OP_PUSHBYTES_7</a>: u8 = 7;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_70"></a>

Push the next 70 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_70">OP_PUSHBYTES_70</a>: u8 = 70;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_71"></a>

Push the next 71 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_71">OP_PUSHBYTES_71</a>: u8 = 71;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_72"></a>

Push the next 72 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_72">OP_PUSHBYTES_72</a>: u8 = 72;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_73"></a>

Push the next 73 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_73">OP_PUSHBYTES_73</a>: u8 = 73;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_74"></a>

Push the next 74 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_74">OP_PUSHBYTES_74</a>: u8 = 74;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_75"></a>

Push the next 75 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_75">OP_PUSHBYTES_75</a>: u8 = 75;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_8"></a>

Push the next 8 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_8">OP_PUSHBYTES_8</a>: u8 = 8;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHBYTES_9"></a>

Push the next 9 bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHBYTES_9">OP_PUSHBYTES_9</a>: u8 = 9;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHDATA1"></a>

Read the next byte as N; push the next N bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHDATA1">OP_PUSHDATA1</a>: u8 = 76;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHDATA2"></a>

Read the next 2 bytes as N; push the next N bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHDATA2">OP_PUSHDATA2</a>: u8 = 77;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHDATA4"></a>

Read the next 4 bytes as N; push the next N bytes as an array onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHDATA4">OP_PUSHDATA4</a>: u8 = 78;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_1"></a>

Push the array <code>0x01</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_1">OP_PUSHNUM_1</a>: u8 = 81;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_10"></a>

Push the array <code>0x0a</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_10">OP_PUSHNUM_10</a>: u8 = 90;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_11"></a>

Push the array <code>0x0b</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_11">OP_PUSHNUM_11</a>: u8 = 91;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_12"></a>

Push the array <code>0x0c</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_12">OP_PUSHNUM_12</a>: u8 = 92;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_13"></a>

Push the array <code>0x0d</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_13">OP_PUSHNUM_13</a>: u8 = 93;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_14"></a>

Push the array <code>0x0e</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_14">OP_PUSHNUM_14</a>: u8 = 94;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_15"></a>

Push the array <code>0x0f</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_15">OP_PUSHNUM_15</a>: u8 = 95;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_16"></a>

Push the array <code>0x10</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_16">OP_PUSHNUM_16</a>: u8 = 96;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_2"></a>

Push the array <code>0x02</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_2">OP_PUSHNUM_2</a>: u8 = 82;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_3"></a>

Push the array <code>0x03</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_3">OP_PUSHNUM_3</a>: u8 = 83;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_4"></a>

Push the array <code>0x04</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_4">OP_PUSHNUM_4</a>: u8 = 84;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_5"></a>

Push the array <code>0x05</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_5">OP_PUSHNUM_5</a>: u8 = 85;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_6"></a>

Push the array <code>0x06</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_6">OP_PUSHNUM_6</a>: u8 = 86;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_7"></a>

Push the array <code>0x07</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_7">OP_PUSHNUM_7</a>: u8 = 87;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_8"></a>

Push the array <code>0x08</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_8">OP_PUSHNUM_8</a>: u8 = 88;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_9"></a>

Push the array <code>0x09</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_9">OP_PUSHNUM_9</a>: u8 = 89;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_PUSHNUM_NEG1"></a>

Push the array <code>0x81</code> onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_PUSHNUM_NEG1">OP_PUSHNUM_NEG1</a>: u8 = 79;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RESERVED"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RESERVED">OP_RESERVED</a>: u8 = 80;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RESERVED1"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RESERVED1">OP_RESERVED1</a>: u8 = 137;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RESERVED2"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RESERVED2">OP_RESERVED2</a>: u8 = 138;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN"></a>

Fail the script immediately. (Must be executed.).


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN">OP_RETURN</a>: u8 = 106;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_187"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_187">OP_RETURN_187</a>: u8 = 187;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_188"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_188">OP_RETURN_188</a>: u8 = 188;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_189"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_189">OP_RETURN_189</a>: u8 = 189;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_190"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_190">OP_RETURN_190</a>: u8 = 190;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_191"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_191">OP_RETURN_191</a>: u8 = 191;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_192"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_192">OP_RETURN_192</a>: u8 = 192;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_193"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_193">OP_RETURN_193</a>: u8 = 193;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_194"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_194">OP_RETURN_194</a>: u8 = 194;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_195"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_195">OP_RETURN_195</a>: u8 = 195;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_196"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_196">OP_RETURN_196</a>: u8 = 196;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_197"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_197">OP_RETURN_197</a>: u8 = 197;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_198"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_198">OP_RETURN_198</a>: u8 = 198;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_199"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_199">OP_RETURN_199</a>: u8 = 199;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_200"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_200">OP_RETURN_200</a>: u8 = 200;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_201"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_201">OP_RETURN_201</a>: u8 = 201;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_202"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_202">OP_RETURN_202</a>: u8 = 202;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_203"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_203">OP_RETURN_203</a>: u8 = 203;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_204"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_204">OP_RETURN_204</a>: u8 = 204;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_205"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_205">OP_RETURN_205</a>: u8 = 205;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_206"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_206">OP_RETURN_206</a>: u8 = 206;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_207"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_207">OP_RETURN_207</a>: u8 = 207;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_208"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_208">OP_RETURN_208</a>: u8 = 208;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_209"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_209">OP_RETURN_209</a>: u8 = 209;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_210"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_210">OP_RETURN_210</a>: u8 = 210;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_211"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_211">OP_RETURN_211</a>: u8 = 211;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_212"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_212">OP_RETURN_212</a>: u8 = 212;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_213"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_213">OP_RETURN_213</a>: u8 = 213;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_214"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_214">OP_RETURN_214</a>: u8 = 214;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_215"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_215">OP_RETURN_215</a>: u8 = 215;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_216"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_216">OP_RETURN_216</a>: u8 = 216;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_217"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_217">OP_RETURN_217</a>: u8 = 217;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_218"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_218">OP_RETURN_218</a>: u8 = 218;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_219"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_219">OP_RETURN_219</a>: u8 = 219;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_220"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_220">OP_RETURN_220</a>: u8 = 220;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_221"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_221">OP_RETURN_221</a>: u8 = 221;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_222"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_222">OP_RETURN_222</a>: u8 = 222;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_223"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_223">OP_RETURN_223</a>: u8 = 223;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_224"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_224">OP_RETURN_224</a>: u8 = 224;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_225"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_225">OP_RETURN_225</a>: u8 = 225;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_226"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_226">OP_RETURN_226</a>: u8 = 226;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_227"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_227">OP_RETURN_227</a>: u8 = 227;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_228"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_228">OP_RETURN_228</a>: u8 = 228;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_229"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_229">OP_RETURN_229</a>: u8 = 229;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_230"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_230">OP_RETURN_230</a>: u8 = 230;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_231"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_231">OP_RETURN_231</a>: u8 = 231;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_232"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_232">OP_RETURN_232</a>: u8 = 232;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_233"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_233">OP_RETURN_233</a>: u8 = 233;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_234"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_234">OP_RETURN_234</a>: u8 = 234;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_235"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_235">OP_RETURN_235</a>: u8 = 235;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_236"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_236">OP_RETURN_236</a>: u8 = 236;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_237"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_237">OP_RETURN_237</a>: u8 = 237;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_238"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_238">OP_RETURN_238</a>: u8 = 238;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_239"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_239">OP_RETURN_239</a>: u8 = 239;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_240"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_240">OP_RETURN_240</a>: u8 = 240;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_241"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_241">OP_RETURN_241</a>: u8 = 241;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_242"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_242">OP_RETURN_242</a>: u8 = 242;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_243"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_243">OP_RETURN_243</a>: u8 = 243;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_244"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_244">OP_RETURN_244</a>: u8 = 244;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_245"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_245">OP_RETURN_245</a>: u8 = 245;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_246"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_246">OP_RETURN_246</a>: u8 = 246;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_247"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_247">OP_RETURN_247</a>: u8 = 247;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_248"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_248">OP_RETURN_248</a>: u8 = 248;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_249"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_249">OP_RETURN_249</a>: u8 = 249;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_250"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_250">OP_RETURN_250</a>: u8 = 250;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_251"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_251">OP_RETURN_251</a>: u8 = 251;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_252"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_252">OP_RETURN_252</a>: u8 = 252;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_253"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_253">OP_RETURN_253</a>: u8 = 253;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RETURN_254"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RETURN_254">OP_RETURN_254</a>: u8 = 254;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RIGHT"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RIGHT">OP_RIGHT</a>: u8 = 129;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RIPEMD160"></a>

Pop the top stack item and push its RIPEMD160 hash.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RIPEMD160">OP_RIPEMD160</a>: u8 = 166;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_ROLL"></a>

Pop the top stack element as N. Move the Nth stack element to the top.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_ROLL">OP_ROLL</a>: u8 = 122;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_ROT"></a>

Rotate the top three stack items, as [top next1 next2] -> [next2 top next1].


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_ROT">OP_ROT</a>: u8 = 123;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_RSHIFT"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_RSHIFT">OP_RSHIFT</a>: u8 = 153;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_SHA1"></a>

Pop the top stack item and push its SHA1 hash.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_SHA1">OP_SHA1</a>: u8 = 167;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_SHA256"></a>

Pop the top stack item and push its SHA256 hash.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_SHA256">OP_SHA256</a>: u8 = 168;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_SIZE"></a>

Pushes the length of the top stack item onto the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_SIZE">OP_SIZE</a>: u8 = 130;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_SUB"></a>

Pop two stack items and push the second minus the top.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_SUB">OP_SUB</a>: u8 = 148;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_SUBSTR"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_SUBSTR">OP_SUBSTR</a>: u8 = 127;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_SWAP"></a>

Swap the top two stack items.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_SWAP">OP_SWAP</a>: u8 = 124;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_TOALTSTACK"></a>

Pop one element from the main stack onto the alt stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_TOALTSTACK">OP_TOALTSTACK</a>: u8 = 107;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_TUCK"></a>

Copy the top stack item to before the second item, as [top next] -> [top next top].


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_TUCK">OP_TUCK</a>: u8 = 125;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_VER"></a>

Synonym for OP_RETURN.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_VER">OP_VER</a>: u8 = 98;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_VERIF"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_VERIF">OP_VERIF</a>: u8 = 101;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_VERIFY"></a>

If the top value is zero or the stack is empty, fail; otherwise, pop the stack.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_VERIFY">OP_VERIFY</a>: u8 = 105;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_VERNOTIF"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_VERNOTIF">OP_VERNOTIF</a>: u8 = 102;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_WITHIN"></a>

Pop the top three items; if the top is >= the second and < the third, push 1, otherwise push 0.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_WITHIN">OP_WITHIN</a>: u8 = 165;
</code></pre>



<a name="0x3_bitcoin_opcode_OP_XOR"></a>

Fail the script unconditionally, does not even need to be executed.


<pre><code><b>const</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_OP_XOR">OP_XOR</a>: u8 = 134;
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_0"></a>

## Function `op_pushbytes_0`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_0">op_pushbytes_0</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_1"></a>

## Function `op_pushbytes_1`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_1">op_pushbytes_1</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_2"></a>

## Function `op_pushbytes_2`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_2">op_pushbytes_2</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_3"></a>

## Function `op_pushbytes_3`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_3">op_pushbytes_3</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_4"></a>

## Function `op_pushbytes_4`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_4">op_pushbytes_4</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_5"></a>

## Function `op_pushbytes_5`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_5">op_pushbytes_5</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_6"></a>

## Function `op_pushbytes_6`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_6">op_pushbytes_6</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_7"></a>

## Function `op_pushbytes_7`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_7">op_pushbytes_7</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_8"></a>

## Function `op_pushbytes_8`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_8">op_pushbytes_8</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_9"></a>

## Function `op_pushbytes_9`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_9">op_pushbytes_9</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_10"></a>

## Function `op_pushbytes_10`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_10">op_pushbytes_10</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_11"></a>

## Function `op_pushbytes_11`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_11">op_pushbytes_11</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_12"></a>

## Function `op_pushbytes_12`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_12">op_pushbytes_12</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_13"></a>

## Function `op_pushbytes_13`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_13">op_pushbytes_13</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_14"></a>

## Function `op_pushbytes_14`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_14">op_pushbytes_14</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_15"></a>

## Function `op_pushbytes_15`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_15">op_pushbytes_15</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_16"></a>

## Function `op_pushbytes_16`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_16">op_pushbytes_16</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_17"></a>

## Function `op_pushbytes_17`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_17">op_pushbytes_17</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_18"></a>

## Function `op_pushbytes_18`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_18">op_pushbytes_18</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_19"></a>

## Function `op_pushbytes_19`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_19">op_pushbytes_19</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_20"></a>

## Function `op_pushbytes_20`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_20">op_pushbytes_20</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_21"></a>

## Function `op_pushbytes_21`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_21">op_pushbytes_21</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_22"></a>

## Function `op_pushbytes_22`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_22">op_pushbytes_22</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_23"></a>

## Function `op_pushbytes_23`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_23">op_pushbytes_23</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_24"></a>

## Function `op_pushbytes_24`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_24">op_pushbytes_24</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_25"></a>

## Function `op_pushbytes_25`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_25">op_pushbytes_25</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_26"></a>

## Function `op_pushbytes_26`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_26">op_pushbytes_26</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_27"></a>

## Function `op_pushbytes_27`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_27">op_pushbytes_27</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_28"></a>

## Function `op_pushbytes_28`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_28">op_pushbytes_28</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_29"></a>

## Function `op_pushbytes_29`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_29">op_pushbytes_29</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_30"></a>

## Function `op_pushbytes_30`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_30">op_pushbytes_30</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_31"></a>

## Function `op_pushbytes_31`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_31">op_pushbytes_31</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_32"></a>

## Function `op_pushbytes_32`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_32">op_pushbytes_32</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_33"></a>

## Function `op_pushbytes_33`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_33">op_pushbytes_33</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_34"></a>

## Function `op_pushbytes_34`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_34">op_pushbytes_34</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_35"></a>

## Function `op_pushbytes_35`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_35">op_pushbytes_35</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_36"></a>

## Function `op_pushbytes_36`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_36">op_pushbytes_36</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_37"></a>

## Function `op_pushbytes_37`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_37">op_pushbytes_37</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_38"></a>

## Function `op_pushbytes_38`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_38">op_pushbytes_38</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_39"></a>

## Function `op_pushbytes_39`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_39">op_pushbytes_39</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_40"></a>

## Function `op_pushbytes_40`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_40">op_pushbytes_40</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_41"></a>

## Function `op_pushbytes_41`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_41">op_pushbytes_41</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_42"></a>

## Function `op_pushbytes_42`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_42">op_pushbytes_42</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_43"></a>

## Function `op_pushbytes_43`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_43">op_pushbytes_43</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_44"></a>

## Function `op_pushbytes_44`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_44">op_pushbytes_44</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_45"></a>

## Function `op_pushbytes_45`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_45">op_pushbytes_45</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_46"></a>

## Function `op_pushbytes_46`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_46">op_pushbytes_46</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_47"></a>

## Function `op_pushbytes_47`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_47">op_pushbytes_47</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_48"></a>

## Function `op_pushbytes_48`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_48">op_pushbytes_48</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_49"></a>

## Function `op_pushbytes_49`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_49">op_pushbytes_49</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_50"></a>

## Function `op_pushbytes_50`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_50">op_pushbytes_50</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_51"></a>

## Function `op_pushbytes_51`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_51">op_pushbytes_51</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_52"></a>

## Function `op_pushbytes_52`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_52">op_pushbytes_52</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_53"></a>

## Function `op_pushbytes_53`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_53">op_pushbytes_53</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_54"></a>

## Function `op_pushbytes_54`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_54">op_pushbytes_54</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_55"></a>

## Function `op_pushbytes_55`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_55">op_pushbytes_55</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_56"></a>

## Function `op_pushbytes_56`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_56">op_pushbytes_56</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_57"></a>

## Function `op_pushbytes_57`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_57">op_pushbytes_57</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_58"></a>

## Function `op_pushbytes_58`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_58">op_pushbytes_58</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_59"></a>

## Function `op_pushbytes_59`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_59">op_pushbytes_59</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_60"></a>

## Function `op_pushbytes_60`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_60">op_pushbytes_60</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_61"></a>

## Function `op_pushbytes_61`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_61">op_pushbytes_61</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_62"></a>

## Function `op_pushbytes_62`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_62">op_pushbytes_62</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_63"></a>

## Function `op_pushbytes_63`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_63">op_pushbytes_63</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_64"></a>

## Function `op_pushbytes_64`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_64">op_pushbytes_64</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_65"></a>

## Function `op_pushbytes_65`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_65">op_pushbytes_65</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_66"></a>

## Function `op_pushbytes_66`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_66">op_pushbytes_66</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_67"></a>

## Function `op_pushbytes_67`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_67">op_pushbytes_67</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_68"></a>

## Function `op_pushbytes_68`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_68">op_pushbytes_68</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_69"></a>

## Function `op_pushbytes_69`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_69">op_pushbytes_69</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_70"></a>

## Function `op_pushbytes_70`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_70">op_pushbytes_70</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_71"></a>

## Function `op_pushbytes_71`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_71">op_pushbytes_71</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_72"></a>

## Function `op_pushbytes_72`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_72">op_pushbytes_72</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_73"></a>

## Function `op_pushbytes_73`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_73">op_pushbytes_73</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_74"></a>

## Function `op_pushbytes_74`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_74">op_pushbytes_74</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushbytes_75"></a>

## Function `op_pushbytes_75`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushbytes_75">op_pushbytes_75</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushdata1"></a>

## Function `op_pushdata1`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushdata1">op_pushdata1</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushdata2"></a>

## Function `op_pushdata2`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushdata2">op_pushdata2</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushdata4"></a>

## Function `op_pushdata4`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushdata4">op_pushdata4</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_neg1"></a>

## Function `op_pushnum_neg1`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_neg1">op_pushnum_neg1</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_reserved"></a>

## Function `op_reserved`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_reserved">op_reserved</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_1"></a>

## Function `op_pushnum_1`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_1">op_pushnum_1</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_2"></a>

## Function `op_pushnum_2`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_2">op_pushnum_2</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_3"></a>

## Function `op_pushnum_3`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_3">op_pushnum_3</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_4"></a>

## Function `op_pushnum_4`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_4">op_pushnum_4</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_5"></a>

## Function `op_pushnum_5`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_5">op_pushnum_5</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_6"></a>

## Function `op_pushnum_6`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_6">op_pushnum_6</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_7"></a>

## Function `op_pushnum_7`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_7">op_pushnum_7</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_8"></a>

## Function `op_pushnum_8`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_8">op_pushnum_8</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_9"></a>

## Function `op_pushnum_9`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_9">op_pushnum_9</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_10"></a>

## Function `op_pushnum_10`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_10">op_pushnum_10</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_11"></a>

## Function `op_pushnum_11`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_11">op_pushnum_11</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_12"></a>

## Function `op_pushnum_12`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_12">op_pushnum_12</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_13"></a>

## Function `op_pushnum_13`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_13">op_pushnum_13</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_14"></a>

## Function `op_pushnum_14`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_14">op_pushnum_14</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_15"></a>

## Function `op_pushnum_15`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_15">op_pushnum_15</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pushnum_16"></a>

## Function `op_pushnum_16`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pushnum_16">op_pushnum_16</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nop"></a>

## Function `op_nop`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nop">op_nop</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_ver"></a>

## Function `op_ver`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_ver">op_ver</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_if_op"></a>

## Function `op_if_op`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_if_op">op_if_op</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_notif"></a>

## Function `op_notif`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_notif">op_notif</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_verif"></a>

## Function `op_verif`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_verif">op_verif</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_vernotif"></a>

## Function `op_vernotif`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_vernotif">op_vernotif</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_else_op"></a>

## Function `op_else_op`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_else_op">op_else_op</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_endif"></a>

## Function `op_endif`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_endif">op_endif</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_verify"></a>

## Function `op_verify`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_verify">op_verify</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return"></a>

## Function `op_return`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return">op_return</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_toaltstack"></a>

## Function `op_toaltstack`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_toaltstack">op_toaltstack</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_fromaltstack"></a>

## Function `op_fromaltstack`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_fromaltstack">op_fromaltstack</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_2drop"></a>

## Function `op_2drop`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_2drop">op_2drop</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_2dup"></a>

## Function `op_2dup`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_2dup">op_2dup</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_3dup"></a>

## Function `op_3dup`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_3dup">op_3dup</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_2over"></a>

## Function `op_2over`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_2over">op_2over</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_2rot"></a>

## Function `op_2rot`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_2rot">op_2rot</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_2swap"></a>

## Function `op_2swap`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_2swap">op_2swap</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_ifdup"></a>

## Function `op_ifdup`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_ifdup">op_ifdup</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_depth"></a>

## Function `op_depth`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_depth">op_depth</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_drop"></a>

## Function `op_drop`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_drop">op_drop</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_dup"></a>

## Function `op_dup`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_dup">op_dup</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nip"></a>

## Function `op_nip`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nip">op_nip</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_over"></a>

## Function `op_over`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_over">op_over</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_pick"></a>

## Function `op_pick`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_pick">op_pick</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_roll"></a>

## Function `op_roll`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_roll">op_roll</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_rot"></a>

## Function `op_rot`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_rot">op_rot</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_swap"></a>

## Function `op_swap`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_swap">op_swap</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_tuck"></a>

## Function `op_tuck`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_tuck">op_tuck</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_cat"></a>

## Function `op_cat`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_cat">op_cat</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_substr"></a>

## Function `op_substr`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_substr">op_substr</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_left"></a>

## Function `op_left`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_left">op_left</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_right"></a>

## Function `op_right`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_right">op_right</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_size"></a>

## Function `op_size`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_size">op_size</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_invert"></a>

## Function `op_invert`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_invert">op_invert</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_and_op"></a>

## Function `op_and_op`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_and_op">op_and_op</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_or_op"></a>

## Function `op_or_op`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_or_op">op_or_op</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_xor"></a>

## Function `op_xor`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_xor">op_xor</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_equal"></a>

## Function `op_equal`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_equal">op_equal</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_equalverify"></a>

## Function `op_equalverify`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_equalverify">op_equalverify</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_reserved1"></a>

## Function `op_reserved1`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_reserved1">op_reserved1</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_reserved2"></a>

## Function `op_reserved2`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_reserved2">op_reserved2</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_1add"></a>

## Function `op_1add`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_1add">op_1add</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_1sub"></a>

## Function `op_1sub`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_1sub">op_1sub</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_2mul"></a>

## Function `op_2mul`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_2mul">op_2mul</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_2div"></a>

## Function `op_2div`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_2div">op_2div</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_negate"></a>

## Function `op_negate`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_negate">op_negate</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_abs"></a>

## Function `op_abs`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_abs">op_abs</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_not"></a>

## Function `op_not`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_not">op_not</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_0notequal"></a>

## Function `op_0notequal`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_0notequal">op_0notequal</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_add"></a>

## Function `op_add`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_add">op_add</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_sub"></a>

## Function `op_sub`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_sub">op_sub</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_mul"></a>

## Function `op_mul`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_mul">op_mul</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_div"></a>

## Function `op_div`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_div">op_div</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_mod"></a>

## Function `op_mod`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_mod">op_mod</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_lshift"></a>

## Function `op_lshift`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_lshift">op_lshift</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_rshift"></a>

## Function `op_rshift`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_rshift">op_rshift</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_booland"></a>

## Function `op_booland`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_booland">op_booland</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_boolor"></a>

## Function `op_boolor`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_boolor">op_boolor</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_numequal"></a>

## Function `op_numequal`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_numequal">op_numequal</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_numequalverify"></a>

## Function `op_numequalverify`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_numequalverify">op_numequalverify</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_numnotequal"></a>

## Function `op_numnotequal`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_numnotequal">op_numnotequal</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_lessthan"></a>

## Function `op_lessthan`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_lessthan">op_lessthan</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_greaterthan"></a>

## Function `op_greaterthan`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_greaterthan">op_greaterthan</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_lessthanorequal"></a>

## Function `op_lessthanorequal`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_lessthanorequal">op_lessthanorequal</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_greaterthanorequal"></a>

## Function `op_greaterthanorequal`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_greaterthanorequal">op_greaterthanorequal</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_min"></a>

## Function `op_min`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_min">op_min</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_max"></a>

## Function `op_max`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_max">op_max</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_within"></a>

## Function `op_within`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_within">op_within</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_ripemd160"></a>

## Function `op_ripemd160`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_ripemd160">op_ripemd160</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_sha1"></a>

## Function `op_sha1`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_sha1">op_sha1</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_sha256"></a>

## Function `op_sha256`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_sha256">op_sha256</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_hash160"></a>

## Function `op_hash160`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_hash160">op_hash160</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_hash256"></a>

## Function `op_hash256`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_hash256">op_hash256</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_codeseparator"></a>

## Function `op_codeseparator`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_codeseparator">op_codeseparator</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_checksig"></a>

## Function `op_checksig`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_checksig">op_checksig</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_checksigverify"></a>

## Function `op_checksigverify`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_checksigverify">op_checksigverify</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_checkmultisig"></a>

## Function `op_checkmultisig`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_checkmultisig">op_checkmultisig</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_checkmultisigverify"></a>

## Function `op_checkmultisigverify`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_checkmultisigverify">op_checkmultisigverify</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nop1"></a>

## Function `op_nop1`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nop1">op_nop1</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_cltv"></a>

## Function `op_cltv`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_cltv">op_cltv</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_csv"></a>

## Function `op_csv`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_csv">op_csv</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nop4"></a>

## Function `op_nop4`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nop4">op_nop4</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nop5"></a>

## Function `op_nop5`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nop5">op_nop5</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nop6"></a>

## Function `op_nop6`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nop6">op_nop6</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nop7"></a>

## Function `op_nop7`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nop7">op_nop7</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nop8"></a>

## Function `op_nop8`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nop8">op_nop8</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nop9"></a>

## Function `op_nop9`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nop9">op_nop9</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_nop10"></a>

## Function `op_nop10`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_nop10">op_nop10</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_checksigadd"></a>

## Function `op_checksigadd`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_checksigadd">op_checksigadd</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_187"></a>

## Function `op_return_187`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_187">op_return_187</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_188"></a>

## Function `op_return_188`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_188">op_return_188</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_189"></a>

## Function `op_return_189`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_189">op_return_189</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_190"></a>

## Function `op_return_190`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_190">op_return_190</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_191"></a>

## Function `op_return_191`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_191">op_return_191</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_192"></a>

## Function `op_return_192`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_192">op_return_192</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_193"></a>

## Function `op_return_193`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_193">op_return_193</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_194"></a>

## Function `op_return_194`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_194">op_return_194</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_195"></a>

## Function `op_return_195`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_195">op_return_195</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_196"></a>

## Function `op_return_196`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_196">op_return_196</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_197"></a>

## Function `op_return_197`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_197">op_return_197</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_198"></a>

## Function `op_return_198`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_198">op_return_198</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_199"></a>

## Function `op_return_199`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_199">op_return_199</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_200"></a>

## Function `op_return_200`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_200">op_return_200</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_201"></a>

## Function `op_return_201`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_201">op_return_201</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_202"></a>

## Function `op_return_202`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_202">op_return_202</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_203"></a>

## Function `op_return_203`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_203">op_return_203</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_204"></a>

## Function `op_return_204`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_204">op_return_204</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_205"></a>

## Function `op_return_205`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_205">op_return_205</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_206"></a>

## Function `op_return_206`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_206">op_return_206</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_207"></a>

## Function `op_return_207`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_207">op_return_207</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_208"></a>

## Function `op_return_208`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_208">op_return_208</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_209"></a>

## Function `op_return_209`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_209">op_return_209</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_210"></a>

## Function `op_return_210`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_210">op_return_210</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_211"></a>

## Function `op_return_211`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_211">op_return_211</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_212"></a>

## Function `op_return_212`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_212">op_return_212</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_213"></a>

## Function `op_return_213`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_213">op_return_213</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_214"></a>

## Function `op_return_214`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_214">op_return_214</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_215"></a>

## Function `op_return_215`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_215">op_return_215</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_216"></a>

## Function `op_return_216`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_216">op_return_216</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_217"></a>

## Function `op_return_217`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_217">op_return_217</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_218"></a>

## Function `op_return_218`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_218">op_return_218</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_219"></a>

## Function `op_return_219`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_219">op_return_219</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_220"></a>

## Function `op_return_220`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_220">op_return_220</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_221"></a>

## Function `op_return_221`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_221">op_return_221</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_222"></a>

## Function `op_return_222`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_222">op_return_222</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_223"></a>

## Function `op_return_223`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_223">op_return_223</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_224"></a>

## Function `op_return_224`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_224">op_return_224</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_225"></a>

## Function `op_return_225`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_225">op_return_225</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_226"></a>

## Function `op_return_226`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_226">op_return_226</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_227"></a>

## Function `op_return_227`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_227">op_return_227</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_228"></a>

## Function `op_return_228`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_228">op_return_228</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_229"></a>

## Function `op_return_229`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_229">op_return_229</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_230"></a>

## Function `op_return_230`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_230">op_return_230</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_231"></a>

## Function `op_return_231`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_231">op_return_231</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_232"></a>

## Function `op_return_232`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_232">op_return_232</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_233"></a>

## Function `op_return_233`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_233">op_return_233</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_234"></a>

## Function `op_return_234`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_234">op_return_234</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_235"></a>

## Function `op_return_235`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_235">op_return_235</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_236"></a>

## Function `op_return_236`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_236">op_return_236</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_237"></a>

## Function `op_return_237`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_237">op_return_237</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_238"></a>

## Function `op_return_238`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_238">op_return_238</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_239"></a>

## Function `op_return_239`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_239">op_return_239</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_240"></a>

## Function `op_return_240`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_240">op_return_240</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_241"></a>

## Function `op_return_241`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_241">op_return_241</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_242"></a>

## Function `op_return_242`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_242">op_return_242</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_243"></a>

## Function `op_return_243`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_243">op_return_243</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_244"></a>

## Function `op_return_244`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_244">op_return_244</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_245"></a>

## Function `op_return_245`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_245">op_return_245</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_246"></a>

## Function `op_return_246`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_246">op_return_246</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_247"></a>

## Function `op_return_247`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_247">op_return_247</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_248"></a>

## Function `op_return_248`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_248">op_return_248</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_249"></a>

## Function `op_return_249`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_249">op_return_249</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_250"></a>

## Function `op_return_250`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_250">op_return_250</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_251"></a>

## Function `op_return_251`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_251">op_return_251</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_252"></a>

## Function `op_return_252`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_252">op_return_252</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_253"></a>

## Function `op_return_253`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_253">op_return_253</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_return_254"></a>

## Function `op_return_254`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_return_254">op_return_254</a>(): u8
</code></pre>



<a name="0x3_bitcoin_opcode_op_invalidopcode"></a>

## Function `op_invalidopcode`



<pre><code><b>public</b> <b>fun</b> <a href="bitcoin_opcode.md#0x3_bitcoin_opcode_op_invalidopcode">op_invalidopcode</a>(): u8
</code></pre>

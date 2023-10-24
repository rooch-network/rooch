pragma circom 2.1.5;

include "circomlib/circuits/bitify.circom";
include "./string.circom";
include "./sha256general.circom";
include "./sha256partial.circom";

template Sha256Bytes(max_num_bytes) {
    signal input in_padded[max_num_bytes];
    signal input in_len_padded_bytes;
    signal output out[256];

    var num_bits = max_num_bytes * 8;
    component sha = Sha256General(num_bits);

    component bytes[max_num_bytes];
    for (var i = 0; i < max_num_bytes; i++) {
        bytes[i] = Num2Bits(8);
        bytes[i].in <== in_padded[i];
        for (var j = 0; j < 8; j++) {
            sha.paddedIn[i*8+j] <== bytes[i].out[7-j];
        }
    }
    sha.in_len_padded_bits <== in_len_padded_bytes * 8;

    for (var i = 0; i < 256; i++) {
        out[i] <== sha.out[i];
    }
}

template Sha256Pad(max_bytes) {
    assert(max_bytes % 64 == 0);

    signal input text[max_bytes];
    signal output padded_text[max_bytes];
    signal output text_len;

    // text length
    component len = Len(max_bytes);
    len.text <== text;

    for (var i = 0; i < max_bytes; i++) {
        padded_text[i] <-- i < len.length ? text[i] : (i == len.length ? 128: 0); // Add the 1 on the end
    }

    text_len <== max_bytes;
}

template Sha256String(max_bytes) {
    signal input text[max_bytes];
    signal output sha[256];

    // text pad
    component sha256Pad = Sha256Pad(max_bytes);
    sha256Pad.text <== text;

    sha <== Sha256Bytes(max_bytes)(sha256Pad.padded_text, sha256Pad.text_len);
}

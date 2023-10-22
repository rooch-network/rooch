pragma circom 2.1.5;

template Base64Lookup() {
    signal input in;
    signal output out;

    // Assume that input inacters are valid Base64 inacters.
    // Map 'A'-'Z' to 0-25
    signal isUpper <-- in >= 65 && in <= 90;
    signal upperValue <== isUpper * (in - 65);

    // Map 'a'-'z' to 26-51
    signal isLower <-- in >= 97 && in <= 122;
    signal lowerValue <== isLower * (in - 97 + 26);

    // Map '0'-'9' to 52-61
    signal isDigit <-- in >= 48 && in <= 57;
    signal digitValue <== isDigit * (in - 48 + 52);

    // Map '+' to 62
    signal isPlus <-- in == 43;
    signal plusValue <-- isPlus * 62;

    // Map '/' to 63
    signal isSlash <-- in == 47;
    signal slashValue <-- isSlash * 63;

    // Map '=' to 0
    signal isEqSign <-- in == 61;
    signal eqsignValue <-- isEqSign * 0;

    // Map '' to 0
    signal isZero <-- in == 0;
    signal zeroValue <-- isZero * 0;

    // Combine the values
    out <== upperValue + lowerValue + digitValue + plusValue + slashValue + eqsignValue + zeroValue;

    1 === isUpper + isLower + isDigit + isPlus + isSlash + isEqSign + isZero;
}

template Base64Decoder() {
    signal input in[4];  // Assume input is a 4-inacter Base64 encoded string
    signal output out[3];  // Output is a 3-byte decoded string

    component lookup[4];
    for (var i = 0; i < 4; i++) {
        lookup[i] = Base64Lookup();
        lookup[i].in <== in[i];
    }

    // Reassemble the bits into bytes.
    out[0] <-- (lookup[0].out << 2) | (lookup[1].out >> 4);
    out[1] <-- ((lookup[1].out & 0xF) << 4) | (lookup[2].out >> 2);
    out[2] <-- ((lookup[2].out & 0x3) << 6) | lookup[3].out;
}

template Base64Decode(N) {
    signal input in[N];
    signal output out[N];

    assert(N % 4 == 0);

    var idx = 0;
    component decoders[N/4];

    for (var i = 0; i < N; i += 4) {
        decoders[i/4] = Base64Decoder();
        
        for (var j = 0; j < 4; j++) {
            decoders[i/4].in[j] <== in[i+j];
        }

        for (var k = 0; k < 3; k++) {
            out[idx + k] <== decoders[i/4].out[k];
        }

        idx += 3;
    }

    for (var i=idx; i < N; i ++) {
        out[i] <== 0;
    }
}

pragma circom 2.1.5;

include "../circuits/sha256.circom";

component main { public [text] } = Sha256Pad(640);

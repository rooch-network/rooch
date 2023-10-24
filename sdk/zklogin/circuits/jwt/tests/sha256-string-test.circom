pragma circom 2.1.5;

include "../helpers/sha256.circom";

component main { public [text] } = Sha256String(256);
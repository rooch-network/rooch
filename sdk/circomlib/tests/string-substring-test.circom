pragma circom 2.1.5;

include "../circuits/string.circom";

component main { public [ text, startIndex, count ] } = SubString(256, 64);

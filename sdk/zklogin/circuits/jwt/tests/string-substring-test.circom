pragma circom 2.1.5;

include "../helpers/string.circom";

component main { public [ text, startIndex, count ] } = SubString(256, 64);

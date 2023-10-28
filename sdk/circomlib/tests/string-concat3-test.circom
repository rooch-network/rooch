pragma circom 2.1.5;

include "../circuits/string.circom";

component main { public [ text1, text2, text3 ] } = Concat3(4, 2, 8);
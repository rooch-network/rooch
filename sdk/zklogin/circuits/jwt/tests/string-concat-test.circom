pragma circom 2.1.5;

include "../helpers/string.circom";

component main { public [ text1, text2 ] } = Concat(4, 8);
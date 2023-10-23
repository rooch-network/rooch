pragma circom 2.1.5;

include "../helpers/string.circom";

component main { public [ text ] } = SplitBy(256, 46, 3); // 46 is '.'
pragma circom 2.1.5;

include "../circuits/jwt.circom";

component main { public [ jwt ] } = JWTSplit(512);
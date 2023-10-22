pragma circom 2.1.5;

include "../helpers/jwt.circom";

component main { public [ jwt ] } = JWTSplit(512);
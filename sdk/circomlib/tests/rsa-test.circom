pragma circom 2.1.5;

include "../circuits/rsa.circom";

component main { public [modulus] } = RSAVerify65537(121, 17);

pragma circom 2.1.5;

include "../helpers/jwt.circom";

component main { public [ jwt_header, jwt_payload, signature, pubkey ] } = JWTVerify(511, 512, 121, 17);
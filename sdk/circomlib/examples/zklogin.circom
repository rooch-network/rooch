pragma circom 2.0.0;

include "../circuits/string.circom";
include "../circuits/jwt.circom";

component main { public [ jwt, signature, pubkey ] } = JWTVerify(512, 121, 17);

template zkLogin(jwt_max_bytes) {
  signal input oauth_jwt[jwt_max_bytes];
  signal input oauth_signature[17];
  signal input oauth_pubKey[17];
  signal input oauth_nonce;
  signal input sequence_number;
  
  signal output rooch_address;

  // JWT Verify
  component jwtVerify = JWTVerify(jwt_max_bytes, 121, 17); // 46 is '.'
  jwtVerify.text <== oauth_jwt;
  jwtVerify.signature <== oauth_signature;
  jwtVerify.pubkey <== oauth_pubKey;

  // Split JWT header and payload
  component splitBy = SplitBy(jwt_max_bytes, 46, 2); // 46 is '.'
  signal jwt_header <== splitBy.out[0];
  signal jwt_payload <== splitBy.out[1];

  // TODO Extract user ID and nonce from JWT
  // TODO Verify if the nonce is correct
  // TODO generate rooch_address
}

component main = zkLogin();

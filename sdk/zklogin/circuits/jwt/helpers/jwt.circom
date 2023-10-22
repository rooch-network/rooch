pragma circom 2.0.0;

include "./string.circom";

template JWTSplit(jwt_max_bytes) {
  signal input jwt[jwt_max_bytes];
  signal output header[jwt_max_bytes];
  signal output payload[jwt_max_bytes];
  signal output signature[jwt_max_bytes];

  // split JWT 
  component splitedJWT = Split(jwt_max_bytes, 46, 3); // 46 is '.'

  splitedJWT.text <== jwt;
  header <== splitedJWT.out[0];
  payload <== splitedJWT.out[1];
  signature <== splitedJWT.out[2];
}
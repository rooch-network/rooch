pragma circom 2.1.5;

include "./string.circom";
include "./sha256.circom";
include "./rsa.circom";

template JWTSplit(max_bytes) {
  signal input jwt[max_bytes];
  signal output header[max_bytes];
  signal output payload[max_bytes];
  signal output signature[max_bytes];

  // split JWT 
  component splitedJWT = SplitBy(max_bytes, 46, 3); // 46 is '.'

  splitedJWT.text <== jwt;
  header <== splitedJWT.out[0];
  payload <== splitedJWT.out[1];
  signature <== splitedJWT.out[2];
}

template JWTVerify(max_bytes, n, k) {
  assert(max_bytes % 64 == 0);
  assert(n * k > 2048); // constraints for 2048 bit RSA
  assert(n < (255 \ 2)); // we want a multiplication to fit into a circom signal

  signal input jwt[max_bytes];
  signal input signature[k];
  signal input pubkey[k];

  // JWT length
  component jwt_len = Len(max_bytes);
  jwt_len.text <== jwt;

  // JWT hash
  signal output sha[256] <== Sha256Bytes(max_bytes)(jwt, jwt_len.length);

  var msg_len = (256 + n) \ n;

  component base_msg[msg_len];
  for (var i = 0; i < msg_len; i++) {
      base_msg[i] = Bits2Num(n);
  }
  for (var i = 0; i < 256; i++) {
      base_msg[i \ n].in[i % n] <== sha[255 - i];
  }
  for (var i = 256; i < n * msg_len; i++) {
      base_msg[i \ n].in[i % n] <== 0;
  }

  // Verify RSA signature
  component rsa = RSAVerify65537(n, k);
  for (var i = 0; i < msg_len; i++) {
      rsa.base_message[i] <== base_msg[i].out;
  }
  for (var i = msg_len; i < k; i++) {
      rsa.base_message[i] <== 0;
  }

  rsa.signature <== signature;
  rsa.modulus <== pubkey;
}


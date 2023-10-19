pragma circom 2.0.0;

// jwt.circom
include "../../node_modules/circomlib/circuits/poseidon.circom";
include "../../node_modules/circomlib/circuits/eddsa.circom";
include "helpers/split.circom";

template verifyJwtSignature() {

  signal input pubKey;
  signal input jwt;

  signal output verified;

  // split JWT 
  component sqrJWT = Split(3, '.');

  sqrJWT.in <== jwt;
  var header = sqrJWT.out[0];
  var payload = sqrJWT.out[1];
  var signature = sqrJWT.out[2];

  // poseidon hash
  component poseidonHash = Poseidon(2);
  
  poseidonHash.inputs[0] <== header;
  poseidonHash.inputs[1] <== payload;  
  var hashed = poseidonHash.out;

  // eddsa verify
  component eddsaVerifier = EdDSAVerifier();

  eddsaVerifier.pubKey <== pubKey;
  eddsaVerifier.msg <== hashed; 
  eddsaVerifier.sig <== signature;

  verified <== eddsaVerifier.out;
}

component main = verifyJwtSignature();
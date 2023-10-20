pragma circom 2.1.5;

template ChatAt() {
  signal input text;
  signal input index;
  signal output ch;

  ch <-- (text >> index*8) & 0xFF;
}

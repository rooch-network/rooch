pragma circom 2.1.5;

template CharCompare(idx) {
  signal input ch;
  signal input targetChar;
  signal output isMatch;

  isMatch <-- ch == targetChar ? 1 : 0;
}

template Len(str_max_len) {
  signal input text[str_max_len];
  signal output length;

  var tmpLen = 0;

  for (var i = 0; i < str_max_len; i++) {
    tmpLen = tmpLen + (text[i] == 0 ? 0 : 1);  // 直接在这里检查每个字符
  }

  length <-- tmpLen;
}

template CharAt(str_max_len) {
  signal input text[str_max_len];
  signal input index;
  signal output ch;

  assert(index >= 0);
  ch <-- index < str_max_len ? text[index] : 0;
}

template IndexOf(str_max_len) {
  signal input text[str_max_len];
  signal input startIndex;
  signal input targetChar;
  signal output index;

  assert(startIndex >= 0 && startIndex < str_max_len);

  var tmpIndex = -1;

  for (var i = 0; i < str_max_len; i++) {
    tmpIndex = tmpIndex + (i >= startIndex && tmpIndex == -1 && (text[i] == targetChar) ? (i + 1) : 0);
  }

  index <-- tmpIndex;
}

template SubString(str_max_len, sub_str_len) {
  signal input text[str_max_len];
  signal input startIndex;
  signal input count;
  signal output substring[sub_str_len];

  assert(startIndex >= 0 && startIndex < str_max_len);
  assert(startIndex + count < str_max_len);
  assert(count >= 0 && count < sub_str_len);
 
  component charAts[sub_str_len];
  for (var i = 0; i < sub_str_len; i++) {
    charAts[i] = CharAt(str_max_len);
    charAts[i].text <== text;
    charAts[i].index <-- startIndex + i;

    substring[i] <-- i < count ? charAts[i].ch : 0;
  }
}

template SplitPart(str_max_len, sep_ch) {
  signal input text[str_max_len];
  signal input startIndex;
  signal output token[str_max_len];
  signal output findIndex;

  assert(startIndex >= 0 && startIndex < str_max_len);

  component len = Len(str_max_len);
  len.text <== text;

  component indexOf = IndexOf(str_max_len);
  indexOf.text <== text;
  indexOf.startIndex <== startIndex;
  indexOf.targetChar <== sep_ch;

  component subStr = SubString(str_max_len, str_max_len);
  subStr.text <== text;
  subStr.startIndex <== startIndex;
  subStr.count <-- indexOf.index == -1 ? len.length - startIndex: indexOf.index - startIndex;

  token <== subStr.substring;
  findIndex <== indexOf.index;
}

template SplitBy(str_max_len, sep_ch, count) {
  signal input text[str_max_len];
  signal output out[count][str_max_len];

  var currentIndex = 0;

  component splitParts[count];
  for (var i = 0; i < count; i++) {
    splitParts[i] = SplitPart(str_max_len, sep_ch);
    splitParts[i].text <== text;
    splitParts[i].startIndex <-- currentIndex;

    out[i] <== splitParts[i].token;
    currentIndex = splitParts[i].findIndex == -1 ? 0 : splitParts[i].findIndex + 1;
  }
}

template Concat(str_max_len1, str_max_len2) {
  signal input text1[str_max_len1];
  signal input text2[str_max_len2];
  signal output out[str_max_len1 + str_max_len2];

  component len1 = Len(str_max_len1);
  len1.text <== text1;
  
  component len2 = Len(str_max_len2);
  len2.text <== text2;

  for (var i = 0; i < str_max_len1 + str_max_len2; i++) {
    out[i] <-- i < len1.length ? text1[i] : (i < len1.length + len2.length ? text2[i - len1.length] : 0);
  }
}

template Concat3(str_max_len1, str_max_len2, str_max_len3) {
  signal input text1[str_max_len1];
  signal input text2[str_max_len2];
  signal input text3[str_max_len3];
  signal output out[str_max_len1 + str_max_len2 + str_max_len3];

  component len1 = Len(str_max_len1);
  len1.text <== text1;
  
  component len2 = Len(str_max_len2);
  len2.text <== text2;

  component len3 = Len(str_max_len3);
  len3.text <== text3;

  for (var i = 0; i < str_max_len1 + str_max_len2 + str_max_len3; i++) {
    out[i] <-- i < len1.length ? text1[i] : (i < len1.length + len2.length ? text2[i - len1.length] : (i < len1.length + len2.length + len3.length ? text3[i - len1.length - len2.length] : 0));
  }
}

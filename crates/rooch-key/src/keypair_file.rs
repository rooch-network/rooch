// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use fastcrypto::traits::EncodeDecodeBase64;
use rooch_types::account::RoochKeyPair;

/// Write Base64 encoded `flag || privkey` to file.
pub fn write_keypair_to_file<P: AsRef<std::path::Path>>(
    keypair: &RoochKeyPair,
    path: P,
) -> anyhow::Result<()> {
    let contents = keypair.encode_base64();
    std::fs::write(path, contents)?;
    Ok(())
}

/// Write Base64 encoded `privkey` to file.
//pub fn write_authority_keypair_to_file<P: AsRef<std::path::Path>>(
//    keypair: &AuthorityKeyPair,
//    path: P,
//) -> anyhow::Result<()> {
//    let contents = keypair.encode_base64();
//    std::fs::write(path, contents)?;
//    Ok(())
//}
//
///// Read from file as Base64 encoded `privkey` and return a AuthorityKeyPair.
//pub fn read_authority_keypair_from_file<P: AsRef<std::path::Path>>(
//    path: P,
//) -> anyhow::Result<AuthorityKeyPair> {
//    let contents = std::fs::read_to_string(path)?;
//    AuthorityKeyPair::decode_base64(contents.as_str().trim()).map_err(|e| anyhow!(e))
//}

/// Read from file as Base64 encoded `flag || privkey` and return a RoochKeypair.
pub fn read_keypair_from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<RoochKeyPair> {
    let contents = std::fs::read_to_string(path)?;
    RoochKeyPair::decode_base64(contents.as_str().trim()).map_err(|e| anyhow!(e))
}
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::consensus::Encodable;
use bitcoin::hashes::{sha256, Hash, HashEngine};
use bitcoin::taproot::{LeafNode, LeafVersion, TaprootBuilder};
use bitcoin::{opcodes, Script, ScriptBuf, TapNodeHash};
use std::str::FromStr;

struct TaprootBuilderWrapper(TaprootBuilder);

impl TaprootBuilderWrapper {
    fn new() -> Self {
        TaprootBuilderWrapper(TaprootBuilder::new())
    }

    fn add_leaf(mut self, depth: u8, script: ScriptBuf) -> Self {
        self.0 = self.0.add_leaf(depth, script).unwrap();
        self
    }

    fn finalize(self) -> TapNodeHash {
        // For simplicity, we're using a dummy internal key here
        let internal_key = dummy_pubkey();
        let secp = bitcoin::secp256k1::Secp256k1::verification_only();
        self.0
            .finalize(&secp, internal_key)
            .unwrap()
            .merkle_root()
            .unwrap()
    }
}

fn dummy_pubkey() -> bitcoin::XOnlyPublicKey {
    // This is the generator point of secp256k1. Private key is known (equal to 1)
    let g :Vec<u8> = hex::decode("0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8").unwrap();
    let hash = sha256::Hash::hash(&g);
    bitcoin::XOnlyPublicKey::from_slice(hash.as_byte_array()).unwrap()
}

#[test]
fn test_taproot_builder() {
    let mut builder = TaprootBuilderWrapper::new();

    let script1 = Script::builder().push_opcode(opcodes::OP_0).into_script();
    let script2 = Script::builder()
        .push_opcode(opcodes::OP_TRUE)
        .into_script();

    builder = builder.add_leaf(1, script1);
    builder = builder.add_leaf(1, script2);

    let root = builder.finalize();

    //println!("Rust implementation root: {:?}", root);
    let expect_root =
        TapNodeHash::from_str("15526cd6108b4765640abe555e75f4bd11d9b1453b9db4cd36cf4189577a6f63")
            .unwrap();
    assert_eq!(root, expect_root);
}

#[test]
fn test_taproot_builder_three_leaves() {
    let mut builder = TaprootBuilderWrapper::new();

    let script1 = Script::builder().push_opcode(opcodes::OP_0).into_script();
    let script2 = Script::builder()
        .push_opcode(opcodes::OP_TRUE)
        .into_script();
    let script3 = Script::builder()
        .push_opcode(opcodes::OP_NOP2)
        .into_script();

    builder = builder.add_leaf(1, script1);
    builder = builder.add_leaf(2, script2);
    builder = builder.add_leaf(2, script3);

    let root = builder.finalize();

    //println!("Rust implementation root: {:?}", root);
    let expect_root =
        TapNodeHash::from_str("d847514fba3bdcfed383ce109a2700baafd6a629e290b22678c8c21ca93aca86")
            .unwrap();
    assert_eq!(root, expect_root);
}

fn leaf_node_tagged_hash(script: &ScriptBuf, ver: LeafVersion) -> sha256::Hash {
    let tag = b"TapLeaf";
    let tag_hash = sha256::Hash::hash(tag);
    let mut engine = sha256::Hash::engine();
    engine.input(&tag_hash[..]);
    engine.input(&tag_hash[..]);

    ver.to_consensus()
        .consensus_encode(&mut engine)
        .expect("engines don't error");
    script
        .consensus_encode(&mut engine)
        .expect("engines don't error");
    sha256::Hash::from_engine(engine)
}

#[test]
fn test_tapnode_hash() {
    let script = Script::builder().push_opcode(opcodes::OP_0).into_script();
    let ver = LeafVersion::TapScript;
    let leaf_node = LeafNode::new_script(script.clone(), ver);
    let hash = leaf_node.node_hash();
    println!("Rust implementation leaf hash: {:?}", hash);
    assert_eq!(
        TapNodeHash::from_str("e7e4d593fcb72926eedbe0d1e311f41acd6f6ef161dcba081a75168ec4dcd379")
            .unwrap(),
        hash
    );

    let hash2 = leaf_node_tagged_hash(&script, ver);
    assert_eq!(hash.to_byte_array(), hash2.to_byte_array());
}

#[test]
fn test_internal_node_hash() {
    let a = LeafNode::new_script(
        Script::builder().push_opcode(opcodes::OP_0).into_script(),
        LeafVersion::TapScript,
    );
    let b = LeafNode::new_script(
        Script::builder()
            .push_opcode(opcodes::OP_TRUE)
            .into_script(),
        LeafVersion::TapScript,
    );
    let a_hash = a.node_hash();
    let b_hash = b.node_hash();
    let hash = TapNodeHash::from_node_hashes(a_hash, b_hash);
    let expect_hash =
        TapNodeHash::from_str("15526cd6108b4765640abe555e75f4bd11d9b1453b9db4cd36cf4189577a6f63")
            .unwrap();
    //println!("Rust implementation internal hash: {:?}", hash);
    assert_eq!(hash, expect_hash);
}

#[test]
fn test_internal_node_hash2() {
    let a =
        TapNodeHash::from_str("e7e4d593fcb72926eedbe0d1e311f41acd6f6ef161dcba081a75168ec4dcd379")
            .unwrap();
    let b =
        TapNodeHash::from_str("a85b2107f791b26a84e7586c28cec7cb61202ed3d01944d832500f363782d675")
            .unwrap();
    let c =
        TapNodeHash::from_str("529d993be5090bb76ae9334283c3796b24169ec184caa6dcf04f39d7dcde9e3d")
            .unwrap();
    assert!(b > c);
    let hash = TapNodeHash::from_node_hashes(b, c);
    let hash2 = TapNodeHash::from_node_hashes(c, b);
    assert_eq!(hash, hash2);
    let expected_hash =
        TapNodeHash::from_str("9de4ce9cf96062eda41e0a9f2d977e38ca1486cc5dc3a66ff6c2fe8dc5301ccf")
            .unwrap();
    assert_eq!(hash, expected_hash);
    println!("cmp: {:?}", a < hash);
    let hash3 = TapNodeHash::from_node_hashes(a, hash);
    let expected_hash2 =
        TapNodeHash::from_str("d847514fba3bdcfed383ce109a2700baafd6a629e290b22678c8c21ca93aca86")
            .unwrap();
    assert_eq!(hash3, expected_hash2);
}

//Make sure the cmp function is same as in Move
//frameworks/rooch-nursery/sources/taproot_builder.move
#[test]
fn test_cmp() {
    let a =
        TapNodeHash::from_str("e7e4d593fcb72926eedbe0d1e311f41acd6f6ef161dcba081a75168ec4dcd379")
            .unwrap();
    let b =
        TapNodeHash::from_str("9de4ce9cf96062eda41e0a9f2d977e38ca1486cc5dc3a66ff6c2fe8dc5301ccf")
            .unwrap();
    let order = std::cmp::Ord::cmp(a.as_byte_array(), b.as_byte_array());
    assert!(order.is_gt())
}

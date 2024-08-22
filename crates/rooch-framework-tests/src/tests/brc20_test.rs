// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::consensus::Decodable;
use hex::FromHex;
use tracing::debug;

fn decode_tx(btx_tx_hex: &str) {
    let btc_tx_bytes = Vec::from_hex(btx_tx_hex).unwrap();
    let btc_tx: bitcoin::Transaction =
        Decodable::consensus_decode(&mut btc_tx_bytes.as_slice()).unwrap();
    debug!("tx_id: {}", btc_tx.txid());
    for (i, input) in btc_tx.input.iter().enumerate() {
        debug!("{}. input: {:?}", i, input.previous_output);
    }
    for (i, output) in btc_tx.output.iter().enumerate() {
        debug!(
            "{}. output: {:?}, public_key: {:?}",
            i,
            output,
            output.script_pubkey.p2wpkh_script_code()
        );
    }

    //let binding_test = binding_test::RustBindingTest::new().unwrap();
    //let brc20_module = binding_test.as_module_binding::<rooch_types::bitcoin::brc20::BRC20Module>();
    //let move_btc_tx: rooch_types::bitcoin::types::Transaction = btc_tx.into();
    //let ops_from_move = brc20_module.get_tick_info(&move_btc_tx).unwrap();
    //debug!("ops_from_move: {:?}", ops_from_move);
    //ops_from_move
}

#[test]
fn test_from_transaction() {
    let _ = tracing_subscriber::fmt::try_init();

    //inscribe mint
    //https://ordinals.com/inscription/24f2585e667e345c7b72a4969b4c70eb0e2106727d876217497c6cf86a8a354ci0
    //https://mempool.space/api/tx/24f2585e667e345c7b72a4969b4c70eb0e2106727d876217497c6cf86a8a354c/hex
    // {
    //     "p": "brc-20",
    //     "op": "mint",
    //     "tick": "ordi",
    //     "amt": "1000"
    //   }
    let btc_tx_hex = "0100000000010168fc0bd080cf62a7bb04a5e3fc1140df4dd34c244edf23e9027d3966f086f25f0000000000fdffffff01102700000000000022512037679ea62eab55ebfd442c53c4ad46b6b75e45d8a8fa9cb31a87d0df268b029a03409baed731180a79d18ac9f54d2ab448e3c1c78df128ba71f471cf75ed5be4db6431d824a1c254bede0d7482ad05a53468b3c737e9f6b4bfe90ba0c064166dd3188d205f308d3670e9d71da3c2d913a44fa0f6daa57f07263b25a23dd3124832753263ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d3800477b200a20202270223a20226272632d3230222c0a2020226f70223a20226d696e74222c0a2020227469636b223a20226f726469222c0a202022616d74223a202231303030220a7d6821c15f308d3670e9d71da3c2d913a44fa0f6daa57f07263b25a23dd312483275326300000000";
    decode_tx(btc_tx_hex);

    //inscribe transfer
    //https://ordinals.com/inscription/885441055c7bb5d1c54863e33f5c3a06e5a14cc4749cb61a9b3ff1dbe52a5bbbi0
    //https://mempool.space/api/tx/885441055c7bb5d1c54863e33f5c3a06e5a14cc4749cb61a9b3ff1dbe52a5bbb/hex
    // {
    //     "p": "brc-20",
    //     "op": "transfer",
    //     "tick": "ordi",
    //     "amt": "100",
    //     "to": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    //     "fee": "1337"
    //   }
    let btc_tx_hex = "01000000000101eb9aadb9ece84438be35112b4681d1001206a504fd28b17a620fc719429eb1230000000000fdffffff01102700000000000022512037679ea62eab55ebfd442c53c4ad46b6b75e45d8a8fa9cb31a87d0df268b029a034091a14e1b53acfec21ea0d3ea0ce6562435e50a20255c73a7234b44c3d6914fc1fc772b481349d20abba86de21420ebb511db9ba12dbaba24cd727677305434edd02052885ab09f6495885e6a1d6cb51e691e3469f1c2e86d8d2442fb44c22253b637ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d38004c897b200a20202270223a20226272632d3230222c0a2020226f70223a20227472616e73666572222c0a2020227469636b223a20226f726469222c0a202022616d74223a2022313030222c0a202022746f223a20223141317a5031655035514765666932444d505466544c35534c6d7637446976664e61222c0a202022666565223a202231333337220a7d6821c152885ab09f6495885e6a1d6cb51e691e3469f1c2e86d8d2442fb44c22253b63700000000";
    decode_tx(btc_tx_hex);

    //transfer
    //https://ordinals.com/inscription/885441055c7bb5d1c54863e33f5c3a06e5a14cc4749cb61a9b3ff1dbe52a5bbbi0
    //https://mempool.space/api/tx/628f019c4e3c30ccc0fd9aae872cb3720294a255127292bf61c38fbee39462fe/hex

    let btc_tx_hex = "02000000000102bb5b2ae5dbf13f9b1ab69c74c44ca1e5063a5c3fe36348c5d1b57b5c054154880000000000ffffffff8eca9f7d2e369e650f439153f503e81dd9960f1030bfb54f9043884a4c63c8bc11000000171600141c6e0ecb1a039c8df94a664ddf130f6e3be90ba5ffffffff0210270000000000001976a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac073d0f000000000017a9142b5fd9fed263646d296cb196bc07747b4c41fdc787014091e2afbe0bf24467275bf90b2fa281e105c6cd0344cd1f0846a89a5369246634d12476f579271a7cadbf918f73de306486f70cc2368274740ea7779a9de4e6c402473044022024e761eeaf29864b4b9bef52f457d2c0302fa5c6b4003c67681cdb006d118404022023aef89d49e6700bca374d2059509cdcacba213c0b8b77a635095c79a63a124f012103b06845003ff20c9e8a1c529003fb32edb1a9d8894e7f2cd37a192c0cbd76fb8f00000000";
    decode_tx(btc_tx_hex);
}

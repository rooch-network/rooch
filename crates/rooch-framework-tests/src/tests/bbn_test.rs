// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    binding_test, bitcoin_block_tester::BitcoinBlockTester,
    tests::bitcoin_data::bitcoin_tx_from_hex,
};
use moveos_types::module_binding::MoveFunctionCaller;
use tracing::{debug, warn};

// Test Babylon v3 transaction
#[tokio::test]
async fn test_block_864790() {
    let _ = tracing_subscriber::fmt::try_init();

    if cfg!(debug_assertions) {
        warn!("test_block_864790 is ignored in debug mode, please run it in release mode");
        return;
    }

    let mut tester = BitcoinBlockTester::new(864790).unwrap();
    tester.execute().unwrap();
    tester.verify_utxo().unwrap();
    //TODO verify bbn tx
}

#[tokio::test]
async fn test_bbn_tx() {
    let _ = tracing_subscriber::fmt::try_init();
    //https://mempool.space/api/tx/a5daf25b85f82de6f93bc08c19abc3d45beeef3fd6d3dac69bf641c061bdcbec/hex
    let tx_hex = "0200000000010499ba804a3414488a46292c42ff1f0f97442bab1ae23d2938dc8a74aa90f07a0800000000000000000003b49a402e959c792da224f49fd84b765043b48fe9744644a526ff9307765ab30000000000000000004cbc8d18517f2c14b0f08c3c8ef183be9d71d10ed63d2c73449c1b8722faccec0000000000000000000dcec7eb4d68bc0af0b923da4bc14c3d45979f7f17edad60e86544c1c07bebcb0000000000000000000300f469b302000000225120ed35a2a991b991c670ec4728df36e55bbf2f3b67f0182619a3e4473ff1879eda0000000000000000496a4762626e31003fc003f3c3f8e274a16b0aae4088ab533f51d4fa45b7af6edd7127ed173e9cb6609b4b8e27e214fd830e69a83a8270a03f7af356f64dde433a7e4b81b2399806fa004890d402000000002251203fc003f3c3f8e274a16b0aae4088ab533f51d4fa45b7af6edd7127ed173e9cb6014084c28ece72b1aa6da758204758013e64bd237cb9be82a861c1d8cf7f59bd2d5bb48fb0eb3e4ce65fd760a343916ff00ad933d49e82b2cf241e8dfa524d536432014006c15b6fee726bfce233b34677fef35a998384ebe8f50a9c858d80f78c4a9f7b081d996e8147096ceb7ad3e89bca79be22b2b2dc0ba9ab0d12ae859efa203c930140f41bb646a690d620c51c16134fce8d2d1e124becbc27361571530431817fd4a44cf4a3656b442254deb6ac893a53edd08acb8f8feeb45e04c221426f5de908f701403f01a045611e777e4a66c453e927c1c2307f4efae606458bd9f7415c9fd143d8e0d8445935795dc7e28c7edd00ef0996952a39a3f7142d721b0fba5cc0a62d6a15320d00";
    let tx = bitcoin_tx_from_hex(tx_hex);

    debug!(
        "OP_RETURN data: {}",
        hex::encode(tx.output[1].script_pubkey.as_bytes())
    );
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let bbn_module = binding_test.as_module_binding::<rooch_types::bitcoin::bbn::BBNModule>();
    let op_return_output_opt = bbn_module.try_get_bbn_op_return_output_from_tx(tx).unwrap();
    assert!(op_return_output_opt.is_some());
    let op_return_output = op_return_output_opt.unwrap();
    debug!("op_return_output: {:?}", op_return_output);
    assert_eq!(op_return_output.vout, 1);
}

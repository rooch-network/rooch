use crate::binding_test;
use bitcoin::consensus::Decodable;
use hex::FromHex;
use rooch_types::framework::ord::Inscription;

#[test]
fn test_ord() {
    tracing_subscriber::fmt::init();
    let binding_test = binding_test::RustBindingTest::new().unwrap();

    //https://mempool.space/api/tx/a03c44005f1871a6068e286a4151b009e3f6184983464782820b56633760333d/hex
    let btc_tx_hex = "02000000000101361cc743a923abc1db73f4fed4d0778cc8ccc092cb20f1c66cada177818e55b20000000000fdffffff022202000000000000225120e5053d2151d14399a3a4825740e14deae6f984e990e0a6872df065a6dad7009c6e04000000000000160014ad45c620bd9b6688c5a7a23e515402d39d02b55203401500c4f407f66ec47c92e1daf34c46f2b52837819119b696e343385b6dba27682dd89f9e4d18354ce0f4a4200ddab8420457392702e1e0b6d51803d25d2bf2647f2016c3a3f18eb4efd24274941ba02c899d151b0473a1bad3512423cbe1b0648ea9ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d3800397b2270223a226272632d3230222c226f70223a227472616e73666572222c227469636b223a226f726469222c22616d74223a2231303030227d6821c102a58d972468a33a79350cf24cb991f28adbbe3e64e88ded5f58f558fff2b67300000000";
    let btc_tx_bytes = Vec::from_hex(btc_tx_hex).unwrap();
    let btc_tx: bitcoin::Transaction =
        Decodable::consensus_decode(&mut btc_tx_bytes.as_slice()).unwrap();
    let inscriptions =
        rooch_framework::natives::rooch_framework::bitcoin::ord::from_transaction(&btc_tx).unwrap();
    //print!("{:?}", inscriptions);
    let ord_module = binding_test.as_module_bundle::<rooch_types::framework::ord::OrdModule>();

    let inscriptions_from_move = ord_module.from_transaction(&btc_tx.into()).unwrap();
    assert_eq!(inscriptions.len(), inscriptions_from_move.len());
    for (inscription, inscription_from_move) in inscriptions.into_iter().zip(inscriptions_from_move)
    {
        assert_eq!(Inscription::from(inscription), inscription_from_move);
    }
}

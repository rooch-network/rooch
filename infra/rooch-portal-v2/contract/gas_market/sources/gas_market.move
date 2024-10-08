module gas_market::gas_market {
    use std::option;
    use std::string::utf8;
    
    use moveos_std::decimal_value::value;
    use moveos_std::address::to_string;
    use moveos_std::event_queue::{Subscriber, consume};
    use moveos_std::event_queue;
    use moveos_std::table;
    use moveos_std::table::Table;
    use moveos_std::tx_context::sender;
    use moveos_std::object::{Object, to_shared};
    use moveos_std::object;
    use moveos_std::signer;

    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::gas_coin::RGas;
    use rooch_framework::account_coin_store;

    use bitcoin_move::utxo::{ReceiveUTXOEvent, unpack_receive_utxo_event};

    use gas_market::trusted_oracle::trusted_price;
    use gas_market::admin::AdminCap;

    #[test_only]
    use moveos_std::decimal_value; 

    struct RGasMarket has key, store {
        rgas_store: Object<CoinStore<RGas>>,
        unit_price: u256,
        receive_btc_address: address,
        market_info: MarketInfo,
        utxo_subscriber: Object<Subscriber<ReceiveUTXOEvent>>,
        is_open: bool
    }

    struct MarketInfo has key, store {
        total_deposit: u256,
        total_withdraw: u256,
        buyer: Table<address, u256>,
        uncheck_info: Table<address, u256>
    }

    // 0.01 U = (BTC decimal * BTC Price decimal / RGas decimal) * 0.01
    //  (10**8 * 10**8 / 10**8) * 0.01
    const DEFAULT_UNIT_PRICE: u256 = 1000000;
    const BTC_USD: vector<u8> = b"BTCUSD";

    const INIT_GAS_AMOUNT: u256 = 50000000_00000000;

    const ErrorMarketNotOpen: u64 = 1;
    const ErrorReceiverAddress: u64 = 2;
    const ErrorTokenPrice: u64 = 3;
    const ErrorNoUncheckTxid: u64 = 4;

    fun init(sender: &signer) {
        let sender_addr = signer::address_of(sender);
        let rgas_store = coin_store::create_coin_store<RGas>();
        let rgas_balance = account_coin_store::balance<RGas>(sender_addr);
        let market_gas_amount = if (rgas_balance > INIT_GAS_AMOUNT) {
            INIT_GAS_AMOUNT
        } else {
            rgas_balance / 3
        };
        deposit_to_rgas_store(sender, &mut rgas_store, market_gas_amount);
        let utxo_subscriber = event_queue::subscribe<ReceiveUTXOEvent>(to_string(&sender()));
        let rgas_market_obj =
            object::new_named_object(
                RGasMarket {
                    rgas_store,
                    unit_price: DEFAULT_UNIT_PRICE,
                    receive_btc_address: sender_addr,
                    market_info: MarketInfo {
                        total_deposit: 0,
                        total_withdraw: 0,
                        buyer: table::new(),
                        uncheck_info: table::new()
                    },
                    utxo_subscriber,
                    is_open: true
                }
            );
        to_shared(rgas_market_obj)
    }

    public entry fun deposit_rgas_coin(
        account: &signer, rgas_market_obj: &mut Object<RGasMarket>, amount: u256
    ) {
        let rgas_market = object::borrow_mut(rgas_market_obj);
        deposit_to_rgas_store(account, &mut rgas_market.rgas_store, amount);
        rgas_market.market_info.total_deposit = rgas_market.market_info.total_deposit + amount
    }

    public entry fun withdraw_rgas_coin(
        rgas_market_obj: &mut Object<RGasMarket>,
        amount: u256,
        _admin: &mut Object<AdminCap>,
    ) {
        let rgas_market = object::borrow_mut(rgas_market_obj);
        let rgas_coin = coin_store::withdraw<RGas>(&mut rgas_market.rgas_store, amount);
        account_coin_store::deposit(sender(), rgas_coin);
        rgas_market.market_info.total_withdraw = rgas_market.market_info.total_withdraw
            + amount
    }

    public fun exists_new_events(rgas_market_obj: &Object<RGasMarket>): bool {
        let rgas_market = object::borrow(rgas_market_obj);
        event_queue::exists_new_events(&rgas_market.utxo_subscriber)
    }

    public entry fun consume_event(
        rgas_market_obj: &mut Object<RGasMarket>,
    ) {
        let rgas_market = object::borrow_mut(rgas_market_obj);
        assert!(rgas_market.is_open, ErrorMarketNotOpen);
        let consume_event = option::extract(&mut consume(&mut rgas_market.utxo_subscriber));
        let (txid, sender, receiver, value) = unpack_receive_utxo_event(consume_event);
        assert!(receiver == rgas_market.receive_btc_address, ErrorReceiverAddress);
        let withdraw_amount = btc_to_rgas(value);
        if (option::is_some(&sender)) {
            let sender_addr = option::extract(&mut sender);
            let rgas_coin =
                coin_store::withdraw<RGas>(&mut rgas_market.rgas_store, withdraw_amount);
            account_coin_store::deposit(sender_addr, rgas_coin);
            if (!table::contains(&rgas_market.market_info.buyer, sender_addr)) {
                table::add(
                    &mut rgas_market.market_info.buyer, sender_addr, withdraw_amount
                )
            } else {
                let total_amount =
                    *table::borrow(&rgas_market.market_info.buyer, sender_addr)
                        + withdraw_amount;
                table::upsert(
                    &mut rgas_market.market_info.buyer, sender_addr, total_amount
                );
            }
        } else {
            if (!table::contains(&rgas_market.market_info.uncheck_info, txid)) {
                table::add(
                    &mut rgas_market.market_info.uncheck_info, txid, withdraw_amount
                )
            } else {
                let total_amount =
                    *table::borrow(&rgas_market.market_info.uncheck_info, txid)
                        + withdraw_amount;
                table::upsert(
                    &mut rgas_market.market_info.uncheck_info, txid, total_amount
                );
            }
        }
    }

    public entry fun consume_uncheck(
        rgas_market_obj: &mut Object<RGasMarket>,
        txid: address,
        sender_addr: address,
        amount: u256,
        _admin: &mut Object<AdminCap>,
    ) {
        let rgas_market = object::borrow_mut(rgas_market_obj);
        let rgas_coin = coin_store::withdraw<RGas>(&mut rgas_market.rgas_store, amount);
        account_coin_store::deposit(sender_addr, rgas_coin);
        assert!(
            table::contains(&rgas_market.market_info.uncheck_info, txid),
            ErrorNoUncheckTxid
        );
        let remaining_amount =
            *table::borrow(&rgas_market.market_info.uncheck_info, txid) - amount;
        if (remaining_amount > 0) {
            table::upsert(
                &mut rgas_market.market_info.uncheck_info, txid, remaining_amount
            );
        } else {
            let _ = table::remove(&mut rgas_market.market_info.uncheck_info, txid);
        };

        if (!table::contains(&rgas_market.market_info.buyer, sender_addr)) {
            table::add(&mut rgas_market.market_info.buyer, sender_addr, amount)
        } else {
            let total_amount =
                *table::borrow(&rgas_market.market_info.buyer, sender_addr) + amount;
            table::upsert(&mut rgas_market.market_info.buyer, sender_addr, total_amount);
        }
    }

    public entry fun remove_uncheck(
        rgas_market_obj: &mut Object<RGasMarket>,
        txid: address,
        _admin: &mut Object<AdminCap>,
    ) {
        let rgas_market = object::borrow_mut(rgas_market_obj);
        table::remove(&mut rgas_market.market_info.uncheck_info, txid);
    }

    public entry fun close_market(
        rgas_market_obj: &mut Object<RGasMarket>,
        _admin: &mut Object<AdminCap>,
    ) {
        let rgas_market = object::borrow_mut(rgas_market_obj);
        rgas_market.is_open = false;
    }

    public entry fun open_market(
        rgas_market_obj: &mut Object<RGasMarket>,
        _admin: &mut Object<AdminCap>,
    ) {
        let rgas_market = object::borrow_mut(rgas_market_obj);
        rgas_market.is_open = true;
    }

    public fun btc_to_rgas(sats_amount: u64): u256 {
        let price_info = trusted_price(utf8(BTC_USD));
        let btc_price = value(&price_info);
        (sats_amount as u256) * btc_price / DEFAULT_UNIT_PRICE
    }

    public fun rgas_to_btc(rgas_amount: u256): u256 {
        let price_info = trusted_price(utf8(BTC_USD));
        let token_price = value(&price_info);
        // TODO If the input quantity of rgas is too small, the return value is 0, and should return u64?
        rgas_amount * DEFAULT_UNIT_PRICE / token_price
    }

    fun deposit_to_rgas_store(
        account: &signer,
        rgas_store: &mut Object<CoinStore<RGas>>,
        amount: u256
    ){
        let rgas_coin = account_coin_store::withdraw<RGas>(account, amount);
        coin_store::deposit(rgas_store, rgas_coin);
    }

    #[test]
    fun test_btc_to_rgas() {
        let price_info = decimal_value::new(5005206000000, 8);
        let token_price = value(&price_info);
        let a = 100000000 * token_price / DEFAULT_UNIT_PRICE;
        assert!(a == 500520600000000, 1);
    }

    #[test]
    fun test_rgas_to_btc() {
        let rgas_amount = 500520600000000;
        let price_info = decimal_value::new(5005206000000, 8);
        let token_price = value(&price_info);
        let b = rgas_amount * DEFAULT_UNIT_PRICE / token_price;
        assert!(b == 100000000, 2);
    }
}

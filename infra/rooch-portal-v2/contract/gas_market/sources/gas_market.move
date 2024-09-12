module gas_market::gas_market {
    use std::option;
    use std::string::utf8;
    use moveos_std::decimal_value::value;
    use gas_market::trusted_oracle::trusted_price;
    use moveos_std::address::to_string;
    use bitcoin_move::utxo::{ReceiveUTXOEvent, unpack_receive_utxo_event};
    use moveos_std::event_queue::{Subscriber, consume};
    use moveos_std::event_queue;
    use moveos_std::table;
    use moveos_std::signer::{module_signer, address_of};
    use moveos_std::account::{move_resource_to, borrow_mut_resource};
    use moveos_std::table::Table;
    use rooch_framework::account_coin_store;
    use moveos_std::tx_context::sender;
    use moveos_std::object::{Object, to_shared, transfer};
    use rooch_framework::coin_store::CoinStore;
    use moveos_std::object;
    use rooch_framework::gas_coin::RGas;
    use rooch_framework::coin_store;
    #[test_only]
    use moveos_std::decimal_value;

    struct AdminCap has store, key {}

    struct RGasMarket has key, store{
        rgas_store: Object<CoinStore<RGas>>,
        unit_price: u256,
        receive_btc_address: address,
        is_open: bool
    }

    struct MarketInfo has key, store{
        total_deposit: u256,
        total_withdraw: u256,
        buyer: Table<address, u256>,
        uncheck_info: Table<address, u256>
    }

    // 0.01 U = (BTC decimal * BTC Price decimal / RGas decimal) * 0.01
    //  (10**8 * 10**8 / 10**8) * 0.01
    const DEFAULT_UNIT_PRICE: u256 = 1000000;
    const BTC_USD: vector<u8> = b"BTCUSD";

    const ErrorMarketNotOpen: u64 = 0;
    const ErrorReceiverAddress: u64 = 1;
    const ErrorTokenPrice: u64 = 2;
    const ErrorNoUncheckTxid: u64 = 3;


    fun init() {
        let rgas_market_obj = object::new_named_object(RGasMarket{
            rgas_store: coin_store::create_coin_store<RGas>(),
            unit_price: DEFAULT_UNIT_PRICE,
            receive_btc_address: sender(),
            is_open: true
        });
        let admin_cap = object::new_named_object(AdminCap{});

        move_resource_to(&module_signer<MarketInfo>(), MarketInfo{
            total_deposit: 0,
            total_withdraw: 0,
            buyer: table::new(),
            uncheck_info: table::new()
        });

        to_shared(event_queue::subscribe<ReceiveUTXOEvent>(to_string(&sender())));
        transfer(admin_cap, sender());
        to_shared(rgas_market_obj)
    }

    public entry fun add_rgas_coin(
        account: &signer,
        rgas_market_obj: &mut Object<RGasMarket>,
        amount: u256
    ){
        let rgas_market = object::borrow_mut(rgas_market_obj);
        assert!(rgas_market.is_open, ErrorMarketNotOpen);
        let rgas_coin = account_coin_store::withdraw<RGas>(account, amount);
        coin_store::deposit(&mut rgas_market.rgas_store, rgas_coin);
        let market_info_mut = borrow_mut_resource<MarketInfo>(address_of(&module_signer<MarketInfo>()));
        market_info_mut.total_deposit = market_info_mut.total_deposit + amount
    }

    public entry fun withdraw_rgas_coin(
        _admin: &mut Object<AdminCap>,
        rgas_market_obj: &mut Object<RGasMarket>,
        amount: u256
    ){
        let rgas_market = object::borrow_mut(rgas_market_obj);
        assert!(rgas_market.is_open, ErrorMarketNotOpen);

        let rgas_coin = coin_store::withdraw<RGas>(&mut rgas_market.rgas_store, amount);
        account_coin_store::deposit(sender(), rgas_coin);
        let market_info_mut = borrow_mut_resource<MarketInfo>(address_of(&module_signer<MarketInfo>()));
        market_info_mut.total_withdraw = market_info_mut.total_withdraw + amount
    }

    public entry fun consume_event(
        rgas_market_obj: &mut Object<RGasMarket>,
        subscriber_obj: &mut Object<Subscriber<ReceiveUTXOEvent>>
    ){
        let rgas_market = object::borrow_mut(rgas_market_obj);
        assert!(rgas_market.is_open, ErrorMarketNotOpen);
        let consume_event = option::extract(&mut consume(subscriber_obj));
        let (txid, sender, receiver, value) = unpack_receive_utxo_event(consume_event);
        assert!(receiver == rgas_market.receive_btc_address, ErrorReceiverAddress);
        let withdraw_amount = btc_to_rgas(value);
        if (option::is_some(&sender)){
            let sender_addr = option::extract(&mut sender);
            let rgas_coin = coin_store::withdraw<RGas>(&mut rgas_market.rgas_store, withdraw_amount);
            account_coin_store::deposit(sender_addr, rgas_coin);
            let market_info_mut = borrow_mut_resource<MarketInfo>(address_of(&module_signer<MarketInfo>()));
            if (!table::contains(&market_info_mut.buyer, sender_addr)){
                table::add(&mut market_info_mut.buyer, sender_addr, withdraw_amount)
            }else {
                let total_amount = *table::borrow(&market_info_mut.buyer, sender_addr) + withdraw_amount;
                table::upsert(&mut market_info_mut.buyer, sender_addr, total_amount);
            }
        }else {
            let market_info_mut = borrow_mut_resource<MarketInfo>(address_of(&module_signer<MarketInfo>()));
            if (!table::contains(&market_info_mut.uncheck_info, txid)){
                table::add(&mut market_info_mut.uncheck_info, txid, withdraw_amount)
            }else {
                let total_amount = *table::borrow(&market_info_mut.uncheck_info, txid) + withdraw_amount;
                table::upsert(&mut market_info_mut.uncheck_info, txid, total_amount);
            }
        }
    }

    public entry fun consume_uncheck(
        _admin: &mut Object<AdminCap>,
        rgas_market_obj: &mut Object<RGasMarket>,
        txid: address,
        sender_addr: address,
        amount: u256
    ){
        let rgas_market = object::borrow_mut(rgas_market_obj);
        assert!(rgas_market.is_open, ErrorMarketNotOpen);
        let rgas_coin = coin_store::withdraw<RGas>(&mut rgas_market.rgas_store, amount);
        account_coin_store::deposit(sender_addr, rgas_coin);
        let market_info_mut = borrow_mut_resource<MarketInfo>(address_of(&module_signer<MarketInfo>()));
        assert!(table::contains(&market_info_mut.uncheck_info, txid), ErrorNoUncheckTxid);
        let remaining_amount = *table::borrow(&market_info_mut.uncheck_info, txid) - amount;
        if (remaining_amount > 0) {
            table::upsert(&mut market_info_mut.uncheck_info, txid, remaining_amount);
        }else {
            let _ = table::remove(&mut market_info_mut.uncheck_info, txid);
        };

        if (!table::contains(&market_info_mut.buyer, sender_addr)){
            table::add(&mut market_info_mut.buyer, sender_addr, amount)
        }else {
            let total_amount = *table::borrow(&market_info_mut.buyer, sender_addr) + amount;
            table::upsert(&mut market_info_mut.buyer, sender_addr, total_amount);
        }
    }

    public entry fun remove_uncheck(
        _admin: &mut Object<AdminCap>,
        txid: address,
    ){
        let market_info_mut = borrow_mut_resource<MarketInfo>(address_of(&module_signer<MarketInfo>()));
        table::remove(&mut market_info_mut.uncheck_info, txid);
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

    #[test]
    fun test_btc_to_rgas(){
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
        let b = rgas_amount * DEFAULT_UNIT_PRICE /  token_price;
        assert!(b == 100000000, 2);
    }


}

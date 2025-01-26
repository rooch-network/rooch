// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
module orderbook::market_v2 {
    use std::option;
    use std::option::{Option, is_some, destroy_none, none, some};
    use std::signer::address_of;
    use std::string;
    use std::string::String;
    use std::vector;
    use std::vector::{length, zip};
    
    use moveos_std::linked_table;
    use moveos_std::linked_table::LinkedTable;
    use moveos_std::object;
    use moveos_std::tx_context::sender;
    use moveos_std::type_info::type_name;
    use moveos_std::object::{Object, ObjectID, to_shared, new_named_object};
    use moveos_std::timestamp::now_milliseconds;
    use moveos_std::table;
    use moveos_std::table::Table;

    use rooch_framework::coin_store::{CoinStore, create_coin_store};
    use rooch_framework::coin::{Self, Coin};
    use rooch_framework::coin_store;
    use rooch_framework::account_coin_store;

    use orderbook::critbit::{CritbitTree, find_leaf, borrow_leaf_by_index, borrow_mut_leaf_by_index,
        remove_leaf_by_index
    };
    use orderbook::critbit;
    use app_admin::admin::AdminCap;
    use moveos_std::event::emit;
    #[test_only]
    use std::u64;

    #[test_only]
    use rooch_framework::account::create_account_for_testing;

    const DEPLOYER: address = @orderbook;


    const VERSION: u64 = 4;


    const BASE_MARKET_FEE: u256 = 20;
    const TRADE_FEE_BASE_RATIO: u256 = 1000;

    const MIN_BID_ORDER_ID: u64 = 1;
    const MIN_ASK_ORDER_ID: u64 = 1 << 63;

    const UNIT_PRICE_SCALE: u256 = 100000;

    const ErrorWrongVersion: u64 = 0;
    const ErrorWrongPaused: u64 = 1;
    const ErrorInputCoin: u64 = 2;
    const ErrorWrongMarket: u64 = 3;
    const ErrorPriceTooLow: u64 = 4;
    const ErrorWrongCreateBid: u64 = 5;
    const ErrorFeeTooHigh: u64 = 6;
    const ErrorInvalidOrderId: u64 = 7;
    const ErrorUnauthorizedCancel: u64 = 8;
    const ErrorOrderLength: u64 = 9;
    const ErrorDeprecated: u64 = 10;
    const ErrorInvalidAmount: u64 = 11;
    const ErrorQuantityTooLow: u64 = 12;


    /// listing info in the market
    struct Order has key, store, drop {
        /// The order id of the order
        order_id: u64,
        /// The unit_price of the order
        unit_price: u64,
        /// the quantity of order
        quantity: u256,
        /// The owner of order
        owner: address,
        /// is bid order or listing order
        is_bid: bool,
    }

    /// Same as Order, but with `copy`
    /// Use this struct as query result
    struct OrderInfo has store, copy, drop{
        order_id: u64,
        unit_price: u64,
        quantity: u256,
        owner: address,
        is_bid: bool,
    }

    struct TickLevel has store {
        price: u64,
        // The key is order order id.
        open_orders: LinkedTable<u64, Order>,
    }

    ///Record some important information of the market
    struct Marketplace<phantom BaseAsset: key + store, phantom QuoteAsset: key + store> has key {
        /// is paused of market
        is_paused: bool,
        /// version of market
        version: u64,
        /// All open bid orders.
        bids: CritbitTree<TickLevel>,
        /// All open ask orders.
        asks: CritbitTree<TickLevel>,
        /// Order id of the next bid order, starting from 0.
        next_bid_order_id: u64,
        /// Order id of the next ask order, starting from 1<<63.
        next_ask_order_id: u64,
        /// Marketplace fee  of the marketplace
        fee: u256,
        /// User order info
        user_order_info: Table<address, LinkedTable<u64, u64>>,
        base_asset: Object<CoinStore<BaseAsset>>,
        quote_asset: Object<CoinStore<QuoteAsset>>,
        /// Stores the trading fees paid in `BaseAsset`.
        base_asset_trading_fees: Object<CoinStore<BaseAsset>>,
        /// Stores the trading fees paid in `QuoteAsset`.
        quote_asset_trading_fees: Object<CoinStore<QuoteAsset>>,
        trade_info: TradeInfo
    }

    struct TradeInfo has store {
        timestamp: u64,
        yesterday_volume: u256,
        today_volume: u256,
        total_volume: u256,
        txs: u64
    }

    // struct AdminCap has key, store {}

    struct MarketplaceHouse has key {
        market_info: LinkedTable<String, ObjectID>,
    }


    struct OrderEvent<phantom BaseAsset: key + store, phantom QuoteAsset: key + store> has copy, drop {
        order_id: u64,
        unit_price: u64,
        quantity: u256,
        owner: address,
        timestamp: u64,
        // 0 is list, 1 is create bid, 2 is cancel bid, 3 is cancel list, 4 is buy, 5 is accept bid
        order_type: u8
    }



    fun init() {
        let market_house = MarketplaceHouse {
            market_info: linked_table::new(),
        };

        //TODO market create event
        // transfer(new_named_object(AdminCap{}), sender());
        to_shared(new_named_object(market_house))
    }

    public entry fun create_market<BaseAsset: key + store, QuoteAsset: key + store>(
        market_house_obj: &mut Object<MarketplaceHouse>,
    ) {
        let market_obj = new_named_object(Marketplace {
            is_paused: false,
            version: VERSION,
            bids: critbit::new(),
            asks: critbit::new(),
            // Order id of the next bid order, starting from 1.
            next_bid_order_id: MIN_BID_ORDER_ID,
            // Order id of the next ask order, starting from 1<<63.
            next_ask_order_id: MIN_ASK_ORDER_ID,
            fee: BASE_MARKET_FEE,
            user_order_info: table::new(),
            base_asset: create_coin_store<BaseAsset>(),
            quote_asset: create_coin_store<QuoteAsset>(),
            base_asset_trading_fees: create_coin_store<BaseAsset>(),
            quote_asset_trading_fees: create_coin_store<QuoteAsset>(),
            trade_info: TradeInfo{
                timestamp: now_milliseconds(),
                yesterday_volume: 0,
                today_volume: 0,
                total_volume: 0,
                txs: 0
            }
        });
        let object_id = object::id(&market_obj);
        let market_house = object::borrow_mut(market_house_obj);
        let type_name = type_name<BaseAsset>();
        string::append(&mut type_name, type_name<QuoteAsset>());
        linked_table::push_back(&mut market_house.market_info, type_name, object_id);
        to_shared(market_obj);
    }

    ///Listing NFT in the collection
    public entry fun list<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        quantity: u256,
        unit_price: u64,
    ) {
        let coin= account_coin_store::withdraw<QuoteAsset>(signer, quantity);
        let market = object::borrow_mut(market_obj);
        // assert!(market.version == VERSION, ErrorWrongVersion);
        assert!(market.is_paused == false, ErrorWrongPaused);
        let order_id = market.next_ask_order_id;
        market.next_ask_order_id = market.next_ask_order_id + 1;
        // TODO here maybe wrap to u512?
        // let price = (unit_price as u256) * quantity;
        assert!(unit_price > 0, ErrorPriceTooLow);
        assert!(quantity > 0, ErrorQuantityTooLow);
        let asks = Order {
            order_id,
            unit_price,
            quantity,
            owner: address_of(signer),
            is_bid: false,
        };
        coin_store::deposit(&mut market.quote_asset, coin);
        let (find_price, index) = critbit::find_leaf(&market.asks, unit_price);
        if (!find_price) {
            index = critbit::insert_leaf(&mut market.asks, unit_price, TickLevel{
                price: unit_price,
                open_orders: linked_table::new()
            });
        };
        let tick_level = critbit::borrow_mut_leaf_by_index(&mut market.asks, index);
        linked_table::push_back(&mut tick_level.open_orders, order_id, asks);
        //
        if (!table::contains(&market.user_order_info, address_of(signer))) {
            table::add(&mut market.user_order_info, address_of(signer), linked_table::new());
        };
        linked_table::push_back(table::borrow_mut(&mut market.user_order_info, address_of(signer)), order_id, unit_price);
        emit(OrderEvent<BaseAsset, QuoteAsset> {
            order_id,
            unit_price,
            quantity,
            owner: address_of(signer),
            timestamp: now_milliseconds(),
            order_type: 0
        })
    }

    public entry fun create_bid<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        unit_price: u64,
        quantity: u256,
    ) {
        let market = object::borrow_mut(market_obj);
        // assert!(market.version == VERSION, ErrorWrongVersion);
        assert!(market.is_paused == false, ErrorWrongPaused);
        assert!(quantity > 0, ErrorQuantityTooLow);
        assert!(unit_price > 0, ErrorWrongCreateBid);
        // TODO here maybe wrap to u512?
        let price = (unit_price as u256) * quantity / UNIT_PRICE_SCALE;
        let paid = account_coin_store::withdraw<BaseAsset>(signer, price);
        let order_id = market.next_bid_order_id;
        market.next_bid_order_id = market.next_bid_order_id + 1;
        let bid = Order {
            order_id,
            unit_price,
            quantity,
            owner: address_of(signer),
            is_bid: true,
        };
        coin_store::deposit(&mut market.base_asset, paid);

        let (find_price, index) = critbit::find_leaf(&market.bids, unit_price);
        if (!find_price) {
            index = critbit::insert_leaf(&mut market.bids, unit_price, TickLevel {
                price: unit_price,
                open_orders: linked_table::new()
            });
        };
        let tick_level = critbit::borrow_mut_leaf_by_index(&mut market.bids, index);
        linked_table::push_back(&mut tick_level.open_orders, order_id, bid);
        if (!table::contains(&market.user_order_info, address_of(signer))) {
            table::add(&mut market.user_order_info, address_of(signer), linked_table::new());
        };
        linked_table::push_back(table::borrow_mut(&mut market.user_order_info, address_of(signer)), order_id, unit_price);
        emit(OrderEvent<BaseAsset, QuoteAsset> {
            order_id,
            unit_price,
            quantity,
            owner: address_of(signer),
            timestamp: now_milliseconds(),
            order_type: 1
        })
    }

    ///Cancel the listing of inscription
    public entry fun cancel_order<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
    ) {
        //Get the list from the collection
        let market = object::borrow_mut(market_obj);
        // assert!(market.version == VERSION, ErrorWrongVersion);

        let usr_open_orders = table::borrow_mut(&mut market.user_order_info, sender());
        let tick_price = *linked_table::borrow(usr_open_orders, order_id);
        let is_bid = order_is_bid(order_id);
        let (tick_exists, tick_index) = find_leaf(if (is_bid) { &market.bids } else { &market.asks }, tick_price);
        assert!(tick_exists, ErrorInvalidOrderId);
        let order = remove_order(
            if (is_bid) { &mut market.bids } else { &mut market.asks },
            usr_open_orders,
            tick_index,
            order_id,
            sender()
        );
        if (is_bid) {
            // TODO here maybe wrap to u512?
            let total_balance = (order.unit_price as u256) * order.quantity / UNIT_PRICE_SCALE;
            account_coin_store::deposit(sender(), coin_store::withdraw(&mut market.base_asset, total_balance));
            emit(OrderEvent<BaseAsset, QuoteAsset> {
                order_id,
                unit_price: order.unit_price,
                quantity: order.quantity,
                owner: order.owner,
                timestamp: now_milliseconds(),
                order_type: 2
            })
        }else {
            account_coin_store::deposit(sender(), coin_store::withdraw(&mut market.quote_asset, order.quantity));
            emit(OrderEvent<BaseAsset, QuoteAsset> {
                order_id,
                unit_price: order.unit_price,
                quantity: order.quantity,
                owner: order.owner,
                timestamp: now_milliseconds(),
                order_type: 3
            })
        }
    }

    public entry fun batch_buy<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_ids: vector<u64>,
        order_owners: vector<address>,
        assert_order_exist: bool,
        receiver: address
    ){
        assert!(length(&order_ids) == length(&order_owners), ErrorOrderLength);
        zip(order_ids, order_owners, |order_id, order_owner|{
            buy(signer, market_obj, order_id, order_owner, assert_order_exist, receiver)
        })
    }

    public entry fun buy<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        order_owner: address,
        assert_order_exist: bool,
        receiver: address
    ){
        let option_coin = do_buy<BaseAsset, QuoteAsset>(signer, market_obj, order_id, order_owner, assert_order_exist);
        if (is_some(&option_coin)) {
            account_coin_store::deposit(receiver, option::extract(&mut option_coin))
        };
        destroy_none(option_coin)
    }

    ///purchase
    public fun do_buy<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        order_owner: address,
        assert_order_exist: bool,
    ): Option<Coin<QuoteAsset>> {
        let market = object::borrow_mut(market_obj);
        assert!(market.is_paused == false, ErrorWrongPaused);
        // assert!(market.version == VERSION, ErrorWrongVersion);
        let usr_open_orders = table::borrow_mut(&mut market.user_order_info, order_owner);
        let tick_price = *linked_table::borrow(usr_open_orders, order_id);
        let (tick_exists, tick_index) = find_leaf(&market.asks, tick_price);
        // Return non-existent orders to none instead of panic during bulk buying
        if (!assert_order_exist && !tick_exists) {
            return option::none()
        };
        assert!(tick_exists, ErrorInvalidOrderId);
        let order = remove_order(&mut market.asks, usr_open_orders, tick_index, order_id, order_owner);
        // TODO here maybe wrap to u512?
        let total_price = order.quantity * (order.unit_price as u256) / UNIT_PRICE_SCALE;
        let trade_coin = account_coin_store::withdraw<BaseAsset>(signer, total_price);
        let trade_info = &mut market.trade_info;
        trade_info.total_volume = trade_info.total_volume + total_price;
        trade_info.txs = trade_info.txs + 1;
        if (now_milliseconds() - trade_info.timestamp > 86400000) {
            trade_info.yesterday_volume = trade_info.today_volume;
            trade_info.today_volume = total_price;
            trade_info.timestamp = now_milliseconds();
        }else {
            trade_info.today_volume = trade_info.today_volume + total_price;
        };

        // TODO here maybe wrap to u512?
        // Here is trade fee is BaseAsset
        let trade_fee = total_price * market.fee / TRADE_FEE_BASE_RATIO;
        coin_store::deposit(&mut market.base_asset_trading_fees, coin::extract(&mut trade_coin, trade_fee));
        account_coin_store::deposit(order.owner, trade_coin);
        emit(OrderEvent<BaseAsset, QuoteAsset> {
            order_id,
            unit_price: order.unit_price,
            quantity: order.quantity,
            owner: order.owner,
            timestamp: now_milliseconds(),
            order_type: 4
        });
        option::some(coin_store::withdraw(&mut market.quote_asset, order.quantity))

    }


    public entry fun buy_from_origin<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        amount: u256,
        order_owner: address,
        assert_order_exist: bool,
        receiver: address
    ){
        let option_coin = do_buy_external<BaseAsset, QuoteAsset>(signer, market_obj, order_id, amount, order_owner, assert_order_exist, none());
        if (is_some(&option_coin)) {
            account_coin_store::deposit(receiver, option::extract(&mut option_coin))
        };
        destroy_none(option_coin)
    }

    public entry fun buy_from_distributor<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        amount: u256,
        order_owner: address,
        assert_order_exist: bool,
        receiver: address,
        distributor: address
    ){
        let option_coin = do_buy_external<BaseAsset, QuoteAsset>(signer, market_obj, order_id, amount, order_owner, assert_order_exist, some(distributor));
        if (is_some(&option_coin)) {
            account_coin_store::deposit(receiver, option::extract(&mut option_coin))
        };
        destroy_none(option_coin)
    }


    public fun do_buy_external<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        amount: u256,
        order_owner: address,
        assert_order_exist: bool,
        distributor: Option<address>
    ): Option<Coin<QuoteAsset>> {
        let market = object::borrow_mut(market_obj);
        assert!(market.is_paused == false, ErrorWrongPaused);
        // assert!(market.version == VERSION, ErrorWrongVersion);
        let usr_open_orders = table::borrow_mut(&mut market.user_order_info, order_owner);
        let tick_price = *linked_table::borrow(usr_open_orders, order_id);
        let (tick_exists, tick_index) = find_leaf(&market.asks, tick_price);
        // Return non-existent orders to none instead of panic during bulk buying
        if (!assert_order_exist && !tick_exists) {
            return option::none()
        };
        assert!(tick_exists, ErrorInvalidOrderId);
        assert!(amount > 0, ErrorQuantityTooLow);
        let order = borrow_mut_order(&mut market.asks, usr_open_orders, tick_index, order_id, order_owner);
        assert!(amount <= order.quantity, ErrorInvalidAmount);
        // TODO here maybe wrap to u512?
        let total_price = amount * (order.unit_price as u256) / UNIT_PRICE_SCALE;
        let trade_coin = account_coin_store::withdraw<BaseAsset>(signer, total_price);
        let trade_info = &mut market.trade_info;
        trade_info.total_volume = trade_info.total_volume + total_price;
        trade_info.txs = trade_info.txs + 1;
        if (now_milliseconds() - trade_info.timestamp > 86400000) {
            trade_info.yesterday_volume = trade_info.today_volume;
            trade_info.today_volume = total_price;
            trade_info.timestamp = now_milliseconds();
        }else {
            trade_info.today_volume = trade_info.today_volume + total_price;
        };

        // TODO here maybe wrap to u512?
        // Here is trade fee is BaseAsset
        let trade_fee = total_price * market.fee / TRADE_FEE_BASE_RATIO;
        if (option::is_some(&distributor)){
            let distributor_address = option::extract(&mut distributor);
            let trade_fee_coin = coin::extract(&mut trade_coin, trade_fee);
            let distributor_fee = trade_fee / 2;
            account_coin_store::deposit(distributor_address, coin::extract(&mut trade_fee_coin, distributor_fee));
            coin_store::deposit(&mut market.base_asset_trading_fees, trade_fee_coin);
        }else{
            coin_store::deposit(&mut market.base_asset_trading_fees, coin::extract(&mut trade_coin, trade_fee));
        };
        account_coin_store::deposit(order.owner, trade_coin);
        emit(OrderEvent<BaseAsset, QuoteAsset> {
            order_id,
            unit_price: order.unit_price,
            quantity: order.quantity,
            owner: order.owner,
            timestamp: now_milliseconds(),
            order_type: 4
        });
        order.quantity = order.quantity - amount;
        if (order.quantity == 0 ) {
            let _ = remove_order(&mut market.asks, usr_open_orders, tick_index, order_id, order_owner);
        };
        option::some(coin_store::withdraw(&mut market.quote_asset, amount))
    }


    public entry fun batch_accept_bid<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_ids: vector<u64>,
        order_owners: vector<address>,
        assert_order_exist: bool,
        receiver: address
    ){
        zip(order_ids, order_owners,|order_id, order_owner|{
            accept_bid(signer, market_obj, order_id, order_owner, assert_order_exist, receiver)
        })
    }


    public entry fun accept_bid<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        order_owner: address,
        assert_order_exist: bool,
        receiver: address
    ){
        let option_coin = do_accept_bid<BaseAsset, QuoteAsset>(signer, market_obj, order_id, order_owner, assert_order_exist);
        if (is_some(&option_coin)) {
            account_coin_store::deposit(receiver, option::extract(&mut option_coin))
        };
        destroy_none(option_coin)
    }

    public fun do_accept_bid<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        order_owner: address,
        assert_order_exist: bool,
        // paid: &mut Coin<QuoteAsset>
    ): Option<Coin<BaseAsset>>
    {
        let market = object::borrow_mut(market_obj);
        assert!(market.is_paused == false, ErrorWrongPaused);
        // assert!(market.version == VERSION, ErrorWrongVersion);
        let usr_open_orders = table::borrow_mut(&mut market.user_order_info, order_owner);
        let tick_price = *linked_table::borrow(usr_open_orders, order_id);
        let (tick_exists, tick_index) = find_leaf(&market.bids, tick_price);
        // Return non-existent orders to none instead of panic during bulk buying
        if (!assert_order_exist && !tick_exists) {
            return option::none()
        };
        assert!(tick_exists, ErrorInvalidOrderId);

        let order = remove_order(&mut market.bids, usr_open_orders, tick_index, order_id, order_owner);
        let trade_coin = account_coin_store::withdraw<QuoteAsset>(signer, order.quantity);
        // assert!(coin::value(paid) >=  order.quantity, ErrorInputCoin);
        // let trade_coin = coin::extract(paid, order.quantity);
        // TODO here maybe wrap to u512?
        let total_price = (order.unit_price as u256) * order.quantity / UNIT_PRICE_SCALE;
        // let total_price = (order.unit_price as u256) * order.quantity;
        let trade_info = &mut market.trade_info;

        trade_info.total_volume = trade_info.total_volume + total_price;
        trade_info.txs = trade_info.txs + 1;
        if (now_milliseconds() - trade_info.timestamp > 86400000) {
            trade_info.yesterday_volume = trade_info.today_volume;
            trade_info.today_volume = total_price;
            trade_info.timestamp = now_milliseconds();
        }else {
            trade_info.today_volume = trade_info.today_volume + total_price;
        };

        // Here trade fee is QuoteAsset
        let trade_fee = order.quantity * market.fee / TRADE_FEE_BASE_RATIO;
        coin_store::deposit(&mut market.quote_asset_trading_fees, coin::extract(&mut trade_coin, trade_fee));
        account_coin_store::deposit(order.owner, trade_coin);
        emit(OrderEvent<BaseAsset, QuoteAsset> {
            order_id,
            unit_price: order.unit_price,
            quantity: order.quantity,
            owner: order.owner,
            timestamp: now_milliseconds(),
            order_type: 5
        });
        option::some(coin_store::withdraw(&mut market.base_asset, total_price))
    }


    public entry fun accept_bid_from_origin<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        amount: u256,
        order_owner: address,
        assert_order_exist: bool,
        receiver: address
    ){
        let option_coin = do_accept_bid_external<BaseAsset, QuoteAsset>(signer, market_obj, order_id, amount, order_owner, assert_order_exist, none());
        if (is_some(&option_coin)) {
            account_coin_store::deposit(receiver, option::extract(&mut option_coin))
        };
        destroy_none(option_coin)
    }

    public entry fun accept_bid_from_distributor<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        amount: u256,
        order_owner: address,
        assert_order_exist: bool,
        receiver: address,
        distributor: address
    ){
        let option_coin = do_accept_bid_external<BaseAsset, QuoteAsset>(signer, market_obj, order_id, amount, order_owner, assert_order_exist, some(distributor));
        if (is_some(&option_coin)) {
            account_coin_store::deposit(receiver, option::extract(&mut option_coin))
        };
        destroy_none(option_coin)
    }

    public fun do_accept_bid_external<BaseAsset: key + store, QuoteAsset: key + store>(
        signer: &signer,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        amount: u256,
        order_owner: address,
        assert_order_exist: bool,
        distributor: Option<address>
    ): Option<Coin<BaseAsset>>
    {
        let market = object::borrow_mut(market_obj);
        assert!(market.is_paused == false, ErrorWrongPaused);
        // assert!(market.version == VERSION, ErrorWrongVersion);
        let usr_open_orders = table::borrow_mut(&mut market.user_order_info, order_owner);
        let tick_price = *linked_table::borrow(usr_open_orders, order_id);
        let (tick_exists, tick_index) = find_leaf(&market.bids, tick_price);
        // Return non-existent orders to none instead of panic during bulk buying
        if (!assert_order_exist && !tick_exists) {
            return option::none()
        };
        assert!(tick_exists, ErrorInvalidOrderId);
        assert!(amount > 0, ErrorQuantityTooLow);
        let order = borrow_mut_order(&mut market.bids, usr_open_orders, tick_index, order_id, order_owner);
        assert!(order.quantity >= amount, ErrorInvalidAmount);
        let trade_coin = account_coin_store::withdraw<QuoteAsset>(signer, amount);
        // TODO here maybe wrap to u512?
        let total_price = (order.unit_price as u256) * amount / UNIT_PRICE_SCALE;
        // let total_price = (order.unit_price as u256) * amount;
        let trade_info = &mut market.trade_info;

        trade_info.total_volume = trade_info.total_volume + total_price;
        trade_info.txs = trade_info.txs + 1;
        if (now_milliseconds() - trade_info.timestamp > 86400000) {
            trade_info.yesterday_volume = trade_info.today_volume;
            trade_info.today_volume = total_price;
            trade_info.timestamp = now_milliseconds();
        }else {
            trade_info.today_volume = trade_info.today_volume + total_price;
        };

        // Here trade fee is QuoteAsset
        let trade_fee = amount * market.fee / TRADE_FEE_BASE_RATIO;
        if (option::is_some(&distributor)){
            let distributor_address = option::extract(&mut distributor);
            let trade_fee_coin = coin::extract(&mut trade_coin, trade_fee);
            let distributor_fee = trade_fee / 2;
            account_coin_store::deposit(distributor_address, coin::extract(&mut trade_fee_coin, distributor_fee));
            coin_store::deposit(&mut market.quote_asset_trading_fees, trade_fee_coin);
        }else{
            coin_store::deposit(&mut market.quote_asset_trading_fees, coin::extract(&mut trade_coin, trade_fee));
        };
        account_coin_store::deposit(order.owner, trade_coin);
        emit(OrderEvent<BaseAsset, QuoteAsset> {
            order_id,
            unit_price: order.unit_price,
            quantity: order.quantity,
            owner: order.owner,
            timestamp: now_milliseconds(),
            order_type: 5
        });
        order.quantity = order.quantity - amount;
        if (order.quantity == 0) {
            let _ = remove_order(&mut market.bids, usr_open_orders, tick_index, order_id, order_owner);
        };

        option::some(coin_store::withdraw(&mut market.base_asset, total_price))
    }



    public entry fun withdraw_profits<BaseAsset: key + store, QuoteAsset: key + store>(
        _admin: &mut Object<AdminCap>,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        receiver: address,
    ) {
        let market = object::borrow_mut(market_obj);
        // assert!(market.version == VERSION, ErrorWrongVersion);
        let quote_amount = coin_store::balance(&market.quote_asset_trading_fees);
        account_coin_store::deposit(receiver, coin_store::withdraw(&mut market.quote_asset_trading_fees, quote_amount));
        let base_amount = coin_store::balance(&market.base_asset_trading_fees);
        account_coin_store::deposit(receiver, coin_store::withdraw(&mut market.base_asset_trading_fees, base_amount));
    }


    public entry fun update_market_fee<BaseAsset: key + store, QuoteAsset: key + store>(
        _admin: &mut Object<AdminCap>,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        fee: u256,
    ) {
        let market = object::borrow_mut(market_obj);
        // assert!(market.version == VERSION, ErrorWrongVersion);
        assert!(fee < TRADE_FEE_BASE_RATIO, ErrorFeeTooHigh);
        market.fee = fee
    }

    public entry fun update_market_status<BaseAsset: key + store, QuoteAsset: key + store>(
        _admin: &mut Object<AdminCap>,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        status: bool,
    ) {
        let market = object::borrow_mut(market_obj);
        // assert!(market.version == VERSION, ErrorWrongVersion);
        market.is_paused = status
    }

    public entry fun migrate_marketplace<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
    ) {
        let market = object::borrow_mut(market_obj);
        assert!(market.version <= VERSION, ErrorWrongVersion);
        market.version = VERSION;
    }

    fun remove_order(
        open_orders: &mut CritbitTree<TickLevel>,
        user_order_info: &mut LinkedTable<u64, u64>,
        tick_index: u64,
        order_id: u64,
        user: address,
    ): Order {
        linked_table::remove(user_order_info, order_id);
        let tick_level = borrow_leaf_by_index(open_orders, tick_index);
        assert!(linked_table::contains(&tick_level.open_orders, order_id), ErrorInvalidOrderId);
        let mut_tick_level = borrow_mut_leaf_by_index(open_orders, tick_index);
        let order = linked_table::remove(&mut mut_tick_level.open_orders, order_id);
        assert!(order.owner == user, ErrorUnauthorizedCancel);
        if (linked_table::is_empty(&mut_tick_level.open_orders)) {
            destroy_empty_level(remove_leaf_by_index(open_orders, tick_index));
        };
        order
    }


    fun borrow_mut_order(
        open_orders: &mut CritbitTree<TickLevel>,
        _user_order_info: &mut LinkedTable<u64, u64>,
        tick_index: u64,
        order_id: u64,
        user: address,
    ): &mut Order {
        // linked_table::remove(user_order_info, order_id);
        let tick_level = borrow_leaf_by_index(open_orders, tick_index);
        assert!(linked_table::contains(&tick_level.open_orders, order_id), ErrorInvalidOrderId);
        let mut_tick_level = borrow_mut_leaf_by_index(open_orders, tick_index);
        let order = linked_table::borrow_mut(&mut mut_tick_level.open_orders, order_id);
        assert!(order.owner == user, ErrorUnauthorizedCancel);
        // if (linked_table::is_empty(&mut_tick_level.open_orders)) {
        //     destroy_empty_level(remove_leaf_by_index(open_orders, tick_index));
        // };
        order
    }

    fun destroy_empty_level(level: TickLevel) {
        let TickLevel {
            price: _,
            open_orders: orders,
        } = level;

        linked_table::destroy_empty(orders);
    }

    public fun query_order_info<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &Object<Marketplace<BaseAsset, QuoteAsset>>,
        query_bid: bool,
        from_price: u64,
        from_price_is_none: bool,
        start_order_id: u64
    ): vector<OrderInfo> {

        let market = object::borrow(market_obj);
        let order_infos = vector::empty<OrderInfo>();

        if (query_bid) {
            if (critbit::is_empty(&market.bids)) {
                return order_infos
            };
            let i = 0;
            let from = if (from_price_is_none) {
                let (key, _) = critbit::max_leaf(&market.bids);
                key
            }else {
                from_price
            };
            while (i < 50) {
                let tick_level = critbit::borrow_leaf_by_key(&market.bids, from);
                let order_count = linked_table::length(&tick_level.open_orders);
                if (order_count == 0) {
                    return order_infos
                };
                let option_order_id = if (!linked_table::contains(&tick_level.open_orders, start_order_id)){
                    linked_table::front(&tick_level.open_orders)
                }else {
                    &option::some(start_order_id)
                };
                while (option::is_some(option_order_id)) {
                    let order_id = option::destroy_some(*option_order_id);
                    let order = linked_table::borrow(&tick_level.open_orders, order_id);
                    vector::push_back(&mut order_infos, OrderInfo {
                        order_id: order.order_id,
                        unit_price: order.unit_price,
                        quantity: order.quantity,
                        owner: order.owner,
                        is_bid: order.is_bid
                    });
                    i = i + 1;
                    if (i >= 50) {
                        return order_infos
                    };
                    option_order_id = linked_table::next(&tick_level.open_orders, order_id)
                };
                let (key, index) = critbit::previous_leaf(&market.bids, from);
                if (index != 0x8000000000000000) {
                    from = key;
                }else {
                    return order_infos
                }
            };
        }else {
            if (critbit::is_empty(&market.asks)) {
                return order_infos
            };
            let i = 0;
            let from = if (from_price_is_none) {
                let (key, _) = critbit::min_leaf(&market.asks);
                key
            }else {
                from_price
            };

            while (i < 50) {
                let tick_level = critbit::borrow_leaf_by_key(&market.asks, from);
                let order_count = linked_table::length(&tick_level.open_orders);
                if (order_count == 0) {
                    return order_infos
                };
                let option_order_id = if (!linked_table::contains(&tick_level.open_orders, start_order_id)){
                    linked_table::front(&tick_level.open_orders)
                }else {
                    &option::some(start_order_id)
                };
                while (option::is_some(option_order_id)) {
                    let order_id = option::destroy_some(*option_order_id);
                    let order = linked_table::borrow(&tick_level.open_orders, order_id);
                    vector::push_back(&mut order_infos, OrderInfo {
                        order_id: order.order_id,
                        unit_price: order.unit_price,
                        quantity: order.quantity,
                        owner: order.owner,
                        is_bid: order.is_bid
                    });
                    i = i + 1;
                    if (i >= 50) {
                        return order_infos
                    };
                    option_order_id = linked_table::next(&tick_level.open_orders, order_id)
                };
                let (key, index) = critbit::next_leaf(&market.asks, from);
                if (index != 0x8000000000000000) {
                    from = key;
                }else {
                    return order_infos
                }
            };
        };
        order_infos
    }

    public fun query_order<BaseAsset: key + store, QuoteAsset: key + store>(
        _market_obj: &Object<Marketplace<BaseAsset, QuoteAsset>>,
        _query_bid: bool,
        _from_price: u64,
        _from_price_is_none: bool,
        _start_order_id: u64
    ): (vector<u64>, vector<u64>, vector<u256>, vector<address>, vector<bool>) {
        abort ErrorDeprecated
    }

    public fun query_user_order_info<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &Object<Marketplace<BaseAsset, QuoteAsset>>,
        user: address,
        from_order: u64,
        from_order_is_none: bool,
        count: u64
    ): vector<OrderInfo> {
        let market = object::borrow(market_obj);
        let order_infos = vector::empty<OrderInfo>();
        if (!table::contains(&market.user_order_info, user)) {
            return order_infos
        };
        let user_table = table::borrow(&market.user_order_info, user);
        let from = if (from_order_is_none) {
            *option::borrow(linked_table::front(user_table))
        }else {
            from_order
        };
        let i = 0;
        while (i < count) {
            let tick_price = *linked_table::borrow(user_table, from);

            let is_bid = order_is_bid(from);
            let open_orders = if (is_bid) { &market.bids } else { &market.asks };
            let (tick_exists, tick_index) = find_leaf(open_orders, tick_price);
            if (tick_exists) {
                let tick_level = borrow_leaf_by_index(open_orders, tick_index);
                let order = linked_table::borrow(&tick_level.open_orders, from);
                vector::push_back(&mut order_infos, OrderInfo {
                    order_id: order.order_id,
                    unit_price: order.unit_price,
                    quantity: order.quantity,
                    owner: order.owner,
                    is_bid: order.is_bid
                });
                i = i + 1;
            }else {
                break
            };
            if (option::is_some(linked_table::next(user_table, from))){
                from = *option::borrow(linked_table::next(user_table, from));
            }else {
                break
            }
        };
        order_infos
    }

    public fun query_user_order<BaseAsset: key + store, QuoteAsset: key + store>(
        _market_obj: &Object<Marketplace<BaseAsset, QuoteAsset>>,
        _user: address,
        _from_order: u64,
        _from_order_is_none: bool,
        _count: u64
    ): (vector<u64>, vector<u64>, vector<u256>, vector<address>, vector<bool>) {
        abort ErrorDeprecated
    }


    fun order_is_bid(order_id: u64): bool {
        return order_id < MIN_ASK_ORDER_ID
    }

    #[test_only]
    struct TestBaseCoin has key, store{}
    #[test_only]
    struct TestQuoteCoin has key, store{}

    #[test_only]
    fun init_for_test(base_decimal: u8, quote_decimal: u8):(Object<Marketplace<TestBaseCoin, TestQuoteCoin>>, Coin<TestBaseCoin>, Coin<TestQuoteCoin>) {
        let base_coin_info = coin::register_extend<TestBaseCoin>(
        string::utf8(b"Test Base Coin"),
            string::utf8(b"TBC"),
            option::none(),
            base_decimal,
        );
        let quote_coin_info = coin::register_extend<TestQuoteCoin>(
        string::utf8(b"Test Quote Coin"),
            string::utf8(b"TQC"),
            option::none(),
            quote_decimal,
        );
        let base_coin = coin::mint_extend<TestBaseCoin>(&mut base_coin_info, (1000 * u64::pow(10, base_decimal) as u256));
        let quote_coin = coin::mint_extend<TestQuoteCoin>(&mut quote_coin_info, (1000 * u64::pow(10, quote_decimal) as u256));
        to_shared(base_coin_info);
        to_shared(quote_coin_info);
        let market_obj = new_named_object(Marketplace {
            is_paused: false,
            version: VERSION,
            bids: critbit::new(),
            asks: critbit::new(),
            // Order id of the next bid order, starting from 0.
            next_bid_order_id: MIN_BID_ORDER_ID,
            // Order id of the next ask order, starting from 1<<63.
            next_ask_order_id: MIN_ASK_ORDER_ID,
            fee: 0,
            user_order_info: table::new(),
            base_asset: create_coin_store<TestBaseCoin>(),
            quote_asset: create_coin_store<TestQuoteCoin>(),
            base_asset_trading_fees: create_coin_store<TestBaseCoin>(),
            quote_asset_trading_fees: create_coin_store<TestQuoteCoin>(),
            trade_info: TradeInfo{
                timestamp: now_milliseconds(),
                yesterday_volume: 0,
                today_volume: 0,
                total_volume: 0,
                txs: 0
            }
        });
        (market_obj, base_coin, quote_coin)
    }

    #[test]
    public fun test_buy_1() {
        rooch_framework::genesis::init_for_test();
        let base_decimal = 0;
        let quote_decimal = 0;
        let account_list = create_account_for_testing(@0x43);
        let address_list = address_of(&account_list);
        let account_buy = create_account_for_testing(@0x44);
        let address_buy = address_of(&account_buy);
        let (market_obj, base_coin, quote_coin )= init_for_test(base_decimal, quote_decimal);
        account_coin_store::deposit(address_list, quote_coin);
        account_coin_store::deposit(address_buy, base_coin);
        // Here we list it at price 10 base/quote, and list 10 quote coin
        list(&account_list, &mut market_obj, (10 * u64::pow(10, quote_decimal) as u256), (10 * UNIT_PRICE_SCALE as u64) / u64::pow(10, quote_decimal));
        buy(&account_buy, &mut market_obj, MIN_ASK_ORDER_ID, address_list, true, address_buy);
        to_shared(market_obj);
        // list account will recieve 10 * 10 = 100 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_list) == 100, 0);
        // buy account will recieve 10 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_buy) == 10, 1);
        // list account will pay 10 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_list) == 990, 2);
        // buy account will pay 10 * 10 = 100 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_buy) == 900, 3);
    }

    #[test]
    public fun test_buy_2() {
        rooch_framework::genesis::init_for_test();
        let base_decimal = 8;
        let quote_decimal = 0;
        let account_list = create_account_for_testing(@0x43);
        let address_list = address_of(&account_list);
        let account_buy = create_account_for_testing(@0x44);
        let address_buy = address_of(&account_buy);
        let (market_obj, base_coin, quote_coin )= init_for_test(base_decimal, quote_decimal);
        account_coin_store::deposit(address_list, quote_coin);
        account_coin_store::deposit(address_buy, base_coin);
        // Here we list it at price 10 base/quote, and list 10.0 quote coin
        list(&account_list, &mut market_obj, (10 * u64::pow(10, quote_decimal) as u256), ((UNIT_PRICE_SCALE as u64) / u64::pow(10, quote_decimal)) * (10 * u64::pow(10, base_decimal)));
        buy(&account_buy, &mut market_obj, MIN_ASK_ORDER_ID, address_list, true, address_buy);
        to_shared(market_obj);
        // list account will recieve 10 * 10 = 100.0000000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_list) == 100_00000000, 0);
        // buy account will recieve 10 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_buy) == 10, 1);
        // list account will pay 10 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_list) == 990, 2);
        // buy account will pay 10 * 10 = 100.0000000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_buy) == 900_00000000, 3);
    }


    #[test]
    public fun test_buy_3() {
        rooch_framework::genesis::init_for_test();
        let base_decimal = 10;
        let quote_decimal = 3;
        let account_list = create_account_for_testing(@0x43);
        let address_list = address_of(&account_list);
        let account_buy = create_account_for_testing(@0x44);
        let address_buy = address_of(&account_buy);
        let (market_obj, base_coin, quote_coin )= init_for_test(base_decimal, quote_decimal);
        account_coin_store::deposit(address_list, quote_coin);
        account_coin_store::deposit(address_buy, base_coin);
        // Here we list it at price 10 base/quote, and list 10.000 quote coin
        list(&account_list, &mut market_obj, (10 * u64::pow(10, quote_decimal) as u256), ((UNIT_PRICE_SCALE as u64) / u64::pow(10, quote_decimal) * 10 * u64::pow(10, base_decimal)));
        buy(&account_buy, &mut market_obj, MIN_ASK_ORDER_ID, address_list, true, address_buy);
        to_shared(market_obj);
        // list account will recieve 10 * 10 = 100.0000000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_list) == 100_0000000000, 0);
        // buy account will recieve 10.000 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_buy) == 10000, 1);
        // list account will pay 10.000 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_list) == 990_000, 2);
        // buy account will pay 10 * 10 = 100.0000000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_buy) == 900_0000000000, 3);
    }


    // u64 max >= 1 * base_decimal * price * UNIT_PRICE_SCALE >  quote_decimal

    #[test]
    public fun test_buy_4() {
        rooch_framework::genesis::init_for_test();
        let base_decimal = 8;
        let quote_decimal = 10;
        let account_list = create_account_for_testing(@0x43);
        let address_list = address_of(&account_list);
        let account_buy = create_account_for_testing(@0x44);
        let address_buy = address_of(&account_buy);
        let (market_obj, base_coin, quote_coin )= init_for_test(base_decimal, quote_decimal);
        account_coin_store::deposit(address_list, quote_coin);
        account_coin_store::deposit(address_buy, base_coin);
        // Here we list it at price 10 base/quote, and list 10.000 quote coin
        list(&account_list, &mut market_obj, (10 * u64::pow(10, quote_decimal) as u256), ((UNIT_PRICE_SCALE as u64) * 10 * u64::pow(10, base_decimal) / u64::pow(10, quote_decimal) ));
        buy(&account_buy, &mut market_obj, MIN_ASK_ORDER_ID, address_list, true, address_buy);
        to_shared(market_obj);
        // list account will recieve 10 * 10 = 100.00000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_list) == 100_00000000, 0);
        // buy account will recieve 10.0000000000 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_buy) == 10_0000000000, 1);
        // list account will pay 10.0000000000 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_list) == 990_0000000000, 2);
        // buy account will pay 10 * 10 = 100.00000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_buy) == 900_00000000, 3);
    }

    #[test]
    public fun test_buy_5() {
        rooch_framework::genesis::init_for_test();
        let base_decimal = 8;
        let quote_decimal = 10;
        let account_list = create_account_for_testing(@0x43);
        let address_list = address_of(&account_list);
        let account_buy = create_account_for_testing(@0x44);
        let address_buy = address_of(&account_buy);
        let (market_obj, base_coin, quote_coin )= init_for_test(base_decimal, quote_decimal);
        account_coin_store::deposit(address_list, quote_coin);
        account_coin_store::deposit(address_buy, base_coin);
        // Here we list it at price 10 base/quote, and list 10.000 quote coin
        list(&account_list, &mut market_obj, (10 * u64::pow(10, quote_decimal) as u256), ((UNIT_PRICE_SCALE as u64) * 10 * u64::pow(10, base_decimal) / u64::pow(10, quote_decimal) ));
        buy_from_origin(&account_buy, &mut market_obj, MIN_ASK_ORDER_ID,
            (5 * u64::pow(10, quote_decimal) as u256), address_list, true, address_buy);
        // list account will recieve 10 * 5 = 50.00000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_list) == 50_00000000, 0);
        // buy account will recieve 5.0000000000 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_buy) == 5_0000000000, 1);
        // list account will pay 10.0000000000 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_list) == 990_0000000000, 2);
        // buy account will pay 10 * 5 = 50.00000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_buy) == 950_00000000, 3);
        buy_from_origin(&account_buy, &mut market_obj, MIN_ASK_ORDER_ID,
            (5 * u64::pow(10, quote_decimal) as u256), address_list, true, address_buy);
        to_shared(market_obj);
    }

    #[test]
    public fun test_accept_bid() {
        rooch_framework::genesis::init_for_test();
        let base_decimal = 8;
        let quote_decimal = 10;
        let account_create_bid = create_account_for_testing(@0x43);
        let address_create_bid = address_of(&account_create_bid);
        let account_accept_bid = create_account_for_testing(@0x44);
        let address_accept_bid = address_of(&account_accept_bid);
        let (market_obj, base_coin, quote_coin )= init_for_test(base_decimal, quote_decimal);
        account_coin_store::deposit(address_accept_bid, quote_coin);
        account_coin_store::deposit(address_create_bid, base_coin);
        // Here we list it at price 10 base/quote, and list 10.000 quote coin
        create_bid(&account_create_bid, &mut market_obj,((UNIT_PRICE_SCALE as u64) * 10 * u64::pow(10, base_decimal) / u64::pow(10, quote_decimal) ), (10 * u64::pow(10, quote_decimal) as u256), );
        accept_bid_from_origin(&account_accept_bid, &mut market_obj, MIN_BID_ORDER_ID,
            (5 * u64::pow(10, quote_decimal) as u256), address_create_bid, true, address_accept_bid);
        // accept bid account will recieve 10 * 5 = 50.00000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_accept_bid) == 50_00000000, 0);
        // create bid account will recieve 5.0000000000 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_create_bid) == 5_0000000000, 1);
        // accept bid account will pay 5.0000000000 quote coin
        assert!(account_coin_store::balance<TestQuoteCoin>(address_accept_bid) == 995_0000000000, 2);
        // create bid account will pay 10 * 10 = 100.00000000 base coin
        assert!(account_coin_store::balance<TestBaseCoin>(address_create_bid) == 900_00000000, 3);
        accept_bid_from_origin(&account_accept_bid, &mut market_obj, MIN_BID_ORDER_ID,
            (5 * u64::pow(10, quote_decimal) as u256), address_create_bid, true, address_accept_bid);
        to_shared(market_obj);
    }
}


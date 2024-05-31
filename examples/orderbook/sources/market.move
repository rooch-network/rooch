// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
module orderbook::market {
    use std::option;
    use std::option::Option;
    use std::string;
    use std::string::String;
    use moveos_std::object;
    use rooch_framework::coin::{Self, Coin};
    use std::vector;
    use moveos_std::event;
    use rooch_framework::account_coin_store;
    use orderbook::linked_table;
    use orderbook::linked_table::LinkedTable;
    use rooch_framework::coin_store;
    use moveos_std::tx_context::sender;
    use moveos_std::type_info::type_name;
    use moveos_std::object::{Object, ObjectID, to_shared, new_named_object, transfer};
    use rooch_framework::coin_store::{CoinStore, create_coin_store};
    use moveos_std::timestamp::now_milliseconds;
    use orderbook::critbit::{CritbitTree, find_leaf, borrow_leaf_by_index, borrow_mut_leaf_by_index,
        remove_leaf_by_index
    };
    use orderbook::critbit;
    use moveos_std::table;
    use moveos_std::table::Table;

    const DEPLOYER: address = @orderbook;


    const VERSION: u64 = 4;


    const BASE_MARKET_FEE: u256 = 20;
    const TRADE_FEE_BASE_RATIO: u256 = 1000;

    const MIN_BID_ORDER_ID: u64 = 1;
    const MIN_ASK_ORDER_ID: u64 = 1 << 63;

    const ErrorWrongVersion: u64 = 0;
    const ErrorWrongPaused: u64 = 1;
    const ErrorInputCoin: u64 = 2;
    const ErrorWrongMarket: u64 = 3;
    const ErrorPriceTooLow: u64 = 4;
    const ErrorWrongCreateBid: u64 = 5;
    const ErrorFeeTooHigh: u64 = 6;
    const ErrorInvalidOrderId: u64 = 7;
    const ErrorUnauthorizedCancel: u64 = 8;


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

    struct TickLevel has store {
        price: u64,
        // The key is order order id.
        open_orders: Object<LinkedTable<u64, Order>>,
        // other price level info
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
        user_order_info: Table<address, Object<LinkedTable<u64, u64>>>,
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

    struct AdminCap has key, store {}

    struct MarketplaceHouse has key {
        market_info: Object<LinkedTable<String, ObjectID>>,
    }



    public entry fun create_market<BaseAsset: key + store, QuoteAsset: key + store>(
        market_house_obj: &mut Object<MarketplaceHouse>,
    ) {
        let market_obj = new_named_object(Marketplace {
            is_paused: false,
            version: VERSION,
            bids: critbit::new(),
            asks: critbit::new(),
            // Order id of the next bid order, starting from 0.
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

    fun init() {
        let market_house = MarketplaceHouse {
            market_info: linked_table::new(),
        };

        //TODO market create event
        transfer(new_named_object(AdminCap{}), sender());
        to_shared(new_named_object(market_house))
    }

    ///Listing NFT in the collection
    public fun list<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        coin: Coin<BaseAsset>,
        unit_price: u64,
    ) {
        let market = object::borrow_mut(market_obj);
        assert!(market.version == VERSION, ErrorWrongVersion);
        assert!(market.is_paused == false, ErrorWrongPaused);
        let quantity = coin::value(&coin);
        let order_id = market.next_ask_order_id;
        market.next_ask_order_id = market.next_ask_order_id + 1;
        // TODO here maybe wrap to u512?
        // let price = (unit_price as u256) * quantity;
        assert!(unit_price > 0, ErrorPriceTooLow);
        let asks = Order {
            order_id,
            unit_price,
            quantity,
            owner: sender(),
            is_bid: false,
        };
        coin_store::deposit(&mut market.base_asset, coin);
        let (find_price, index) = critbit::find_leaf(&market.asks, unit_price);
        if (find_price) {
            critbit::insert_leaf(&mut market.asks, unit_price, TickLevel{
                price: unit_price,
                open_orders: linked_table::new()
            });
        };
        let tick_level = critbit::borrow_mut_leaf_by_index(&mut market.asks, index);
        linked_table::push_back(&mut tick_level.open_orders, order_id, asks);

        if (!table::contains(&market.user_order_info, sender())) {
            table::add(&mut market.user_order_info, sender(), linked_table::new());
        };
        linked_table::push_back(table::borrow_mut(&mut market.user_order_info, sender()), order_id, unit_price);

    }


    public fun create_bid<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        paid: &mut Coin<QuoteAsset>,
        unit_price: u64,
        quantity: u256,
    ) {
        let market = object::borrow_mut(market_obj);
        assert!(market.version == VERSION, ErrorWrongVersion);
        assert!(market.is_paused == false, ErrorWrongPaused);
        assert!(quantity > 0, ErrorWrongCreateBid);
        assert!(unit_price > 0, ErrorWrongCreateBid);
        // TODO here maybe wrap to u512?
        let price = (unit_price as u256) * quantity;
        assert!(price <= coin::value(paid), ErrorInputCoin);
        let order_id = market.next_bid_order_id;
        market.next_bid_order_id = market.next_bid_order_id + 1;
        let bid = Order {
            order_id,
            unit_price,
            quantity,
            owner: sender(),
            is_bid: true,
        };
        coin_store::deposit(&mut market.quote_asset, coin::extract(paid, price));

        let (find_price, index) = critbit::find_leaf(&market.bids, unit_price);
        if (!find_price) {
            critbit::insert_leaf(&mut market.bids, unit_price, TickLevel {
                price: unit_price,
                open_orders: linked_table::new()
            });
        };
        let tick_level = critbit::borrow_mut_leaf_by_index(&mut market.bids, index);
        linked_table::push_back(&mut tick_level.open_orders, order_id, bid);
    }

    ///Cancel the listing of inscription
    public entry fun cancel_order<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
    ) {
        //Get the list from the collection
        let market = object::borrow_mut(market_obj);
        assert!(market.version == VERSION, ErrorWrongVersion);

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
            let total_balance = (order.unit_price as u256) * order.quantity;
            account_coin_store::deposit(sender(), coin_store::withdraw(&mut market.quote_asset, total_balance))
        }else {
            account_coin_store::deposit(sender(), coin_store::withdraw(&mut market.base_asset, order.quantity))
        }
    }


    ///purchase
    public fun buy<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        assert_order_exist: bool,
        paid: &mut Coin<BaseAsset>,
    ): Option<Coin<QuoteAsset>> {
        let market = object::borrow_mut(market_obj);
        assert!(market.is_paused == false, ErrorWrongPaused);
        assert!(market.version == VERSION, ErrorWrongVersion);
        let usr_open_orders = table::borrow_mut(&mut market.user_order_info, sender());
        let tick_price = *linked_table::borrow(usr_open_orders, order_id);
        let (tick_exists, tick_index) = find_leaf(&market.asks, tick_price);
        // Return non-existent orders to none instead of panic during bulk buying
        if (!assert_order_exist && !tick_exists) {
            return option::none()
        };
        assert!(tick_exists, ErrorInvalidOrderId);
        let order = remove_order(&mut market.asks, usr_open_orders, tick_index, order_id, sender());
        // TODO here maybe wrap to u512?
        let total_price = order.quantity * (order.unit_price as u256);
        let trade_coin = coin::extract(paid, total_price);
        assert!(coin::value(paid) >= total_price, ErrorInputCoin);
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
        coin_store::deposit(&mut market.base_asset, trade_coin);
        option::some(coin_store::withdraw(&mut market.quote_asset, order.quantity))
    }

    public fun accept_bid<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        order_id: u64,
        assert_order_exist: bool,
        paid: &mut Coin<QuoteAsset>
    ): Option<Coin<BaseAsset>>
    {
        let market = object::borrow_mut(market_obj);
        assert!(market.is_paused == false, ErrorWrongPaused);
        assert!(market.version == VERSION, ErrorWrongVersion);
        let usr_open_orders = table::borrow_mut(&mut market.user_order_info, sender());
        let tick_price = *linked_table::borrow(usr_open_orders, order_id);
        let (tick_exists, tick_index) = find_leaf(&market.bids, tick_price);
        // Return non-existent orders to none instead of panic during bulk buying
        if (!assert_order_exist && !tick_exists) {
            return option::none()
        };
        assert!(tick_exists, ErrorInvalidOrderId);

        let order = remove_order(&mut market.bids, usr_open_orders, tick_index, order_id, sender());
        assert!(coin::value(paid) >=  order.quantity, ErrorInputCoin);
        let trade_coin = coin::extract(paid, order.quantity);

        // TODO here maybe wrap to u512?
        let total_price = (order.unit_price as u256) * order.quantity;
        let trade_info = &mut market.trade_info;

        trade_info.total_volume = trade_info.total_volume + total_price;
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
        coin_store::deposit(&mut market.quote_asset_trading_fees, trade_coin);

        option::some(coin_store::withdraw(&mut market.base_asset, total_price))
    }



    public entry fun withdraw_profits<BaseAsset: key + store, QuoteAsset: key + store>(
        _admin: &mut Object<AdminCap>,
        market_obj: &mut Object<Marketplace<BaseAsset, QuoteAsset>>,
        receiver: address,
    ) {
        let market = object::borrow_mut(market_obj);
        assert!(market.version == VERSION, ErrorWrongVersion);
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
        assert!(market.version == VERSION, ErrorWrongVersion);
        assert!(fee < TRADE_FEE_BASE_RATIO, ErrorFeeTooHigh);
        market.fee = fee
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
        user_order_info: &mut Object<LinkedTable<u64, u64>>,
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

    fun destroy_empty_level(level: TickLevel) {
        let TickLevel {
            price: _,
            open_orders: orders,
        } = level;

        linked_table::destroy_empty(orders);
    }

    struct QueryOrderEvent has copy, drop {
        order_ids: vector<u64>,
        unit_prices: vector<u64>,
        quantitys: vector<u256>,
        owners: vector<address>,
        is_bids: vector<bool>
    }

    public fun query_order<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &Object<Marketplace<BaseAsset, QuoteAsset>>,
        query_bid: bool,
        from_order: Option<u64>,
        start: u64
    ): vector<u64> {
        let market = object::borrow(market_obj);
        let order_ids = vector<u64>[];
        let unit_prices = vector<u64>[];
        let quantitys = vector<u256>[];
        let owners = vector<address>[];
        let is_bids = vector<bool>[];

        if (query_bid) {
            let i = 0;
            let from = if (option::is_none(&from_order)) {
                let (key, _) = critbit::max_leaf(&market.bids);
                key
            }else {
                *option::borrow(&from_order)
            };
            let count = start;
            while (i < 50) {
                let tick_level = critbit::borrow_leaf_by_key(&market.bids, from);
                let order_count = linked_table::length(&tick_level.open_orders);

                while (order_count > count) {
                    let order = linked_table::borrow(&tick_level.open_orders, count);
                    vector::push_back(&mut order_ids, order.order_id);
                    vector::push_back(&mut unit_prices, order.unit_price);
                    vector::push_back(&mut quantitys, order.quantity);
                    vector::push_back(&mut owners, order.owner);
                    vector::push_back(&mut is_bids, order.is_bid);

                    count = count + 1;
                    i = i + 1;
                    if (i >= 50) {
                        event::emit(
                            QueryOrderEvent{
                                order_ids,
                                unit_prices,
                                quantitys,
                                owners,
                                is_bids
                            }
                        );
                        return order_ids
                    }
                };
                count = 0;
                let (key, index) = critbit::previous_leaf(&market.bids, from);
                if (index != 0x8000000000000000) {
                    from = key;
                }else {
                    event::emit(
                        QueryOrderEvent{
                            order_ids,
                            unit_prices,
                            quantitys,
                            owners,
                            is_bids
                        }
                    );
                    return order_ids
                }
            };
        }else {
            let i = 0;
            let from = if (option::is_none(&from_order)) {
                let (key, _) = critbit::min_leaf(&market.asks);
                key
            }else {
                *option::borrow(&from_order)
            };
            let count = start;
            while (i < 50) {
                let tick_level = critbit::borrow_leaf_by_key(&market.asks, from);
                let order_count = linked_table::length(&tick_level.open_orders);

                while (order_count > count) {
                    let order = linked_table::borrow(&tick_level.open_orders, count);
                    vector::push_back(&mut order_ids, order.order_id);
                    vector::push_back(&mut unit_prices, order.unit_price);
                    vector::push_back(&mut quantitys, order.quantity);
                    vector::push_back(&mut owners, order.owner);
                    vector::push_back(&mut is_bids, order.is_bid);

                    count = count + 1;
                    i = i + 1;
                    if (i >= 50) {
                        event::emit(
                            QueryOrderEvent{
                                order_ids,
                                unit_prices,
                                quantitys,
                                owners,
                                is_bids
                            }
                        );
                        return order_ids
                    }
                };
                count = 0;
                let (key, index) = critbit::next_leaf(&market.asks, from);
                if (index != 0x8000000000000000) {
                    from = key;
                }else {
                    event::emit(
                        QueryOrderEvent{
                            order_ids,
                            unit_prices,
                            quantitys,
                            owners,
                            is_bids
                        }
                    );
                    return order_ids
                }
            };
        };
        event::emit(
            QueryOrderEvent{
                order_ids,
                unit_prices,
                quantitys,
                owners,
                is_bids
            }
        );
        return order_ids
    }

    public fun query_user_order<BaseAsset: key + store, QuoteAsset: key + store>(
        market_obj: &Object<Marketplace<BaseAsset, QuoteAsset>>,
        user: address,
        from_order: Option<u64>,
        count: u64
    ): vector<u64>{
        let market = object::borrow(market_obj);
        let user_table = table::borrow(&market.user_order_info, user);
        let order_ids = vector<u64>[];
        let unit_prices = vector<u64>[];
        let quantitys = vector<u256>[];
        let owners = vector<address>[];
        let is_bids = vector<bool>[];
        let from = if (option::is_none(&from_order)) {
            *option::borrow(linked_table::front(user_table))
        }else {
            *option::borrow(&from_order)
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
                vector::push_back(&mut order_ids, order.order_id);
                vector::push_back(&mut unit_prices, order.unit_price);
                vector::push_back(&mut quantitys, order.quantity);
                vector::push_back(&mut owners, order.owner);
                vector::push_back(&mut is_bids, order.is_bid);
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
        event::emit(
            QueryOrderEvent{
                order_ids,
                unit_prices,
                quantitys,
                owners,
                is_bids
            }
        );
        return order_ids
    }


    fun order_is_bid(order_id: u64): bool {
        return order_id < MIN_ASK_ORDER_ID
    }
}

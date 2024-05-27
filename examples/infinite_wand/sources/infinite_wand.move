// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module infinite_wand::infinite_wand {

    use std::string;
    use rooch_framework::timestamp::now_milliseconds;
    use moveos_std::table::Table;
    use moveos_std::table;
    use rooch_framework::account_coin_store;
    use rooch_framework::coin::CoinInfo;
    use rooch_framework::coin_store;
    use moveos_std::display;
    use moveos_std::tx_context::sender;
    use moveos_std::object;
    use rooch_framework::coin;
    use rooch_framework::coin_store::CoinStore;
    use moveos_std::object::Object;


    const ErrorSendLimitExceeded: u64 = 0;
    const ErrorReceiveLimitExceeded: u64 = 1;
    const ErrorAddressNotInWhitelist: u64 = 2;
    const ErrorAddressAlreadyInWhitelist: u64 = 3;

    const DECIMALS: u8 = 1u8;
    const SEND_LIMIT: u256 = 10_000_000;
    const RECEIVE_LIMIT: u256 = 50_000_000;
    const THIRTY_DAYS_MILLISECONDS: u64 = 2592000000;

    struct AdminCap has key, store{}

    struct InfiniteGold has key {}

    struct InfiniteWand has key, store {
        nft_id: u64,
        level: u64,
        gold: Object<CoinStore<InfiniteGold>>
    }
    struct Global has key, store {
        wand_id: u64,
        swap_detail: Table<address, SwapRuler>,
        whitelist: Table<address, bool>
    }

    struct SwapRuler has key, store {
        receive_amount: u256,
        send_amount: u256,
        timestamp: u64
    }

    fun init() {
        let coin_info_obj = coin::register_extend<InfiniteGold>(
            string::utf8(b"PreToken of InfiniteGames DAO"),
            string::utf8(b"InfiniteGold"),
            DECIMALS,
        );
        let admin_cap = object::new_named_object(AdminCap {});
        let global_obj = object::new_named_object(Global {
            wand_id: 0,
            swap_detail: table::new(),
            whitelist: table::new()
        });
        let nft_display_object = display::object_display<InfiniteWand>();
        display::set_value(nft_display_object, string::utf8(b"name"), string::utf8(b"InfiniteWand#{value.nft_id}"));
        display::set_value(nft_display_object, string::utf8(b"description"), string::utf8(b"PreToken of InfiniteGames DAO"));
        display::set_value(nft_display_object, string::utf8(b"image_url"), string::utf8(b"https://base_url/{level}"));
        object::to_shared(coin_info_obj);
        object::transfer(admin_cap, sender());
        object::to_shared(global_obj);
    }

    public fun mint_infinite_wand(_admin_cap: &mut Object<AdminCap>, global_obj: &mut Object<Global>, receiver: address) {
        let global = object::borrow_mut(global_obj);
        let nft = InfiniteWand {
            nft_id: global.wand_id,
            level: 0,
            gold: coin_store::create_coin_store_extend()
        };

        let nft_obj = object::new(
            nft
        );
        object::transfer(nft_obj, receiver);
        global.wand_id = global.wand_id + 1;
    }

    public fun mint_infinite_gold(_admin_cap: &mut Object<AdminCap>, coin_info_obj: &mut Object<CoinInfo<InfiniteGold>>, credit: u256, receiver: address) {
        let coin = coin::mint_extend<InfiniteGold>(coin_info_obj, credit);
        // must set InfiniteGold is accept coin
        account_coin_store::deposit_extend(receiver, coin);
    }

    public fun add_whitelist(_admin_cap: &mut Object<AdminCap>, global_obj: &mut Object<Global>, addr: address) {
        let global = object::borrow_mut(global_obj);
        assert!(!table::contains(&global.whitelist, addr), ErrorAddressAlreadyInWhitelist);
        table::add(&mut global.whitelist, addr, true);
    }

    public fun remove_whitelist(_admin_cap: &mut Object<AdminCap>, global_obj: &mut Object<Global>, addr: address) {
        let global = object::borrow_mut(global_obj);
        assert!(table::contains(&global.whitelist, addr), ErrorAddressNotInWhitelist);
        table::remove(&mut global.whitelist, addr);
    }

    public entry fun transfer(receiver: address, amount: u256, global_obj: &mut Object<Global>) {
        let global = object::borrow_mut(global_obj);
        let sender_is_whitelist = table::contains(&global.whitelist, sender());
        let receiver_is_whitelist = table::contains(&global.whitelist, receiver);

        if (sender_is_whitelist || receiver_is_whitelist){
            account_coin_store::transfer_extend<InfiniteGold>(sender(), receiver, amount);
            return
        };

        let now = now_milliseconds();
        if (!table::contains(&global.swap_detail, sender())) {
            table::add(&mut global.swap_detail, sender(), SwapRuler{
                receive_amount: 0,
                send_amount: 0,
                timestamp: now
            })
        };
        if (!table::contains(&global.swap_detail, receiver)) {
            table::add(&mut global.swap_detail, receiver, SwapRuler{
                receive_amount: 0,
                send_amount: 0,
                timestamp: now
            })
        };
        let sender_detail =  table::borrow_mut(&mut global.swap_detail, sender());
        if (sender_detail.timestamp + THIRTY_DAYS_MILLISECONDS > now) {
            sender_detail.timestamp = now;
            sender_detail.send_amount = 0;
            sender_detail.receive_amount = 0;
        };
        assert!(sender_detail.send_amount + amount <= SEND_LIMIT, ErrorSendLimitExceeded);
        sender_detail.send_amount = sender_detail.send_amount + amount;
        let receiver_detail =  table::borrow_mut(&mut global.swap_detail, receiver);
        if (receiver_detail.timestamp + THIRTY_DAYS_MILLISECONDS > now) {
            receiver_detail.timestamp = now;
            receiver_detail.send_amount = 0;
            receiver_detail.receive_amount = 0;
        };
        assert!(receiver_detail.receive_amount + amount <= RECEIVE_LIMIT, ErrorReceiveLimitExceeded);
        account_coin_store::transfer_extend<InfiniteGold>(sender(), receiver, amount);
        receiver_detail.receive_amount = receiver_detail.receive_amount + amount;
    }

    public fun stake(credit: u256, infinite_wand_obj: &mut Object<InfiniteWand>) {
        let coin = account_coin_store::withdraw_extend<InfiniteGold>(sender(), credit);
        let infinite_wand = object::borrow_mut(infinite_wand_obj);
        coin_store::deposit_extend(&mut infinite_wand.gold, coin);
        let total_credit = coin_store::balance(&infinite_wand.gold);
        let new_level = calculate_level(total_credit);
        infinite_wand.level = new_level;
    }

    public fun un_stake(credit: u256, infinite_wand_obj: &mut Object<InfiniteWand>) {
        let infinite_wand = object::borrow_mut(infinite_wand_obj);
        let coin = coin_store::withdraw_extend(&mut infinite_wand.gold, credit);
        let new_level = calculate_level(coin_store::balance(&infinite_wand.gold));
        infinite_wand.level = new_level;
        account_coin_store::deposit_extend(sender(), coin);
    }

    public fun calculate_level(credit: u256): u64 {
        if (credit < 16000) {
            // level is 0 ~ 19
            return (credit/800 as u64)
        }else if (credit < 36000){
            // level is 20 ~ 39
            return (20 +((credit - 16000)/1000) as u64)
        }else if (credit < 96000){
            // level is 40 ~ 59
            return (40+((credit-96000)/3000) as u64)
        }else if (credit < 196000){
            // level is 60 ~ 79
            return (60+((credit-196000)/5000) as u64)
        }else if (credit < 396000) {
            // level is 80 ~ 99
            return (80+((credit-396000)/10000) as u64)
        }else {
            return 100
        }
    }

}

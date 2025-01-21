module grow_bitcoin::grow_point_v3 {

    use std::option;
    use std::string;
    use std::string::String;
    use moveos_std::timestamp::now_seconds;
    use rooch_framework::account_coin_store;
    use moveos_std::object;
    use moveos_std::signer::address_of;
    use rooch_framework::coin::CoinInfo;
    use moveos_std::object::Object;
    use moveos_std::signer;
    use moveos_std::account;
    use rooch_framework::coin;

    friend grow_bitcoin::grow_information_v3;

    /// The `Rooch Grow Point`
    struct BITXP has key {}

    const DECIMALS: u8 = 1u8;

    const ErrorPoinBoxAleardyClaimed: u64 = 1;
    const ErrorPoinBoxNotClaimed: u64 = 2;
    const ErrorLeaderboardNotOpen: u64 = 3;

    struct Leaderboard has key, store {
        coin_info: Object<CoinInfo<BITXP>>,
    }

    struct PointBox has key, store {
        project_id: String,
        value: u256,
        // The project team can award different levels of rewards based on the voting time
        timestamp: u64
    }


    fun init() {
        let coin_info_obj = coin::register_extend<BITXP>(
            string::utf8(b"Grow Bitcoin Point"),
            string::utf8(b"BITXP"),
            option::none(),
            DECIMALS,
        );
        let grow_leaderboard_signer = signer::module_signer<Leaderboard>();
        account::move_resource_to(&grow_leaderboard_signer, Leaderboard{
            coin_info: coin_info_obj
        })
    }

    public(friend) fun mint_point_box(project_id: String, value: u256, receiver: address): Object<PointBox> {
        let grow_leaderboard_signer = signer::module_signer<Leaderboard>();
        let leaderboard = account::borrow_mut_resource<Leaderboard>(address_of(&grow_leaderboard_signer));
        let coin = coin::mint_extend(&mut leaderboard.coin_info, value);
        account_coin_store::deposit_extend(receiver, coin);
        object::new(PointBox{
            project_id,
            value,
            timestamp: now_seconds()
        })
    }


    public entry fun destory_point_box(point_box_obj: Object<PointBox>) {
        let PointBox{
            project_id: _,
            value: _,
            timestamp: _
        } = object::remove(point_box_obj);
    }


    public fun value(point_box_obj: &Object<PointBox>): u256{
        object::borrow(point_box_obj).value
    }

    public fun project_id(point_box_obj: &Object<PointBox>): String {
        object::borrow(point_box_obj).project_id
    }

    public fun timestamp(point_box_obj: &Object<PointBox>): u64 {
        object::borrow(point_box_obj).timestamp
    }
}

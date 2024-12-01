module invitation_record::invitation {

    use std::option;
    use std::string::String;
    use std::vector;
    use moveos_std::hash;
    use rooch_framework::transaction;
    use rooch_framework::transaction::TransactionSequenceInfo;
    use moveos_std::timestamp;
    use moveos_std::bcs;
    use moveos_std::tx_context;
    use rooch_framework::simple_rng::bytes_to_u64;
    use moveos_std::timestamp::now_seconds;
    use moveos_std::table_vec;
    use moveos_std::table_vec::TableVec;
    use rooch_framework::bitcoin_address;
    use twitter_binding::twitter_account::{verify_and_binding_twitter_account, check_binding_tweet};
    use moveos_std::tx_context::sender;
    use rooch_framework::account_coin_store;
    use rooch_framework::gas_coin::RGas;
    use rooch_framework::coin_store::CoinStore;
    use rooch_framework::coin_store;
    use gas_faucet::gas_faucet::{RGasFaucet, claim};
    use moveos_std::table;
    use moveos_std::object;
    use app_admin::admin::AdminCap;
    use moveos_std::object::{Object, to_shared, ObjectID};
    use moveos_std::table::Table;
    #[test_only]
    use bitcoin_move::utxo;
    #[test_only]
    use gas_faucet::gas_faucet;
    #[test_only]
    use moveos_std::signer::address_of;
    #[test_only]
    use rooch_framework::account::create_account_for_testing;
    #[test_only]
    use rooch_framework::gas_coin::faucet_for_test;


    const ErrorFaucetNotOpen: u64 = 1;
    const ErrorFaucetNotEnoughRGas: u64 = 2;
    const ErrorNoRemainingLuckeyTicket: u64 = 3;

    const ONE_RGAS: u256 = 1_00000000;
    const ErrorInvalidArg: u64 = 0;

    struct UserInvitationRecords has key, store {
        invitation_records: Table<address, u256>,
        lottery_records: TableVec<LotteryInfo>,
        total_invitations: u64,
        remaining_luckey_ticket: u64,
        invitation_reward_amount: u256,
        lottery_reward_amount: u256,
    }

    struct LotteryInfo has store {
        timestamp: u64,
        reward_amount: u256,
    }

    struct InvitationConf has key {
        rgas_store: Object<CoinStore<RGas>>,
        invitation_records: Table<address, UserInvitationRecords>,
        is_open: bool,
        unit_invitation_amount: u256,
    }

    fun init() {
        let invitation_obj = object::new_named_object(InvitationConf{
            rgas_store: coin_store::create_coin_store<RGas>(),
            invitation_records: table::new(),
            is_open: true,
            unit_invitation_amount: ONE_RGAS * 5
        });
        to_shared(invitation_obj)
    }

    public entry fun deposit_rgas_coin(
        account: &signer,
        faucet_obj: &mut Object<InvitationConf>,
        amount: u256
    ){
        let faucet = object::borrow_mut(faucet_obj);
        deposit_to_rgas_store(account, &mut faucet.rgas_store, amount);
    }

    public entry fun withdraw_rgas_coin(
        faucet_obj: &mut Object<InvitationConf>,
        amount: u256,
        _admin: &mut Object<AdminCap>,
    ){
        let faucet = object::borrow_mut(faucet_obj);
        let rgas_coin = coin_store::withdraw(&mut faucet.rgas_store, amount);
        account_coin_store::deposit<RGas>(sender(), rgas_coin);
    }

        /// Anyone can call this function to help the claimer claim the faucet
    public entry fun claim_from_faucet(faucet_obj: &mut Object<RGasFaucet>, invitation_obj: &mut Object<InvitationConf>, claimer: address, utxo_ids: vector<ObjectID>, inviter: address){
        let invitation_conf = object::borrow_mut(invitation_obj);
        assert!(invitation_conf.is_open, ErrorFaucetNotOpen);
        if (!table::contains(&invitation_conf.invitation_records, inviter)) {
            table::add(&mut invitation_conf.invitation_records, inviter, UserInvitationRecords{
                invitation_records: table::new(),
                lottery_records: table_vec::new(),
                total_invitations: 0u64,
                remaining_luckey_ticket: 0u64,
                invitation_reward_amount: 0u256,
                lottery_reward_amount: 0u256,
            })
        };
        let user_invitation_records = table::borrow_mut(&mut invitation_conf.invitation_records, inviter);
        let invitation_amount = table::borrow_mut_with_default(&mut user_invitation_records.invitation_records, claimer, 0u256);
        *invitation_amount = *invitation_amount + invitation_conf.unit_invitation_amount;
        user_invitation_records.total_invitations = user_invitation_records.total_invitations + 1u64;
        user_invitation_records.invitation_reward_amount = user_invitation_records.invitation_reward_amount + invitation_conf.unit_invitation_amount;
        user_invitation_records.remaining_luckey_ticket = user_invitation_records.remaining_luckey_ticket + 1u64;
        let rgas_coin = coin_store::withdraw(&mut invitation_conf.rgas_store, invitation_conf.unit_invitation_amount);
        account_coin_store::deposit<RGas>(inviter, rgas_coin);

        claim(faucet_obj, claimer, utxo_ids);
    }

    public entry fun claim_from_twitter(tweet_id: String,  invitation_obj: &mut Object<InvitationConf>, inviter: address){
        let bitcoin_address = check_binding_tweet(tweet_id);
        let claimer = bitcoin_address::to_rooch_address(&bitcoin_address);
        let invitation_conf = object::borrow_mut(invitation_obj);
        assert!(invitation_conf.is_open, ErrorFaucetNotOpen);
        if (!table::contains(&invitation_conf.invitation_records, inviter)) {
            table::add(&mut invitation_conf.invitation_records, inviter, UserInvitationRecords{
                invitation_records: table::new(),
                lottery_records: table_vec::new(),
                total_invitations: 0u64,
                remaining_luckey_ticket: 0u64,
                invitation_reward_amount: 0u256,
                lottery_reward_amount: 0u256,
            })
        };
        let user_invitation_records = table::borrow_mut(&mut invitation_conf.invitation_records, inviter);
        let invitation_amount = table::borrow_mut_with_default(&mut user_invitation_records.invitation_records, claimer, 0u256);
        *invitation_amount = *invitation_amount + invitation_conf.unit_invitation_amount;
        user_invitation_records.total_invitations = user_invitation_records.total_invitations + 1u64;
        user_invitation_records.invitation_reward_amount = user_invitation_records.invitation_reward_amount + invitation_conf.unit_invitation_amount;
        user_invitation_records.remaining_luckey_ticket = user_invitation_records.remaining_luckey_ticket + 1u64;
        let rgas_coin = coin_store::withdraw(&mut invitation_conf.rgas_store, invitation_conf.unit_invitation_amount);
        account_coin_store::deposit<RGas>(inviter, rgas_coin);
        verify_and_binding_twitter_account(tweet_id);

    }

    entry fun lottery(invitation_obj: &mut Object<InvitationConf>, amount: u64){
        let invitation_conf = object::borrow_mut(invitation_obj);
        let user_invitation_records = table::borrow_mut(&mut invitation_conf.invitation_records, sender());
        assert!(user_invitation_records.remaining_luckey_ticket >= amount, ErrorNoRemainingLuckeyTicket);
        while (amount > 0) {
            let reward_amount = rand_u64_range(10_000_000, 100_000_000, amount);
            if (reward_amount % 150 == 0) {
                reward_amount = reward_amount * 1000
            };
            let rgas_coin = coin_store::withdraw(&mut invitation_conf.rgas_store, (reward_amount as u256));
            account_coin_store::deposit<RGas>(sender(), rgas_coin);
            table_vec::push_back(&mut user_invitation_records.lottery_records, LotteryInfo {
                timestamp: now_seconds(),
                reward_amount: (reward_amount as u256),
            });
            user_invitation_records.remaining_luckey_ticket = user_invitation_records.remaining_luckey_ticket - 1u64;
            user_invitation_records.lottery_reward_amount = user_invitation_records.lottery_reward_amount + (reward_amount as u256);
            amount = amount - 1u64;
        }
    }

    public entry fun close_invitation(
        invitation_obj: &mut Object<InvitationConf>,
        _admin: &mut Object<AdminCap>,
    ){
        let invitation = object::borrow_mut(invitation_obj);
        invitation.is_open = false;
    }

    public entry fun open_invitation(
        invitation_obj: &mut Object<InvitationConf>,
        _admin: &mut Object<AdminCap>,
    ) {
        let invitation = object::borrow_mut(invitation_obj);
        invitation.is_open = true;
    }

    public entry fun set_invitation_unit_amount(
        invitation_obj: &mut Object<InvitationConf>,
        unit_invitation_amount: u256,
        _admin: &mut Object<AdminCap>,
    ) {
        let invitation = object::borrow_mut(invitation_obj);
        invitation.unit_invitation_amount = unit_invitation_amount;
    }

    fun deposit_to_rgas_store(
        account: &signer,
        rgas_store: &mut Object<CoinStore<RGas>>,
        amount: u256
    ){
        let rgas_coin = account_coin_store::withdraw<RGas>(account, amount);
        coin_store::deposit(rgas_store, rgas_coin);
    }

    fun seed(index: u64): vector<u8> {
        // get sequence number
        let sequence_number = tx_context::sequence_number();
        let sequence_number_bytes = bcs::to_bytes(&sequence_number);

        // get sender address
        let sender_addr = tx_context::sender();
        let sender_addr_bytes = bcs::to_bytes(&sender_addr);

        // get now milliseconds timestamp
        let timestamp_ms = timestamp::now_milliseconds();
        let timestamp_ms_bytes = bcs::to_bytes(&timestamp_ms);

        let index_bytes = bcs::to_bytes(&index);
        // construct a seed
        let seed_bytes = vector::empty<u8>();

        // get the tx accumulator root if exists
        let tx_sequence_info_opt = tx_context::get_attribute<TransactionSequenceInfo>();
        if (option::is_some(&tx_sequence_info_opt)) {
            let tx_sequence_info = option::extract(&mut tx_sequence_info_opt);
            let tx_accumulator_root = transaction::tx_accumulator_root(&tx_sequence_info);
            let tx_accumulator_root_bytes = bcs::to_bytes(&tx_accumulator_root);
            vector::append(&mut seed_bytes, tx_accumulator_root_bytes);
        } else {
            // if it doesn't exist, get the tx hash
            let tx_hash = tx_context::tx_hash();
            let tx_hash_bytes = bcs::to_bytes(&tx_hash);
            vector::append(&mut seed_bytes, tx_hash_bytes);
        };

        vector::append(&mut seed_bytes, timestamp_ms_bytes);
        vector::append(&mut seed_bytes, sender_addr_bytes);
        vector::append(&mut seed_bytes, sequence_number_bytes);
        vector::append(&mut seed_bytes, index_bytes);
        // hash seed bytes and return a seed
        let seed = hash::sha3_256(seed_bytes);
        seed
    }

    fun rand_u64_range(low: u64, high: u64, index: u64): u64 {
        assert!(high > low, ErrorInvalidArg);
        let value = rand_u64(index);
        (value % (high - low)) + low
    }

    fun rand_u64(index: u64): u64 {
        let seed_bytes = seed(index);
        bytes_to_u64(seed_bytes)
    }


        #[test(sender=@0x42)]
    fun test_claim_with_invitation(sender: &signer){
        bitcoin_move::genesis::init_for_test();
        create_account_for_testing(@0x42);
        create_account_for_testing(@0x43);
        let invitation_obj = object::new_named_object(InvitationConf{
            invitation_records: table::new(),
            is_open: true,
            unit_invitation_amount: ONE_RGAS * 5,
            rgas_store: coin_store::create_coin_store<RGas>(),
        });
        faucet_for_test(address_of(sender), 5000000_00000000);
        gas_faucet::init_for_test(sender);
        deposit_rgas_coin(sender, &mut invitation_obj,500000_00000000);
        object::to_shared(invitation_obj);
        let faucet_obj = object::borrow_mut_object_shared<RGasFaucet>(object::named_object_id<RGasFaucet>());
        let invitation_obj = object::borrow_mut_object_shared<InvitationConf>(object::named_object_id<InvitationConf>());
        let tx_id = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let sat_value = 100000000;
        let test_utxo = utxo::new_for_testing(tx_id, 0u32, sat_value);
        let test_utxo_id = object::id(&test_utxo);
        utxo::transfer_for_testing(test_utxo, @0x43);
        claim_from_faucet(faucet_obj, invitation_obj, @0x43, vector[test_utxo_id], @0x42);
        let invitation_obj = object::borrow_mut_object_shared<InvitationConf>(object::named_object_id<InvitationConf>());
       let invitation = object::borrow(invitation_obj);
        let records = table::borrow(&invitation.invitation_records, @0x42);
        let invitation_user_record = table::borrow(&records.invitation_records, @0x43);
        assert!(invitation_user_record == &500000000, 1);
        assert!(records.invitation_reward_amount == 500000000, 2);
        assert!(records.total_invitations == 1, 3);
    }
}

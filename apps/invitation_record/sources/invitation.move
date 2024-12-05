module invitation_record::invitation {

    use std::option;
    use std::string::String;
    use std::vector;
    use moveos_std::signer;
    use moveos_std::consensus_codec;
    use twitter_binding::tweet_v2;
    use rooch_framework::bitcoin_address::BitcoinAddress;
    use rooch_framework::ecdsa_k1;
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
    use twitter_binding::twitter_account::{verify_and_binding_twitter_account, check_binding_tweet, check_user_claimed};
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
    use std::string;

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
    const ErrorNoInvitationSignature: u64 = 4;
    const ErrorNoInvitationBitcoinSignature: u64 = 5;
    const ErrorNotClaimerAddress: u64 = 6;
    const ErrorCannotInviteOneself: u64 = 7;
    const ErrorInvalidSignature: u64 = 8;

    const ONE_RGAS: u256 = 1_00000000;
    const INIT_GAS_AMOUNT: u256 = 1000000_00000000;
    const ErrorInvalidArg: u64 = 0;

    const MessagePrefix : vector<u8> = b"Bitcoin Signed Message:\n";


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
        let invitation_obj = object::new_named_object(InvitationConf{
            rgas_store,
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
    public entry fun claim_from_faucet(
        faucet_obj: &mut Object<RGasFaucet>,
        invitation_obj: &mut Object<InvitationConf>,
        claimer_bitcoin_address: String,
        utxo_ids: vector<ObjectID>,
        inviter: address,
        public_key: vector<u8>,
        signature: vector<u8>,
        message: vector<u8>,
    ){
        let bitcoin_address = bitcoin_address::from_string(&claimer_bitcoin_address);
        let full_message = encode_full_message(MessagePrefix, message);
        verify_btc_signature(bitcoin_address, public_key, signature, full_message);
        let claimer = bitcoin_address::to_rooch_address(&bitcoin_address);
        assert!(inviter != claimer, ErrorCannotInviteOneself);
        let invitation_conf = object::borrow_mut(invitation_obj);
        assert!(invitation_conf.is_open, ErrorFaucetNotOpen);
        if (inviter == @rooch_framework){
            claim(faucet_obj, claimer, utxo_ids);
            return
        };
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

    public entry fun claim_from_twitter(
        tweet_id: String,
        invitation_obj: &mut Object<InvitationConf>,
        inviter: address,
        public_key: vector<u8>,
        signature: vector<u8>,
        message: vector<u8>,
    ){
        let bitcoin_address = check_binding_tweet(tweet_id);
        let claimer = bitcoin_address::to_rooch_address(&bitcoin_address);
        assert!(inviter != claimer, ErrorCannotInviteOneself);
        let full_message = encode_full_message(MessagePrefix, message);
        verify_btc_signature(bitcoin_address, public_key, signature, full_message);
        let invitation_conf = object::borrow_mut(invitation_obj);
        assert!(invitation_conf.is_open, ErrorFaucetNotOpen);
        let tweet_obj = tweet_v2::borrow_tweet_object(tweet_id);
        let tweet = object::borrow(tweet_obj);
        let author_id = *tweet_v2::tweet_author_id(tweet);
        if (inviter == @rooch_framework || check_user_claimed(author_id)){
            verify_and_binding_twitter_account(tweet_id);
            return
        };
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

    public fun invitation_user_record(invitation_obj: &Object<InvitationConf>, account: address): &UserInvitationRecords{
        let invitation_conf = object::borrow(invitation_obj);
        table::borrow(&invitation_conf.invitation_records, account)
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

    fun encode_full_message(message_prefix: vector<u8>, message_info: vector<u8>): vector<u8> {
        encode_full_message_consensus(message_prefix, message_info)
    }

    fun starts_with(haystack: &vector<u8>, needle: &vector<u8>): bool {
        let haystack_len = vector::length(haystack);
        let needle_len = vector::length(needle);

        if (needle_len > haystack_len) {
            return false
        };

        let i = 0;
        while (i < needle_len) {
            if (vector::borrow(haystack, i) != vector::borrow(needle, i)) {
                return false
            };
            i = i + 1;
        };

        true
    }

    fun encode_full_message_consensus(message_prefix: vector<u8>, message_info: vector<u8>): vector<u8> {

        let encoder = consensus_codec::encoder();
        consensus_codec::emit_var_slice(&mut encoder, message_prefix);
        consensus_codec::emit_var_slice(&mut encoder, message_info);
        consensus_codec::unpack_encoder(encoder)
    }

    public fun verify_btc_signature(bitcoin_address: BitcoinAddress, public_key: vector<u8>, signature: vector<u8>, message: vector<u8>) {
        let message_hash = hash::sha2_256(message);
        assert!(
            ecdsa_k1::verify(
                &signature,
                &public_key,
                &message_hash,
                ecdsa_k1::sha256()
            ),
            ErrorNoInvitationSignature
        );
        assert!(
            bitcoin_address::verify_bitcoin_address_with_public_key(&bitcoin_address, &public_key),
            ErrorNoInvitationBitcoinSignature
        );
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


    #[test(sender=@0xf0919849a42aa204673b15e586614963649a634851589dfbfde326816bed4161)]
    fun test_claim_with_invitation(sender: &signer){
        bitcoin_move::genesis::init_for_test();
        create_account_for_testing(@0xf0919849a42aa204673b15e586614963649a634851589dfbfde326816bed4161);
        create_account_for_testing(@0x7efa53965d5cdd8c3a6f69e4001a6920a53d427a8b4b99de1d1ceb8bd2e0dc5d);
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
        let signature = x"5a1a4923742b43c73db01430fb0bea005eb54e9d764dbada3f00155981827ab076355636bbd89920ae12b50d91acfd8d5b31e078785afd3fd23928def8b53e41";
        let message = b"hello, rooch";
        let pk = x"02645681b3197f99f8763bccb34fab611778bf61806c2bd2fd8f335e87ed8c23fd";
        let claimer_address= string::utf8(b"bc1pewcwnlshuxedpfywzk9vztnvpj54zmd0a29ydseygtuu7kfjcm9qjngxn0");
        utxo::transfer_for_testing(test_utxo, bitcoin_address::to_rooch_address(&bitcoin_address::from_string(&claimer_address)));
        claim_from_faucet(faucet_obj, invitation_obj, claimer_address, vector[test_utxo_id], @0x7efa53965d5cdd8c3a6f69e4001a6920a53d427a8b4b99de1d1ceb8bd2e0dc5d, pk, signature, message);
        let invitation_obj = object::borrow_mut_object_shared<InvitationConf>(object::named_object_id<InvitationConf>());
        let invitation = object::borrow(invitation_obj);
        let records = table::borrow(&invitation.invitation_records, @0x7efa53965d5cdd8c3a6f69e4001a6920a53d427a8b4b99de1d1ceb8bd2e0dc5d);
        let invitation_user_record = table::borrow(&records.invitation_records, @0xf0919849a42aa204673b15e586614963649a634851589dfbfde326816bed4161);
        assert!(invitation_user_record == &500000000, 1);
        assert!(records.invitation_reward_amount == 500000000, 2);
        assert!(records.total_invitations == 1, 3);
    }
}

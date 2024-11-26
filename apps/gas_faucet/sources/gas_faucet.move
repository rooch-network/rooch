module gas_faucet::gas_faucet {

    use std::vector;

    use moveos_std::signer;
    use moveos_std::object::{Self, Object, ObjectID, to_shared};
    use moveos_std::tx_context::{sender};
    use moveos_std::table::{Self, Table};

    use rooch_framework::coin_store::CoinStore;
    use rooch_framework::gas_coin::RGas;
    use rooch_framework::coin_store;
    use rooch_framework::account_coin_store;
    use rooch_framework::chain_id;

    use bitcoin_move::utxo::{Self, UTXO};

    use app_admin::admin::AdminCap;

    #[test_only]
    use std::signer::address_of;
    #[test_only]
    use moveos_std::account::create_account_for_testing;
    #[test_only]
    use rooch_framework::gas_coin::faucet_for_test;

    const INIT_GAS_AMOUNT: u256 = 5000000_00000000;
    const ONE_RGAS: u256 = 1_00000000;

    //0.01 BTC
    const SAT_LEVEL_ONE: u64 = 1000000;
    //0.1 BTC
    const SAT_LEVEL_TWO: u64 = 10000000;

    const ErrorFaucetNotOpen: u64 = 1;
    const ErrorInvalidUTXO: u64 = 2;
    const ErrorFaucetNotEnoughRGas: u64 = 3;
    const ErrorAlreadyClaimed: u64 = 4;
    const ErrorUTXOValueIsZero: u64 = 5;


    struct RGasFaucet has key{
        rgas_store: Object<CoinStore<RGas>>,
        claim_records: Table<address, u256>, 
        is_open: bool,
        /// Is allow user to claim multiple times
        allow_repeat: bool,
        /// Is require utxo to claim
        require_utxo: bool,
    }

    struct UserInvitationRecords has key, store {
        invitation_records: Table<address, u256>,
        total_invitations: u64,
        total_claim_amount: u256,
    }

    struct InvitationConf has key {
        invitation_records: Table<address, UserInvitationRecords>,
        is_open: bool,
        unit_invitation_amount: u256,
    }


    fun init(sender: &signer) {
      let sender_addr = signer::address_of(sender);
      let rgas_store = coin_store::create_coin_store<RGas>();
      let rgas_balance = account_coin_store::balance<RGas>(sender_addr);
      let faucet_gas_amount = if(rgas_balance > INIT_GAS_AMOUNT) {
        INIT_GAS_AMOUNT
      } else {
        rgas_balance/3
      };
      Self::deposit_to_rgas_store(sender, &mut rgas_store, faucet_gas_amount);
      let allow_repeat = !chain_id::is_main();
      let require_utxo = chain_id::is_main();
      let faucet_obj = object::new_named_object(RGasFaucet{
          rgas_store,
          claim_records: table::new(),
          is_open: true,
          allow_repeat,
          require_utxo,
      });
      to_shared(faucet_obj)
    }

    public entry fun init_invitation(unit_invitation_amount: u256, _admin_cap: &mut Object<AdminCap>) {
        let invitation_obj = object::new_named_object(InvitationConf{
            invitation_records: table::new(),
            is_open: true,
            unit_invitation_amount
        });
        to_shared(invitation_obj)
    }

    /// Anyone can call this function to help the claimer claim the faucet
    public entry fun claim(faucet_obj: &mut Object<RGasFaucet>, claimer: address, utxo_ids: vector<ObjectID>){
      let claim_rgas_amount = Self::check_claim(faucet_obj, claimer, utxo_ids);
      let faucet = object::borrow_mut(faucet_obj);
      let rgas_coin = coin_store::withdraw(&mut faucet.rgas_store, claim_rgas_amount);
      account_coin_store::deposit<RGas>(claimer, rgas_coin);
      let total_claim_amount = table::borrow_mut_with_default(&mut faucet.claim_records, claimer, 0u256);
      *total_claim_amount = *total_claim_amount + claim_rgas_amount;
    }


    /// Anyone can call this function to help the claimer claim the faucet
    public entry fun claim_with_invitation(faucet_obj: &mut Object<RGasFaucet>, invitation_obj: &mut Object<InvitationConf>, claimer: address, utxo_ids: vector<ObjectID>, inviter: address){
        let claim_rgas_amount = Self::check_claim(faucet_obj, claimer, utxo_ids);
        let invitation_conf = object::borrow_mut(invitation_obj);
        assert!(invitation_conf.is_open, ErrorFaucetNotOpen);
        if (!table::contains(&invitation_conf.invitation_records, inviter)) {
            table::add(&mut invitation_conf.invitation_records, inviter, UserInvitationRecords{
                invitation_records: table::new(),
                total_invitations: 0u64,
                total_claim_amount: 0u256,
            })
        };
        let user_invitation_records = table::borrow_mut(&mut invitation_conf.invitation_records, inviter);
        let invitation_amount = table::borrow_mut_with_default(&mut user_invitation_records.invitation_records, claimer, 0u256);
        *invitation_amount = *invitation_amount + claim_rgas_amount;
        user_invitation_records.total_invitations = user_invitation_records.total_invitations + 1u64;
        user_invitation_records.total_claim_amount = user_invitation_records.total_claim_amount + claim_rgas_amount;
        let faucet = object::borrow_mut(faucet_obj);
        let rgas_coin = coin_store::withdraw(&mut faucet.rgas_store, invitation_conf.unit_invitation_amount);
        account_coin_store::deposit<RGas>(inviter, rgas_coin);

        claim(faucet_obj, claimer, utxo_ids);
    }

    public entry fun deposit_rgas_coin(
        account: &signer,
        faucet_obj: &mut Object<RGasFaucet>,
        amount: u256
    ){
        let faucet = object::borrow_mut(faucet_obj);
        deposit_to_rgas_store(account, &mut faucet.rgas_store, amount);
    }

    public entry fun withdraw_rgas_coin( 
        faucet_obj: &mut Object<RGasFaucet>,
        amount: u256,
        _admin: &mut Object<AdminCap>,
    ){
        let faucet = object::borrow_mut(faucet_obj);
        let rgas_coin = coin_store::withdraw(&mut faucet.rgas_store, amount);
        account_coin_store::deposit<RGas>(sender(), rgas_coin);
    }

    fun deposit_to_rgas_store(
        account: &signer,
        rgas_store: &mut Object<CoinStore<RGas>>,
        amount: u256
    ){
        let rgas_coin = account_coin_store::withdraw<RGas>(account, amount);
        coin_store::deposit(rgas_store, rgas_coin);
    }

    /// A view function to check the amount of RGas that can be claimed
    /// Return the amount of RGas that can be claimed
    /// Abort if the claimer is not allowed to claim
    public fun check_claim(faucet_obj: &mut Object<RGasFaucet>, claimer: address, utxo_ids: vector<ObjectID>): u256 {
      let faucet = object::borrow(faucet_obj);
      assert!(faucet.is_open, ErrorFaucetNotOpen);

      if (!faucet.allow_repeat && table::contains(&faucet.claim_records, claimer)) {
        abort ErrorAlreadyClaimed
      };
      
      let total_sat_amount = Self::total_sat_amount(claimer, utxo_ids);
      if (faucet.require_utxo && total_sat_amount == 0) {
        abort ErrorUTXOValueIsZero
      };
      let claim_rgas_amount = Self::sat_amount_to_rgas(total_sat_amount);
      if (claim_rgas_amount == 0) {
        claim_rgas_amount = ONE_RGAS;
      };
      let remaining_rgas_amount = coin_store::balance(&faucet.rgas_store);
      if (claim_rgas_amount > remaining_rgas_amount) {
        abort ErrorFaucetNotEnoughRGas
      };
      claim_rgas_amount
    }

    public fun balance(faucet_obj: &Object<RGasFaucet>): u256 {
        let faucet = object::borrow(faucet_obj);
        coin_store::balance(&faucet.rgas_store)
    }

    public entry fun close_faucet(
        faucet_obj: &mut Object<RGasFaucet>,
        _admin: &mut Object<AdminCap>,
    ){
        let faucet = object::borrow_mut(faucet_obj);
        faucet.is_open = false;
    }

    public entry fun open_faucet(
        faucet_obj: &mut Object<RGasFaucet>,
        _admin: &mut Object<AdminCap>,
    ) {
        let faucet = object::borrow_mut(faucet_obj);
        faucet.is_open = true;
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

    public entry fun set_allow_repeat(
        faucet_obj: &mut Object<RGasFaucet>,
        allow_repeat: bool,
        _admin: &mut Object<AdminCap>,
    ) {
        let faucet = object::borrow_mut(faucet_obj);
        faucet.allow_repeat = allow_repeat;
    }

    public entry fun set_require_utxo(
        faucet_obj: &mut Object<RGasFaucet>,
        require_utxo: bool,
        _admin: &mut Object<AdminCap>,
    ) {
        let faucet = object::borrow_mut(faucet_obj);
        faucet.require_utxo = require_utxo;
    }

    fun sat_amount_to_rgas(sat_amount: u64): u256{
      if (sat_amount == 0) {
        0
      }else if(sat_amount <= SAT_LEVEL_ONE){
        ONE_RGAS
      }else if(sat_amount <= SAT_LEVEL_TWO){
        2 * ONE_RGAS
      }else{
        3 * ONE_RGAS
      }
    }

    fun total_sat_amount(claim_address: address, utxo_ids: vector<ObjectID>): u64{
      let total_sat_amount = 0;
      vector::for_each(utxo_ids, |utxo_id| {
        let utxo_obj = object::borrow_object<UTXO>(utxo_id);
        assert!(object::owner(utxo_obj) == claim_address, ErrorInvalidUTXO);
        total_sat_amount = total_sat_amount + utxo::value(object::borrow(utxo_obj));
      });
      total_sat_amount
    }

    #[test(sender=@0x42)]
    fun test_claim_with_invitation(sender: &signer){
        bitcoin_move::genesis::init_for_test();
        create_account_for_testing(@0x42);
        create_account_for_testing(@0x43);
        let invitation_obj = object::new_named_object(InvitationConf{
            invitation_records: table::new(),
            is_open: true,
            unit_invitation_amount: 1,
        });
        faucet_for_test(address_of(sender), INIT_GAS_AMOUNT);
        init(sender);
        object::to_shared(invitation_obj);
        let faucet_obj = object::borrow_mut_object_shared<RGasFaucet>(object::named_object_id<RGasFaucet>());
        let invitation_obj = object::borrow_mut_object_shared<InvitationConf>(object::named_object_id<InvitationConf>());
        let tx_id = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let sat_value = 100000000;
        let test_utxo = utxo::new_for_testing(tx_id, 0u32, sat_value);
        let test_utxo_id = object::id(&test_utxo);
        utxo::transfer_for_testing(test_utxo, @0x43);
        claim_with_invitation(faucet_obj, invitation_obj, @0x43, vector[test_utxo_id], @0x42);
        let invitation_obj = object::borrow_mut_object_shared<InvitationConf>(object::named_object_id<InvitationConf>());
       let invitation = object::borrow(invitation_obj);
        let records = table::borrow(&invitation.invitation_records, @0x42);
        let invitation_user_record = table::borrow(&records.invitation_records, @0x43);
        assert!(invitation_user_record == &300000000, 1);
        assert!(records.total_claim_amount == 300000000, 2);
        assert!(records.total_invitations == 1, 3);
    }

}

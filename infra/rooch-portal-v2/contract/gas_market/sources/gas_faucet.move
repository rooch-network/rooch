module gas_market::gas_faucet {

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

    use gas_market::gas_market::AdminCap;

    const INIT_GAS_AMOUNT: u256 = 5000000_00000000;
    const ONE_RGAS: u256 = 1_00000000;

    //0.01 BTC
    const SAT_LEVEL_ONE: u64 = 1000000;
    //0.1 BTC
    const SAT_LEVEL_TWO: u64 = 10000000;

    const ErrorAirdropNotOpen: u64 = 0;
    const ErrorInvalidUTXO: u64 = 1;
    const ErrorAirdropNotEnoughRGas: u64 = 2;
    const ErrorAlreadyClaimed: u64 = 3;
    const ErrorUTXOValueIsZero: u64 = 4;


    struct RGasFaucet has key{
        rgas_store: Object<CoinStore<RGas>>,
        claim_records: Table<address, u256>, 
        is_open: bool,
        /// Is allow user to claim multiple times
        allow_repeat: bool,
        /// Is require utxo to claim
        require_utxo: bool,
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

    /// Anyone can call this function to help the claimer claim the faucet
    public entry fun claim(faucet_obj: &mut Object<RGasFaucet>, claimer: address, utxo_ids: vector<ObjectID>){      
      let faucet = object::borrow_mut(faucet_obj);
      assert!(faucet.is_open, ErrorAirdropNotOpen);
      if (!faucet.allow_repeat) {
        assert!(!table::contains(&faucet.claim_records, claimer), ErrorAlreadyClaimed);
      };
      let total_sat_amount = Self::total_sat_amount(claimer, utxo_ids);
      let claim_rgas_amount = Self::sat_amount_to_rgas(total_sat_amount);
      if (claim_rgas_amount == 0) {
        if(faucet.require_utxo){
          abort ErrorUTXOValueIsZero
        }else{
          claim_rgas_amount = ONE_RGAS;
        }
      };

      let remaining_rgas_amount = coin_store::balance(&faucet.rgas_store);
      assert!(claim_rgas_amount <= remaining_rgas_amount, ErrorAirdropNotEnoughRGas);
      let rgas_coin = coin_store::withdraw(&mut faucet.rgas_store, claim_rgas_amount);
      account_coin_store::deposit<RGas>(claimer, rgas_coin);
      let total_claim_amount = table::borrow_mut_with_default(&mut faucet.claim_records, claimer, 0u256);
      *total_claim_amount = *total_claim_amount + claim_rgas_amount;
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
        _admin: &mut Object<AdminCap>,
        faucet_obj: &mut Object<RGasFaucet>,
        amount: u256
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

    /// A view function to get the amount of RGas that can be claimed
    public fun get_claimable_rgas(claimer: address, utxo_ids: vector<ObjectID>): u256 {
      let total_sat_amount = Self::total_sat_amount(claimer, utxo_ids);
      Self::sat_amount_to_rgas(total_sat_amount)
    }

    public entry fun close_faucet(
        _admin: &mut Object<AdminCap>,
        faucet_obj: &mut Object<RGasFaucet>
    ){
        let faucet = object::borrow_mut(faucet_obj);
        faucet.is_open = false;
    }

    public entry fun open_faucet(
        _admin: &mut Object<AdminCap>,
        faucet_obj: &mut Object<RGasFaucet>
    ) {
        let faucet = object::borrow_mut(faucet_obj);
        faucet.is_open = true;
    }

    public entry fun set_allow_repeat(
        _admin: &mut Object<AdminCap>,
        faucet_obj: &mut Object<RGasFaucet>,
        allow_repeat: bool
    ) {
        let faucet = object::borrow_mut(faucet_obj);
        faucet.allow_repeat = allow_repeat;
    }

    public entry fun set_require_utxo(
        _admin: &mut Object<AdminCap>,
        faucet_obj: &mut Object<RGasFaucet>,
        require_utxo: bool
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

}

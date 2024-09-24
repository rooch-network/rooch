module gas_market::gas_airdrop {

    use std::vector;

    use moveos_std::signer;
    use moveos_std::object::{Self, Object, ObjectID, to_shared};
    use moveos_std::tx_context::{sender};

    use rooch_framework::coin_store::CoinStore;
    use rooch_framework::gas_coin::RGas;
    use rooch_framework::coin_store;
    use rooch_framework::account_coin_store;

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


    struct RGasAirdrop has key{
        rgas_store: Object<CoinStore<RGas>>,
        is_open: bool
    }


    fun init(sender: &signer) {
      let sender_addr = signer::address_of(sender);
      let rgas_store = coin_store::create_coin_store<RGas>();
      let rgas_balance = account_coin_store::balance<RGas>(sender_addr);
      let airdrop_gas_amount = if(rgas_balance > INIT_GAS_AMOUNT) {
        INIT_GAS_AMOUNT
      } else {
        rgas_balance/3
      };
      Self::deposit_to_rgas_store(sender, &mut rgas_store, airdrop_gas_amount);
      let rgas_airdrop_obj = object::new_named_object(RGasAirdrop{
          rgas_store,
          is_open: true
      });
      to_shared(rgas_airdrop_obj)
    }

    /// Anyone can call this function to help the claimer claim the airdrop
    public fun claim(claimer: address, utxo_ids: vector<ObjectID>){
      let total_sat_amount = Self::total_sat_amount(claimer, utxo_ids);
      let claim_rgas_amount = Self::sat_amount_to_rgas(total_sat_amount);
      let airdrop_obj_id = object::named_object_id<RGasAirdrop>();
      let airdrop_obj = object::borrow_mut_object_extend<RGasAirdrop>(airdrop_obj_id);
      let airdrop = object::borrow_mut(airdrop_obj);

      let remaining_rgas_amount = coin_store::balance(&airdrop.rgas_store);
      assert!(claim_rgas_amount <= remaining_rgas_amount, ErrorAirdropNotEnoughRGas);
      let rgas_coin = coin_store::withdraw(&mut airdrop.rgas_store, claim_rgas_amount);
      account_coin_store::deposit<RGas>(claimer, rgas_coin);
    }

    public entry fun deposit_rgas_coin(
        account: &signer,
        rgas_airdrop_obj: &mut Object<RGasAirdrop>,
        amount: u256
    ){
        let rgas_airdrop = object::borrow_mut(rgas_airdrop_obj);
        deposit_to_rgas_store(account, &mut rgas_airdrop.rgas_store, amount);
    }

    public entry fun withdraw_rgas_coin(
        _admin: &mut Object<AdminCap>,
        rgas_airdrop_obj: &mut Object<RGasAirdrop>,
        amount: u256
    ){
        let rgas_airdrop = object::borrow_mut(rgas_airdrop_obj);
        let rgas_coin = coin_store::withdraw(&mut rgas_airdrop.rgas_store, amount);
        account_coin_store::deposit<RGas>(sender(), rgas_coin);
    }

    public entry fun close_airdrop(
        _admin: &mut Object<AdminCap>,
        rgas_airdrop_obj: &mut Object<RGasAirdrop>
    ){
        let rgas_airdrop = object::borrow_mut(rgas_airdrop_obj);
        rgas_airdrop.is_open = false;
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

    fun sat_amount_to_rgas(sat_amount: u64): u256{
      if(sat_amount <= SAT_LEVEL_ONE){
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

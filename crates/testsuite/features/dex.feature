Feature: RoochDAO Apps contract tests

   
    @serial
    Scenario: rooch_dex
      Given a server for rooch_dex

      Then cmd: "account create --json"
      Then cmd: "account create --json"
      Then cmd: "account list --json" 
      
      Then cmd: "move run --sender {{$.account[2].account0.address}} --function rooch_framework::gas_coin::faucet_entry --args u256:1000000000000000 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "move run --sender {{$.account[2].account1.address}} --function rooch_framework::gas_coin::faucet_entry --args u256:1000000000000000 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
    
      # publish rooch_dex via default address
      Then cmd: "move publish -p ../../apps/app_admin  --named-addresses app_admin=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      Then cmd: "move publish -p ../../apps/rooch_dex --named-addresses app_admin=default,rooch_dex=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # publish examples coins via default address
      Then cmd: "move publish -p ../../examples/coins  --named-addresses coins=default --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "object -t default::admin::AdminCap"
      Then cmd: "object -t default::fixed_supply_coin::Treasury"
     
      #Get some test coins
      Then cmd: "move run --function default::fixed_supply_coin::faucet --args object:{{$.object[1].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      # Create toke pair
      Then cmd: "move run --function default::router::create_token_pair --type-args 0x3::gas_coin::RGas --type-args default::fixed_supply_coin::FSC --args u64:100000000000 --args u64:100000000000 --args u64:0 --args u64:0 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "object -t '0x3::coin::CoinInfo<default::swap::LPToken<0x3::gas_coin::RGas,default::fixed_supply_coin::FSC>>'"

      #Add add_liquidity
      Then cmd: "move run --function default::router::add_liquidity --type-args 0x3::gas_coin::RGas --type-args default::fixed_supply_coin::FSC  --args u64:100000000000 --args u64:100000000000 --args u64:0 --args u64:0 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      #Check lp token balance
      Then cmd: "account balance --coin-type default::swap::LPToken<0x3::gas_coin::RGas,default::fixed_supply_coin::FSC> --json"
      Then assert: "{{$.account[3].RDexLP.balance}} != 0"

      #Before swap the balance should be 0
      Then cmd: "account balance --coin-type default::fixed_supply_coin::FSC --address {{$.account[2].account1.address}} --json"
      Then assert: "{{$.account[-1].FSC.balance}} == 0"

      #SWAP
      Then cmd: "move run --sender {{$.account[2].account1.address}} --function default::router::swap_with_exact_input --type-args 0x3::gas_coin::RGas --type-args default::fixed_supply_coin::FSC --args u64:100000000 --args u64:0 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
      
      Then cmd: "account balance --coin-type default::fixed_supply_coin::FSC --address {{$.account[2].account1.address}} --json"
      Then assert: "{{$.account[-1].FSC.balance}} != 0"

      #Create incentive pool
      Then cmd: "move run --function default::admin_util::create_xp_incentive_pool --type-args 0x3::gas_coin::RGas --type-args default::fixed_supply_coin::FSC --args u128:10000 --args u256:10000000000 --args u64:0 --args object:{{$.object[0].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      Then cmd: "object -t 'default::liquidity_incentive::FarmingAsset<0x3::gas_coin::RGas,default::fixed_supply_coin::FSC,default::liquid_xp::LiquidXP>'"

      #stake the LPToken
      Then cmd: "move run --function default::liquidity_incentive::stake --type-args 0x3::gas_coin::RGas --type-args default::fixed_supply_coin::FSC --type-args default::liquid_xp::LiquidXP --args u256:{{$.account[3].RDexLP.balance}} --args object:{{$.object[-1].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      #Fast forward timestamp
      Then cmd: "move run --function 0x3::timestamp::fast_forward_seconds_for_local --args u64:1000 --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      #Harvest
      Then cmd: "move run --function default::liquidity_incentive::harvest --type-args 0x3::gas_coin::RGas --type-args default::fixed_supply_coin::FSC --type-args default::liquid_xp::LiquidXP --args object:{{$.object[-1].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

      #Check LPXP balance
      Then cmd: "account balance --coin-type default::liquid_xp::LiquidXP --json"
      Then assert: "{{$.account[-1].LPXP.balance}} != 0"

      #Unstake
      Then cmd: "move run --function default::liquidity_incentive::unstake --type-args 0x3::gas_coin::RGas --type-args default::fixed_supply_coin::FSC --type-args default::liquid_xp::LiquidXP --args object:{{$.object[-1].data[0].id}} --json"
      Then assert: "{{$.move[-1].execution_info.status.type}} == executed"



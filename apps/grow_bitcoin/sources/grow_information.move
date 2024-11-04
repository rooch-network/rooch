module grow_bitcoin::grow_information {

    use std::string::String;
    use std::vector::length;
    use rooch_framework::account_coin_store;
    use moveos_std::timestamp::now_milliseconds;
    use moveos_std::event::emit;
    use moveos_std::tx_context::sender;
    use grow_bitcoin::grow_point::mint_point_box;
    use rooch_framework::coin;
    use rooch_framework::coin::Coin;
    use rooch_framework::coin_store;
    use app_admin::admin::AdminCap;
    use moveos_std::table;
    use moveos_std::object;
    use moveos_std::table::Table;
    use grow_bitcoin::grow_bitcoin::GROW;
    use moveos_std::object::{Object, ObjectID, to_shared};
    use rooch_framework::coin_store::CoinStore;


    const ErrorProjectAleardyExist: u64 = 1;
    const ErrorGrowMetaLength: u64 = 2;
    const ErrorVoteNotOpen:u64 = 3;

    struct GrowProject has key, store {
        id: u64,
        vote_store: Object<CoinStore<GROW>>,
        vote_detail: Table<address, u256>,
        metadata: GrowMeta
    }

    struct GrowMeta has store, drop {
        key: vector<String>,
        value: vector<String>
    }

    struct GrowProjectList has key, store {
        project_list: Table<u64, ObjectID>,
        is_open: bool
    }

    struct VoteEvent has copy, store, drop {
        id: u64,
        value: u256,
        timestamp: u64
    }

    fun init(){
        let grow_project_list_obj= object::new_named_object(GrowProjectList{
            project_list: table::new(),
            is_open: true
        });
        object::to_shared(grow_project_list_obj)
    }

    public entry fun new_project(_admin: &mut Object<AdminCap>, grow_project_list_obj: &mut Object<GrowProjectList>, id: u64) {
        let grow_project_list = object::borrow_mut(grow_project_list_obj);
        assert!(!table::contains(&grow_project_list.project_list, id), ErrorProjectAleardyExist);
        let new_grow_project = object::new(GrowProject{
            id,
            vote_store: coin_store::create_coin_store(),
            vote_detail: table::new(),
            metadata: GrowMeta{
                key: vector[],
                value: vector[]
            }
        });
        let grow_project_id = object::id(&new_grow_project);
        table::add(&mut grow_project_list.project_list, id, grow_project_id);
        to_shared(new_grow_project)
    }

    public entry fun update_project_meta(_admin: &mut Object<AdminCap>, grow_project_obj: &mut Object<GrowProject>, key: vector<String>, value: vector<String>){
        assert!(length(&key) == length(&value), ErrorGrowMetaLength);
        let grow_project = object::borrow_mut(grow_project_obj);
        grow_project.metadata = GrowMeta {
            key,
            value
        }
    }

    public entry fun vote_entry(
        account: &signer,
        grow_project_obj: &mut Object<GrowProject>,
        grow_project_list_obj: &Object<GrowProjectList>,
        grow_value: u256
    ){
        let coin = account_coin_store::withdraw<GROW>(account, grow_value);
        vote(grow_project_obj, grow_project_list_obj, coin)
    }

    public fun vote(grow_project_obj: &mut Object<GrowProject>, grow_project_list_obj: &Object<GrowProjectList>, coin: Coin<GROW>) {
        let coin_value = coin::value(&coin);
        let grow_project = object::borrow_mut(grow_project_obj);
        assert!(object::borrow(grow_project_list_obj).is_open, ErrorVoteNotOpen);
        coin_store::deposit(&mut grow_project.vote_store, coin);
        if (!table::contains(&grow_project.vote_detail, sender())){
            table::add(&mut grow_project.vote_detail, sender(), coin_value)
        }else {
            *table::borrow_mut(&mut grow_project.vote_detail, sender()) + coin_value;
        };
        emit(VoteEvent{
            id: grow_project.id,
            value: coin_value,
            timestamp: now_milliseconds()
        });
        let point_box = mint_point_box(grow_project.id, coin_value, sender());
        object::transfer(point_box, sender());
    }


    public entry fun open_vote(_admin: &mut Object<AdminCap>, grow_project_list_obj: &mut Object<GrowProjectList>){
        object::borrow_mut(grow_project_list_obj).is_open = true
    }

    public entry fun close_vote(_admin: &mut Object<AdminCap>, grow_project_list_obj: &mut Object<GrowProjectList>){
        object::borrow_mut(grow_project_list_obj).is_open = false
    }

    public fun borrow_grow_project(grow_project_list_obj: &Object<GrowProjectList>, id: u64): &Object<GrowProject> {
        let grow_project_list = object::borrow(grow_project_list_obj);
        let object_id = table::borrow(&grow_project_list.project_list, id);
        object::borrow_object(*object_id)
    }
}

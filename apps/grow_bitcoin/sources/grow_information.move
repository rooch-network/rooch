module grow_bitcoin::grow_information_v3 {

    use std::signer::address_of;
    use std::string::String;
    use std::vector::length;
    use moveos_std::account;
    use rooch_framework::account_coin_store;
    use moveos_std::timestamp::now_milliseconds;
    use moveos_std::event::emit;
    use moveos_std::tx_context::sender;
    use grow_bitcoin::grow_point_v3::mint_point_box;
    use rooch_framework::coin;
    use rooch_framework::coin::Coin;
    use rooch_framework::coin_store;
    use app_admin::admin;
    use moveos_std::table;
    use moveos_std::object;
    use moveos_std::table::Table;
    use grow_bitcoin::grow_bitcoin::GROW;
    use moveos_std::object::{Object, transfer, ObjectID};
    use rooch_framework::coin_store::CoinStore;


    const ErrorProjectAleardyExist: u64 = 1;
    const ErrorGrowMetaLength: u64 = 2;
    const ErrorVoteNotOpen:u64 = 3;

    struct GrowProject has key, store {
        id: String,
        vote_store: Object<CoinStore<GROW>>,
        vote_value: u256,
        vote_detail: Table<address, u256>,
        metadata: GrowMeta
    }

    struct GrowMeta has store, drop {
        key: vector<String>,
        value: vector<String>
    }

    struct GrowProjectList has key, store {
        project_list: Table<String, GrowProject>,
        is_open: bool
    }

    struct UserVoteInfo has key {
        vote_info: Table<String, u256>
    }

    struct VoteEvent has copy, store, drop {
        id: String,
        value: u256,
        timestamp: u64
    }

    struct ProjectCap has store, key {}

    fun init(){
        let grow_project_list_obj= object::new_named_object(GrowProjectList{
            project_list: table::new(),
            is_open: true
        });
        object::to_shared(grow_project_list_obj);
        object::transfer(object::new_named_object(ProjectCap{}), sender())
    }

    public entry fun create_admin(_admin: &mut Object<admin::AdminCap>, receiver: address){
        let new_admin = object::new(ProjectCap{});
        transfer(new_admin, receiver)
    }

    public entry fun delete_admin(_admin: &mut Object<admin::AdminCap>, admin_id: ObjectID){
        let admin_obj = object::take_object_extend<ProjectCap>(admin_id);
        let ProjectCap{} = object::remove(admin_obj);
    }

    public entry fun new_project(grow_project_list_obj: &mut Object<GrowProjectList>, id: String, _admin: &mut Object<ProjectCap>) {
        let grow_project_list = object::borrow_mut(grow_project_list_obj);
        assert!(!table::contains(&grow_project_list.project_list, id), ErrorProjectAleardyExist);
        table::add(&mut grow_project_list.project_list, id, GrowProject{
            id,
            vote_store: coin_store::create_coin_store(),
            vote_value: 0,
            vote_detail: table::new(),
            metadata: GrowMeta{
                key: vector[],
                value: vector[]
            }
        });
    }

    public entry fun update_project_meta(grow_project_list_obj: &mut Object<GrowProjectList>, id: String, key: vector<String>, value: vector<String>, _admin: &mut Object<ProjectCap>){
        assert!(length(&key) == length(&value), ErrorGrowMetaLength);
        let grow_project = borrow_mut_grow_project(grow_project_list_obj, id);
        grow_project.metadata = GrowMeta {
            key,
            value
        }
    }

    public entry fun vote_entry(
        account: &signer,
        grow_project_list_obj: &mut Object<GrowProjectList>,
        id: String,
        grow_value: u256
    ){
        let coin = account_coin_store::withdraw<GROW>(account, grow_value);
        vote(account, grow_project_list_obj, id, coin)
    }
    public fun vote(
        account: &signer,
        grow_project_list_obj: &mut Object<GrowProjectList>,
        id: String,
        coin: Coin<GROW>
    ) {
        let coin_value = coin::value(&coin);
        assert!(object::borrow(grow_project_list_obj).is_open, ErrorVoteNotOpen);
        let grow_project = borrow_mut_grow_project(grow_project_list_obj, id);
        coin_store::deposit(&mut grow_project.vote_store, coin);
        let vote_detail = table::borrow_mut_with_default(&mut grow_project.vote_detail, sender(), 0);
        *vote_detail = *vote_detail + coin_value;

        grow_project.vote_value = coin_store::balance(&grow_project.vote_store);
        if (!account::exists_resource<UserVoteInfo>(address_of(account))) {
            account::move_resource_to(account, UserVoteInfo{
                vote_info: table::new()
            })
        };
        let user_vote_info = account::borrow_mut_resource<UserVoteInfo>(address_of(account));
        let vote_info =  table::borrow_mut_with_default(&mut user_vote_info.vote_info, id, 0);
        *vote_info = *vote_info + coin_value;
        emit(VoteEvent{
            id: grow_project.id,
            value: coin_value,
            timestamp: now_milliseconds()
        });
        let point_box = mint_point_box(grow_project.id, coin_value, sender());
        object::transfer(point_box, sender());
    }

    public fun get_vote(grow_project_list_obj: &Object<GrowProjectList>, user: address, id: String): u256 {
        let grow_project = borrow_grow_project(grow_project_list_obj, id);
        *table::borrow(&grow_project.vote_detail, user)
    }


    public entry fun open_vote(grow_project_list_obj: &mut Object<GrowProjectList>, _admin: &mut Object<ProjectCap>){
        object::borrow_mut(grow_project_list_obj).is_open = true
    }

    public entry fun close_vote(grow_project_list_obj: &mut Object<GrowProjectList>, _admin: &mut Object<ProjectCap>){
        object::borrow_mut(grow_project_list_obj).is_open = false
    }

    public fun borrow_grow_project(grow_project_list_obj: &Object<GrowProjectList>, id: String): &GrowProject {
        let grow_project_list = object::borrow(grow_project_list_obj);
        table::borrow(&grow_project_list.project_list, id)
    }

    fun borrow_mut_grow_project(grow_project_list_obj: &mut Object<GrowProjectList>, id: String): &mut GrowProject {
        let grow_project_list = object::borrow_mut(grow_project_list_obj);
        table::borrow_mut(&mut grow_project_list.project_list, id)
    }
}

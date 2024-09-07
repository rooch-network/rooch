// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// The move version inscription_updater
/// https://github.com/ordinals/ord/blob/e59bd3e73d30ed9bc0b252ba2084bba670d6b0db/src/index/updater/inscription_updater.rs
module bitcoin_move::inscription_updater{

    use std::vector;
    use std::option::{Self, Option};
    use std::string::String;
    
    use moveos_std::object::{Self, Object};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::sort;
    use moveos_std::event;
    use moveos_std::type_info;

    use bitcoin_move::network;
    use bitcoin_move::types::{Self, Transaction, TxOut};
    use bitcoin_move::ord::{Self, InscriptionID, SatPoint, Inscription, Envelope, InscriptionRecord};
    use bitcoin_move::pending_block::{Self, PendingBlock};
    use bitcoin_move::utxo::{Self, UTXO, SealOut};
    use bitcoin_move::script_buf;

    friend bitcoin_move::bitcoin;

    const ORDINAL_GENESIS_HEIGHT:u64 = 767430;

    const ErrorUTXOBalanceNotMatch: u64 = 1;
    const ErrorFlotsamNotProcessed: u64 = 2;

    struct FlotsamNew has copy, drop, store{
        cursed: bool,
        fee: u64,
        hidden: bool,
        parents: vector<InscriptionID>,
        pointer: Option<u64>,
        reinscription: bool,
        unbound: bool,
        vindicated: bool,
        envelope: Envelope<InscriptionRecord>,
    }

    struct Flotsam has copy, drop, store {
        inscription_id: InscriptionID,
        offset: u64,
        new: Option<FlotsamNew>,
        old: Option<SatPoint>,
    }


    /// Triggered when a new inscription is created
    /// @param block_height: The block height at which the inscription is created
    /// @param charms: The charm value of the inscription, representing its special attributes
    /// @param inscription_id: The unique identifier of the newly created inscription
    /// @param location: The location of the inscription, which may be None
    /// @param parent_inscription_ids: A list of parent inscription IDs, used to represent relationships between inscriptions
    /// @param sequence_number: The sequence number of the inscription
    struct InscriptionCreatedEvent has copy, drop, store {
        block_height: u64,
        charms: u16,
        inscription_id: InscriptionID,
        location: Option<SatPoint>,
        parent_inscription_ids: vector<InscriptionID>,
        sequence_number: u32,
    }
    
    /// Triggered when an inscription is transferred
    /// @param block_height: The block height at which the inscription is transferred
    /// @param inscription_id: The unique identifier of the inscription being transferred
    /// @param new_location: The new location of the inscription
    /// @param old_location: The old location of the inscription
    /// @param sequence_number: The sequence number of the inscription
    /// @param is_burned: A boolean indicating whether the inscription is burned
    struct InscriptionTransferredEvent has copy, drop, store {
        block_height: u64,
        inscription_id: InscriptionID,
        new_location: SatPoint,
        old_location: SatPoint,
        sequence_number: u32,
        is_burned: bool,
    }

    struct InscriptionUpdater has store {
        block_height: u64,
        seal_protocol: String,
        flotsams: vector<Flotsam>,
        lost_sats: u64,
        reward: u64,
        blessed_inscription_count: u32,
        cursed_inscription_count: u32,
        unbound_inscription_count: u32,
        next_sequence_number: u32,
    }

    struct Location has copy, drop, store {
        new_satpoint: SatPoint,
        flotsam: Flotsam,
        is_op_return: bool,
        owner: address,
    }

    struct Range has copy, drop, store {
        start: u64,
        end: u64,
    }

    struct ReinscribeCounter has copy, drop, store{
        inscription_id: InscriptionID,
        count: u64,
    }

    public(friend) fun process_tx(pending_block: &mut Object<PendingBlock>, tx: &Transaction, input_utxos: &mut vector<UTXO>): vector<SealOut> {
        let block_height = pending_block::block_height(pending_block);
        let seal_outs = vector::empty();
        if(!need_process_oridinals(block_height)){
            return seal_outs
        };
        let check_utxo_input = utxo::check_utxo_input();
        let seal_protocol = type_info::type_name<Inscription>();

        let txid = types::tx_id(tx);
        let txinput = types::tx_input(tx);
        let txoutput: &vector<TxOut> = types::tx_output(tx);
        let is_coinbase = types::is_coinbase_tx(tx);

        let id_counter = 0;
        let inscribed_offsets = simple_map::new<u64, ReinscribeCounter>();
        let floating_inscriptions = vector::empty();

        let jubilant = block_height >= network::jubilee_height();

        let total_input_value = 0;

        let output_len = vector::length(txoutput);
        let output_idx = 0;
        let total_output_value = 0;
        while(output_idx < output_len){
            total_output_value = total_output_value + types::txout_value(vector::borrow(txoutput, output_idx));
            output_idx = output_idx + 1;
        };

        let envelopes = ord::parse_inscription_from_tx(tx);
        //reverse the envelopes for pop back to iterate
        vector::reverse(&mut envelopes);

        let updater = if (pending_block::exists_intermediate<InscriptionUpdater>(pending_block)){
            pending_block::take_intermediate<InscriptionUpdater>(pending_block)
        }else{
            let inscription_store = ord::borrow_inscription_store();
            let blessed_inscription_count = ord::blessed_inscription_count(inscription_store);
            let cursed_inscription_count = ord::cursed_inscription_count(inscription_store);
            let unbound_inscription_count = ord::unbound_inscription_count(inscription_store);
            let lost_sats = ord::lost_sats(inscription_store);
            let next_sequence_number = ord::next_sequence_number(inscription_store);
            InscriptionUpdater{
                block_height,
                seal_protocol,
                flotsams: vector::empty(),
                lost_sats,
                reward: network::subsidy_by_height(block_height),
                blessed_inscription_count,
                cursed_inscription_count,
                unbound_inscription_count,
                next_sequence_number,
            }
        };
        
        let input_idx = 0;
        let input_len = vector::length(txinput);
        while(input_idx < input_len){
            let input = vector::borrow(txinput, input_idx);
            // skip subsidy since no inscriptions possible
            if (types::is_null_outpoint(types::txin_previous_output(input))){
                total_input_value = total_input_value + network::subsidy_by_height(block_height);
                input_idx = input_idx + 1;
                continue
            };
            let utxo = vector::borrow_mut(input_utxos, input_idx);

            //Process inscription transfer
            let seals = utxo::remove_seals_internal<Inscription>(utxo);
            
            let seal_idx = 0;
            let seals_len = vector::length(&seals);
            while (seal_idx < seals_len) {
                let seal_object_id = *vector::borrow(&mut seals, seal_idx);
                let inscription_obj = ord::borrow_object(seal_object_id);
                let inscription = object::borrow(inscription_obj);
                let inscription_id = *ord::id(inscription);

                let old_location = ord::location(inscription);
                
                let offset = total_input_value + ord::satpoint_offset(old_location);

                let flotsam = Flotsam {
                    inscription_id: inscription_id,
                    offset: offset,
                    new: option::none(),
                    old: option::some(*old_location),
                };
                vector::push_back(&mut floating_inscriptions, flotsam);
                if(simple_map::contains_key(&inscribed_offsets, &offset)){
                    let counter = simple_map::borrow_mut(&mut inscribed_offsets, &offset);
                    counter.count = counter.count + 1;
                }else{
                    simple_map::add(&mut inscribed_offsets, offset, ReinscribeCounter{inscription_id, count: 1});
                };
                seal_idx = seal_idx + 1;
            };

            // Process inscription creation
            let offset = total_input_value;
            total_input_value = total_input_value + utxo::value(utxo);
            let ins_idx = 0;
            let ins_len = vector::length(&envelopes);

            while (ins_idx < ins_len) {
                let envelope = vector::pop_back(&mut envelopes);
                let input = ord::envelope_input(&envelope);
                let payload = ord::envelope_payload(&envelope);
                if (input != (input_idx as u32)){
                    vector::push_back(&mut envelopes, envelope);
                    break
                };
                let inscription_id = ord::new_inscription_id(txid, id_counter);
                let pointer = *ord::inscription_record_pointer(payload);
                let parents = *ord::inscription_record_parents(payload);
                
                let unrecongized_even_field = ord::inscription_record_unrecognized_even_field(payload);
                
                //handle curse before fix the offset via pointer
                let curse = handle_curse_inscription(&envelope, offset, &inscribed_offsets);
                let offset = if (option::is_some(&pointer)){
                    let p = option::destroy_some(pointer);
                    if (p < total_output_value){
                        p
                    }else{
                        offset
                    }
                }else{
                    offset
                };

                let flotsam = Flotsam{
                    inscription_id: inscription_id,
                    offset: offset,
                    new: option::some(FlotsamNew{
                        cursed: option::is_some(&curse) && !jubilant,
                        fee: 0,
                        //We do not handle the hidden
                        hidden: false,
                        parents,
                        pointer,
                        reinscription: simple_map::contains_key(&inscribed_offsets, &offset),
                        unbound: (utxo::value(utxo) == 0 
                            || curse == option::some(curse_unrecognized_even_field())
                            || unrecongized_even_field),
                        vindicated: option::is_some(&curse) && jubilant,
                        envelope: envelope,
                    }),
                    old: option::none(),
                };
                vector::push_back(&mut floating_inscriptions, flotsam);
                id_counter = id_counter + 1;
                ins_idx = ins_idx + 1;
            };
            input_idx = input_idx + 1;
        };

        //We do not validate the parent here.
        //And we also not store the fee

        if(is_coinbase) {
            //remove all the flotsams from the previous txs
            let flotsams = vector::trim(&mut updater.flotsams, 0);
            vector::append(&mut floating_inscriptions, flotsams);
        };

        sort::sort_by_key(&mut floating_inscriptions, |flotsam| {
            let flotsam: &Flotsam = flotsam;
            &flotsam.offset
        });

        let range_to_vout = simple_map::new();
        let new_locations = vector::empty();
        let output_value = 0;

        let output_idx = 0;
        let output_len = vector::length(txoutput);
        let flotsam_idx = 0;
        let flotsam_len = vector::length(&floating_inscriptions);
        while(output_idx < output_len){
            //Skip the output process if there is no flotsam
            if(flotsam_len == 0){
                break
            };
            let output = vector::borrow(txoutput, output_idx);
            let value = types::txout_value(output);
            let output_script_buf = types::txout_script_pubkey(output);
            let is_op_return = script_buf::is_op_return(output_script_buf);
            let output_address = types::txout_object_address(output);
            let end = output_value + value;
            
            while(flotsam_idx < flotsam_len){
                let flotsam = vector::borrow(&floating_inscriptions, flotsam_idx);
                if (flotsam.offset >= end){
                    break
                };

                let new_satpoint = ord::new_satpoint(types::new_outpoint(txid, (output_idx as u32)), flotsam.offset - output_value);

                vector::push_back(&mut new_locations,
                Location{
                    new_satpoint,
                    flotsam: *flotsam,
                    is_op_return,
                    owner: output_address,
                });
                flotsam_idx = flotsam_idx + 1;
            };
            
            simple_map::add(&mut range_to_vout, output_idx, Range{start: output_value, end: end});
            output_value = end;

            output_idx = output_idx + 1;
        };

        let new_locations_len = vector::length(&new_locations);
        let new_locations_idx = 0;
        while(new_locations_idx < new_locations_len){
            let location = vector::borrow(&new_locations, new_locations_idx);
            let new_satpoint = if (option::is_some(&location.flotsam.new)){
            let new_info = option::borrow(&location.flotsam.new);
            if (option::is_some(&new_info.pointer)) {
                let pointer = *option::borrow(&new_info.pointer);
                if (pointer < output_value) {
                    let (found, vout, start) = find_range_for_pointer(&range_to_vout, pointer);
                    if (found) {
                        //The ordinals code update the flotsam offset via pointer.
                        //And use the offset to calculate Sat, but we do not handle the Sat,
                        //So this update is useless. 
                        //location.flotsam.offset = pointer;
                        ord::new_satpoint(types::new_outpoint(txid, (vout as u32)), pointer - start)
                    } else {
                        location.new_satpoint
                    }
                } else {
                    location.new_satpoint
                }
            } else {
                location.new_satpoint
            }
            } else {
                location.new_satpoint
            };

            update_inscription_location(&mut updater, &mut seal_outs, &location.flotsam, new_satpoint, location.is_op_return, location.owner);
            new_locations_idx = new_locations_idx + 1;
        };

        if (is_coinbase) {
            while(flotsam_idx < flotsam_len){
                let flotsam = vector::borrow(&floating_inscriptions, flotsam_idx);
                let new_satpoint = ord::new_satpoint(types::null_outpoint(), updater.lost_sats + flotsam.offset - output_value);
                update_inscription_location(&mut updater, &mut seal_outs, flotsam, new_satpoint, false, @bitcoin_move);
                flotsam_idx = flotsam_idx + 1;
            };
            updater.lost_sats = updater.lost_sats + updater.reward - output_value;
        }else{
            while(flotsam_idx < flotsam_len){
                let flotsam = vector::borrow_mut(&mut floating_inscriptions, flotsam_idx);
                let offset = updater.reward + flotsam.offset - output_value;
                flotsam.offset = offset;
                vector::push_back(&mut updater.flotsams, *flotsam);
                flotsam_idx = flotsam_idx + 1;
            };
            let tx_fee = if (total_input_value >= total_output_value){
                total_input_value - total_output_value
            }else{
                if(check_utxo_input){
                    abort ErrorUTXOBalanceNotMatch
                };
                0
            };
            updater.reward = updater.reward + tx_fee;
        };
        
        let inscription_store = ord::borrow_mut_inscription_store();
        ord::update_cursed_inscription_count(inscription_store, updater.cursed_inscription_count);
        ord::update_blessed_inscription_count(inscription_store, updater.blessed_inscription_count);
        ord::update_unbound_inscription_count(inscription_store, updater.unbound_inscription_count);
        ord::update_lost_sats(inscription_store, updater.lost_sats);
        ord::update_next_sequence_number(inscription_store, updater.next_sequence_number);

        if(is_coinbase){
            //The updater lifetime is the same as the pending_block
            //The coinbase is the last tx to process, so we can drop the updater here
            drop(updater);
        }else{
            pending_block::add_intermediate(pending_block, updater);
        };
        seal_outs
    }

    fun update_inscription_location(updater: &mut InscriptionUpdater, seal_outs: &mut vector<SealOut>, flotsam: &Flotsam, new_satpoint: SatPoint, is_op_return: bool, owner: address) {
        let inscription_id = flotsam.inscription_id;
        let (unbound, inscription_obj_id) = if (option::is_some(&flotsam.old)){
            let old_satpoint = *option::borrow(&flotsam.old);
            

            let inscription_obj_id = ord::derive_inscription_id(inscription_id);
            let inscription_obj = ord::take_object(inscription_obj_id);
            let inscription = object::borrow(&inscription_obj);
            let sequence_number = ord::sequence_number(inscription);
            
        
            ord::transfer_object(inscription_obj, owner, new_satpoint, is_op_return);
            
            event::emit(InscriptionTransferredEvent{
                block_height: updater.block_height,
                inscription_id: inscription_id,
                new_location: new_satpoint,
                old_location: old_satpoint,
                sequence_number,
                is_burned: is_op_return,
            });
            (false, inscription_obj_id)
        }else{
            let FlotsamNew{
                cursed,
                fee:_,
                hidden:_,
                parents,
                pointer:_,
                reinscription,
                unbound,
                vindicated,
                envelope,
            } = *option::borrow(&flotsam.new);
            let inscription_number = if(cursed) {
                let number = updater.cursed_inscription_count;
                updater.cursed_inscription_count = updater.cursed_inscription_count + 1;
                //cursed number start from -1
                (number + 1)
            }else{
                let number = updater.blessed_inscription_count;
                updater.blessed_inscription_count = updater.blessed_inscription_count + 1;
                number
            };
            let sequence_number = updater.next_sequence_number;
            updater.next_sequence_number = updater.next_sequence_number + 1;

            let charms = 0u16;

            if(cursed) {
                charms = ord::set_charm(charms, ord::charm_cursed_flag());
            };

            if(reinscription) {
                charms = ord::set_charm(charms, ord::charm_reinscription_flag());
            };   

            //We do not handle the Sat
            
            if(is_op_return) {
                charms = ord::set_charm(charms, ord::charm_burned_flag());
            };

            if(*ord::satpoint_outpoint(&new_satpoint) == types::null_outpoint()) {
                charms = ord::set_charm(charms, ord::charm_lost_flag());
            };

            let location = if(unbound) {
                charms = ord::set_charm(charms, ord::charm_unbound_flag());
                ord::new_satpoint(types::unbound_outpoint(), (updater.unbound_inscription_count as u64))
            }else{
                new_satpoint
            };

            if(vindicated) {
                charms = ord::set_charm(charms, ord::charm_vindicated_flag());
            };
            let inscription_obj_id = ord::create_object(
                inscription_id,
                location,
                sequence_number,
                inscription_number,
                cursed,
                charms,
                envelope,
                owner,
            );
            event::emit(InscriptionCreatedEvent{
                block_height: updater.block_height,
                charms,
                inscription_id,
                location: option::some(new_satpoint),
                parent_inscription_ids: parents,
                sequence_number,
            });
            (unbound, inscription_obj_id)
        };

        if(unbound) {
            updater.unbound_inscription_count = updater.unbound_inscription_count + 1;
        }else{
            let vout = ord::satpoint_vout(&new_satpoint);
            let seal = utxo::new_utxo_seal(updater.seal_protocol, inscription_obj_id);
            vector::push_back(seal_outs, utxo::new_seal_out(vout, seal));
        };
    }


    public(friend) fun need_process_oridinals(block_height: u64) : bool {
        block_height >= network::first_inscription_height()
    }

    fun find_range_for_pointer(range_to_vout: &SimpleMap<u64, Range>, pointer: u64): (bool, u64, u64) {
        let keys = simple_map::keys(range_to_vout);
        let i = 0;
        let len = vector::length(&keys);
        while (i < len) {
            let vout = *vector::borrow(&keys, i);
            let range = *simple_map::borrow(range_to_vout, &vout);
            if (pointer >= range.start && pointer < range.end) {
                return (true, vout, range.start)
            };
            i = i + 1;
        };
        (false, 0, 0)
    }

    fun drop(updater: InscriptionUpdater) {
        let InscriptionUpdater {
            block_height:_,
            seal_protocol:_,
            flotsams,
            lost_sats:_,
            reward:_,
            blessed_inscription_count:_,
            cursed_inscription_count:_,
            unbound_inscription_count:_,
            next_sequence_number:_,
        } = updater;
        //The flotsams must be empty after the process_tx
        assert!(vector::is_empty(&flotsams), ErrorFlotsamNotProcessed);
        vector::destroy_empty(flotsams);
    }

    //======================= Curse =======================

    /// Curse Inscription
    const CURSE_DUPLICATE_FIELD: vector<u8> = b"DuplicateField";

    public fun curse_duplicate_field(): vector<u8> {
        CURSE_DUPLICATE_FIELD
    }

    const CURSE_INCOMPLETE_FIELD: vector<u8> = b"IncompleteField";

    public fun curse_incompleted_field(): vector<u8> {
        CURSE_INCOMPLETE_FIELD
    }

    const CURSE_NOT_AT_OFFSET_ZERO: vector<u8> = b"NotAtOffsetZero";

    public fun curse_not_at_offset_zero(): vector<u8> {
        CURSE_NOT_AT_OFFSET_ZERO
    }

    const CURSE_NOT_IN_FIRST_INPUT: vector<u8> = b"NotInFirstInput";

    public fun curse_not_in_first_input(): vector<u8> {
        CURSE_NOT_IN_FIRST_INPUT
    }

    const CURSE_POINTER: vector<u8> = b"Pointer";

    public fun curse_pointer(): vector<u8> {
        CURSE_POINTER
    }

    const CURSE_PUSHNUM: vector<u8> = b"Pushnum";

    public fun curse_pushnum(): vector<u8> {
        CURSE_PUSHNUM
    }

    const CURSE_REINSCRIPTION: vector<u8> = b"Reinscription";

    public fun curse_reinscription(): vector<u8> {
        CURSE_REINSCRIPTION
    }

    const CURSE_STUTTER: vector<u8> = b"Stutter";

    public fun curse_stutter(): vector<u8> {
        CURSE_STUTTER
    }

    const CURSE_UNRECOGNIZED_EVEN_FIELD: vector<u8> = b"UnrecognizedEvenField";

    public fun curse_unrecognized_even_field(): vector<u8> {
        CURSE_UNRECOGNIZED_EVEN_FIELD
    }

    fun handle_curse_inscription(e: &Envelope<InscriptionRecord>, offset: u64, inscribed_offset: &SimpleMap<u64, ReinscribeCounter>): option::Option<vector<u8>> {
        let record = ord::envelope_payload(e);
        let curse = if (ord::inscription_record_unrecognized_even_field(record)) {
            option::some(CURSE_UNRECOGNIZED_EVEN_FIELD)
        } else if (ord::inscription_record_duplicate_field(record)) {
            option::some(CURSE_DUPLICATE_FIELD)
        } else if (ord::inscription_record_incomplete_field(record)) {
            option::some(CURSE_INCOMPLETE_FIELD)
        } else if (ord::envelope_input(e) != 0) {
            option::some(CURSE_NOT_IN_FIRST_INPUT)
        } else if (ord::envelope_offset(e) != 0) {
            option::some(CURSE_NOT_AT_OFFSET_ZERO)
        } else if (option::is_some(ord::inscription_record_pointer(record))) {
            option::some(CURSE_POINTER)
        } else if (ord::envelope_pushnum(e)) {
            option::some(CURSE_PUSHNUM)
        } else if (ord::envelope_stutter(e)) {
            option::some(CURSE_STUTTER)
        }else{
            if(simple_map::contains_key(inscribed_offset, &offset)){
                let counter = *simple_map::borrow(inscribed_offset, &offset);
                if(counter.count > 1){
                    option::some(CURSE_REINSCRIPTION)
                }else{
                    //the counter inscription id is from input, so it should exist
                    let inscription = ord::borrow_inscription(counter.inscription_id);
                    let is_cursed = ord::is_cursed(inscription);
                    let charms = ord::charms(inscription);
                    let initial_inscription_was_cursed_or_vindicated = is_cursed || ord::is_set_charm(charms, ord::charm_vindicated_flag());
                    if(initial_inscription_was_cursed_or_vindicated){
                        option::none()
                    }else{
                        option::some(CURSE_REINSCRIPTION)
                    }
                }
            }else{
                option::none()
            }
        };
        curse
    }
}
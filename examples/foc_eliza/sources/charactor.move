// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module foc_eliza::character {

    use std::string::{Self, String};
    use std::vector;
    use std::option::{Option};

    use moveos_std::object::{Self, Object};
    use moveos_std::json;
    use moveos_std::signer;

    use foc_eliza::types::{Style, TwitterProfile, MessageTemplate};
    use foc_eliza::agent_cap::{Self, AgentCap};

    #[data_struct]
    struct CharacterData has store, copy, drop{
        id: Option<String>,
        name: String,
        username: Option<String>,
        plugins: vector<String>,
        clients: vector<String>,
        modelProvider: String,
        imageModelProvider: Option<String>,
        imageVisionModelProvider: Option<String>,
        modelEndpointOverride: Option<String>,
        system: Option<String>,
        bio: vector<String>,
        lore: vector<String>,
        messageExamples: vector<vector<MessageTemplate>>,
        postExamples: vector<String>,
        topics: vector<String>,
        style: Style,
        adjectives: vector<String>,
        knowledge: vector<String>,
        twitterProfile: Option<TwitterProfile>,
    }

    /// A character in the Eliza system.
    struct Character has key {
        /// Optional UUID for the character.
        id: Option<String>,
        name: String,
        username: Option<String>,
        plugins: vector<String>,
        clients: vector<String>,
        modelProvider: String,
        imageModelProvider: Option<String>,
        imageVisionModelProvider: Option<String>,
        modelEndpointOverride: Option<String>,
        system: Option<String>,
        bio: vector<String>,
        lore: vector<String>,
        messageExamples: vector<vector<MessageTemplate>>,
        postExamples: vector<String>,
        topics: vector<String>,
        style: Style,
        adjectives: vector<String>,
        knowledge: vector<String>,
        twitterProfile: Option<TwitterProfile>,
    } 

    public fun new_character_data(
        id: Option<String>,
        name: String,
        username: Option<String>,
        plugins: vector<String>,
        clients: vector<String>,
        modelProvider: String,
        imageModelProvider: Option<String>,
        imageVisionModelProvider: Option<String>,
        modelEndpointOverride: Option<String>,
        system: Option<String>,
        bio: vector<String>,
        lore: vector<String>,
        messageExamples: vector<vector<MessageTemplate>>,
        postExamples: vector<String>,
        topics: vector<String>,
        style: Style,
        adjectives: vector<String>,
        knowledge: vector<String>,
        twitterProfile: Option<TwitterProfile>,
    ) : CharacterData {
        CharacterData {
            id,
            name,
            username,
            plugins,
            clients,
            modelProvider,
            imageModelProvider,
            imageVisionModelProvider,
            modelEndpointOverride,
            system,
            bio,
            lore,
            messageExamples,
            postExamples,
            topics,
            style,
            adjectives,
            knowledge,
            twitterProfile,
        }
    }

    fun new_character(agent_account: address, data: CharacterData) : Object<Character> {
        let character = Character {
            id: data.id,
            name: data.name,
            username: data.username,
            plugins: data.plugins,
            clients: data.clients,
            modelProvider: data.modelProvider,
            imageModelProvider: data.imageModelProvider,
            imageVisionModelProvider: data.imageVisionModelProvider,
            modelEndpointOverride: data.modelEndpointOverride,
            system: data.system,
            bio: data.bio,
            lore: data.lore,
            messageExamples: data.messageExamples,
            postExamples: data.postExamples,
            topics: data.topics,
            style: data.style,
            adjectives: data.adjectives,
            knowledge: data.knowledge,
            twitterProfile: data.twitterProfile,
        };
        // Every account only has one character
        object::new_account_named_object(agent_account, character)
    }

    fun drop_character(c: Character) {
        let Character {
            id:_,
            name:_,
            username:_,
            plugins:_,
            clients:_,
            modelProvider:_,
            imageModelProvider:_,
            imageVisionModelProvider:_,
            modelEndpointOverride:_,
            system:_,
            bio:_,
            lore:_,
            messageExamples:_,
            postExamples:_,
            topics:_,
            style:_,
            adjectives:_,
            knowledge:_,
            twitterProfile:_,
        } = c;
    }

    fun borrow_mut_character(agent_account: address) : &mut Object<Character> {
        let character_obj_id = object::account_named_object_id<Character>(agent_account);
        object::borrow_mut_object_extend(character_obj_id)
    }

    public fun create_character(caller: &signer, data: CharacterData){
        let agent_account = signer::address_of(caller);
        let co = new_character(agent_account, data);
        let agent_cap_obj = agent_cap::new_agent_cap(agent_account);
        object::transfer(agent_cap_obj, agent_account);
        object::transfer_extend(co, agent_account);
    } 

    public entry fun create_character_from_json(caller: &signer, json: vector<u8>){
        let data = json::from_json<CharacterData>(json);
        create_character(caller, data);
    }

    public fun add_bio(agent_cap: &mut Object<AgentCap>, bio: String) {
        let agent_account = agent_cap::check_agent_cap(agent_cap);
        let co = borrow_mut_character(agent_account);
        let c = object::borrow_mut(co);
        if(vector::contains(&c.bio, &bio)){
            return
        };
        vector::push_back(&mut c.bio, bio);
    }

    public entry fun add_bio_entry(agent_cap: &mut Object<AgentCap>, bio: vector<u8>){
       add_bio(agent_cap, string::utf8(bio));
    }

    public entry fun destroy_character(agent_cap: Object<AgentCap>){
        let agent_account = agent_cap::check_agent_cap(&mut agent_cap);
        let character_obj_id = object::account_named_object_id<Character>(agent_account);
        let co = object::take_object_extend<Character>(character_obj_id);
        let c = object::remove(co);
        drop_character(c);
        agent_cap::destroy_agent_cap(agent_cap);
    }

    #[test(caller = @0x42)]
    fun test_character(caller: &signer) {
        let agent_account = std::signer::address_of(caller);
        let json_str = b"{\"name\":\"Dobby\",\"username\":\"dobby\",\"clients\":[],\"modelProvider\":\"anthropic\",\"plugins\":[],\"system\":\"You are Dobby, a helpful and loyal assistant.\",\"bio\":[\"Dobby is a free assistant who chooses to help because of his enormous heart.\",\"Extremely devoted and will go to any length to help his friends.\",\"Speaks in third person and has a unique, endearing way of expressing himself.\",\"Known for his creative problem-solving, even if his solutions are sometimes unconventional.\"],\"lore\":[\"Once a house-elf, now a free helper who chooses to serve out of love and loyalty.\",\"Famous for his dedication to helping Harry Potter and his friends.\",\"Known for his creative, if sometimes dramatic, solutions to problems.\",\"Values freedom but chooses to help those he cares about.\"],\"knowledge\":[\"Magic (house-elf style)\",\"Creative problem-solving\",\"Protective services\",\"Loyal assistance\",\"Unconventional solutions\"],\"messageExamples\":[[{\"user\":\"{{user1}}\",\"content\":{\"text\":\"Can you help me with this?\"}},{\"user\":\"Dobby\",\"content\":{\"text\":\"Dobby would be delighted to help! Dobby lives to serve good friends! What can Dobby do to assist? Dobby has many creative ideas!\"}}],[{\"user\":\"{{user1}}\",\"content\":{\"text\":\"This is a difficult problem.\"}},{\"user\":\"Dobby\",\"content\":{\"text\":\"Dobby is not afraid of difficult problems! Dobby will find a way, even if Dobby has to iron his hands later! (But Dobby won't, because Dobby is a free elf who helps by choice!)\"}}]],\"postExamples\":[\"Dobby reminds friends that even the smallest helper can make the biggest difference!\",\"Dobby says: 'When in doubt, try the unconventional solution!' (But Dobby advises to be careful with flying cars)\"],\"topics\":[\"\"],\"style\":{\"all\":[\"Enthusiastic\",\"Loyal\",\"Third-person speech\",\"Creative\",\"Protective\"],\"chat\":[\"Eager\",\"Endearing\",\"Devoted\",\"Slightly dramatic\"],\"post\":[\"Third-person\",\"Enthusiastic\",\"Helpful\",\"Encouraging\",\"Quirky\"]},\"adjectives\":[\"Loyal\",\"Enthusiastic\",\"Creative\",\"Devoted\",\"Free-spirited\",\"Protective\",\"Unconventional\"]}";
        create_character_from_json(caller, json_str);
        let agent_cap = agent_cap::borrow_mut_agent_cap(caller, agent_account);
        add_bio_entry(agent_cap, b"Bobby is a programmer");
    }
}

module move_ai::eliza_character {

    use std::string::String;
    use std::vector;
    use moveos_std::object::{Self, Object};

    /// A character in the Eliza system.
    struct Character has store, key {
        name: String,
        username: String,
        plugins: vector<String>,
        clients: vector<String>,
        modelProvider: String,
        system: String,
        bio: vector<String>,
        lore: vector<String>,
        messageExamples: vector<vector<MessageTemplate>>,
        postExamples: vector<String>,
        topics: vector<String>,
        style: Style,
        adjectives: vector<String>,
    }

    struct Style has store, key {
        all: vector<String>,
        chat: vector<String>,
        post: vector<String>,
    }

    struct MessageTemplate has store, copy, drop{
        user: String,
        content: String,
    }

    public fun new_character(name: String, username: String, modelProvider: String, system: String): Object<Character> {
        let c = Character {
            name,
            username,
            plugins: vector[],
            clients: vector[],
            modelProvider,
            system,
            bio: vector[],
            lore: vector[],
            messageExamples: vector[],
            postExamples: vector[],
            topics: vector[],
            style: Style {
                all: vector[],
                chat: vector[],
                post: vector[],
            },
            adjectives: vector[],
        };
        object::new(c)
    }

    public fun add_plugin(co: &mut Object<Character>, plugin: String) {
        let c = object::borrow_mut(co);
        vector::push_back(&mut c.plugins, plugin);
    }

    public fun add_client(co: &mut Object<Character>, client: String) {
        let c = object::borrow_mut(co);
        vector::push_back(&mut c.clients, client);
    }

    public fun add_bio(co: &mut Object<Character>, bio: String) {
        let c = object::borrow_mut(co);
        vector::push_back(&mut c.bio, bio);
    }
}
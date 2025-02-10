module onchain_ai_chat::message {
    use std::string::String;
    use moveos_std::timestamp;

    /// Message types
    const MESSAGE_TYPE_USER: u8 = 0;
    const MESSAGE_TYPE_AI: u8 = 1;

    #[data_struct]
    struct Message has store, copy, drop {
        id: u64,
        sender: address,
        content: String,
        timestamp: u64,
        message_type: u8,
    }

    // Constructor
    public fun new_message(id: u64, sender: address, content: String, message_type: u8): Message {
        Message {
            id,
            sender,
            content,
            timestamp: timestamp::now_milliseconds(),
            message_type,
        }
    }

    // Getters
    public fun get_id(message: &Message): u64 {
        message.id
    }

    public fun get_content(message: &Message): String {
        message.content
    }

    public fun get_type(message: &Message): u8 {
        message.message_type
    }

    public fun get_timestamp(message: &Message): u64 {
        message.timestamp
    }

    public fun get_sender(message: &Message): address {
        message.sender
    }

    // Constants
    public fun type_user(): u8 { MESSAGE_TYPE_USER }
    public fun type_ai(): u8 { MESSAGE_TYPE_AI }

    #[test]
    fun test_message_creation() {
        use std::string;
        
        let msg = new_message(1, @0x1, string::utf8(b"test content"), type_user());
        assert!(get_id(&msg) == 1, 0);
        assert!(get_content(&msg) == string::utf8(b"test content"), 1);
        assert!(get_type(&msg) == type_user(), 2);
        assert!(get_sender(&msg) == @0x1, 3);
    }
}
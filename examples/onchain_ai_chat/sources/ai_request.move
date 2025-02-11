module onchain_ai_chat::ai_request {
    use std::string::{Self, String};
    use std::vector;
    use moveos_std::json;
    use onchain_ai_chat::message::{Self, Message};

    /// Base system prompt
    const SYSTEM_PROMPT: vector<u8> = b"You are a knowledgeable AI assistant. Begin each response with a relevant title as a Markdown h1 header (# Title). Provide clear explanations using Markdown formatting. Format code with language-specific code blocks. Keep responses under 2000 characters. Break down complex topics into digestible parts.";

    /// Summary hint - add when message_id % 10 == 0
    const SUMMARY_HINT: vector<u8> = b"Since this is a milestone in our conversation, please start your response with a brief summary of our discussion so far, then proceed with your answer to the current question. Keep the overall response natural and flowing.";

    const SUMMARY_POINT: u64 = 20;

    #[data_struct]
    struct ChatMessage has store, copy, drop {
        /// Must be "user" or "assistant" in JSON
        role: String,
        content: String,
    }

    #[data_struct]
    struct ChatRequest has store, copy, drop {
        model: String,
        messages: vector<ChatMessage>,
        temperature: u64,
    }

    public fun new_chat_request(content: String, previous_messages: &vector<Message>): ChatRequest {
        let messages = vector::empty();
        let len = vector::length(previous_messages);

        // Add base system prompt
        vector::push_back(&mut messages, ChatMessage {
            role: string::utf8(b"system"),
            content: string::utf8(SYSTEM_PROMPT),
        });

        // Check if we need summary (every 10 messages)
        if (len > 0) {
            let last_msg = vector::borrow(previous_messages, len - 1);
            if (message::get_id(last_msg) % SUMMARY_POINT == 0) {
                // Add summary hint
                vector::push_back(&mut messages, ChatMessage {
                    role: string::utf8(b"system"),
                    content: string::utf8(SUMMARY_HINT),
                });
            };
        };

        let i = 0;
        
        while (i < len) {
            let msg = vector::borrow(previous_messages, i);
            vector::push_back(&mut messages, ChatMessage {
                role: if (message::get_type(msg) == message::type_ai()) {
                    string::utf8(b"assistant")
                } else {
                    string::utf8(b"user")
                },
                content: message::get_content(msg),
            });
            i = i + 1;
        };

        // Add current message
        vector::push_back(&mut messages, ChatMessage {
            role: string::utf8(b"user"),
            content,
        });

        ChatRequest {
            model: string::utf8(b"gpt-4o"),
            messages,
            temperature: 1, //Because there no float type in Move, how to pass float value?
        }
    }

    public fun to_json(request: &ChatRequest): vector<u8> {
        json::to_json(request)
    }

    #[test]
    fun test_chat_request() {
        use std::string;
        
        let messages = vector::empty();
        let content = string::utf8(b"Hello AI");
        let request = new_chat_request(content, &messages);
        
        // Convert to JSON and verify structure
        let json_bytes = to_json(&request);
        let json_str = string::utf8(json_bytes);
        
        // Expected: {"model":"gpt-4o","messages":[{"role":"user","content":"Hello AI"}],"temperature":7}
        assert!(string::index_of(&json_str, &string::utf8(b"gpt-4o")) != 18446744073709551615, 1);
        assert!(string::index_of(&json_str, &string::utf8(b"Hello AI")) != 18446744073709551615, 2);
        assert!(string::index_of(&json_str, &string::utf8(b"user")) != 18446744073709551615, 3);
    }
}
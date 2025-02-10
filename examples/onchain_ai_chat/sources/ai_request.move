module onchain_ai_chat::ai_request {
    use std::string::{Self, String};
    use std::vector;
    use moveos_std::json;
    use onchain_ai_chat::message::{Self, Message};

    /// System prompt to guide AI behavior
    const SYSTEM_PROMPT: vector<u8> = b"You are a knowledgeable AI assistant. Begin each response with a relevant title as a Markdown h1 header (# Title) that summarizes the current conversation. After the title, provide your response using clear Markdown formatting. Format code with language-specific code blocks. Keep responses under 2000 characters. Acknowledge any uncertainties. Break down complex topics into digestible parts.";

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

        // Add system prompt
        vector::push_back(&mut messages, ChatMessage {
            role: string::utf8(b"system"),
            content: string::utf8(SYSTEM_PROMPT),
        });

        let i = 0;
        let len = vector::length(previous_messages);
        
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
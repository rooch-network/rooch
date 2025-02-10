module onchain_ai_chat::ai_response {
    use std::string::{Self, String};
    use std::option::Option;
    use moveos_std::json;
    use std::vector;

    #[data_struct]
    struct Usage has copy, drop, store {
        prompt_tokens: u64,
        completion_tokens: u64,
        total_tokens: u64,
    }

    #[data_struct]
    struct Message has copy, drop, store {
        role: String,
        content: String,
        refusal: Option<String>,
    }

    #[data_struct]
    struct Choice has copy, drop, store {
        index: u64,
        message: Message,
        finish_reason: String,
    }

    #[data_struct]
    struct ChatCompletion has copy, drop, store {
        id: String,
        object: String,
        created: u64,
        model: String,
        choices: vector<Choice>,
        usage: Usage,
    }

    public fun parse_chat_completion(json_str: String): ChatCompletion {
        json::from_json<ChatCompletion>(string::into_bytes(json_str))
    }

    public fun parse_chat_completion_option(json_str: String): Option<ChatCompletion> {
        json::from_json_option<ChatCompletion>(string::into_bytes(json_str))
    }

    /// Get the message content from the first choice in the completion
    public fun get_message_content(completion: &ChatCompletion): String {
        let choice = vector::borrow(&completion.choices, 0);
        choice.message.content
    }

    /// Get the refusal reason if present
    public fun get_refusal(completion: &ChatCompletion): Option<String> {
        let choice = vector::borrow(&completion.choices, 0);
        choice.message.refusal
    }

    /// Get the finish reason from the first choice
    public fun get_finish_reason(completion: &ChatCompletion): String {
        let choice = vector::borrow(&completion.choices, 0);
        choice.finish_reason
    }

    /// Get total tokens used
    public fun get_total_tokens(completion: &ChatCompletion): u64 {
        completion.usage.total_tokens
    }

    /// Get completion model name
    public fun get_model(completion: &ChatCompletion): String {
        completion.model
    }

    /// Get assistant's role
    public fun get_assistant_role(completion: &ChatCompletion): String {
        let choice = vector::borrow(&completion.choices, 0);
        choice.message.role
    }

    /// Check if the completion has any refusal
    public fun has_refusal(completion: &ChatCompletion): bool {
        let choice = vector::borrow(&completion.choices, 0);
        std::option::is_some(&choice.message.refusal)
    }

    #[test]
    fun test_parse_chat_completion() {
        let json_str = string::utf8(b"{\"id\":\"chatcmpl-Az3dpZskp51c4DlFzLVmgjoM3B56Z\",\"object\":\"chat.completion\",\"created\":1739115369,\"model\":\"gpt-4-0613\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Hello! How can I assist you today?\",\"refusal\":null},\"logprobs\":null,\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":8,\"completion_tokens\":10,\"total_tokens\":18}}");

        let completion = parse_chat_completion(json_str);
        
        // Test basic fields
        assert!(completion.id == string::utf8(b"chatcmpl-Az3dpZskp51c4DlFzLVmgjoM3B56Z"), 1);
        assert!(completion.object == string::utf8(b"chat.completion"), 2);
        assert!(completion.created == 1739115369, 3);
        assert!(completion.model == string::utf8(b"gpt-4-0613"), 4);

        // Test choices
        assert!(vector::length(&completion.choices) == 1, 5);
        let choice = vector::borrow(&completion.choices, 0);
        assert!(choice.index == 0, 6);
        assert!(choice.finish_reason == string::utf8(b"stop"), 7);

        // Test message
        let message = &choice.message;
        assert!(message.role == string::utf8(b"assistant"), 8);
        assert!(message.content == string::utf8(b"Hello! How can I assist you today?"), 9);
        assert!(std::option::is_none(&message.refusal), 10);

        // Test usage
        assert!(completion.usage.prompt_tokens == 8, 11);
        assert!(completion.usage.completion_tokens == 10, 12);
        assert!(completion.usage.total_tokens == 18, 13);
    }

    #[test]
    fun test_getters() {
        let json_str = string::utf8(b"{\"id\":\"chatcmpl-Az3dpZskp51c4DlFzLVmgjoM3B56Z\",\"object\":\"chat.completion\",\"created\":1739115369,\"model\":\"gpt-4-0613\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Hello! How can I assist you today?\",\"refusal\":null},\"logprobs\":null,\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":8,\"completion_tokens\":10,\"total_tokens\":18}}");

        let completion = parse_chat_completion(json_str);
        
        assert!(get_message_content(&completion) == string::utf8(b"Hello! How can I assist you today?"), 1);
        assert!(get_finish_reason(&completion) == string::utf8(b"stop"), 2);
        assert!(get_total_tokens(&completion) == 18, 3);
        assert!(get_model(&completion) == string::utf8(b"gpt-4-0613"), 4);
        assert!(get_assistant_role(&completion) == string::utf8(b"assistant"), 5);
        assert!(!has_refusal(&completion), 6);
        assert!(std::option::is_none(&get_refusal(&completion)), 7);
    }
}
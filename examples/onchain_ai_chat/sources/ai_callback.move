module onchain_ai_chat::ai_callback {
    use moveos_std::object;
    use moveos_std::string_utils;
    use moveos_std::json;
    use onchain_ai_chat::room::{Self, Room};
    use onchain_ai_chat::ai_service;
    use onchain_ai_chat::ai_response;
    use verity::oracles;
    use std::option;
    use std::vector;
    use std::string::{Self, String};

    public entry fun process_response() {
        let pending_requests = ai_service::get_pending_requests();
        
        vector::for_each(pending_requests, |request| {
            let (room_id, request_id) = ai_service::unpack_pending_request(request);
            
            let response_status = oracles::get_response_status(&request_id);
            
            if (response_status != 0) {
                let response = oracles::get_response(&request_id);
                let response_content = option::destroy_some(response);
                let room_obj = object::borrow_mut_object_shared<Room>(room_id);
                let room = object::borrow_mut(room_obj);
                let message = if (response_status == 200){
                    let json_str_opt = json::from_json_option<String>(string::into_bytes(response_content));
                    let json_str = if(option::is_some(&json_str_opt)){
                        option::destroy_some(json_str_opt)
                    }else{
                        response_content
                    };
                    let chat_completion_opt = ai_response::parse_chat_completion_option(json_str);
                    if(option::is_some(&chat_completion_opt)){
                        let chat_completion = option::destroy_some(chat_completion_opt);
                        let message_content = ai_response::get_message_content(&chat_completion);
                        let refusal = ai_response::get_refusal(&chat_completion);
                        if(option::is_some(&refusal)){
                            let refusal_reason = option::destroy_some(refusal);
                            string::append(&mut message_content, string::utf8(b", refusal: "));
                            string::append(&mut message_content, refusal_reason);
                        };
                        message_content
                    }else{
                        response_content
                    }
                }else{
                    let error_message = string::utf8(b"AI Oracle response error, error code: ");
                    string::append(&mut error_message, string_utils::to_string_u32((response_status as u32)));
                    string::append(&mut error_message, string::utf8(b", response: "));
                    string::append(&mut error_message, response_content);
                    error_message
                };
                room::add_ai_response(room, message);
                ai_service::remove_request(request_id);
            };
        });
        
    }
}
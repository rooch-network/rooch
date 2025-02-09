module onchain_ai_chat::ai_callback {
    use moveos_std::object;
    use moveos_std::string_utils;
    use onchain_ai_chat::room::{Self, Room};
    use onchain_ai_chat::ai_service;
    use verity::oracles;
    use std::option;
    use std::vector;
    use std::string;

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
                if (response_status != 200) {
                    let error_message = string::utf8(b"AI Oracle response error, error code: ");
                    string::append(&mut error_message, string_utils::to_string_u32((response_status as u32)));
                    string::append(&mut error_message, string::utf8(b", response: "));
                    string::append(&mut error_message, response_content);
                };
                room::add_ai_response(room, response_content);
                ai_service::remove_request(request_id);
            };
        });
        
    }
}
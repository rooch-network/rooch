// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module foc_eliza::types {

    use std::string::String;
    use std::option::Option;

    #[data_struct]
    struct TwitterProfile has store, copy, drop {
        id: String,
        username: String,
        screenName: String,
        bio: String,
        nicknames: vector<String>,
    }

    #[data_struct]
    struct Style has store, copy, drop {
        all: vector<String>,
        chat: vector<String>,
        post: vector<String>,
    }

    public fun new_style(all: vector<String>, chat: vector<String>, post: vector<String>) : Style {
        Style {
            all,
            chat,
            post,
        }
    }

    #[data_struct]
    struct Media has store, copy, drop {
        id: String,
        url: String,
        title: String,
        source: String,
        description: String,
        text: String,
        contentType: String,
    }

    public fun new_media(id: String, url: String, title: String, source: String, description: String, text: String, contentType: String) : Media {
        Media {
            id,
            url,
            title,
            source,
            description,
            text,
            contentType,
        }
    }

    #[data_struct]
    struct Content has store, copy, drop {
        text: String,
        action: Option<String>,
        source: Option<String>,
        url: Option<String>,
        inReplyTo: Option<String>,
        attachments: vector<Media>,
    }

    public fun new_content(text: String, action: Option<String>, source: Option<String>, url: Option<String>, inReplyTo: Option<String>, attachments: vector<Media>) : Content {
        Content {
            text,
            action,
            source,
            url,
            inReplyTo,
            attachments,
        }
    }

    #[data_struct]
    struct MessageTemplate has store, copy, drop{
        user: String,
        content: Content,
    }

    public fun new_message_template(user: String, content: Content) : MessageTemplate {
        MessageTemplate {
            user,
            content,
        }
    }
}

module rooch_blog::article_created {

    use moveos_std::object_id::ObjectID;
    use rooch_blog::article::{Self, ArticleCreated};
    use std::option;
    use std::string::String;

    public fun id(article_created: &ArticleCreated): option::Option<ObjectID> {
        article::article_created_id(article_created)
    }

    public fun title(article_created: &ArticleCreated): String {
        article::article_created_title(article_created)
    }

    public fun body(article_created: &ArticleCreated): String {
        article::article_created_body(article_created)
    }

}

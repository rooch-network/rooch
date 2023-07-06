module rooch_blog::article_deleted {

    use moveos_std::object_id::ObjectID;
    use rooch_blog::article::{Self, ArticleDeleted};

    public fun id(article_deleted: &ArticleDeleted): ObjectID {
        article::article_deleted_id(article_deleted)
    }

}

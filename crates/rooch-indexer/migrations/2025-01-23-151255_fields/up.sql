CREATE TABLE fields
(
    id                VARCHAR        NOT NULL       PRIMARY KEY,
    parent_id         VARCHAR        NOT NULL,
    field_key         VARCHAR        NOT NULL,
    sort_key          BIGINT         NOT NULL,
    created_at        BIGINT         NOT NULL,
    updated_at        BIGINT         NOT NULL,
    UNIQUE (parent_id, field_key)
);

CREATE INDEX idx_fields_parent_id_sort_key ON fields (parent_id, sort_key);
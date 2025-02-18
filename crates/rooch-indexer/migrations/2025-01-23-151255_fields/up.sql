CREATE TABLE fields
(
    id                VARCHAR       NOT NULL,
    field_key         VARCHAR       NOT NULL,
    name              VARCHAR       NOT NULL,
    val               BIGINT        NOT NULL,
    created_at        BIGINT         NOT NULL,
    updated_at        BIGINT         NOT NULL,
    PRIMARY KEY (id, field_key)
--    UNIQUE (id, name)
);

CREATE INDEX idx_fields_value ON fields (id, val);
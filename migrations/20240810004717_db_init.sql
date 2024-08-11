-- Add migration script here
CREATE TABLE users (
    id uuid NOT NULL,
    primary key (id),
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    created_at timestamptz NOT NULL
);

CREATE TABLE user_tokens (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    refresh_token TEXT NOT NULL,
    created_at timestamptz NOT NULL,
    user_id uuid NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE todos (
    id uuid NOT NULL,
    primary key (id),
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at timestamptz NOT NULL,
    updated_at timestamptz NULL,
    owner_id uuid NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);
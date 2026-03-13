CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT UNIQUE NOT NULL CHECK (char_length(username) BETWEEN 3 AND 30),
    email TEXT UNIQUE NOT NULL CHECK (char_length(email) <= 255),
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
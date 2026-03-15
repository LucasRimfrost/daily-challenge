CREATE TABLE games (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed with the first game
INSERT INTO games (id, name, description, sort_order)
VALUES (
    'trivia',
    'Daily Trivia',
    'Test your knowledge with a new question every day',
    1
);
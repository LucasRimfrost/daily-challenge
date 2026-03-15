-- Register the game
INSERT INTO games (id, name, description, sort_order)
VALUES (
    'code_output',
    'What''s the Output?',
    'Read the code, predict the output. No compiler needed — just your brain.',
    2
);

-- Challenge table
CREATE TABLE code_output_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    language TEXT NOT NULL CHECK (language IN ('python', 'javascript', 'rust')),
    code_snippet TEXT NOT NULL,
    expected_output TEXT NOT NULL,
    difficulty TEXT NOT NULL CHECK (difficulty IN ('easy', 'medium', 'hard')),
    hint TEXT,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    scheduled_date DATE UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Submissions
CREATE TABLE code_output_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    challenge_id UUID NOT NULL REFERENCES code_output_challenges(id) ON DELETE CASCADE,
    answer TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL DEFAULT false,
    attempt_number INTEGER NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT code_output_unique_attempt UNIQUE (user_id, challenge_id, attempt_number),
    CONSTRAINT code_output_valid_attempt CHECK (attempt_number > 0)
);

-- Stats (per-game streaks and scores)
CREATE TABLE code_output_stats (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    current_streak INTEGER NOT NULL DEFAULT 0,
    longest_streak INTEGER NOT NULL DEFAULT 0,
    total_solved INTEGER NOT NULL DEFAULT 0,
    total_attempts INTEGER NOT NULL DEFAULT 0,
    last_solved_date DATE
);
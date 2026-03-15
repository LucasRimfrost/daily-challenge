-- Rename existing game tables to trivia-specific names
ALTER TABLE challenges RENAME TO trivia_challenges;
ALTER TABLE submissions RENAME TO trivia_submissions;
ALTER TABLE user_stats RENAME TO trivia_stats;

-- Rename constraints and indexes that reference old names
ALTER TABLE trivia_submissions RENAME CONSTRAINT unique_attempt TO trivia_unique_attempt;
ALTER TABLE trivia_submissions RENAME CONSTRAINT valid_attempt_number TO trivia_valid_attempt_number;
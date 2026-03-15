// ---- Auth ----

export interface User {
  id: string;
  username: string;
  email: string;
}

export interface UserStats {
  current_streak: number;
  longest_streak: number;
  total_solved: number;
  total_attempts: number;
}

export interface UserWithStats extends User {
  stats: UserStats;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface ForgotPasswordRequest {
  email: string;
}

export interface ResetPasswordRequest {
  token: string;
  new_password: string;
}

export interface MessageResponse {
  message: string;
}

export interface UpdateProfileRequest {
  username: string;
}

export interface UpdateEmailRequest {
  new_email: string;
  current_password: string;
}

export interface UpdatePasswordRequest {
  current_password: string;
  new_password: string;
}

// ---- Challenge ----

export interface Challenge {
  id: string;
  title: string;
  description: string;
  difficulty: "easy" | "medium" | "hard";
  hint: string | null;
  max_attempts: number;
  scheduled_date: string;
  attempts_used: number;
  is_solved: boolean;
  correct_answer: string | null;
}

export interface SubmitRequest {
  answer: string;
  challenge_id?: string;
}

export interface ArchiveEntry {
  id: string;
  title: string;
  difficulty: "easy" | "medium" | "hard";
  scheduled_date: string;
  is_solved: boolean;
  attempts_used: number;
  max_attempts: number;
}

export interface SubmitResponse {
  is_correct: boolean;
  attempt_number: number;
  attempts_remaining: number;
  hint: string | null;
}

export interface HistoryEntry {
  challenge_id: string;
  title: string;
  difficulty: "easy" | "medium" | "hard";
  scheduled_date: string;
  is_correct: boolean;
  attempt_number: number;
  submitted_at: string;
}

// ---- Code Output ----

export interface CodeOutputChallenge {
  id: string;
  title: string;
  description: string;
  language: string;
  code_snippet: string;
  difficulty: "easy" | "medium" | "hard";
  hint: string | null;
  max_attempts: number;
  scheduled_date: string;
  attempts_used: number;
  is_solved: boolean;
  correct_answer: string | null;
}

export interface CodeOutputArchiveEntry {
  id: string;
  title: string;
  language: string;
  difficulty: "easy" | "medium" | "hard";
  scheduled_date: string;
  is_solved: boolean;
  attempts_used: number;
  max_attempts: number;
}

export interface CodeOutputHistoryEntry {
  challenge_id: string;
  title: string;
  language: string;
  difficulty: "easy" | "medium" | "hard";
  scheduled_date: string;
  is_correct: boolean;
  attempt_number: number;
  submitted_at: string;
}

// ---- Games ----

export interface Game {
  id: string;
  name: string;
  description: string;
  icon: string | null;
  is_active: boolean;
  sort_order: number;
}

// ---- Leaderboard ----

export interface LeaderboardEntry {
  username: string;
  current_streak: number;
  longest_streak: number;
  total_solved: number;
}

// ---- Errors ----

export interface ApiError {
  error: string;
}

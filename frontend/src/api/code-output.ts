import { api } from "./client";
import type {
  CodeOutputArchiveEntry,
  CodeOutputChallenge,
  CodeOutputHistoryEntry,
  SubmitRequest,
  SubmitResponse,
} from "./types";

const BASE = "/api/v1/code-output";

export function getToday(): Promise<CodeOutputChallenge> {
  return api<CodeOutputChallenge>(`${BASE}/today`);
}

export function getChallengeByDate(date: string): Promise<CodeOutputChallenge> {
  return api<CodeOutputChallenge>(`${BASE}/${date}`);
}

export function getArchive(): Promise<CodeOutputArchiveEntry[]> {
  return api<CodeOutputArchiveEntry[]>(`${BASE}/archive`);
}

export function submitAnswer(data: SubmitRequest): Promise<SubmitResponse> {
  return api<SubmitResponse>(`${BASE}/submit`, {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function getHistory(limit = 30): Promise<CodeOutputHistoryEntry[]> {
  return api<CodeOutputHistoryEntry[]>(`${BASE}/history?limit=${limit}`);
}

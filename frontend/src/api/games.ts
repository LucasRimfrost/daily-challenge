import { api } from "./client";
import type { Game } from "./types";

export function getGames(): Promise<Game[]> {
  return api<Game[]>("/api/v1/games");
}

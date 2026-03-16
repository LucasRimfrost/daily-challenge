import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { getGames } from "@/api/games";
import { getToday as getTriviaToday } from "@/api/trivia";
import { getToday as getCodeOutputToday } from "@/api/code-output";
import { ApiRequestError } from "@/api/client";
import type { Game } from "@/api/types";
import { useAuth } from "@/hooks/useAuth";
import {
  Check,
  ChevronRight,
  Code,
  Flame,
  Gamepad2,
  Lightbulb,
  Zap,
} from "lucide-react";
import { Button } from "@/components/ui/button";

const gameIcons: Record<string, React.ElementType> = {
  trivia: Lightbulb,
  code_output: Code,
};

const gameIconColors: Record<string, string> = {
  trivia: "from-amber-500/20 to-orange-500/20 text-amber-500",
  code_output: "from-emerald-500/20 to-cyan-500/20 text-emerald-500",
};

const gameAccentBorder: Record<string, string> = {
  trivia: "border-l-amber-500/60",
  code_output: "border-l-emerald-500/60",
};

type GameStatus = "available" | "completed" | "no_challenge";

export function HubPage() {
  const { user } = useAuth();
  const [games, setGames] = useState<Game[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [gameStatuses, setGameStatuses] = useState<Map<string, GameStatus>>(new Map());

  const fetchGames = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const data = await getGames();
      data.sort((a, b) => a.sort_order - b.sort_order);
      setGames(data.filter((g) => g.is_active));
    } catch (err) {
      if (err instanceof ApiRequestError) {
        setError(err.message);
      } else {
        setError("Failed to load games.");
      }
    } finally {
      setLoading(false);
    }
  }, []);

  // Check today's status for each game
  useEffect(() => {
    if (!user) return;
    const statuses = new Map<string, GameStatus>();

    Promise.allSettled([
      getTriviaToday()
        .then((c) => {
          const done = c.is_solved || c.attempts_used >= c.max_attempts;
          statuses.set("trivia", done ? "completed" : "available");
        })
        .catch((err) => {
          if (err instanceof ApiRequestError && err.status === 404) {
            statuses.set("trivia", "no_challenge");
          }
        }),
      getCodeOutputToday()
        .then((c) => {
          const done = c.is_solved || c.attempts_used >= c.max_attempts;
          statuses.set("code_output", done ? "completed" : "available");
        })
        .catch((err) => {
          if (err instanceof ApiRequestError && err.status === 404) {
            statuses.set("code_output", "no_challenge");
          }
        }),
    ]).then(() => setGameStatuses(statuses));
  }, [user]);

  useEffect(() => {
    fetchGames();
  }, [fetchGames]);

  const today = new Date().toLocaleDateString("en-US", {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  if (loading) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <div className="mx-auto mb-3 size-8 animate-spin rounded-full border-2 border-muted border-t-primary" />
          <p className="text-sm text-muted-foreground">Loading games...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <Gamepad2 className="mx-auto mb-3 size-10 text-muted-foreground/50" />
          <p className="text-lg font-medium">{error}</p>
          <Button
            variant="outline"
            size="sm"
            className="mt-4"
            onClick={fetchGames}
          >
            Try again
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl">
      <div className="mb-10 text-center">
        <div className="mx-auto mb-4 flex size-14 items-center justify-center rounded-2xl bg-primary">
          <Zap className="size-7 text-primary-foreground" />
        </div>
        <h1 className="text-3xl font-bold tracking-tight">BrainForge</h1>
        <p className="mt-1.5 text-sm text-muted-foreground">{today}</p>
      </div>

      <div className="grid gap-5">
        {games.map((game) => {
          const Icon = gameIcons[game.id] ?? Gamepad2;
          const iconColor =
            gameIconColors[game.id] ?? "from-primary/20 to-primary/10 text-primary";
          const status = gameStatuses.get(game.id);
          const isCompleted = status === "completed";
          const noChallenge = status === "no_challenge";

          return (
            <Link
              key={game.id}
              to={`/${game.id.replace(/_/g, "-")}`}
              className="group rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
            >
              <Card className={`border-l-[3px] transition-all duration-200 hover:shadow-lg hover:shadow-primary/5 group-hover:border-primary/30 group-hover:scale-[1.02] group-active:scale-[0.99] ${gameAccentBorder[game.id] ?? ""}`}>
                <CardContent className="flex items-center gap-5 p-6">
                  <div
                    className={`flex size-14 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br ${iconColor}`}
                  >
                    <Icon className="size-7" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="text-lg font-semibold">{game.name}</p>
                    <p className="mt-1 text-sm text-muted-foreground">
                      {game.description}
                    </p>
                    {isCompleted && (
                      <p className="mt-1.5 flex items-center gap-1 text-xs text-green-600 dark:text-green-500">
                        <Check className="size-3" strokeWidth={3} />
                        Completed today
                      </p>
                    )}
                  </div>
                  <div className="flex items-center gap-3">
                    {user &&
                      !isCompleted &&
                      (() => {
                        const streak = game.id === "trivia"
                          ? user.trivia_stats.current_streak
                          : game.id === "code_output"
                            ? user.code_output_stats.current_streak
                            : 0;
                        return streak > 0 ? (
                          <div className="flex items-center gap-1 rounded-md bg-amber-500/10 px-2 py-1 text-sm font-medium dark:bg-muted">
                            <Flame className="size-3.5 text-orange-500" />
                            <span>{streak}</span>
                          </div>
                        ) : null;
                      })()}
                    {noChallenge ? (
                      <span className="hidden text-sm text-muted-foreground sm:inline">
                        No challenge today
                      </span>
                    ) : (
                      <span className="hidden text-sm font-medium text-primary sm:inline">
                        Play now
                      </span>
                    )}
                    <ChevronRight className="size-5 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
                  </div>
                </CardContent>
              </Card>
            </Link>
          );
        })}
      </div>

      {games.length === 0 && (
        <div className="flex flex-col items-center gap-2 py-12 text-center">
          <Gamepad2 className="size-10 text-muted-foreground/50" />
          <p className="font-medium">No games available yet</p>
          <p className="text-sm text-muted-foreground">Check back soon!</p>
        </div>
      )}
    </div>
  );
}

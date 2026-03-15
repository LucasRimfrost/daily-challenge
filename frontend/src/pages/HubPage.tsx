import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { getGames } from "@/api/games";
import { ApiRequestError } from "@/api/client";
import type { Game } from "@/api/types";
import { useAuth } from "@/hooks/useAuth";
import { ChevronRight, Flame, Gamepad2, Zap } from "lucide-react";
import { Button } from "@/components/ui/button";

export function HubPage() {
  const { user } = useAuth();
  const [games, setGames] = useState<Game[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

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

  useEffect(() => {
    fetchGames();
  }, [fetchGames]);

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
          <Button variant="outline" size="sm" className="mt-4" onClick={fetchGames}>
            Try again
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl">
      <div className="mb-8 text-center">
        <div className="mx-auto mb-3 flex size-12 items-center justify-center rounded-xl bg-primary">
          <Zap className="size-6 text-primary-foreground" />
        </div>
        <h1 className="text-2xl font-bold tracking-tight">Daily Challenge</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Pick a game and test your skills
        </p>
      </div>

      <div className="grid gap-4">
        {games.map((game) => (
          <Link key={game.id} to={`/${game.id}`} className="group">
            <Card className="transition-all duration-200 hover:shadow-md group-hover:border-primary/30">
              <CardContent className="flex items-center gap-4 p-5">
                <div className="flex size-12 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
                  <Gamepad2 className="size-6" />
                </div>
                <div className="min-w-0 flex-1">
                  <p className="font-semibold">{game.name}</p>
                  <p className="mt-0.5 text-sm text-muted-foreground">
                    {game.description}
                  </p>
                </div>
                <div className="flex items-center gap-3">
                  {user && (
                    <div className="flex items-center gap-1 rounded-md bg-muted px-2 py-1 text-sm font-medium">
                      <Flame className="size-3.5 text-orange-500" />
                      <span>{user.stats.current_streak}</span>
                    </div>
                  )}
                  <ChevronRight className="size-5 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
                </div>
              </CardContent>
            </Card>
          </Link>
        ))}
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

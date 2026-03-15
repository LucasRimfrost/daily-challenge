import { useEffect, useState } from "react";
import { toast } from "sonner";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { getLeaderboard } from "@/api/leaderboard";
import { ApiRequestError } from "@/api/client";
import type { LeaderboardEntry } from "@/api/types";
import { useAuth } from "@/hooks/useAuth";
import { cn } from "@/lib/utils";
import { Code, Flame, Medal, RefreshCw, Trophy, Users } from "lucide-react";
import { Button } from "@/components/ui/button";

function RankCell({ rank }: { rank: number }) {
  if (rank === 1)
    return <Trophy className="mx-auto size-5 text-yellow-500" />;
  if (rank === 2)
    return <Medal className="mx-auto size-5 text-gray-400" />;
  if (rank === 3)
    return <Medal className="mx-auto size-5 text-amber-700" />;
  return <span className="text-muted-foreground">{rank}</span>;
}

export function LeaderboardPage() {
  const { user } = useAuth();
  const [tab, setTab] = useState<"trivia" | "code-output">("trivia");
  const [entries, setEntries] = useState<LeaderboardEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  function load() {
    setLoading(true);
    setError(false);
    getLeaderboard()
      .then(setEntries)
      .catch((err) => {
        setError(true);
        toast.error(
          err instanceof ApiRequestError
            ? err.message
            : "Failed to load leaderboard",
        );
      })
      .finally(() => setLoading(false));
  }

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (loading) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <div className="mx-auto mb-3 size-8 animate-spin rounded-full border-2 border-muted border-t-primary" />
          <p className="text-sm text-muted-foreground">Loading leaderboard...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <Trophy className="mx-auto mb-3 size-10 text-muted-foreground/50" />
          <p className="font-medium">Couldn't load leaderboard</p>
          <p className="mt-1 text-sm text-muted-foreground">
            Check your connection and try again.
          </p>
          <Button variant="outline" size="sm" className="mt-4" onClick={load}>
            <RefreshCw className="mr-2 size-3.5" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-xl">
            <Trophy className="size-5 text-yellow-500" />
            Leaderboard
          </CardTitle>
          <CardDescription>
            Top players ranked by total challenges solved
          </CardDescription>
          <div className="mt-3 flex gap-1 rounded-lg bg-muted p-1">
            <button
              className={cn(
                "flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors",
                tab === "trivia"
                  ? "bg-background shadow-sm"
                  : "text-muted-foreground hover:text-foreground",
              )}
              onClick={() => setTab("trivia")}
            >
              <Trophy className="size-3.5" />
              Daily Trivia
            </button>
            <button
              className={cn(
                "flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors",
                tab === "code-output"
                  ? "bg-background shadow-sm"
                  : "text-muted-foreground hover:text-foreground",
              )}
              onClick={() => setTab("code-output")}
            >
              <Code className="size-3.5" />
              What's the Output?
            </button>
          </div>
        </CardHeader>
        <CardContent>
          {tab === "code-output" ? (
            <div className="flex flex-col items-center gap-2 py-12 text-center">
              <Code className="size-10 text-muted-foreground/50" />
              <p className="font-medium">Coming soon</p>
              <p className="text-sm text-muted-foreground">
                The What's the Output? leaderboard is on its way!
              </p>
            </div>
          ) : entries.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-12 text-center">
              <Users className="size-10 text-muted-foreground/50" />
              <p className="font-medium">No players yet</p>
              <p className="text-sm text-muted-foreground">
                Be the first to solve a challenge!
              </p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-12 text-center">#</TableHead>
                  <TableHead>Player</TableHead>
                  <TableHead className="text-right">Streak</TableHead>
                  <TableHead className="hidden text-right sm:table-cell">
                    Best
                  </TableHead>
                  <TableHead className="text-right">Solved</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {entries.map((entry, i) => {
                  const rank = i + 1;
                  const isCurrentUser = user?.username === entry.username;

                  return (
                    <TableRow
                      key={entry.username}
                      className={cn(
                        isCurrentUser && "bg-primary/5",
                      )}
                    >
                      <TableCell className="text-center font-medium">
                        <RankCell rank={rank} />
                      </TableCell>
                      <TableCell className="font-medium">
                        <span className="flex items-center gap-2">
                          {entry.username}
                          {isCurrentUser && (
                            <span className="rounded-full bg-primary/10 px-1.5 py-0.5 text-[10px] font-semibold text-primary">
                              YOU
                            </span>
                          )}
                        </span>
                      </TableCell>
                      <TableCell className="text-right">
                        <span className="inline-flex items-center gap-1">
                          <Flame className="size-3.5 text-orange-500" />
                          {entry.current_streak}
                        </span>
                      </TableCell>
                      <TableCell className="hidden text-right text-muted-foreground sm:table-cell">
                        {entry.longest_streak}
                      </TableCell>
                      <TableCell className="text-right font-semibold">
                        {entry.total_solved}
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { getArchive } from "@/api/code-output";
import { ApiRequestError } from "@/api/client";
import type { CodeOutputArchiveEntry } from "@/api/types";
import { cn } from "@/lib/utils";
import { difficultyConfig, getLanguageLabel } from "@/lib/game";
import {
  Archive,
  Calendar,
  CheckCircle,
  Circle,
  Clock,
  XCircle,
} from "lucide-react";

type ChallengeStatus = "solved" | "failed" | "in_progress" | "not_attempted";

function getStatus(entry: CodeOutputArchiveEntry): ChallengeStatus {
  if (entry.is_solved) return "solved";
  if (entry.attempts_used >= entry.max_attempts) return "failed";
  if (entry.attempts_used > 0) return "in_progress";
  return "not_attempted";
}

const statusConfig: Record<
  ChallengeStatus,
  { border: string; bg: string; icon: React.ReactNode; label: string }
> = {
  solved: {
    border: "border-green-500/30",
    bg: "bg-green-500/5 hover:bg-green-500/10",
    icon: <CheckCircle className="size-4 text-green-500" />,
    label: "Solved",
  },
  failed: {
    border: "border-red-500/30",
    bg: "bg-red-500/5 hover:bg-red-500/10",
    icon: <XCircle className="size-4 text-red-500" />,
    label: "Failed",
  },
  in_progress: {
    border: "border-yellow-500/30",
    bg: "bg-yellow-500/5 hover:bg-yellow-500/10",
    icon: <Clock className="size-4 text-yellow-500" />,
    label: "In progress",
  },
  not_attempted: {
    border: "border-border",
    bg: "bg-card hover:bg-muted/50",
    icon: <Circle className="size-4 text-muted-foreground" />,
    label: "Not attempted",
  },
};

export function CodeOutputArchivePage() {
  const [entries, setEntries] = useState<CodeOutputArchiveEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const fetchArchive = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const data = await getArchive();
      data.sort(
        (a, b) =>
          new Date(b.scheduled_date).getTime() -
          new Date(a.scheduled_date).getTime(),
      );
      setEntries(data);
    } catch (err) {
      if (err instanceof ApiRequestError) {
        setError(err.message);
      } else {
        setError("Failed to load archive.");
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchArchive();
  }, [fetchArchive]);

  if (loading) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <div className="mx-auto mb-3 size-8 animate-spin rounded-full border-2 border-muted border-t-primary" />
          <p className="text-sm text-muted-foreground">Loading archive...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <Archive className="mx-auto mb-3 size-10 text-muted-foreground/50" />
          <p className="text-lg font-medium">{error}</p>
          <Button
            variant="outline"
            size="sm"
            className="mt-4"
            onClick={fetchArchive}
          >
            Try again
          </Button>
        </div>
      </div>
    );
  }

  if (entries.length === 0) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <Archive className="mx-auto mb-3 size-10 text-muted-foreground/50" />
          <p className="text-lg font-medium">No previous challenges yet</p>
          <p className="mt-1 text-sm text-muted-foreground">
            Check back tomorrow!
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-4xl">
      <div className="mb-6">
        <h1 className="text-2xl font-bold tracking-tight">
          What's the Output? Archive
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          {entries.length} past challenge{entries.length === 1 ? "" : "s"}
        </p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {entries.map((entry) => {
          const status = getStatus(entry);
          const cfg = statusConfig[status];
          const diff =
            difficultyConfig[entry.difficulty] ?? difficultyConfig.medium;

          return (
            <Link
              key={entry.id}
              to={`/code-output/${entry.scheduled_date}`}
              className="group"
            >
              <Card
                className={cn(
                  "transition-all duration-200",
                  cfg.border,
                  cfg.bg,
                  "group-hover:shadow-md",
                )}
              >
                <CardContent className="flex flex-col gap-3 p-4">
                  <div className="flex items-start justify-between gap-2">
                    <div className="min-w-0 flex-1">
                      <p className="truncate font-medium leading-tight">
                        {entry.title}
                      </p>
                      <div className="mt-1 flex items-center gap-2">
                        <p className="flex items-center gap-1 text-xs text-muted-foreground">
                          <Calendar className="size-3" />
                          {entry.scheduled_date}
                        </p>
                        <Badge
                          variant="outline"
                          className="border-neutral-700 bg-neutral-800 text-[10px] text-neutral-300"
                        >
                          {getLanguageLabel(entry.language)}
                        </Badge>
                      </div>
                    </div>
                    <Badge
                      variant="outline"
                      className={cn(
                        "shrink-0 text-xs capitalize",
                        diff.class,
                      )}
                    >
                      {diff.label}
                    </Badge>
                  </div>

                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                      {cfg.icon}
                      <span>{cfg.label}</span>
                    </div>
                    {entry.attempts_used > 0 && (
                      <span className="text-xs text-muted-foreground">
                        {entry.attempts_used}/{entry.max_attempts} attempts
                      </span>
                    )}
                  </div>
                </CardContent>
              </Card>
            </Link>
          );
        })}
      </div>
    </div>
  );
}

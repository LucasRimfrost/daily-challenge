import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
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
import { getHistory } from "@/api/code-output";
import { ApiRequestError } from "@/api/client";
import type { CodeOutputHistoryEntry } from "@/api/types";
import { cn } from "@/lib/utils";
import { difficultyConfig, getLanguageLabel } from "@/lib/game";
import {
  CheckCircle,
  History,
  RefreshCw,
  ScrollText,
  XCircle,
} from "lucide-react";

export function CodeOutputHistoryPage() {
  const [entries, setEntries] = useState<CodeOutputHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  function load() {
    setLoading(true);
    setError(false);
    getHistory()
      .then(setEntries)
      .catch((err) => {
        setError(true);
        toast.error(
          err instanceof ApiRequestError
            ? err.message
            : "Failed to load history",
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
          <p className="text-sm text-muted-foreground">Loading history...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <History className="mx-auto mb-3 size-10 text-muted-foreground/50" />
          <p className="font-medium">Couldn't load history</p>
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

  const solved = entries.filter((e) => e.is_correct).length;

  return (
    <div className="mx-auto max-w-2xl">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-xl">
            <History className="size-5" />
            What's the Output? History
          </CardTitle>
          <CardDescription>
            {entries.length === 0
              ? "Your past challenges will appear here"
              : `${solved} of ${entries.length} solved`}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {entries.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-12 text-center">
              <ScrollText className="size-10 text-muted-foreground/50" />
              <p className="font-medium">No history yet</p>
              <p className="text-sm text-muted-foreground">
                Complete today's challenge to get started!
              </p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-10" />
                  <TableHead>Challenge</TableHead>
                  <TableHead className="hidden sm:table-cell">Date</TableHead>
                  <TableHead className="text-center">Difficulty</TableHead>
                  <TableHead className="text-right">Attempts</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {entries.map((entry) => {
                  const diff =
                    difficultyConfig[entry.difficulty] ??
                    difficultyConfig.medium;

                  return (
                    <TableRow
                      key={`${entry.challenge_id}-${entry.submitted_at}`}
                    >
                      <TableCell>
                        {entry.is_correct ? (
                          <CheckCircle className="size-4 text-green-500" />
                        ) : (
                          <XCircle className="size-4 text-destructive" />
                        )}
                      </TableCell>
                      <TableCell>
                        <div>
                          <p className="font-medium">{entry.title}</p>
                          <div className="flex items-center gap-1.5">
                            <p className="text-xs text-muted-foreground sm:hidden">
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
                      </TableCell>
                      <TableCell className="hidden text-muted-foreground sm:table-cell">
                        {entry.scheduled_date}
                      </TableCell>
                      <TableCell className="text-center">
                        <Badge
                          variant="outline"
                          className={cn("capitalize", diff.class)}
                        >
                          {diff.label}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right text-muted-foreground">
                        {entry.attempt_number}
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

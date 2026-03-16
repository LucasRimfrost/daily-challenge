import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
} from "react";
import { useParams } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import {
  getArchive,
  getChallengeByDate,
  getToday,
  submitAnswer,
} from "@/api/code-output";
import { ChallengeNav } from "@/components/ChallengeNav";
import { toast } from "sonner";
import { ApiRequestError } from "@/api/client";
import type { CodeOutputChallenge, SubmitResponse } from "@/api/types";
import { useAuth } from "@/hooks/useAuth";
import { cn } from "@/lib/utils";
import { difficultyConfig, getLanguageLabel } from "@/lib/game";
import {
  Check,
  CheckCircle,
  ClipboardCheck,
  Copy,
  Flame,
  Lightbulb,
  Send,
  Share2,
  Terminal,
  XCircle,
} from "lucide-react";
import { AttemptDots } from "@/components/AttemptDots";
import Prism from "prismjs";
import "prismjs/components/prism-python";
import "prismjs/components/prism-javascript";
import "prismjs/components/prism-rust";

function generateShareText(challenge: CodeOutputChallenge): string {
  const pattern = Array.from({ length: challenge.attempts_used })
    .map((_, i) => {
      if (challenge.is_solved && i === challenge.attempts_used - 1)
        return "\u{1F7E9}";
      return "\u2B1B";
    })
    .join("");

  return `What's the Output? ${challenge.scheduled_date} ${pattern} ${challenge.attempts_used}/${challenge.max_attempts}`;
}

export function CodeOutputPage() {
  const { date } = useParams<{ date?: string }>();
  const { user, refresh } = useAuth();
  const [challenge, setChallenge] = useState<CodeOutputChallenge | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState("");
  const [answer, setAnswer] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [lastResult, setLastResult] = useState<SubmitResponse | null>(null);
  const [shaking, setShaking] = useState(false);
  const [poppedDot, setPoppedDot] = useState(-1);
  const [hint, setHint] = useState<string | null>(null);
  const [answerError, setAnswerError] = useState("");
  const [copied, setCopied] = useState(false);
  const [codeCopied, setCodeCopied] = useState(false);
  const [hintVisible, setHintVisible] = useState(false);
  const [guesses, setGuesses] = useState<string[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);

  const highlightedLines = useMemo(() => {
    if (!challenge) return [];
    const langMap: Record<string, string> = {
      python: "python",
      javascript: "javascript",
      rust: "rust",
    };
    const lang = langMap[challenge.language] ?? "javascript";
    const grammar = Prism.languages[lang];
    if (!grammar) return challenge.code_snippet.split("\n");
    return Prism.highlight(challenge.code_snippet, grammar, lang).split("\n");
  }, [challenge]);

  const fetchChallenge = useCallback(async () => {
    setLoading(true);
    setLoadError("");
    setChallenge(null);
    setLastResult(null);
    setHint(null);
    setHintVisible(false);
    setAnswer("");
    setGuesses([]);
    try {
      const data = date ? await getChallengeByDate(date) : await getToday();
      setChallenge(data);
      setGuesses(data.previous_guesses ?? []);
      if (data.attempts_used >= 2) {
        setHint(data.hint);
      }
    } catch (err) {
      if (err instanceof ApiRequestError && err.status === 404) {
        setLoadError(
          date
            ? "Challenge not found for this date."
            : "No challenge available today. Check back tomorrow!",
        );
      } else {
        setLoadError("Failed to load challenge.");
      }
    } finally {
      setLoading(false);
    }
  }, [date]);

  useEffect(() => {
    fetchChallenge();
  }, [fetchChallenge]);

  useEffect(() => {
    if (
      challenge &&
      !challenge.is_solved &&
      challenge.attempts_used < challenge.max_attempts
    ) {
      inputRef.current?.focus();
    }
  }, [challenge]);

  async function handleShare() {
    if (!challenge) return;
    const text = generateShareText(challenge);
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!challenge) return;
    if (!answer.trim()) {
      setAnswerError("Enter the expected output");
      return;
    }

    setSubmitting(true);
    setLastResult(null);

    try {
      const trimmed = answer.trim();
      setGuesses((prev) => [...prev, trimmed]);
      const result = await submitAnswer({
        answer: trimmed,
        challenge_id: challenge.id,
      });
      setLastResult(result);
      setAnswer("");

      if (result.hint) setHint(result.hint);

      setPoppedDot(result.attempt_number - 1);
      setTimeout(() => setPoppedDot(-1), 400);

      if (result.is_correct) {
        refresh();
        fetchChallenge();
      } else if (result.attempts_remaining === 0) {
        fetchChallenge();
      } else {
        setShaking(true);
        setTimeout(() => setShaking(false), 500);
        setChallenge((c) =>
          c ? { ...c, attempts_used: result.attempt_number } : c,
        );
        setTimeout(() => inputRef.current?.focus(), 100);
      }
    } catch (err) {
      if (err instanceof ApiRequestError) {
        toast.error(err.message);
      }
    } finally {
      setSubmitting(false);
    }
  }

  if (loading) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <div className="mx-auto mb-3 size-8 animate-spin rounded-full border-2 border-muted border-t-primary" />
          <p className="text-sm text-muted-foreground">
            Loading today's challenge...
          </p>
        </div>
      </div>
    );
  }

  if (loadError) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <Terminal className="mx-auto mb-3 size-10 text-muted-foreground/50" />
          <p className="text-lg font-medium">{loadError}</p>
          <p className="mt-1 text-sm text-muted-foreground">
            New challenges drop daily at midnight.
          </p>
        </div>
      </div>
    );
  }

  if (!challenge) return null;

  const exhausted =
    challenge.attempts_used >= challenge.max_attempts && !challenge.is_solved;
  const done = challenge.is_solved || exhausted;
  const diff = difficultyConfig[challenge.difficulty] ?? difficultyConfig.medium;

  return (
    <div className="mx-auto max-w-2xl">
      {date && (
        <ChallengeNav
          currentDate={challenge.scheduled_date}
          basePath="/code-output"
          getArchive={getArchive}
        />
      )}
      <Card>
        {/* Header */}
        <CardHeader>
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <CardTitle className="text-xl leading-tight">
                {challenge.title}
              </CardTitle>
              <p className="mt-1 text-sm text-muted-foreground">
                {challenge.scheduled_date}
              </p>
            </div>
            <Badge
              variant="outline"
              className={cn("shrink-0 capitalize", diff.class)}
            >
              {diff.label}
            </Badge>
          </div>
        </CardHeader>

        <CardContent className="grid gap-6">
          {/* Description */}
          <p className="leading-relaxed">{challenge.description}</p>

          {/* Code snippet */}
          <div className="code-block overflow-hidden rounded-2xl">
            <div className="code-block-header flex items-center justify-between px-4 py-2">
              <Badge
                variant="outline"
                className="border-neutral-600 bg-neutral-700 text-xs text-neutral-300"
              >
                {getLanguageLabel(challenge.language)}
              </Badge>
              <button
                onClick={() => {
                  navigator.clipboard.writeText(challenge.code_snippet);
                  setCodeCopied(true);
                  setTimeout(() => setCodeCopied(false), 2000);
                }}
                title="Copy code"
                className="rounded-md p-1.5 text-neutral-500 transition-colors hover:bg-neutral-700 hover:text-neutral-300"
              >
                {codeCopied ? (
                  <Check className="size-4 text-green-400" />
                ) : (
                  <Copy className="size-4" />
                )}
              </button>
            </div>
            <div className="overflow-x-auto p-4">
              <pre className="text-sm leading-loose">
                <code>
                  {highlightedLines.map((line, i) => (
                    <div key={i} className="flex">
                      <span className="mr-6 inline-block w-6 select-none text-right text-neutral-600">
                        {i + 1}
                      </span>
                      <span
                        className="text-neutral-200"
                        dangerouslySetInnerHTML={{ __html: line || " " }}
                      />
                    </div>
                  ))}
                </code>
              </pre>
            </div>
          </div>

          {/* Expected Output — directly below code when revealed */}
          {challenge.correct_answer && (
            <div className="code-block animate-slide-up-fade -mt-3 overflow-hidden rounded-2xl">
              <div className="code-block-header px-4 py-2">
                <p className="text-xs font-medium text-neutral-400">
                  Expected Output
                </p>
              </div>
              <pre className="p-4 text-sm text-green-400">
                {challenge.correct_answer}
              </pre>
            </div>
          )}

          {done ? (
            <>
              {/* Result status with attempt dots */}
              {challenge.is_solved ? (
                <div className="animate-slide-up-fade rounded-lg border border-green-500/20 bg-green-500/5 px-4 py-5 text-center">
                  <CheckCircle className="mx-auto mb-2 size-8 text-green-500" />
                  <p className="text-lg font-semibold text-green-700 dark:text-green-400">
                    Correct!
                  </p>
                  <div className="mt-2 flex justify-center">
                    <AttemptDots
                      maxAttempts={challenge.max_attempts}
                      attemptsUsed={challenge.attempts_used}
                      isSolved={challenge.is_solved}
                      guesses={guesses}
                      size="sm"
                    />
                  </div>
                  <p className="mt-2 text-sm text-muted-foreground">
                    Solved in {challenge.attempts_used} attempt
                    {challenge.attempts_used === 1 ? "" : "s"}
                  </p>
                  {!date && user && user.code_output_stats.current_streak > 0 && (
                    <div className="mt-3 inline-flex items-center gap-1.5 rounded-full bg-muted px-3 py-1 text-sm font-medium">
                      <Flame className="size-4 text-orange-500" />
                      {user.code_output_stats.current_streak} day streak
                    </div>
                  )}
                </div>
              ) : (
                <div className="animate-slide-up-fade rounded-lg border border-muted bg-muted/50 px-4 py-5 text-center">
                  <XCircle className="mx-auto mb-2 size-8 text-muted-foreground" />
                  <p className="text-lg font-semibold">Out of attempts</p>
                  <div className="mt-2 flex justify-center">
                    <AttemptDots
                      maxAttempts={challenge.max_attempts}
                      attemptsUsed={challenge.attempts_used}
                      isSolved={false}
                      guesses={guesses}
                      size="sm"
                    />
                  </div>
                  <p className="mt-2 text-sm text-muted-foreground">
                    Better luck tomorrow!
                  </p>
                </div>
              )}

              {/* Share result button */}
              <div className="flex justify-center">
                <Button variant="outline" size="sm" onClick={handleShare}>
                  {copied ? (
                    <ClipboardCheck className="mr-2 size-4" />
                  ) : (
                    <Share2 className="mr-2 size-4" />
                  )}
                  {copied ? "Copied!" : "Share Result"}
                </Button>
              </div>
            </>
          ) : (
            <>
              <Separator />

              {/* Attempt indicators (in-progress) */}
              <div className="flex flex-col items-center gap-3">
                <AttemptDots
                  maxAttempts={challenge.max_attempts}
                  attemptsUsed={challenge.attempts_used}
                  isSolved={false}
                  guesses={guesses}
                  poppedDot={poppedDot}
                />
                <p className="text-xs text-muted-foreground">
                  {`${challenge.max_attempts - challenge.attempts_used} attempt${challenge.max_attempts - challenge.attempts_used === 1 ? "" : "s"} remaining`}
                </p>
              </div>

              {/* Hint status */}
              {!hint && (
                <p className="flex items-center justify-center gap-1.5 text-xs text-muted-foreground">
                  <Lightbulb className="size-3.5" />
                  Hint unlocks after 2 failed attempts
                </p>
              )}

              {/* Feedback after submission */}
              {lastResult && !lastResult.is_correct && (
                <div
                  key={lastResult.attempt_number}
                  className={cn(
                    "animate-slide-up-fade flex items-center gap-2.5 rounded-lg px-4 py-3 text-sm font-medium",
                    shaking && "animate-shake",
                    "bg-destructive/10 text-destructive",
                  )}
                >
                  <XCircle className="size-4 shrink-0" />
                  Not quite. {lastResult.attempts_remaining} attempt
                  {lastResult.attempts_remaining === 1 ? "" : "s"} left.
                </div>
              )}

              {/* Inline hint reveal */}
              {hint && hintVisible && (
                <div className="animate-slide-up-fade flex items-start gap-2.5 rounded-lg border border-yellow-500/20 bg-yellow-500/5 px-4 py-3 text-sm">
                  <Lightbulb className="mt-0.5 size-4 shrink-0 text-yellow-500" />
                  <div>
                    <p className="mb-0.5 text-xs font-medium text-yellow-700 dark:text-yellow-400">
                      Hint
                    </p>
                    <p className="text-foreground">{hint}</p>
                  </div>
                </div>
              )}

              {/* Input form */}
              <form noValidate onSubmit={handleSubmit} className="grid gap-2">
                <div className="flex items-center gap-2">
                  <Input
                    ref={inputRef}
                    value={answer}
                    onChange={(e) => {
                      setAnswer(e.target.value);
                      setAnswerError("");
                    }}
                    placeholder="Type the expected output..."
                    disabled={submitting}
                    autoComplete="off"
                    className="flex-1 font-mono"
                    aria-invalid={!!answerError || undefined}
                  />
                  {hint && (
                    <Button
                      type="button"
                      variant="ghost"
                      size="lg"
                      onClick={() => setHintVisible((v) => !v)}
                      className={cn(
                        "shrink-0",
                        hintVisible
                          ? "text-yellow-500"
                          : "text-muted-foreground hover:text-yellow-500",
                      )}
                      aria-label={hintVisible ? "Hide hint" : "Show hint"}
                    >
                      <Lightbulb className="size-4" />
                    </Button>
                  )}
                  <Button type="submit" disabled={submitting} size="lg">
                    {submitting ? (
                      <div className="size-4 animate-spin rounded-full border-2 border-primary-foreground/30 border-t-primary-foreground" />
                    ) : (
                      <Send className="size-4" />
                    )}
                  </Button>
                </div>
                <p className="text-xs text-muted-foreground">
                  Output is case-sensitive
                </p>
                {answerError && (
                  <p className="text-sm text-destructive">{answerError}</p>
                )}
              </form>
            </>
          )}
        </CardContent>

        {/* Stats footer when done */}
        {done && user && (
          <CardFooter className="border-t">
            <div className="flex w-full items-center justify-around text-center text-sm">
              <div>
                <p className="text-lg font-bold">{user.code_output_stats.total_solved}</p>
                <p className="text-xs text-muted-foreground">Solved</p>
              </div>
              <Separator orientation="vertical" className="h-8" />
              <div>
                <p className="text-lg font-bold">
                  {user.code_output_stats.current_streak}
                </p>
                <p className="text-xs text-muted-foreground">Streak</p>
              </div>
              <Separator orientation="vertical" className="h-8" />
              <div>
                <p className="text-lg font-bold">
                  {user.code_output_stats.longest_streak}
                </p>
                <p className="text-xs text-muted-foreground">Best</p>
              </div>
            </div>
          </CardFooter>
        )}
      </Card>
    </div>
  );
}

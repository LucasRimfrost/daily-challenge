import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
} from "react";
import { useParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
  ClipboardCheck,
  Copy,
  Flame,
  Lightbulb,
  Send,
  Share2,
  Terminal,
} from "lucide-react";
import { SegmentedProgressBar } from "@/components/SegmentedProgressBar";
import { HintPopover } from "@/components/HintPopover";
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
  const [toastText, setToastText] = useState("");
  const [toastExiting, setToastExiting] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const fetchIdRef = useRef(0);
  const toastTimersRef = useRef<ReturnType<typeof setTimeout>[]>([]);

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
    const id = ++fetchIdRef.current;
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
      if (fetchIdRef.current !== id) return; // stale response
      setChallenge(data);
      setGuesses(data.previous_guesses ?? []);
      if (data.attempts_used >= 2) {
        setHint(data.hint);
      }
    } catch (err) {
      if (fetchIdRef.current !== id) return;
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
      if (fetchIdRef.current === id) setLoading(false);
    }
  }, [date]);

  useEffect(() => {
    fetchChallenge();
  }, [fetchChallenge]);

  // Clean up toast timers on unmount
  useEffect(() => () => { toastTimersRef.current.forEach(clearTimeout); }, []);

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
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error("Failed to copy to clipboard");
    }
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
        // Incorrect — shake input and show auto-dismissing toast
        setShaking(true);
        setTimeout(() => setShaking(false), 650);

        // Clear any pending toast timers
        toastTimersRef.current.forEach(clearTimeout);
        toastTimersRef.current = [];

        // Show toast: enter → hold 2s → exit → cleanup
        setToastExiting(false);
        setToastText(`Not quite — ${result.attempts_remaining} attempt${result.attempts_remaining === 1 ? "" : "s"} left`);
        toastTimersRef.current.push(
          setTimeout(() => setToastExiting(true), 2000),
          setTimeout(() => { setToastText(""); setToastExiting(false); }, 2450),
        );
        setChallenge((c) =>
          c ? { ...c, attempts_used: result.attempt_number } : c,
        );
        setTimeout(() => inputRef.current?.focus(), 100);
      }
    } catch (err) {
      setGuesses((prev) => prev.slice(0, -1));
      if (err instanceof ApiRequestError) {
        toast.error(err.message);
      } else {
        toast.error("Connection error. Please try again.");
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
  const remaining = challenge.max_attempts - challenge.attempts_used;

  return (
    <div className="mx-auto max-w-2xl">
      {date && (
        <ChallengeNav
          currentDate={challenge.scheduled_date}
          basePath="/code-output"
          getArchive={getArchive}
        />
      )}

      <div className="relative rounded-2xl bg-card text-card-foreground border shadow-sm p-6 sm:p-8">
        {/* Date */}
        <p className="text-[13px] text-muted-foreground/60 mb-1.5">
          {challenge.scheduled_date}
        </p>

        {/* Title + language + difficulty */}
        <div className="flex items-center justify-between gap-3 mb-6">
          <div className="flex items-center gap-2.5 min-w-0">
            <h1 className="text-[22px] font-medium leading-tight">
              {challenge.title}
            </h1>
            <span className="shrink-0 rounded-full bg-neutral-100 dark:bg-neutral-800 px-2 py-0.5 text-xs font-mono text-muted-foreground">
              {getLanguageLabel(challenge.language)}
            </span>
          </div>
          <span
            className={cn(
              "shrink-0 rounded-full px-2.5 py-0.5 text-xs font-medium",
              diff.class,
            )}
          >
            {diff.label}
          </span>
        </div>

        {/* Description */}
        <p className="leading-relaxed text-foreground/90 mb-6">
          {challenge.description}
        </p>

        {/* Code snippet */}
        <div className="code-block overflow-hidden rounded-2xl">
          <div className="code-block-header flex items-center justify-between px-4 py-2.5">
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-1.5">
                <div className="size-2.5 rounded-full bg-red-500/60" />
                <div className="size-2.5 rounded-full bg-yellow-500/60" />
                <div className="size-2.5 rounded-full bg-green-500/60" />
              </div>
              <span className="text-xs text-muted-foreground">
                {getLanguageLabel(challenge.language)}
              </span>
            </div>
            <button
              onClick={async () => {
                try {
                  await navigator.clipboard.writeText(challenge.code_snippet);
                  setCodeCopied(true);
                  setTimeout(() => setCodeCopied(false), 2000);
                } catch {
                  toast.error("Failed to copy code");
                }
              }}
              title="Copy code"
              aria-label="Copy code"
              className="rounded-md p-1.5 text-neutral-400 dark:text-neutral-500 transition-colors hover:bg-neutral-200 hover:text-neutral-700 dark:hover:bg-neutral-700 dark:hover:text-neutral-300"
            >
              {codeCopied ? (
                <Check className="size-4 text-green-600 dark:text-green-400" />
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
                    <span className="mr-6 inline-block w-6 select-none text-right text-neutral-400 dark:text-neutral-600">
                      {i + 1}
                    </span>
                    <span
                      className="text-neutral-800 dark:text-neutral-200"
                      dangerouslySetInnerHTML={{ __html: line || " " }}
                    />
                  </div>
                ))}
              </code>
            </pre>
          </div>
        </div>

        {/* Thin divider */}
        <div className="bg-border/50 my-8" style={{ height: "0.5px" }} />

        {/* ── Auto-dismissing toast ── */}
        {toastText && (
          <div className="pointer-events-none absolute inset-x-0 top-5 sm:top-6 z-10 flex justify-center">
            <div
              className={cn(
                "pointer-events-auto whitespace-nowrap rounded-full bg-foreground text-background px-5 py-2.5 text-sm font-medium shadow-lg",
                toastExiting ? "animate-toast-exit" : "animate-toast-enter",
              )}
            >
              {toastText}
            </div>
          </div>
        )}

        {/* ── Unsolved ── */}
        {!done && (
          <>
            {/* Attempt progress bar */}
            <div className="flex flex-col gap-2">
              <SegmentedProgressBar
                maxAttempts={challenge.max_attempts}
                attemptsUsed={challenge.attempts_used}
                isSolved={false}
                guesses={guesses}
                animatingSegment={poppedDot}
              />
              <p className="text-sm text-muted-foreground text-center">
                Attempt {Math.min(challenge.attempts_used + 1, challenge.max_attempts)} of {challenge.max_attempts}
              </p>
            </div>

            {/* Hint unlock notice */}
            {!hint && (
              <p className="flex items-center justify-center gap-1.5 text-xs text-muted-foreground/40 mt-3">
                <Lightbulb className="size-3" />
                Hint unlocks after 2 failed attempts
              </p>
            )}

            {/* Answer input */}
            <form noValidate onSubmit={handleSubmit} className="mt-8">
              <div className={cn("flex items-center gap-2", shaking && "animate-shake")}>
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
                  <HintPopover
                    hint={hint}
                    visible={hintVisible}
                    onToggle={() => setHintVisible((v) => !v)}
                  />
                )}
                <Button type="submit" disabled={submitting} size="lg">
                  {submitting ? (
                    <div className="size-4 animate-spin rounded-full border-2 border-primary-foreground/30 border-t-primary-foreground" />
                  ) : (
                    <Send className="size-4" />
                  )}
                </Button>
              </div>
              <p className="text-xs text-muted-foreground/50 mt-2">
                Output is case-sensitive
              </p>
              {answerError && (
                <p className="text-sm text-destructive mt-1">{answerError}</p>
              )}
            </form>
          </>
        )}

        {/* ── Solved ── */}
        {challenge.is_solved && (
          <>
            {/* Answer reveal — success */}
            <div className="text-center py-4">
              <div className="animate-result-success result-reveal result-reveal-success inline-block rounded-xl px-8 py-5">
                <span className="result-label result-label-success block text-[11px] font-medium mb-1.5">
                  Result
                </span>
                <pre className="font-mono text-[28px] sm:text-[32px] font-medium result-text-success leading-tight whitespace-pre-wrap">
                  {challenge.correct_answer}
                </pre>
              </div>
            </div>

            <p className="text-sm text-green-600 dark:text-green-400 text-center mt-3">
              Solved in {challenge.attempts_used} attempt
              {challenge.attempts_used === 1 ? "" : "s"}
            </p>

            {/* Streak badge */}
            {!date && user && user.code_output_stats.current_streak > 0 && (
              <p className="text-center text-sm text-muted-foreground mt-2 mb-6">
                <Flame className="inline size-4 text-orange-500 mr-1 -mt-0.5" />
                {user.code_output_stats.current_streak} day streak
              </p>
            )}

            {/* Stats */}
            {user && (
              <>
                <div className="bg-border/50 mt-6" style={{ height: "0.5px" }} />
                <div className="grid grid-cols-3 text-center py-5">
                  <div>
                    <p className="text-lg font-bold">{user.code_output_stats.total_solved}</p>
                    <p className="text-xs text-muted-foreground">Solved</p>
                  </div>
                  <div
                    className="border-border/50"
                    style={{ borderLeftWidth: "0.5px", borderRightWidth: "0.5px", borderStyle: "solid" }}
                  >
                    <p className="text-lg font-bold">{user.code_output_stats.current_streak}</p>
                    <p className="text-xs text-muted-foreground">Streak</p>
                  </div>
                  <div>
                    <p className="text-lg font-bold">{user.code_output_stats.longest_streak}</p>
                    <p className="text-xs text-muted-foreground">Best</p>
                  </div>
                </div>
              </>
            )}

            {/* Share button */}
            <Button
              className="w-full mt-4 rounded-lg"
              size="lg"
              onClick={handleShare}
            >
              {copied ? (
                <ClipboardCheck className="mr-2 size-4" />
              ) : (
                <Share2 className="mr-2 size-4" />
              )}
              {copied ? "Copied!" : "Share Result"}
            </Button>
          </>
        )}

        {/* ── Exhausted ── */}
        {exhausted && (
          <>
            <div className="text-center py-4">
              <p className="text-sm text-muted-foreground mb-4">The correct output was</p>

              {/* Answer reveal — failure */}
              {challenge.correct_answer && (
                <div className="animate-fail-reveal result-reveal result-reveal-fail inline-block rounded-xl px-8 py-5">
                  <pre className="font-mono text-[28px] sm:text-[32px] font-medium text-foreground leading-tight whitespace-pre-wrap">
                    {challenge.correct_answer}
                  </pre>
                </div>
              )}

              <p className="text-sm text-muted-foreground mt-5">
                Better luck tomorrow!
              </p>
            </div>

            {/* Share button */}
            <Button
              className="w-full mt-4 rounded-lg"
              size="lg"
              onClick={handleShare}
            >
              {copied ? (
                <ClipboardCheck className="mr-2 size-4" />
              ) : (
                <Share2 className="mr-2 size-4" />
              )}
              {copied ? "Copied!" : "Share Result"}
            </Button>
          </>
        )}
      </div>
    </div>
  );
}

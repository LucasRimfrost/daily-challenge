import {
  Fragment,
  useCallback,
  useEffect,
  useRef,
  useState,
  type FormEvent,
} from "react";
import { useParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { getArchive, getChallengeByDate, getToday, submitAnswer } from "@/api/trivia";
import { ChallengeNav } from "@/components/ChallengeNav";
import { toast } from "sonner";
import { ApiRequestError } from "@/api/client";
import type { Challenge, SubmitResponse } from "@/api/types";
import { useAuth } from "@/hooks/useAuth";
import { cn } from "@/lib/utils";
import { difficultyConfig } from "@/lib/game";
import {
  ArrowRight,
  ClipboardCheck,
  Flame,
  Lightbulb,
  Send,
  Share2,
  Trophy,
  XCircle,
} from "lucide-react";
import { SegmentedProgressBar } from "@/components/SegmentedProgressBar";
import { HintPopover } from "@/components/HintPopover";

/** Extract arrow-separated sequences (e.g. "87 → 165 → 726 → ?") into visual pills */
function parseDescription(description: string): { text: string; pills: string[] | null } {
  const arrowMatch = description.match(/((?:[\w?.,-]+\s*(?:→|->)\s*){2,}[\w?.,-]+)/);
  if (arrowMatch) {
    const seqStr = arrowMatch[1];
    const pills = seqStr.split(/\s*(?:→|->)\s*/).filter(Boolean);
    if (pills.length >= 3) {
      let text = description.replace(seqStr, "").replace(/\s{2,}/g, " ").trim();
      if (text.endsWith(":")) text = text.slice(0, -1).trim();
      return { text: text || description, pills };
    }
  }
  return { text: description, pills: null };
}

function generateShareText(challenge: Challenge): string {
  const pattern = Array.from({ length: challenge.attempts_used })
    .map((_, i) => {
      if (challenge.is_solved && i === challenge.attempts_used - 1) return "\u{1F7E9}";
      return "\u2B1B";
    })
    .join("");

  return `BrainForge ${challenge.scheduled_date} ${pattern} ${challenge.attempts_used}/${challenge.max_attempts}`;
}

export function ChallengePage() {
  const { date } = useParams<{ date?: string }>();
  const { user, refresh } = useAuth();
  const [challenge, setChallenge] = useState<Challenge | null>(null);
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
  const [hintVisible, setHintVisible] = useState(false);
  const [guesses, setGuesses] = useState<string[]>([]);
  const [toastText, setToastText] = useState("");
  const [toastExiting, setToastExiting] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const fetchIdRef = useRef(0);
  const toastTimersRef = useRef<ReturnType<typeof setTimeout>[]>([]);

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
      if (data.attempts_used >= 3) {
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

  // Auto-focus input when challenge loads
  useEffect(() => {
    if (challenge && !challenge.is_solved && challenge.attempts_used < challenge.max_attempts) {
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
      setAnswerError("Enter an answer");
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

      // Animate the newly filled dot
      setPoppedDot(result.attempt_number - 1);
      setTimeout(() => setPoppedDot(-1), 400);

      if (result.is_correct) {
        // Refresh auth to update streak in navbar
        refresh();
        fetchChallenge();
      } else if (result.attempts_remaining === 0) {
        // Out of attempts — refetch to get correct_answer
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
        // Re-focus for next attempt
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
          <p className="text-sm text-muted-foreground">Loading today's challenge...</p>
        </div>
      </div>
    );
  }

  if (loadError) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center">
          <Trophy className="mx-auto mb-3 size-10 text-muted-foreground/50" />
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
  const { text: questionText, pills } = parseDescription(challenge.description);

  return (
    <div className="mx-auto max-w-2xl">
      {date && (
        <ChallengeNav
          currentDate={challenge.scheduled_date}
          basePath="/trivia"
          getArchive={getArchive}
        />
      )}

      <div className="relative rounded-2xl bg-card text-card-foreground border shadow-sm p-6 sm:p-8">
      {/* Date */}
      <p className="text-[13px] text-muted-foreground/60 mb-1.5">
        {challenge.scheduled_date}
      </p>

      {/* Title + difficulty pill */}
      <div className="flex items-center justify-between gap-3 mb-8">
        <h1 className="text-[22px] font-medium leading-tight min-w-0">
          {challenge.title}
        </h1>
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
      <p className="leading-relaxed text-foreground/90 whitespace-pre-wrap">
        {questionText}
      </p>

      {/* Pill chain for arrow sequences */}
      {pills && (
        <div className="flex items-center gap-2 flex-wrap mt-4">
          {pills.map((item, i) => (
            <Fragment key={i}>
              <span className="font-mono text-sm bg-muted px-3 py-1.5 rounded-full">
                {item}
              </span>
              {i < pills.length - 1 && (
                <ArrowRight className="size-3.5 text-muted-foreground/50" />
              )}
            </Fragment>
          ))}
        </div>
      )}

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
              isSolved={challenge.is_solved}
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
              Hint unlocks after 3 failed attempts
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
                placeholder="Type your answer..."
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
            {answerError && (
              <p className="text-sm text-destructive mt-2">{answerError}</p>
            )}
          </form>
        </>
      )}

      {/* ── Solved ── */}
      {challenge.is_solved && (
        <>
          {/* Answer reveal — success */}
          <div className="text-center py-6">
            <div className="animate-result-success result-reveal result-reveal-success inline-block rounded-xl px-8 py-5">
              <span className="result-label result-label-success block text-[11px] font-medium mb-1.5">
                Answer
              </span>
              <p className="font-mono text-[28px] sm:text-[32px] font-medium result-text-success leading-tight">
                {challenge.correct_answer}
              </p>
            </div>
            <p className="text-sm text-green-600 dark:text-green-400 mt-3">
              Solved in {challenge.attempts_used} attempt
              {challenge.attempts_used === 1 ? "" : "s"}
            </p>
          </div>

          {/* Streak badge */}
          {!date && user && user.trivia_stats.current_streak > 0 && (
            <p className="text-center text-sm text-muted-foreground mb-6">
              <Flame className="inline size-4 text-orange-500 mr-1 -mt-0.5" />
              {user.trivia_stats.current_streak} day streak
            </p>
          )}

          {/* Stats */}
          {user && (
            <>
              <div className="bg-border/50" style={{ height: "0.5px" }} />
              <div className="grid grid-cols-3 text-center py-5">
                <div>
                  <p className="text-lg font-bold">{user.trivia_stats.total_solved}</p>
                  <p className="text-xs text-muted-foreground">Solved</p>
                </div>
                <div
                  className="border-border/50"
                  style={{ borderLeftWidth: "0.5px", borderRightWidth: "0.5px", borderStyle: "solid" }}
                >
                  <p className="text-lg font-bold">{user.trivia_stats.current_streak}</p>
                  <p className="text-xs text-muted-foreground">Streak</p>
                </div>
                <div>
                  <p className="text-lg font-bold">{user.trivia_stats.longest_streak}</p>
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
          <div className="text-center py-6">
            <XCircle className="mx-auto mb-3 size-8 text-muted-foreground/50" />
            <p className="text-sm text-muted-foreground mb-4">The answer was</p>
            {challenge.correct_answer && (
              <div className="animate-fail-reveal result-reveal result-reveal-fail inline-block rounded-xl px-8 py-5">
                <p className="font-mono text-[28px] sm:text-[32px] font-medium text-foreground leading-tight">
                  {challenge.correct_answer}
                </p>
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

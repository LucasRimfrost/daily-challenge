import { useEffect, useRef, useState } from "react";
import { Lightbulb } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface HintPopoverProps {
  hint: string;
  visible: boolean;
  onToggle: () => void;
}

export function HintPopover({ hint, visible, onToggle }: HintPopoverProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const onToggleRef = useRef(onToggle);
  onToggleRef.current = onToggle;

  // Keep the bubble mounted during exit animation, then unmount
  const [render, setRender] = useState(visible);

  useEffect(() => {
    if (visible) {
      setRender(true);
    } else {
      const timer = setTimeout(() => setRender(false), 180);
      return () => clearTimeout(timer);
    }
  }, [visible]);

  // Click outside to dismiss
  useEffect(() => {
    if (!visible) return;
    function handleClickOutside(e: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        onToggleRef.current();
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [visible]);

  return (
    <div ref={containerRef} className="relative shrink-0">
      <Button
        type="button"
        variant="ghost"
        size="lg"
        onClick={onToggle}
        className={cn(
          "shrink-0",
          visible
            ? "text-yellow-500"
            : "text-muted-foreground hover:text-yellow-500",
        )}
        aria-label={visible ? "Hide hint" : "Show hint"}
      >
        <Lightbulb className="size-4" />
      </Button>

      {render && (
        <div
          className={cn(
            "absolute bottom-full right-0 z-20 mb-3 w-[280px] max-w-[calc(100vw-3rem)] sm:w-[320px]",
            visible
              ? "animate-hint-bubble-in"
              : "animate-hint-bubble-out pointer-events-none",
          )}
        >
          {/* Bubble */}
          <div className="hint-bubble-bg rounded-xl border border-border px-4 py-3">
            <p className="text-sm leading-relaxed text-foreground/85">
              {hint}
            </p>
          </div>
          {/* Tail pointing down at lightbulb */}
          <div className="hint-bubble-bg absolute -bottom-[6px] right-[14px] size-3 rotate-45 border-b border-r border-border" />
        </div>
      )}
    </div>
  );
}

export const difficultyConfig: Record<
  string,
  { label: string; class: string }
> = {
  easy: {
    label: "Easy",
    class:
      "bg-green-500/10 text-green-700 dark:text-green-400 border-green-500/20",
  },
  medium: {
    label: "Medium",
    class:
      "bg-yellow-500/10 text-yellow-700 dark:text-yellow-400 border-yellow-500/20",
  },
  hard: {
    label: "Hard",
    class: "bg-red-500/10 text-red-700 dark:text-red-400 border-red-500/20",
  },
};

const languageNames: Record<string, string> = {
  python: "Python",
  javascript: "JavaScript",
  typescript: "TypeScript",
  rust: "Rust",
  go: "Go",
  java: "Java",
  c: "C",
  cpp: "C++",
  csharp: "C#",
  ruby: "Ruby",
  php: "PHP",
  swift: "Swift",
  kotlin: "Kotlin",
};

export function getLanguageLabel(lang: string): string {
  return languageNames[lang] ?? lang.charAt(0).toUpperCase() + lang.slice(1);
}

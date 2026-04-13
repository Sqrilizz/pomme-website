import removeMarkdown from "remove-markdown";

export function countWords(markdownOrMdx: string): number {
  const plainText = removeMarkdown(markdownOrMdx);
  const s = plainText.replace(/\s+/g, " ").trim();
  if (!s) return 0;
  return s.split(" ").filter(Boolean).length;
}

export function minutesToRead(wordCount: number, wpm = 225): number {
  return Math.max(1, Math.round(wordCount / wpm));
}

export function truncateForPreview(markdownOrMdx: string, maxLength: number): string {
  const plainText = removeMarkdown(markdownOrMdx);
  const s = plainText.replace(/\s+/g, " ").trim();
  if (!s) return "";
  if (s.length <= maxLength) return s;
  return s.slice(0, maxLength - 3) + "...";
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(0)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
}

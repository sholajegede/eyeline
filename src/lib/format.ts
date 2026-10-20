export function spokenWordCount(text: string): number {
  return text
    .replace(/\[[^\]]*\]/g, " ")
    .replace(/^\s*#{1,3}\s.*$/gm, " ")
    .split(/\s+/)
    .filter(Boolean).length;
}

export function readingMinutes(words: number, wordsPerMinute = 150): string {
  if (words === 0) return "0 min";
  const minutes = words / wordsPerMinute;
  return minutes < 1 ? `${Math.max(1, Math.round(minutes * 60))} sec` : `${minutes.toFixed(1)} min`;
}

export function relativeTime(unixSeconds: number, now = Date.now()): string {
  if (!unixSeconds) return "";
  const diff = Math.max(0, Math.floor(now / 1000) - unixSeconds);
  if (diff < 60) return "just now";
  if (diff < 3600) return `${Math.floor(diff / 60)} min ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} h ago`;
  if (diff < 86400 * 30) return `${Math.floor(diff / 86400)} d ago`;
  return new Date(unixSeconds * 1000).toLocaleDateString();
}

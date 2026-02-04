/**
 * Utilities for extracting excerpts and highlighting search matches
 */

export interface Excerpt {
  excerpt: string;
  matchStart: number;
  matchLength: number;
}

export interface TextSegment {
  text: string;
  isMatch: boolean;
}

/**
 * Escapes special regex characters in a string
 */
function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Extracts an excerpt from content centered around the first occurrence of the query.
 *
 * @param content - The full document content
 * @param query - The search query (phrase match)
 * @param contextChars - Number of characters to include before and after the match (default: 100)
 * @returns Excerpt with match position, or null if no match found
 */
export function extractExcerpt(
  content: string,
  query: string,
  contextChars: number = 100
): Excerpt | null {
  // Handle empty or whitespace-only queries
  if (!query || !query.trim()) {
    return null;
  }

  // Find first occurrence (case-insensitive)
  const lowerContent = content.toLowerCase();
  const lowerQuery = query.toLowerCase().trim();
  const matchIndex = lowerContent.indexOf(lowerQuery);

  if (matchIndex === -1) {
    return null;
  }

  // Calculate excerpt boundaries
  const matchEnd = matchIndex + lowerQuery.length;
  let excerptStart = Math.max(0, matchIndex - contextChars);
  let excerptEnd = Math.min(content.length, matchEnd + contextChars);

  // Adjust to word boundaries if possible (don't cut words in half)
  if (excerptStart > 0) {
    // Find the next space after excerptStart
    const nextSpace = content.indexOf(' ', excerptStart);
    if (nextSpace !== -1 && nextSpace < matchIndex) {
      excerptStart = nextSpace + 1;
    }
  }

  if (excerptEnd < content.length) {
    // Find the previous space before excerptEnd
    const prevSpace = content.lastIndexOf(' ', excerptEnd);
    if (prevSpace !== -1 && prevSpace > matchEnd) {
      excerptEnd = prevSpace;
    }
  }

  // Extract the excerpt
  let excerpt = content.substring(excerptStart, excerptEnd);

  // Add ellipsis
  if (excerptStart > 0) {
    excerpt = '...' + excerpt;
  }
  if (excerptEnd < content.length) {
    excerpt = excerpt + '...';
  }

  // Calculate match position within the excerpt
  const matchStartInExcerpt = excerptStart > 0
    ? matchIndex - excerptStart + 3  // +3 for "..."
    : matchIndex - excerptStart;

  return {
    excerpt,
    matchStart: matchStartInExcerpt,
    matchLength: lowerQuery.length,
  };
}

/**
 * Splits text into segments, marking which parts match the query.
 * Used for rendering with highlighted matches.
 *
 * @param text - The text to split (e.g., an excerpt)
 * @param query - The search query (phrase match)
 * @returns Array of text segments with match indicators
 */
export function highlightMatches(text: string, query: string): TextSegment[] {
  // Handle empty or whitespace-only queries
  if (!query || !query.trim()) {
    return [{ text, isMatch: false }];
  }

  const trimmedQuery = query.trim();
  const escapedQuery = escapeRegex(trimmedQuery);

  // Create regex for case-insensitive matching
  const regex = new RegExp(escapedQuery, 'gi');
  const segments: TextSegment[] = [];

  let lastIndex = 0;
  let match;

  while ((match = regex.exec(text)) !== null) {
    // Add non-matching text before this match
    if (match.index > lastIndex) {
      segments.push({
        text: text.substring(lastIndex, match.index),
        isMatch: false,
      });
    }

    // Add the matching text
    segments.push({
      text: match[0], // Use actual matched text to preserve case
      isMatch: true,
    });

    lastIndex = match.index + match[0].length;
  }

  // Add remaining non-matching text
  if (lastIndex < text.length) {
    segments.push({
      text: text.substring(lastIndex),
      isMatch: false,
    });
  }

  return segments;
}

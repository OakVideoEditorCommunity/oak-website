import { marked } from 'marked'
import DOMPurify from 'isomorphic-dompurify'

/**
 * Renders a Markdown string to sanitized HTML.
 *
 * The function runs on both server and client, so release notes can be
 * server-side rendered safely.
 */
export function renderMarkdown(md: string): string {
  const raw = marked.parse(md, { async: false }) as string
  return DOMPurify.sanitize(raw)
}

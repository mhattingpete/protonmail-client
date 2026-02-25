import { useState } from "react";
import type { EmailDetail } from "../../types/index.ts";
import { senderDisplay, dateFromUtc } from "../../types/index.ts";
import { sanitizeHtml } from "../../lib/sanitize.ts";
import { useComposeStore } from "../../stores/composeStore.ts";

interface ThreadViewProps {
  emails: EmailDetail[];
}

function formatFullDate(timestamp: number): string {
  return dateFromUtc(timestamp).toLocaleString([], {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function ThreadMessage({ email }: { email: EmailDetail }) {
  const [expanded, setExpanded] = useState(false);
  const openReply = useComposeStore((s) => s.openReply);

  const bodyHtml = email.html_body
    ? sanitizeHtml(email.html_body)
    : email.text_body
      ? `<pre style="white-space:pre-wrap;font-family:inherit">${email.text_body.replace(/</g, "&lt;")}</pre>`
      : "<p>No content</p>";

  const iframeSrcDoc = `
    <!DOCTYPE html>
    <html>
    <head>
      <style>
        body { color: #e2e8f0; background: transparent; font-family: system-ui, sans-serif; font-size: 14px; margin: 0; padding: 12px; }
        a { color: #a78bfa; }
        img { max-width: 100%; height: auto; }
      </style>
    </head>
    <body>${bodyHtml}</body>
    </html>
  `;

  const sender = senderDisplay(email);

  return (
    <div className="border-b border-white/5 last:border-0">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center gap-3 px-4 py-3 text-left hover:bg-white/5"
      >
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-purple-500/20 text-xs text-purple-300">
          {sender.charAt(0).toUpperCase()}
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="truncate text-sm font-medium text-gray-200">
              {sender}
            </span>
            <span className="text-xs text-gray-500">
              {formatFullDate(email.date_utc)}
            </span>
          </div>
          {!expanded && email.preview && (
            <div className="truncate text-xs text-gray-500">
              {email.preview.slice(0, 100)}
            </div>
          )}
        </div>
        <span className="text-xs text-gray-500">{expanded ? "\u25B2" : "\u25BC"}</span>
      </button>

      {expanded && (
        <div className="px-4 pb-4">
          <iframe
            srcDoc={iframeSrcDoc}
            sandbox="allow-same-origin"
            title="Email content"
            className="min-h-[200px] w-full rounded border-0 bg-transparent"
          />
          <div className="mt-2 flex gap-2">
            <button
              onClick={() =>
                openReply(
                  email.uid,
                  email.mailbox_id,
                  email.sender_email || "",
                  email.subject ?? "",
                )
              }
              className="rounded bg-purple-600/20 px-3 py-1 text-xs text-purple-300 hover:bg-purple-600/30"
            >
              Reply
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default function ThreadView({ emails }: ThreadViewProps) {
  if (emails.length === 0) {
    return (
      <div className="flex h-full items-center justify-center text-gray-500">
        No messages in thread
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      <div className="border-b border-white/10 px-4 py-3">
        <h2 className="text-lg font-semibold text-gray-100">
          {emails[0].subject ?? "(no subject)"}
        </h2>
        <span className="text-xs text-gray-500">
          {emails.length} message{emails.length > 1 ? "s" : ""}
        </span>
      </div>
      {emails.map((email) => (
        <ThreadMessage key={`${email.mailbox_id}:${email.uid}`} email={email} />
      ))}
    </div>
  );
}

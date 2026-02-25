import { useMailStore } from "../../stores/mailStore.ts";
import { useComposeStore } from "../../stores/composeStore.ts";
import { sanitizeHtml } from "../../lib/sanitize.ts";
import { senderDisplay, dateFromUtc } from "../../types/index.ts";

function formatFullDate(timestamp: number): string {
  return dateFromUtc(timestamp).toLocaleString([], {
    weekday: "short",
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function ActionButton({ label, onClick, icon, danger }: {
  label: string;
  onClick: () => void;
  icon: string;
  danger?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-[12px] font-medium transition-colors hover:bg-white/[0.06]"
      style={{ color: danger ? "#ec5858" : "var(--pm-text-secondary)" }}
      title={label}
    >
      <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round">
        <path d={icon} />
      </svg>
      {label}
    </button>
  );
}

export default function MailView() {
  const email = useMailStore((s) => s.selectedEmail);
  const deleteEmail = useMailStore((s) => s.deleteEmail);
  const archiveEmail = useMailStore((s) => s.archiveEmail);
  const markUnread = useMailStore((s) => s.markUnread);
  const openReply = useComposeStore((s) => s.openReply);
  const openForward = useComposeStore((s) => s.openForward);

  if (!email) {
    return (
      <div className="flex h-full items-center justify-center" style={{ color: "var(--pm-text-muted)" }}>
        <div className="flex flex-col items-center gap-2">
          <svg className="h-12 w-12 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1}>
            <path d="M22 6c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6zm-2 0l-8 5-8-5h16zm0 12H4V8l8 5 8-5v10z" />
          </svg>
          <span className="text-sm">Select an email to read</span>
        </div>
      </div>
    );
  }

  const bodyHtml = email.html_body
    ? sanitizeHtml(email.html_body)
    : email.text_body
      ? `<pre style="white-space:pre-wrap;font-family:inherit">${email.text_body.replace(/</g, "&lt;")}</pre>`
      : "<p style='color:#6b6583'>Loading email content...</p>";

  const iframeSrcDoc = `
    <!DOCTYPE html>
    <html>
    <head>
      <style>
        body { color: #e5e1f5; background: transparent; font-family: system-ui, -apple-system, sans-serif; font-size: 14px; line-height: 1.6; margin: 0; padding: 16px; }
        a { color: #6d4aff; }
        img { max-width: 100%; height: auto; }
        blockquote { border-left: 3px solid #3a3650; margin-left: 0; padding-left: 12px; color: #9b95b0; }
        pre { background: rgba(255,255,255,0.03); padding: 12px; border-radius: 6px; overflow-x: auto; }
        table { border-collapse: collapse; } td, th { padding: 4px 8px; }
      </style>
    </head>
    <body>${bodyHtml}</body>
    </html>
  `;

  const sender = senderDisplay(email);
  const replyTo = email.sender_email || "";

  return (
    <div className="flex h-full flex-col">
      {/* Email header */}
      <div className="border-b px-5 py-4" style={{ borderColor: "var(--pm-border)" }}>
        <h2 className="text-base font-semibold" style={{ color: "var(--pm-text-primary)" }}>
          {email.subject ?? "(no subject)"}
        </h2>

        <div className="mt-2 flex items-start gap-3">
          <div className="mt-0.5 flex-1">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium" style={{ color: "var(--pm-text-primary)" }}>
                {sender}
              </span>
              {email.sender_email && sender !== email.sender_email && (
                <span className="text-[12px]" style={{ color: "var(--pm-text-muted)" }}>
                  &lt;{email.sender_email}&gt;
                </span>
              )}
            </div>
            <div className="text-[12px]" style={{ color: "var(--pm-text-muted)" }}>
              {formatFullDate(email.date_utc)}
            </div>
          </div>
        </div>

        {/* Action buttons */}
        <div className="mt-3 flex gap-1">
          <ActionButton
            label="Reply"
            onClick={() => openReply(email.uid, email.mailbox_id, replyTo, email.subject ?? "")}
            icon="M3 10l9-7 9 7v11H3V10zM9 21v-6h6v6"
          />
          <ActionButton
            label="Forward"
            onClick={() => openForward(email.uid, email.mailbox_id, email.subject ?? "", email.text_body ?? "")}
            icon="M14 2l6 6-6 6V10H4V6h10V2z"
          />
          <ActionButton
            label="Archive"
            onClick={() => archiveEmail(email.uid, email.mailbox_id)}
            icon="M21 8v13H3V8M1 3h22v5H1zM10 12h4"
          />
          <ActionButton
            label="Unread"
            onClick={() => markUnread(email.uid, email.mailbox_id)}
            icon="M22 6c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6zm-2 0l-8 5-8-5h16z"
          />
          <ActionButton
            label="Delete"
            onClick={() => deleteEmail(email.uid, email.mailbox_id)}
            icon="M3 6h18M8 6V4h8v2M19 6v14H5V6M10 11v6M14 11v6"
            danger
          />
        </div>
      </div>

      {/* Email body */}
      <div className="flex-1 overflow-hidden">
        <iframe
          srcDoc={iframeSrcDoc}
          sandbox="allow-same-origin"
          title="Email content"
          className="h-full w-full border-0 bg-transparent"
        />
      </div>
    </div>
  );
}

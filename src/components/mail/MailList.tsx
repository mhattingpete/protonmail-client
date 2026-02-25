import { useMailStore } from "../../stores/mailStore.ts";
import { isRead, isStarred, senderDisplay, dateFromUtc } from "../../types/index.ts";

// Consistent avatar colors based on sender name
const AVATAR_COLORS = [
  "#6d4aff", "#db60d6", "#ec5858", "#f78400", "#1ea885",
  "#3cbcf0", "#415df0", "#a26ef0", "#c44150", "#e0a03d",
];

function avatarColor(name: string): string {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  return AVATAR_COLORS[Math.abs(hash) % AVATAR_COLORS.length];
}

function initials(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length >= 2) {
    return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  }
  return name.slice(0, 2).toUpperCase();
}

function formatDate(timestamp: number): string {
  const date = dateFromUtc(timestamp);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();
  if (isToday) {
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (date.toDateString() === yesterday.toDateString()) {
    return "Yesterday";
  }
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}

export default function MailList() {
  const emails = useMailStore((s) => s.emails);
  const selectedEmail = useMailStore((s) => s.selectedEmail);
  const selectEmail = useMailStore((s) => s.selectEmail);
  const toggleStar = useMailStore((s) => s.toggleStar);
  const loading = useMailStore((s) => s.loading);

  if (loading && emails.length === 0) {
    return (
      <div className="flex h-full items-center justify-center" style={{ color: "var(--pm-text-muted)" }}>
        <div className="flex flex-col items-center gap-3">
          <div
            className="h-6 w-6 animate-spin rounded-full border-2 border-t-transparent"
            style={{ borderColor: "var(--pm-accent)", borderTopColor: "transparent" }}
          />
          <span className="text-sm">Syncing emails...</span>
        </div>
      </div>
    );
  }

  if (emails.length === 0) {
    return (
      <div className="flex h-full items-center justify-center" style={{ color: "var(--pm-text-muted)" }}>
        <span className="text-sm">No emails</span>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      {emails.map((email) => {
        const isSelected =
          selectedEmail?.uid === email.uid &&
          selectedEmail?.mailbox_id === email.mailbox_id;
        const read = isRead(email);
        const starred = isStarred(email);
        const sender = senderDisplay(email);

        return (
          <div
            key={`${email.mailbox_id}:${email.uid}`}
            onClick={() => selectEmail(email)}
            className="flex cursor-pointer items-center gap-3 border-b px-3 py-2.5 transition-colors"
            style={{
              borderColor: "var(--pm-border)",
              background: isSelected
                ? "var(--pm-bg-selected)"
                : "transparent",
            }}
            onMouseOver={(e) => {
              if (!isSelected) e.currentTarget.style.background = "var(--pm-bg-hover)";
            }}
            onMouseOut={(e) => {
              if (!isSelected) e.currentTarget.style.background = "transparent";
            }}
          >
            {/* Avatar */}
            <div
              className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[11px] font-bold text-white"
              style={{ background: avatarColor(sender) }}
            >
              {initials(sender)}
            </div>

            {/* Star */}
            <button
              onClick={(e) => { e.stopPropagation(); toggleStar(email); }}
              className="shrink-0 transition-colors"
              style={{ color: starred ? "#f5c542" : "var(--pm-text-muted)" }}
            >
              <svg className="h-4 w-4" viewBox="0 0 24 24" fill={starred ? "currentColor" : "none"} stroke="currentColor" strokeWidth={starred ? 0 : 1.5}>
                <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
              </svg>
            </button>

            {/* Content */}
            <div className="min-w-0 flex-1">
              <div className="flex items-baseline gap-2">
                <span
                  className={`truncate text-[13px] ${!read ? "font-semibold" : "font-normal"}`}
                  style={{ color: !read ? "var(--pm-text-primary)" : "var(--pm-text-secondary)" }}
                >
                  {sender}
                </span>
                <span className="ml-auto shrink-0 text-[11px]" style={{ color: "var(--pm-text-muted)" }}>
                  {formatDate(email.date_utc)}
                </span>
              </div>
              <div className="flex items-center gap-1.5">
                <span
                  className={`truncate text-[13px] ${!read ? "font-medium" : "font-normal"}`}
                  style={{ color: !read ? "var(--pm-text-primary)" : "var(--pm-text-secondary)" }}
                >
                  {email.subject ?? "(no subject)"}
                </span>
                {email.has_attachments === 1 && (
                  <svg className="h-3.5 w-3.5 shrink-0" style={{ color: "var(--pm-text-muted)" }} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M21.44 11.05l-9.19 9.19a6 6 0 01-8.49-8.49l9.19-9.19a4 4 0 015.66 5.66l-9.2 9.19a2 2 0 01-2.83-2.83l8.49-8.48" />
                  </svg>
                )}
              </div>
              {email.preview && (
                <div className="truncate text-[12px]" style={{ color: "var(--pm-text-muted)" }}>
                  {email.preview.slice(0, 100)}
                </div>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

import { useMailStore } from "../../stores/mailStore.ts";
import { useComposeStore } from "../../stores/composeStore.ts";
import { useAccountStore } from "../../stores/accountStore.ts";

const FOLDER_ICONS: Record<string, string> = {
  inbox: "M20 6H4l8 5 8-5zM4 8v10h16V8l-8 5-8-5z",
  starred: "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z",
  sent: "M2.01 21L23 12 2.01 3 2 10l15 2-15 2z",
  drafts: "M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z",
  archive: "M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM12 17.5L6.5 12H10v-2h4v2h3.5L12 17.5z",
  spam: "M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z",
  trash: "M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z",
  "all mail": "M22 6c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6zm-2 0l-8 5-8-5h16zm0 12H4V8l8 5 8-5v10z",
};

const FOLDER_ORDER: Record<string, number> = {
  inbox: 0, starred: 1, sent: 2, drafts: 3, archive: 4, spam: 5, trash: 6, "all mail": 7,
};

function sortMailboxes<T extends { name: string }>(mailboxes: T[]): T[] {
  return [...mailboxes].sort((a, b) => {
    const oa = FOLDER_ORDER[a.name.toLowerCase()] ?? 99;
    const ob = FOLDER_ORDER[b.name.toLowerCase()] ?? 99;
    return oa - ob || a.name.localeCompare(b.name);
  });
}

function FolderIcon({ name }: { name: string }) {
  const path = FOLDER_ICONS[name.toLowerCase()];
  if (!path) {
    // Generic folder icon
    return (
      <svg className="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="currentColor">
        <path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" />
      </svg>
    );
  }
  return (
    <svg className="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="currentColor">
      <path d={path} />
    </svg>
  );
}

export default function Sidebar() {
  const mailboxes = useMailStore((s) => s.mailboxes);
  const selectedMailbox = useMailStore((s) => s.selectedMailbox);
  const selectMailbox = useMailStore((s) => s.selectMailbox);
  const openCompose = useComposeStore((s) => s.open);
  const accountEmail = useAccountStore((s) => s.email);

  return (
    <aside className="flex h-full w-[220px] flex-col" style={{ background: "var(--pm-bg-sidebar)" }}>
      <div className="p-3">
        <button
          onClick={openCompose}
          className="w-full rounded-lg py-2.5 text-sm font-semibold text-white transition-colors"
          style={{ background: "var(--pm-accent)" }}
          onMouseOver={(e) => (e.currentTarget.style.background = "var(--pm-accent-hover)")}
          onMouseOut={(e) => (e.currentTarget.style.background = "var(--pm-accent)")}
        >
          New message
        </button>
      </div>

      <nav className="flex-1 overflow-y-auto px-2 pb-2">
        {sortMailboxes(mailboxes).map((mailbox) => {
          const isSelected = selectedMailbox === mailbox.id;
          const unread = mailbox.unread_count ?? 0;

          return (
            <button
              key={mailbox.id}
              onClick={() => selectMailbox(mailbox.id)}
              className={`group flex w-full items-center gap-2.5 rounded-lg px-3 py-1.5 text-[13px] transition-colors ${
                isSelected
                  ? "text-white"
                  : "hover:bg-white/[0.04]"
              }`}
              style={isSelected ? { background: "var(--pm-bg-selected)", color: "var(--pm-accent)" } : { color: "var(--pm-text-secondary)" }}
            >
              <FolderIcon name={mailbox.name} />
              <span className="flex-1 text-left truncate">{mailbox.name}</span>
              {unread > 0 && (
                <span className="text-[11px] font-semibold" style={{ color: "var(--pm-accent)" }}>
                  {unread}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      {accountEmail && (
        <div className="border-t px-3 py-2 text-[11px]" style={{ borderColor: "var(--pm-border)", color: "var(--pm-text-muted)" }}>
          {accountEmail}
        </div>
      )}
    </aside>
  );
}

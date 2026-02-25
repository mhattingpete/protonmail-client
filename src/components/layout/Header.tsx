import { useState } from "react";
import { useMailStore } from "../../stores/mailStore.ts";

export default function Header() {
  const [query, setQuery] = useState("");
  const search = useMailStore((s) => s.search);
  const clearSearch = useMailStore((s) => s.clearSearch);
  const searchQuery = useMailStore((s) => s.searchQuery);
  const refresh = useMailStore((s) => s.refresh);
  const loading = useMailStore((s) => s.loading);
  const selectedMailbox = useMailStore((s) => s.selectedMailbox);
  const mailboxes = useMailStore((s) => s.mailboxes);

  const currentMailbox = mailboxes.find((m) => m.id === selectedMailbox);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      search(query.trim());
    } else {
      clearSearch();
    }
  };

  return (
    <header
      className="flex items-center gap-3 border-b px-4 py-2"
      style={{ background: "var(--pm-bg-content)", borderColor: "var(--pm-border)" }}
    >
      {/* Mailbox name */}
      <h2 className="text-sm font-semibold" style={{ color: "var(--pm-text-primary)" }}>
        {currentMailbox?.name ?? "Inbox"}
      </h2>

      {/* Sync button */}
      <button
        onClick={refresh}
        disabled={loading}
        className="rounded-md p-1.5 transition-colors hover:bg-white/[0.06]"
        style={{ color: "var(--pm-text-secondary)" }}
        title="Refresh"
      >
        <svg
          className={`h-4 w-4 ${loading ? "animate-spin" : ""}`}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M21 2v6h-6" />
          <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
          <path d="M3 22v-6h6" />
          <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
        </svg>
      </button>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Search bar */}
      <form onSubmit={handleSearch} className="flex items-center">
        <div className="relative">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search..."
            className="w-56 rounded-lg px-3 py-1.5 pl-8 text-[13px] outline-none transition-all focus:w-72 focus:ring-1"
            style={{
              background: "rgba(255,255,255,0.04)",
              color: "var(--pm-text-primary)",
              borderColor: "var(--pm-border)",
              border: "1px solid var(--pm-border)",
            }}
          />
          <svg
            className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2"
            style={{ color: "var(--pm-text-muted)" }}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
        </div>
        {searchQuery && (
          <button
            type="button"
            onClick={() => { setQuery(""); clearSearch(); }}
            className="ml-2 text-xs transition-colors hover:text-white"
            style={{ color: "var(--pm-text-muted)" }}
          >
            Clear
          </button>
        )}
      </form>
    </header>
  );
}

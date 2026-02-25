import { useState } from "react";
import { useAccountStore } from "./stores/accountStore.ts";
import PasswordPrompt from "./components/auth/PasswordPrompt.tsx";
import Layout from "./components/layout/Layout.tsx";
import MailList from "./components/mail/MailList.tsx";
import MailView from "./components/mail/MailView.tsx";
import ComposeModal from "./components/mail/ComposeModal.tsx";
import RuleBuilder from "./components/rules/RuleBuilder.tsx";

type View = "mail" | "rules";

export default function App() {
  const connected = useAccountStore((s) => s.connected);
  const [view, setView] = useState<View>("mail");

  if (!connected) {
    return <PasswordPrompt />;
  }

  return (
    <Layout>
      <div className="flex h-full">
        {view === "mail" ? (
          <>
            <div className="w-80 shrink-0 overflow-hidden" style={{ borderRight: "1px solid var(--pm-border)" }}>
              <MailList />
            </div>
            <div className="flex-1 overflow-hidden">
              <MailView />
            </div>
          </>
        ) : (
          <div className="flex-1 overflow-hidden">
            <RuleBuilder />
          </div>
        )}
      </div>

      <ComposeModal />

      {/* View toggle */}
      <div
        className="fixed bottom-4 right-4 flex gap-0.5 rounded-lg p-1"
        style={{ background: "var(--pm-bg-sidebar)", border: "1px solid var(--pm-border)" }}
      >
        {(["mail", "rules"] as const).map((v) => (
          <button
            key={v}
            onClick={() => setView(v)}
            className="rounded-md px-3 py-1.5 text-xs font-medium capitalize transition-colors"
            style={{
              background: view === v ? "var(--pm-bg-selected)" : "transparent",
              color: view === v ? "var(--pm-accent)" : "var(--pm-text-muted)",
            }}
          >
            {v}
          </button>
        ))}
      </div>
    </Layout>
  );
}

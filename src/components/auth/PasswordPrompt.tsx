import { useState } from "react";
import { useAccountStore } from "../../stores/accountStore.ts";
import { useMailStore } from "../../stores/mailStore.ts";

export default function PasswordPrompt() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const setBridgePassword = useAccountStore((s) => s.setBridgePassword);
  const loading = useAccountStore((s) => s.loading);
  const error = useAccountStore((s) => s.error);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim() || !password.trim()) return;
    await setBridgePassword(password, email);
    // If connection succeeded, trigger initial data fetch
    const { connected } = useAccountStore.getState();
    if (connected) {
      useMailStore.getState().fetchMailboxes();
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center" style={{ background: "var(--pm-bg-primary)" }}>
      <div className="w-full max-w-sm rounded-xl p-8 shadow-2xl" style={{ background: "var(--pm-bg-sidebar)", border: "1px solid var(--pm-border)" }}>
        <h1 className="mb-2 text-xl font-bold" style={{ color: "var(--pm-accent)" }}>ProtonMail Client</h1>
        <p className="mb-6 text-sm" style={{ color: "var(--pm-text-secondary)" }}>
          Connect to your ProtonMail account via Hydroxide bridge.
        </p>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="account-email" className="mb-1 block text-xs" style={{ color: "var(--pm-text-muted)" }}>
              ProtonMail Email
            </label>
            <input
              id="account-email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="you@proton.me"
              autoFocus
              className="w-full rounded-lg px-4 py-2.5 text-sm outline-none transition-colors"
              style={{
                background: "rgba(255,255,255,0.04)",
                color: "var(--pm-text-primary)",
                border: "1px solid var(--pm-border)",
              }}
            />
          </div>

          <div>
            <label htmlFor="bridge-password" className="mb-1 block text-xs" style={{ color: "var(--pm-text-muted)" }}>
              Bridge Password
            </label>
            <input
              id="bridge-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Bridge password"
              className="w-full rounded-lg px-4 py-2.5 text-sm outline-none transition-colors"
              style={{
                background: "rgba(255,255,255,0.04)",
                color: "var(--pm-text-primary)",
                border: "1px solid var(--pm-border)",
              }}
            />
          </div>

          {error && (
            <p className="text-xs text-red-400">{error}</p>
          )}

          <button
            type="submit"
            disabled={loading || !email.trim() || !password.trim()}
            className="w-full rounded-lg py-2.5 text-sm font-medium text-white transition-colors disabled:opacity-50"
            style={{ background: "var(--pm-accent)" }}
          >
            {loading ? "Connecting..." : "Connect"}
          </button>
        </form>
      </div>
    </div>
  );
}

import type { ReactNode } from "react";
import Sidebar from "./Sidebar.tsx";
import Header from "./Header.tsx";

interface LayoutProps {
  children: ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  return (
    <div className="flex h-screen text-gray-100" style={{ background: "var(--pm-bg-primary)" }}>
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Header />
        <main className="flex-1 overflow-hidden" style={{ background: "var(--pm-bg-content)" }}>
          {children}
        </main>
      </div>
    </div>
  );
}

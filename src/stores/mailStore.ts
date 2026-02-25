import { create } from "zustand";
import type { Email, EmailDetail } from "../types/index.ts";
import type { Mailbox } from "../types/index.ts";
import { isRead, isStarred } from "../types/index.ts";
import * as tauri from "../lib/tauri.ts";

/** Match emails by their composite key (uid + mailbox_id). */
function emailMatch(a: Email, b: Email): boolean {
  return a.uid === b.uid && a.mailbox_id === b.mailbox_id;
}

interface MailState {
  emails: Email[];
  selectedEmail: EmailDetail | null;
  mailboxes: Mailbox[];
  selectedMailbox: string; // Mailbox.id from backend
  searchQuery: string;
  loading: boolean;

  fetchMailboxes: () => Promise<void>;
  fetchEmails: (mailboxId: string) => Promise<void>;
  selectEmail: (email: Email) => Promise<void>;
  selectMailbox: (mailboxId: string) => void;
  toggleStar: (email: Email) => Promise<void>;
  markRead: (uid: number, mailboxId: string) => Promise<void>;
  markUnread: (uid: number, mailboxId: string) => Promise<void>;
  deleteEmail: (uid: number, mailboxId: string) => Promise<void>;
  moveEmail: (uid: number, fromMailbox: string, toMailbox: string) => Promise<void>;
  archiveEmail: (uid: number, mailboxId: string) => Promise<void>;
  search: (query: string) => Promise<void>;
  clearSearch: () => void;
  refresh: () => void;
}

export const useMailStore = create<MailState>((set, get) => ({
  emails: [],
  selectedEmail: null,
  mailboxes: [],
  selectedMailbox: "",
  searchQuery: "",
  loading: false,

  fetchMailboxes: async () => {
    try {
      const mailboxes = await tauri.listMailboxes();
      console.log("[mailStore] fetched mailboxes:", mailboxes);
      set({ mailboxes });
      // Auto-select INBOX if none selected, fall back to first mailbox
      if (!get().selectedMailbox && mailboxes.length > 0) {
        const inbox = mailboxes.find((m) => m.name === "INBOX");
        const target = inbox ? inbox.id : mailboxes[0].id;
        console.log("[mailStore] auto-selecting mailbox:", target);
        get().selectMailbox(target);
      }
    } catch (err) {
      console.error("[mailStore] fetchMailboxes error:", err);
    }
  },

  fetchEmails: async (mailboxId: string) => {
    if (!mailboxId) return;
    set({ loading: true });
    try {
      console.log("[mailStore] fetching emails for:", mailboxId);
      const emails = await tauri.listEmails(mailboxId);
      console.log("[mailStore] fetched", emails.length, "emails");
      set({ emails, loading: false });
    } catch (err) {
      console.error("[mailStore] fetchEmails error:", err);
      set({ emails: [], loading: false });
    }
  },

  selectEmail: async (email: Email) => {
    try {
      const detail = await tauri.getEmail(email.uid, email.mailbox_id);
      set({ selectedEmail: detail });
      if (!isRead(email)) {
        await tauri.markRead(email.uid, email.mailbox_id);
        // Update the flags in the list to include \\Seen
        set((state) => ({
          emails: state.emails.map((e) => {
            if (emailMatch(e, email)) {
              const flags = e.flags ? JSON.parse(e.flags) as string[] : [];
              if (!flags.includes("\\Seen")) flags.push("\\Seen");
              return { ...e, flags: JSON.stringify(flags) };
            }
            return e;
          }),
          selectedEmail: state.selectedEmail
            ? (() => {
                const flags = state.selectedEmail.flags
                  ? JSON.parse(state.selectedEmail.flags) as string[]
                  : [];
                if (!flags.includes("\\Seen")) flags.push("\\Seen");
                return { ...state.selectedEmail, flags: JSON.stringify(flags) };
              })()
            : null,
        }));
      }
    } catch {
      // If fetching detail fails, still show basic info
      set({ selectedEmail: { ...email, html_body: null, text_body: null } });
    }
  },

  selectMailbox: (mailboxId: string) => {
    set({ selectedMailbox: mailboxId, selectedEmail: null, searchQuery: "" });
    get().fetchEmails(mailboxId);
  },

  toggleStar: async (email: Email) => {
    if (isStarred(email)) {
      await tauri.unstarEmail(email.uid, email.mailbox_id);
    } else {
      await tauri.starEmail(email.uid, email.mailbox_id);
    }
    set((state) => ({
      emails: state.emails.map((e) => {
        if (!emailMatch(e, email)) return e;
        const flags = e.flags ? JSON.parse(e.flags) as string[] : [];
        if (isStarred(e)) {
          return { ...e, flags: JSON.stringify(flags.filter((f: string) => f !== "\\Flagged")) };
        }
        return { ...e, flags: JSON.stringify([...flags, "\\Flagged"]) };
      }),
    }));
  },

  markRead: async (uid: number, mailboxId: string) => {
    await tauri.markRead(uid, mailboxId);
    set((state) => ({
      emails: state.emails.map((e) => {
        if (e.uid !== uid || e.mailbox_id !== mailboxId) return e;
        const flags = e.flags ? JSON.parse(e.flags) as string[] : [];
        if (!flags.includes("\\Seen")) flags.push("\\Seen");
        return { ...e, flags: JSON.stringify(flags) };
      }),
    }));
  },

  markUnread: async (uid: number, mailboxId: string) => {
    await tauri.markUnread(uid, mailboxId);
    set((state) => ({
      emails: state.emails.map((e) => {
        if (e.uid !== uid || e.mailbox_id !== mailboxId) return e;
        const flags = e.flags ? JSON.parse(e.flags) as string[] : [];
        return { ...e, flags: JSON.stringify(flags.filter((f: string) => f !== "\\Seen")) };
      }),
    }));
  },

  deleteEmail: async (uid: number, mailboxId: string) => {
    await tauri.deleteEmail(uid, mailboxId);
    set((state) => ({
      emails: state.emails.filter((e) => !(e.uid === uid && e.mailbox_id === mailboxId)),
      selectedEmail:
        state.selectedEmail?.uid === uid && state.selectedEmail?.mailbox_id === mailboxId
          ? null
          : state.selectedEmail,
    }));
  },

  moveEmail: async (uid: number, fromMailbox: string, toMailbox: string) => {
    await tauri.moveEmail(uid, fromMailbox, toMailbox);
    set((state) => ({
      emails: state.emails.filter((e) => !(e.uid === uid && e.mailbox_id === fromMailbox)),
      selectedEmail:
        state.selectedEmail?.uid === uid && state.selectedEmail?.mailbox_id === fromMailbox
          ? null
          : state.selectedEmail,
    }));
  },

  archiveEmail: async (uid: number, mailboxId: string) => {
    await tauri.archiveEmail(uid, mailboxId);
    set((state) => ({
      emails: state.emails.filter((e) => !(e.uid === uid && e.mailbox_id === mailboxId)),
      selectedEmail:
        state.selectedEmail?.uid === uid && state.selectedEmail?.mailbox_id === mailboxId
          ? null
          : state.selectedEmail,
    }));
  },

  search: async (query: string) => {
    set({ searchQuery: query, loading: true });
    try {
      const emails = await tauri.searchEmails(query);
      set({ emails, loading: false });
    } catch {
      set({ emails: [], loading: false });
    }
  },

  clearSearch: () => {
    set({ searchQuery: "" });
    get().fetchEmails(get().selectedMailbox);
  },

  refresh: () => {
    const { selectedMailbox, fetchEmails, fetchMailboxes } = get();
    fetchMailboxes();
    if (selectedMailbox) {
      fetchEmails(selectedMailbox);
    }
  },
}));

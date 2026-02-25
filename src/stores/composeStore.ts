import { create } from "zustand";
import * as tauri from "../lib/tauri.ts";

interface ComposeState {
  isOpen: boolean;
  to: string;
  subject: string;
  body: string;
  replyUid: number | null;
  replyMailboxId: string | null;
  forwardUid: number | null;
  forwardMailboxId: string | null;

  open: () => void;
  openReply: (uid: number, mailboxId: string, to: string, subject: string) => void;
  openForward: (uid: number, mailboxId: string, subject: string, bodyText: string) => void;
  close: () => void;
  setTo: (to: string) => void;
  setSubject: (subject: string) => void;
  setBody: (body: string) => void;
  send: () => Promise<void>;
}

export const useComposeStore = create<ComposeState>((set, get) => ({
  isOpen: false,
  to: "",
  subject: "",
  body: "",
  replyUid: null,
  replyMailboxId: null,
  forwardUid: null,
  forwardMailboxId: null,

  open: () => {
    set({
      isOpen: true,
      to: "",
      subject: "",
      body: "",
      replyUid: null,
      replyMailboxId: null,
      forwardUid: null,
      forwardMailboxId: null,
    });
  },

  openReply: (uid: number, mailboxId: string, to: string, subject: string) => {
    set({
      isOpen: true,
      to,
      subject: subject.startsWith("Re:") ? subject : `Re: ${subject}`,
      body: "",
      replyUid: uid,
      replyMailboxId: mailboxId,
      forwardUid: null,
      forwardMailboxId: null,
    });
  },

  openForward: (uid: number, mailboxId: string, subject: string, bodyText: string) => {
    set({
      isOpen: true,
      to: "",
      subject: subject.startsWith("Fwd:") ? subject : `Fwd: ${subject}`,
      body: `\n\n---------- Forwarded message ----------\n${bodyText}`,
      replyUid: null,
      replyMailboxId: null,
      forwardUid: uid,
      forwardMailboxId: mailboxId,
    });
  },

  close: () => {
    set({ isOpen: false });
  },

  setTo: (to) => set({ to }),
  setSubject: (subject) => set({ subject }),
  setBody: (body) => set({ body }),

  send: async () => {
    const { to, subject, body, replyUid, replyMailboxId, forwardUid, forwardMailboxId } = get();
    if (replyUid !== null && replyMailboxId) {
      await tauri.replyEmail(replyUid, replyMailboxId, body);
    } else if (forwardUid !== null && forwardMailboxId) {
      await tauri.forwardEmail(forwardUid, forwardMailboxId, to, body);
    } else {
      await tauri.sendEmail(to, subject, body);
    }
    set({ isOpen: false });
  },
}));

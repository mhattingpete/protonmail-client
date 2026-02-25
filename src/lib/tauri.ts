import { invoke } from "@tauri-apps/api/core";
import type {
  Email,
  EmailDetail,
  Mailbox,
  AutomationRule,
  Classification,
} from "../types/index.ts";

// Email operations
export const listEmails = (mailboxId: string, limit?: number, offset?: number) =>
  invoke<Email[]>("list_emails", { mailboxId, limit, offset });

export const getEmail = (uid: number, mailboxId: string) =>
  invoke<EmailDetail>("get_email", { uid, mailboxId });

export const markRead = (uid: number, mailboxId: string) =>
  invoke<void>("mark_read", { uid, mailboxId });

export const markUnread = (uid: number, mailboxId: string) =>
  invoke<void>("mark_unread", { uid, mailboxId });

export const starEmail = (uid: number, mailboxId: string) =>
  invoke<void>("star_email", { uid, mailboxId });

export const unstarEmail = (uid: number, mailboxId: string) =>
  invoke<void>("unstar_email", { uid, mailboxId });

export const deleteEmail = (uid: number, mailboxId: string) =>
  invoke<void>("delete_email", { uid, mailboxId });

export const moveEmail = (uid: number, fromMailbox: string, toMailbox: string) =>
  invoke<void>("move_email", { uid, fromMailbox, toMailbox });

export const archiveEmail = (uid: number, mailboxId: string) =>
  invoke<void>("archive_email", { uid, mailboxId });

// Compose operations
export const sendEmail = (to: string, subject: string, body: string, inReplyTo?: string) =>
  invoke<void>("send_email", { to, subject, body, inReplyTo: inReplyTo ?? null });

export const replyEmail = (uid: number, mailboxId: string, body: string) =>
  invoke<void>("reply_email", { uid, mailboxId, body });

export const forwardEmail = (uid: number, mailboxId: string, to: string, body: string) =>
  invoke<void>("forward_email", { uid, mailboxId, to, body });

// Mailbox operations
export const listMailboxes = () =>
  invoke<Mailbox[]>("list_mailboxes");

export const createFolder = (name: string) =>
  invoke<void>("create_folder", { name });

// Search
export const searchEmails = (query: string, limit?: number) =>
  invoke<Email[]>("search_emails", { query, limit: limit ?? null });

// Automation rules
export const listRules = () =>
  invoke<AutomationRule[]>("list_rules");

export const createRule = (name: string, conditions: string, actions: string) =>
  invoke<AutomationRule>("create_rule", { name, conditions, actions });

export const updateRule = (id: string, name: string, enabled: boolean, conditions: string, actions: string) =>
  invoke<AutomationRule>("update_rule", { id, name, enabled, conditions, actions });

export const deleteRule = (id: string) =>
  invoke<void>("delete_rule", { id });

// Classification
export const classifyEmail = (uid: number, mailboxId: string) =>
  invoke<Classification>("classify_email", { uid, mailboxId });

// Bridge / Connection
export const setBridgePassword = (password: string, email: string) =>
  invoke<void>("set_bridge_password", { password, email });

export const getConnectionStatus = () =>
  invoke<boolean>("get_connection_status");

export const getAccountEmail = () =>
  invoke<string>("get_account_email");

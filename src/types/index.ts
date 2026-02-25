/** Raw Email as returned by the Rust backend (serde-serialized). */
export interface Email {
  uid: number;
  mailbox_id: string;
  message_id: string | null;
  subject: string | null;
  sender_name: string | null;
  sender_email: string | null;
  recipients: string | null; // JSON string of array
  date_utc: number; // Unix timestamp (seconds)
  flags: string | null; // JSON string: ["\\Seen", "\\Flagged"]
  has_attachments: number; // 0 or 1
  preview: string | null;
  classification: string | null;
  classification_confidence: number | null;
}

/** EmailDetail returned by get_email — Email fields plus body content. */
export interface EmailDetail extends Email {
  html_body: string | null;
  text_body: string | null;
}

export interface Mailbox {
  id: string;
  account_id: string;
  name: string;
  path: string;
  uid_validity: number | null;
  last_synced_uid: number | null;
  unread_count: number | null;
}

export interface Classification {
  label: string;
  confidence: number;
  tier: string;
}

export type RuleConditionField = "from" | "to" | "subject" | "body";
export type RuleConditionOperator = "contains" | "equals" | "starts_with" | "ends_with" | "regex";

export interface RuleCondition {
  field: RuleConditionField;
  operator: RuleConditionOperator;
  value: string;
}

export type RuleActionType = "move" | "label" | "classify" | "star" | "mark_read" | "delete";

export interface RuleAction {
  action_type: RuleActionType;
  value: string;
}

export interface AutomationRule {
  id: string;
  name: string;
  enabled: boolean;
  conditions: RuleCondition[];
  actions: RuleAction[];
}

// ---- Helpers for working with Email fields ----

/** Parse the flags JSON string into an array. */
export function parseFlags(flags: string | null): string[] {
  if (!flags) return [];
  try {
    return JSON.parse(flags);
  } catch {
    return [];
  }
}

/** Check whether the email has been read (\\Seen flag). */
export function isRead(email: Email): boolean {
  return parseFlags(email.flags).includes("\\Seen");
}

/** Check whether the email is starred (\\Flagged flag). */
export function isStarred(email: Email): boolean {
  return parseFlags(email.flags).includes("\\Flagged");
}

/** Get a display name for the sender: prefer sender_name, fall back to sender_email. */
export function senderDisplay(email: Email): string {
  return email.sender_name || email.sender_email || "Unknown";
}

/** Format a unix timestamp (seconds) to a Date. */
export function dateFromUtc(timestamp: number): Date {
  return new Date(timestamp * 1000);
}

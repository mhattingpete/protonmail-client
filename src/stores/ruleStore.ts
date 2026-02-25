import { create } from "zustand";
import type { AutomationRule, RuleCondition, RuleAction } from "../types/index.ts";
import * as tauri from "../lib/tauri.ts";

interface RuleState {
  rules: AutomationRule[];
  loading: boolean;

  fetchRules: () => Promise<void>;
  createRule: (name: string, conditions: RuleCondition[], actions: RuleAction[]) => Promise<void>;
  updateRule: (rule: AutomationRule) => Promise<void>;
  deleteRule: (id: string) => Promise<void>;
}

export const useRuleStore = create<RuleState>((set) => ({
  rules: [],
  loading: false,

  fetchRules: async () => {
    set({ loading: true });
    const rules = await tauri.listRules();
    set({ rules, loading: false });
  },

  createRule: async (name, conditions, actions) => {
    const rule = await tauri.createRule(
      name,
      JSON.stringify(conditions),
      JSON.stringify(actions),
    );
    set((state) => ({ rules: [...state.rules, rule] }));
  },

  updateRule: async (rule) => {
    const updated = await tauri.updateRule(
      rule.id,
      rule.name,
      rule.enabled,
      JSON.stringify(rule.conditions),
      JSON.stringify(rule.actions),
    );
    set((state) => ({
      rules: state.rules.map((r) => (r.id === rule.id ? updated : r)),
    }));
  },

  deleteRule: async (id) => {
    await tauri.deleteRule(id);
    set((state) => ({
      rules: state.rules.filter((r) => r.id !== id),
    }));
  },
}));

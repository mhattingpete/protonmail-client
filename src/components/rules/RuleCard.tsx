import type { AutomationRule } from "../../types/index.ts";
import { useRuleStore } from "../../stores/ruleStore.ts";

interface RuleCardProps {
  rule: AutomationRule;
  onEdit: (rule: AutomationRule) => void;
}

export default function RuleCard({ rule, onEdit }: RuleCardProps) {
  const deleteRule = useRuleStore((s) => s.deleteRule);
  const updateRule = useRuleStore((s) => s.updateRule);

  const toggleEnabled = () => {
    updateRule({ ...rule, enabled: !rule.enabled });
  };

  return (
    <div className={`rounded-lg border border-white/10 p-4 ${rule.enabled ? "bg-white/[0.02]" : "bg-white/[0.01] opacity-60"}`}>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <button
            onClick={toggleEnabled}
            className={`h-5 w-9 rounded-full transition-colors ${
              rule.enabled ? "bg-purple-500" : "bg-gray-600"
            }`}
          >
            <div
              className={`h-4 w-4 rounded-full bg-white transition-transform ${
                rule.enabled ? "translate-x-[18px]" : "translate-x-0.5"
              }`}
            />
          </button>
          <h3 className="text-sm font-medium text-gray-200">{rule.name}</h3>
        </div>

        <div className="flex gap-1">
          <button
            onClick={() => onEdit(rule)}
            className="rounded px-2 py-1 text-xs text-gray-400 hover:bg-white/5 hover:text-gray-200"
          >
            Edit
          </button>
          <button
            onClick={() => deleteRule(rule.id)}
            className="rounded px-2 py-1 text-xs text-red-400 hover:bg-red-500/10"
          >
            Delete
          </button>
        </div>
      </div>

      <div className="mt-2 text-xs text-gray-500">
        {rule.conditions.length} condition{rule.conditions.length !== 1 ? "s" : ""}
        {" · "}
        {rule.actions.length} action{rule.actions.length !== 1 ? "s" : ""}
      </div>
    </div>
  );
}

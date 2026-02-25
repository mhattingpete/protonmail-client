import { useState, useEffect } from "react";
import type {
  AutomationRule,
  RuleCondition,
  RuleAction,
  RuleActionType,
} from "../../types/index.ts";
import { useRuleStore } from "../../stores/ruleStore.ts";
import ConditionRow from "./ConditionRow.tsx";
import RuleCard from "./RuleCard.tsx";

const ACTION_TYPES: { value: RuleActionType; label: string }[] = [
  { value: "move", label: "Move to folder" },
  { value: "label", label: "Apply label" },
  { value: "classify", label: "Classify as" },
  { value: "star", label: "Star" },
  { value: "mark_read", label: "Mark as read" },
  { value: "delete", label: "Delete" },
];

const emptyCondition: RuleCondition = { field: "from", operator: "contains", value: "" };
const emptyAction: RuleAction = { action_type: "move", value: "" };

export default function RuleBuilder() {
  const rules = useRuleStore((s) => s.rules);
  const fetchRules = useRuleStore((s) => s.fetchRules);
  const createRule = useRuleStore((s) => s.createRule);
  const updateRule = useRuleStore((s) => s.updateRule);

  const [editing, setEditing] = useState<AutomationRule | null>(null);
  const [name, setName] = useState("");
  const [conditions, setConditions] = useState<RuleCondition[]>([{ ...emptyCondition }]);
  const [actions, setActions] = useState<RuleAction[]>([{ ...emptyAction }]);
  const [showForm, setShowForm] = useState(false);

  useEffect(() => {
    fetchRules();
  }, [fetchRules]);

  const resetForm = () => {
    setEditing(null);
    setName("");
    setConditions([{ ...emptyCondition }]);
    setActions([{ ...emptyAction }]);
    setShowForm(false);
  };

  const handleEdit = (rule: AutomationRule) => {
    setEditing(rule);
    setName(rule.name);
    setConditions([...rule.conditions]);
    setActions([...rule.actions]);
    setShowForm(true);
  };

  const handleSave = async () => {
    if (!name.trim()) return;
    if (editing) {
      await updateRule({ ...editing, name, conditions, actions });
    } else {
      await createRule(name, conditions, actions);
    }
    resetForm();
  };

  const updateCondition = (index: number, condition: RuleCondition) => {
    setConditions(conditions.map((c, i) => (i === index ? condition : c)));
  };

  const removeCondition = (index: number) => {
    if (conditions.length > 1) {
      setConditions(conditions.filter((_, i) => i !== index));
    }
  };

  const updateAction = (index: number, action: RuleAction) => {
    setActions(actions.map((a, i) => (i === index ? action : a)));
  };

  const removeAction = (index: number) => {
    if (actions.length > 1) {
      setActions(actions.filter((_, i) => i !== index));
    }
  };

  return (
    <div className="h-full overflow-y-auto p-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-gray-100">Automation Rules</h2>
        {!showForm && (
          <button
            onClick={() => setShowForm(true)}
            className="rounded-lg bg-purple-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-purple-500"
          >
            New Rule
          </button>
        )}
      </div>

      {showForm && (
        <div className="mt-4 space-y-4 rounded-lg border border-white/10 bg-white/[0.02] p-4">
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Rule name"
            className="w-full rounded bg-white/5 px-3 py-2 text-sm text-gray-200 outline-none ring-1 ring-white/10 focus:ring-purple-500/50"
          />

          <div>
            <h4 className="mb-2 text-xs font-medium uppercase text-gray-500">
              Conditions
            </h4>
            <div className="space-y-2">
              {conditions.map((condition, i) => (
                <ConditionRow
                  key={i}
                  condition={condition}
                  onChange={(c) => updateCondition(i, c)}
                  onRemove={() => removeCondition(i)}
                />
              ))}
            </div>
            <button
              onClick={() => setConditions([...conditions, { ...emptyCondition }])}
              className="mt-2 text-xs text-purple-400 hover:text-purple-300"
            >
              + Add condition
            </button>
          </div>

          <div>
            <h4 className="mb-2 text-xs font-medium uppercase text-gray-500">
              Actions
            </h4>
            <div className="space-y-2">
              {actions.map((action, i) => (
                <div key={i} className="flex items-center gap-2">
                  <select
                    value={action.action_type}
                    onChange={(e) =>
                      updateAction(i, {
                        ...action,
                        action_type: e.target.value as RuleActionType,
                      })
                    }
                    className="rounded bg-white/5 px-2 py-1.5 text-sm text-gray-200 outline-none ring-1 ring-white/10"
                  >
                    {ACTION_TYPES.map((t) => (
                      <option key={t.value} value={t.value}>
                        {t.label}
                      </option>
                    ))}
                  </select>
                  <input
                    type="text"
                    value={action.value}
                    onChange={(e) =>
                      updateAction(i, { ...action, value: e.target.value })
                    }
                    placeholder="Value"
                    className="flex-1 rounded bg-white/5 px-2 py-1.5 text-sm text-gray-200 outline-none ring-1 ring-white/10"
                  />
                  <button
                    onClick={() => removeAction(i)}
                    className="rounded px-2 py-1 text-xs text-red-400 hover:bg-red-500/10"
                  >
                    ✕
                  </button>
                </div>
              ))}
            </div>
            <button
              onClick={() => setActions([...actions, { ...emptyAction }])}
              className="mt-2 text-xs text-purple-400 hover:text-purple-300"
            >
              + Add action
            </button>
          </div>

          <div className="flex gap-2">
            <button
              onClick={handleSave}
              className="rounded-lg bg-purple-600 px-4 py-2 text-sm font-medium text-white hover:bg-purple-500"
            >
              {editing ? "Update Rule" : "Create Rule"}
            </button>
            <button
              onClick={resetForm}
              className="rounded-lg bg-white/5 px-4 py-2 text-sm text-gray-300 hover:bg-white/10"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      <div className="mt-4 space-y-2">
        {rules.map((rule) => (
          <RuleCard key={rule.id} rule={rule} onEdit={handleEdit} />
        ))}
        {rules.length === 0 && !showForm && (
          <p className="text-sm text-gray-500">No automation rules yet.</p>
        )}
      </div>
    </div>
  );
}

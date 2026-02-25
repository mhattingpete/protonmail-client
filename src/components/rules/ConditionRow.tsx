import type { RuleCondition, RuleConditionField, RuleConditionOperator } from "../../types/index.ts";

const FIELDS: { value: RuleConditionField; label: string }[] = [
  { value: "from", label: "From" },
  { value: "to", label: "To" },
  { value: "subject", label: "Subject" },
  { value: "body", label: "Body" },
];

const OPERATORS: { value: RuleConditionOperator; label: string }[] = [
  { value: "contains", label: "Contains" },
  { value: "equals", label: "Equals" },
  { value: "starts_with", label: "Starts with" },
  { value: "ends_with", label: "Ends with" },
  { value: "regex", label: "Regex" },
];

interface ConditionRowProps {
  condition: RuleCondition;
  onChange: (condition: RuleCondition) => void;
  onRemove: () => void;
}

export default function ConditionRow({ condition, onChange, onRemove }: ConditionRowProps) {
  return (
    <div className="flex items-center gap-2">
      <select
        value={condition.field}
        onChange={(e) => onChange({ ...condition, field: e.target.value as RuleConditionField })}
        className="rounded bg-white/5 px-2 py-1.5 text-sm text-gray-200 outline-none ring-1 ring-white/10"
      >
        {FIELDS.map((f) => (
          <option key={f.value} value={f.value}>
            {f.label}
          </option>
        ))}
      </select>

      <select
        value={condition.operator}
        onChange={(e) => onChange({ ...condition, operator: e.target.value as RuleConditionOperator })}
        className="rounded bg-white/5 px-2 py-1.5 text-sm text-gray-200 outline-none ring-1 ring-white/10"
      >
        {OPERATORS.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>

      <input
        type="text"
        value={condition.value}
        onChange={(e) => onChange({ ...condition, value: e.target.value })}
        placeholder="Value"
        className="flex-1 rounded bg-white/5 px-2 py-1.5 text-sm text-gray-200 outline-none ring-1 ring-white/10"
      />

      <button
        onClick={onRemove}
        className="rounded px-2 py-1 text-xs text-red-400 hover:bg-red-500/10"
      >
        ✕
      </button>
    </div>
  );
}

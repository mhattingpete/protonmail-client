const BADGE_COLORS: Record<string, string> = {
  primary: "bg-blue-500/20 text-blue-400",
  urgent: "bg-red-500/20 text-red-400",
  transactions: "bg-green-500/20 text-green-400",
  updates: "bg-yellow-500/20 text-yellow-400",
  newsletters: "bg-gray-500/20 text-gray-400",
  social: "bg-purple-500/20 text-purple-400",
};

interface CategoryBadgeProps {
  classification: string;
}

export default function CategoryBadge({ classification }: CategoryBadgeProps) {
  const colorClass = BADGE_COLORS[classification.toLowerCase()] ?? "bg-gray-500/20 text-gray-400";
  const label = classification.charAt(0).toUpperCase() + classification.slice(1);

  return (
    <span
      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${colorClass}`}
    >
      {label}
    </span>
  );
}

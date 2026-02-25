import { useEffect, useState } from "react";
import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { useComposeStore } from "../../stores/composeStore.ts";

export default function ComposeModal() {
  const isOpen = useComposeStore((s) => s.isOpen);
  const to = useComposeStore((s) => s.to);
  const subject = useComposeStore((s) => s.subject);
  const setTo = useComposeStore((s) => s.setTo);
  const setSubject = useComposeStore((s) => s.setSubject);
  const setBody = useComposeStore((s) => s.setBody);
  const send = useComposeStore((s) => s.send);
  const close = useComposeStore((s) => s.close);
  const [sending, setSending] = useState(false);

  const editor = useEditor({
    extensions: [StarterKit],
    content: "",
    editorProps: {
      attributes: {
        class:
          "prose prose-invert prose-sm max-w-none min-h-[200px] px-3 py-2 outline-none",
      },
    },
    onUpdate: ({ editor: ed }) => {
      setBody(ed.getHTML());
    },
  });

  useEffect(() => {
    if (isOpen && editor) {
      editor.commands.clearContent();
    }
  }, [isOpen, editor]);

  if (!isOpen) return null;

  const handleSend = async () => {
    setSending(true);
    try {
      await send();
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center p-4 sm:items-center">
      <div className="fixed inset-0 bg-black/50" onClick={close} />
      <div className="relative w-full max-w-2xl rounded-xl bg-[#1a1a2e] shadow-2xl ring-1 ring-white/10">
        <div className="flex items-center justify-between border-b border-white/10 px-4 py-3">
          <h3 className="text-sm font-semibold text-gray-200">New Message</h3>
          <button
            onClick={close}
            className="text-gray-400 hover:text-gray-200"
          >
            {"\u2715"}
          </button>
        </div>

        <div className="space-y-0">
          <div className="flex items-center border-b border-white/5 px-4 py-2">
            <label className="w-12 text-xs text-gray-500">To</label>
            <input
              type="text"
              value={to}
              onChange={(e) => setTo(e.target.value)}
              className="flex-1 bg-transparent text-sm text-gray-200 outline-none"
              placeholder="recipient@example.com"
            />
          </div>

          <div className="flex items-center border-b border-white/5 px-4 py-2">
            <label className="w-12 text-xs text-gray-500">Subject</label>
            <input
              type="text"
              value={subject}
              onChange={(e) => setSubject(e.target.value)}
              className="flex-1 bg-transparent text-sm text-gray-200 outline-none"
              placeholder="Subject"
            />
          </div>

          <div className="min-h-[200px] border-b border-white/5 text-sm text-gray-200">
            <EditorContent editor={editor} />
          </div>
        </div>

        <div className="flex items-center justify-between px-4 py-3">
          <div className="flex gap-2">
            {editor && (
              <>
                <button
                  onClick={() => editor.chain().focus().toggleBold().run()}
                  className={`rounded px-2 py-1 text-xs ${
                    editor.isActive("bold")
                      ? "bg-purple-500/30 text-purple-300"
                      : "text-gray-400 hover:bg-white/5"
                  }`}
                >
                  B
                </button>
                <button
                  onClick={() => editor.chain().focus().toggleItalic().run()}
                  className={`rounded px-2 py-1 text-xs italic ${
                    editor.isActive("italic")
                      ? "bg-purple-500/30 text-purple-300"
                      : "text-gray-400 hover:bg-white/5"
                  }`}
                >
                  I
                </button>
              </>
            )}
          </div>
          <button
            onClick={handleSend}
            disabled={sending || !to.trim()}
            className="rounded-lg bg-purple-600 px-6 py-2 text-sm font-medium text-white transition-colors hover:bg-purple-500 disabled:opacity-50"
          >
            {sending ? "Sending..." : "Send"}
          </button>
        </div>
      </div>
    </div>
  );
}

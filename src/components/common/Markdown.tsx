import React, { useRef } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import rehypeHighlight from "rehype-highlight";
import { Copy } from "lucide-react";
import { copyText } from "@/lib/utils";
import { useI18n } from "@/lib/i18n";

function Pre({ children }: { children?: React.ReactNode }) {
  const ref = useRef<HTMLPreElement>(null);
  const { t } = useI18n();
  return (
    <div className="code-block">
      <button
        type="button"
        className="code-copy"
        onClick={() => copyText(ref.current?.innerText ?? "")}
        title={t("message.copy")}
      >
        <Copy size={13} />
      </button>
      <pre ref={ref}>{children}</pre>
    </div>
  );
}

export function Markdown({ children }: { children: string }) {
  return (
    <div className="markdown">
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkMath]}
        rehypePlugins={[rehypeKatex, [rehypeHighlight, { detect: true, ignoreMissing: true }]]}
        components={{
          pre: Pre,
          a: ({ href, children: linkChildren }) => (
            <a
              href={href}
              target="_blank"
              rel="noreferrer"
              onClick={(e) => {
                e.preventDefault();
                if (href) window.open(href, "_blank");
              }}
            >
              {linkChildren}
            </a>
          ),
          table: ({ children: tableChildren }) => (
            <div className="table-wrap">
              <table>{tableChildren}</table>
            </div>
          ),
        }}
      >
        {children}
      </ReactMarkdown>
    </div>
  );
}

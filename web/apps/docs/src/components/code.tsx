'use client';

import { useState } from 'react';

// Token types for YAML highlighting
type TokenType = 'key' | 'colon' | 'string' | 'number' | 'boolean' | 'null' | 'comment' | 'dash' | 'bracket' | 'special' | 'text';

interface Token {
  type: TokenType;
  value: string;
}

const TOKEN_COLORS: Record<TokenType, string> = {
  key: 'text-[#7dd3fc]',        // sky-300 - keys
  colon: 'text-zinc-500',       // dim
  string: 'text-[#86efac]',     // green-300 - strings
  number: 'text-[#fcd34d]',     // amber-300 - numbers
  boolean: 'text-[#fcd34d]',    // amber-300 - booleans
  null: 'text-[#fcd34d]',       // amber-300 - null
  comment: 'text-zinc-600 italic', // very dim
  dash: 'text-zinc-500',        // dim
  bracket: 'text-zinc-500',     // dim
  special: 'text-[#c4b5fd]',    // violet-300 - wildcards
  text: 'text-zinc-400',        // default text - subtle
};

function tokenizeLine(line: string): Token[] {
  const tokens: Token[] = [];
  let remaining = line;

  // Handle leading whitespace
  const leadingSpace = remaining.match(/^(\s*)/);
  if (leadingSpace && leadingSpace[1]) {
    tokens.push({ type: 'text', value: leadingSpace[1] });
    remaining = remaining.slice(leadingSpace[1].length);
  }

  // Full line comment
  if (remaining.startsWith('#')) {
    tokens.push({ type: 'comment', value: remaining });
    return tokens;
  }

  // List item
  if (remaining.startsWith('- ')) {
    tokens.push({ type: 'dash', value: '- ' });
    remaining = remaining.slice(2);
  }

  // Key-value pair
  const keyMatch = remaining.match(/^([a-zA-Z_][a-zA-Z0-9_-]*)(:)(\s*)/);
  if (keyMatch) {
    tokens.push({ type: 'key', value: keyMatch[1] });
    tokens.push({ type: 'colon', value: keyMatch[2] });
    if (keyMatch[3]) {
      tokens.push({ type: 'text', value: keyMatch[3] });
    }
    remaining = remaining.slice(keyMatch[0].length);
  }

  // Process value
  if (remaining) {
    // Inline comment at end
    const commentIdx = remaining.indexOf(' #');
    let value = remaining;
    let comment = '';
    if (commentIdx > -1) {
      value = remaining.slice(0, commentIdx);
      comment = remaining.slice(commentIdx);
    }

    if (value) {
      // Quoted string
      const stringMatch = value.match(/^(["'])(.*)\1$/);
      if (stringMatch) {
        tokens.push({ type: 'string', value: value });
      }
      // Array brackets
      else if (value.match(/^\[.*\]$/)) {
        // Parse array contents
        tokens.push({ type: 'bracket', value: '[' });
        const inner = value.slice(1, -1);
        const items = inner.split(/,\s*/);
        items.forEach((item, i) => {
          if (item.startsWith('"') || item.startsWith("'")) {
            tokens.push({ type: 'string', value: item });
          } else if (item === '*') {
            tokens.push({ type: 'special', value: item });
          } else {
            tokens.push({ type: 'text', value: item });
          }
          if (i < items.length - 1) {
            tokens.push({ type: 'text', value: ', ' });
          }
        });
        tokens.push({ type: 'bracket', value: ']' });
      }
      // Boolean
      else if (/^(true|false)$/.test(value.trim())) {
        tokens.push({ type: 'boolean', value: value });
      }
      // Null
      else if (/^(null|~)$/.test(value.trim())) {
        tokens.push({ type: 'null', value: value });
      }
      // Number
      else if (/^-?\d+(\.\d+)?$/.test(value.trim())) {
        tokens.push({ type: 'number', value: value });
      }
      // Special glob/wildcard
      else if (value.trim() === '"*"' || value.trim() === "'*'" || value.trim() === '*') {
        tokens.push({ type: 'special', value: value });
      }
      // Plain text value
      else {
        tokens.push({ type: 'text', value: value });
      }
    }

    // Add inline comment
    if (comment) {
      tokens.push({ type: 'comment', value: comment });
    }
  }

  return tokens;
}

// YAML syntax highlighting
function highlightYaml(code: string): React.ReactNode[] {
  const lines = code.split('\n');

  return lines.map((line, lineIndex) => {
    const tokens = tokenizeLine(line);

    return (
      <span key={lineIndex}>
        {tokens.map((token, i) => (
          <span key={i} className={TOKEN_COLORS[token.type]}>
            {token.value}
          </span>
        ))}
        {lineIndex < lines.length - 1 && '\n'}
      </span>
    );
  });
}

// Copy button component
function CopyButton({ text, className = '' }: { text: string; className?: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={handleCopy}
      className={`flex h-7 w-7 items-center justify-center rounded-md border border-[var(--border)] bg-[var(--card)] text-[var(--muted)] transition-all hover:border-[var(--muted)] hover:text-[var(--foreground)] active:scale-95 ${className}`}
      title="Copy to clipboard"
    >
      {copied ? (
        <svg className="h-3.5 w-3.5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
        </svg>
      ) : (
        <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
      )}
    </button>
  );
}

// YAML code block with syntax highlighting
export function YamlBlock({ children, filename }: { children: string; filename?: string }) {
  return (
    <div className="group relative my-4 overflow-hidden rounded-lg border border-zinc-200 dark:border-zinc-800 bg-zinc-950">
      {filename && (
        <div className="flex items-center justify-between border-b border-zinc-800 bg-zinc-900 px-4 py-2">
          <span className="text-xs font-mono text-zinc-500">{filename}</span>
          <CopyButton text={children} />
        </div>
      )}
      <div className="relative">
        {!filename && (
          <CopyButton
            text={children}
            className="absolute right-3 top-3 opacity-0 group-hover:opacity-100 transition-opacity"
          />
        )}
        <pre className="overflow-x-auto p-4 pr-12">
          <code className="text-sm font-mono leading-relaxed">{highlightYaml(children)}</code>
        </pre>
      </div>
    </div>
  );
}

// Command block with copy button
export function CommandBlock({ children }: { children: string }) {
  const command = children.trim();

  return (
    <div className="group relative my-4 flex items-center justify-between gap-4 rounded-lg border border-zinc-200 dark:border-zinc-800 bg-zinc-950 px-4 py-3">
      <code className="font-mono text-sm">
        <span className="text-zinc-600 select-none">$ </span>
        <span className="text-zinc-300">{command}</span>
      </code>
      <CopyButton text={command} className="opacity-0 group-hover:opacity-100 transition-opacity" />
    </div>
  );
}

// Generic code block (non-YAML)
export function CodeBlock({ children, title }: { children: string; title?: string }) {
  return (
    <div className="group relative my-4">
      {title && <p className="text-xs text-zinc-500 mb-1">{title}</p>}
      <div className="relative overflow-hidden rounded-lg border border-zinc-200 dark:border-zinc-800 bg-zinc-950">
        <CopyButton
          text={children}
          className="absolute right-3 top-3 opacity-0 group-hover:opacity-100 transition-opacity"
        />
        <pre className="overflow-x-auto p-4 pr-12">
          <code className="text-zinc-300 text-sm font-mono whitespace-pre">{children}</code>
        </pre>
      </div>
    </div>
  );
}

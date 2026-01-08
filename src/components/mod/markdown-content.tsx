'use client'

import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import type { Components } from 'react-markdown'

interface MarkdownContentProps {
  content: string
  className?: string
}

export function MarkdownContent({ content, className = '' }: MarkdownContentProps) {
  const components: Components = {
    h1: ({ node, ...props }) => (
      <h1 className="text-3xl font-bold mb-4 mt-6 first:mt-0" {...props} />
    ),
    h2: ({ node, ...props }) => (
      <h2 className="text-2xl font-semibold mb-3 mt-5" {...props} />
    ),
    h3: ({ node, ...props }) => (
      <h3 className="text-xl font-semibold mb-2 mt-4" {...props} />
    ),
    h4: ({ node, ...props }) => (
      <h4 className="text-lg font-semibold mb-2 mt-3" {...props} />
    ),
    h5: ({ node, ...props }) => (
      <h5 className="text-base font-semibold mb-1 mt-2" {...props} />
    ),
    h6: ({ node, ...props }) => (
      <h6 className="text-sm font-semibold mb-1 mt-2" {...props} />
    ),
    p: ({ node, ...props }) => <p className="mb-4 leading-7" {...props} />,
    a: ({ node, ...props }) => (
      <a
        className="text-primary underline underline-offset-4 hover:text-primary/80"
        target="_blank"
        rel="noopener noreferrer"
        {...props}
      />
    ),
    ul: ({ node, ...props }) => (
      <ul className="list-disc list-inside mb-4 ml-4" {...props} />
    ),
    ol: ({ node, ...props }) => (
      <ol className="list-decimal list-inside mb-4 ml-4" {...props} />
    ),
    li: ({ node, ...props }) => <li className="mb-1" {...props} />,
    blockquote: ({ node, ...props }) => (
      <blockquote
        className="border-l-4 border-muted-foreground/30 pl-4 italic my-4"
        {...props}
      />
    ),
    code: ({ node, inline, ...props }) =>
      inline ? (
        <code
          className="bg-muted px-1.5 py-0.5 rounded text-sm font-mono"
          {...props}
        />
      ) : (
        <code
          className="block bg-muted p-4 rounded-lg text-sm font-mono overflow-x-auto my-4"
          {...props}
        />
      ),
    pre: ({ node, ...props }) => (
      <pre className="bg-muted p-4 rounded-lg overflow-x-auto my-4" {...props} />
    ),
    table: ({ node, ...props }) => (
      <div className="overflow-x-auto my-4">
        <table className="min-w-full divide-y divide-border" {...props} />
      </div>
    ),
    thead: ({ node, ...props }) => (
      <thead className="bg-muted/50" {...props} />
    ),
    tbody: ({ node, ...props }) => (
      <tbody className="divide-y divide-border" {...props} />
    ),
    tr: ({ node, ...props }) => <tr {...props} />,
    th: ({ node, ...props }) => (
      <th className="px-4 py-2 text-left text-sm font-semibold" {...props} />
    ),
    td: ({ node, ...props }) => (
      <td className="px-4 py-2 text-sm" {...props} />
    ),
    hr: ({ node, ...props }) => <hr className="my-6 border-border" {...props} />,
    img: ({ node, ...props }) => (
      <img className="max-w-full h-auto rounded-lg my-4" {...props} />
    ),
  }

  return (
    <div className={`prose prose-stone dark:prose-invert max-w-none ${className}`}>
      <ReactMarkdown remarkPlugins={[remarkGfm]} components={components}>
        {content}
      </ReactMarkdown>
    </div>
  )
}

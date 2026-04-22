import React, { forwardRef } from 'react';

interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(({ className = '', ...props }, ref) => {
  return (
    <textarea
      ref={ref}
      className={`bg-zinc-900 border border-zinc-700 text-white px-3 py-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 resize-none ${className}`}
      {...props}
    />
  );
});

Textarea.displayName = 'Textarea';

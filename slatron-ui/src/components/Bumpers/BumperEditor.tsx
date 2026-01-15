import React from 'react'
import Editor from '@monaco-editor/react'

interface BumperEditorProps {
    value: string
    onChange: (value: string) => void
    readOnly?: boolean
    theme?: string
}

export const BumperEditor: React.FC<BumperEditorProps> = ({
    value,
    onChange,
    readOnly = false,
    theme = "vs-dark"
}) => {
    return (
        <div className="h-full w-full bg-[#1e1e1e] border border-[var(--border-color)] rounded-lg overflow-hidden flex flex-col">
            <div className="p-2 border-b border-[var(--border-color)] bg-[var(--bg-tertiary)] flex justify-between items-center z-10">
                <span className="text-xs font-mono text-[var(--text-secondary)]">template.mlt</span>
                <span className="text-xs text-[var(--text-secondary)]">XML Template</span>
            </div>
            <div className="flex-1 relative">
                <Editor
                    height="100%"
                    defaultLanguage="xml"
                    theme={theme}
                    value={value}
                    onChange={(val) => onChange(val || '')}
                    options={{
                        minimap: { enabled: false },
                        fontSize: 14,
                        padding: { top: 16 },
                        fontFamily: 'JetBrains Mono, monospace',
                        scrollBeyondLastLine: false,
                        automaticLayout: true,
                        tabSize: 2,
                        readOnly: readOnly,
                        domReadOnly: readOnly
                    }}
                />
            </div>
        </div>
    )
}

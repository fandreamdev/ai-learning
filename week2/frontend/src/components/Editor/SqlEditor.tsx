import Editor from '@monaco-editor/react'

interface SqlEditorProps {
  value: string
  onChange: (value: string) => void
  height?: string | number
  readOnly?: boolean
}

export function SqlEditor({
  value,
  onChange,
  height = '100%',
  readOnly = false,
}: SqlEditorProps) {
  return (
    <Editor
      height={height}
      defaultLanguage="sql"
      value={value}
      onChange={(nextValue) => onChange(nextValue ?? '')}
      theme="vs-light"
      options={{
        minimap: { enabled: false },
        fontSize: 14,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        automaticLayout: true,
        readOnly,
      }}
    />
  )
}

export default SqlEditor

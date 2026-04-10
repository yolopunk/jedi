export interface SlashCommand {
  name: string
  description: string
  icon: string
  template: string  // prompt template with {args} placeholder
  args?: boolean     // whether this command accepts arguments
}

export const SLASH_COMMANDS: SlashCommand[] = [
  { name: '/commit', description: 'Commit changes', icon: '📦', template: 'Commit the current changes with a clear commit message: {args}', args: true },
  { name: '/review', description: 'Review code', icon: '🔍', template: 'Review the code changes in detail and provide feedback: {args}', args: true },
  { name: '/verify', description: 'Verify implementation', icon: '✅', template: 'Verify the implementation works correctly: {args}', args: true },
  { name: '/test', description: 'Run tests', icon: '🧪', template: 'Run tests for: {args}', args: true },
  { name: '/explain', description: 'Explain code', icon: '📖', template: 'Explain this code in detail: {args}', args: true },
  { name: '/refactor', description: 'Refactor code', icon: '🔧', template: 'Refactor the following code: {args}', args: true },
  { name: '/agent', description: 'Spawn worker agent', icon: '🤖', template: '{args}', args: true },
  { name: '/stop', description: 'Stop worker', icon: '⏹', template: 'Stop the worker: {args}', args: true },
]

export function parseSlashCommand(input: string): { command: SlashCommand; args: string } | null {
  const trimmed = input.trim()
  for (const cmd of SLASH_COMMANDS) {
    if (trimmed.startsWith(cmd.name)) {
      const rest = trimmed.slice(cmd.name.length).trim()
      return { command: cmd, args: rest }
    }
  }
  return null
}

export function formatCommandPrompt(input: string): string {
  const parsed = parseSlashCommand(input)
  if (!parsed) return input
  const args = parsed.args || 'no specific parameters'
  return parsed.command.template.replace('{args}', args)
}

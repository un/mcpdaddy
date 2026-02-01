export type UpstreamPreset = {
  id: string;
  displayName: string;
  description: string;
  command: string;
  args: string[];
  requiredEnvKeys: string[];
  docsUrl?: string;
};

export const UPSTREAM_PRESETS: UpstreamPreset[] = [
  {
    id: 'github',
    displayName: 'GitHub',
    description: 'Official GitHub MCP server (tools for repos, issues, PRs, etc.).',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-github'],
    requiredEnvKeys: ['GITHUB_PERSONAL_ACCESS_TOKEN'],
    docsUrl: 'https://modelcontextprotocol.io/examples.md',
  },
  {
    id: 'notion',
    displayName: 'Notion',
    description: 'Notion MCP server (workspace pages, databases, search).',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-notion'],
    requiredEnvKeys: ['NOTION_TOKEN'],
  },
  {
    id: 'vercel',
    displayName: 'Vercel',
    description: 'Vercel MCP server (projects, deployments, logs).',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-vercel'],
    requiredEnvKeys: ['VERCEL_TOKEN'],
  },
  {
    id: 'google',
    displayName: 'Google',
    description: 'Google MCP server (varies by integration; often uses a service account).',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-google'],
    requiredEnvKeys: ['GOOGLE_APPLICATION_CREDENTIALS'],
  },
];

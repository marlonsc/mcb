import * as http from 'http';
import { randomUUID } from 'crypto';
import bcrypt from 'bcryptjs';

const mcpHost = process.env.MCB_TEST_HOST || 'localhost';
const mcpPort = Number(process.env.MCB_TEST_PORT || '18080');
const apiKeyHeader = 'X-API-Key';
const adminKeyHeader = 'X-Admin-Key';

type JsonRpcError = {
  message?: string;
  code?: number;
};

type JsonRpcResponse = {
  result?: unknown;
  error?: JsonRpcError;
};

type ToolResult = {
  isError?: boolean;
  content?: Array<{ text?: string }>;
};

export type E2eAdminAuth = {
  orgId: string;
  userId: string;
  apiKey: string;
  headers: Record<string, string>;
};

let authPromise: Promise<E2eAdminAuth> | undefined;
let requestId = 1;

function firstResultText(result: unknown): string {
  const toolResult = result as ToolResult;
  return toolResult.content?.map(item => item.text || '').join('\n') || '';
}

function parseSseOrJson(raw: string): JsonRpcResponse {
  const dataLine = raw
    .split(/\r?\n/)
    .find(line => line.startsWith('data:'));
  const payload = dataLine ? dataLine.slice('data:'.length).trim() : raw.trim();
  if (!payload) {
    throw new Error('MCP response was empty');
  }
  return JSON.parse(payload) as JsonRpcResponse;
}

function mcpPost(body: string): Promise<JsonRpcResponse> {
  return new Promise((resolve, reject) => {
    const data = Buffer.from(body, 'utf8');
    const req = http.request({
      hostname: mcpHost,
      port: mcpPort,
      path: '/mcp',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json, text/event-stream',
        'Content-Length': data.length,
      },
    }, (res) => {
      let acc = '';
      const timer = setTimeout(() => {
        res.destroy(new Error(`MCP response timed out: ${acc}`));
      }, 15000);

      res.setEncoding('utf8');
      res.on('data', (chunk: string) => {
        acc += chunk;
        if (acc.includes('\n\n')) {
          clearTimeout(timer);
          res.destroy();
          try {
            resolve(parseSseOrJson(acc));
          } catch (error) {
            reject(error);
          }
        }
      });
      res.on('end', () => {
        clearTimeout(timer);
        try {
          resolve(parseSseOrJson(acc));
        } catch (error) {
          reject(error);
        }
      });
      res.on('error', reject);
    });
    req.on('error', reject);
    req.setTimeout(20000, () => {
      req.destroy(new Error('MCP request timed out'));
    });
    req.write(data);
    req.end();
  });
}

async function rpc(method: string, params: unknown): Promise<unknown> {
  const response = await mcpPost(JSON.stringify({
    jsonrpc: '2.0',
    id: requestId++,
    method,
    params,
  }));
  if (response.error) {
    throw new Error(`MCP ${method} failed: ${response.error.message || response.error.code}`);
  }
  return response.result;
}

export async function callMcpTool(name: string, args: Record<string, unknown>): Promise<unknown> {
  const result = await rpc('tools/call', {
    name,
    arguments: args,
  });
  const toolResult = result as ToolResult;
  if (toolResult.isError) {
    throw new Error(`MCP tool ${name} failed: ${firstResultText(result)}`);
  }
  return result;
}

async function createAdminAuth(): Promise<E2eAdminAuth> {
  await rpc('initialize', {
    protocolVersion: '2024-11-05',
    capabilities: {},
    clientInfo: { name: 'playwright-auth-seed', version: '1.0.0' },
  });

  const suffix = `${process.pid}-${Date.now()}-${randomUUID().slice(0, 8)}`;
  const orgId = `playwright-org-${suffix}`;
  const userId = `playwright-user-${suffix}`;
  const apiKeyId = `playwright-key-${suffix}`;
  const apiKey = `mcb-playwright-${suffix}`;
  const keyHash = bcrypt.hashSync(apiKey, 4);
  const createdAt = Math.floor(Date.now() / 1000);

  await callMcpTool('entity', {
    action: 'create',
    resource: 'org',
    org_id: orgId,
    data: {
      id: orgId,
      name: 'Playwright E2E Organization',
      slug: orgId,
      settings_json: '{}',
      created_at: createdAt,
      updated_at: createdAt,
    },
  });

  await callMcpTool('entity', {
    action: 'create',
    resource: 'user',
    org_id: orgId,
    data: {
      id: userId,
      org_id: orgId,
      email: `${userId}@example.test`,
      display_name: 'Playwright Admin',
      role: 'Admin',
      api_key_hash: keyHash,
      created_at: createdAt,
      updated_at: createdAt,
    },
  });

  await callMcpTool('entity', {
    action: 'create',
    resource: 'api_key',
    org_id: orgId,
    data: {
      id: apiKeyId,
      user_id: userId,
      org_id: orgId,
      key_hash: keyHash,
      name: 'playwright-admin-key',
      scopes_json: '[]',
      expires_at: null,
      revoked_at: null,
      created_at: createdAt,
    },
  });

  return {
    orgId,
    userId,
    apiKey,
    headers: {
      [apiKeyHeader]: apiKey,
      [adminKeyHeader]: apiKey,
    },
  };
}

export async function ensureE2eAdminAuth(): Promise<E2eAdminAuth> {
  authPromise ??= createAdminAuth();
  return authPromise;
}

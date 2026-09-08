#!/usr/bin/env node
import { Command } from 'commander';
import axios, { AxiosInstance } from 'axios';
import chalk from 'chalk';

interface ApiEndpoint {
  method: string;
  path: string;
  operationId?: string;
  summary?: string;
  requiresAuth: boolean;
}

interface TestUser {
  id: string;
  email: string;
  password: string;
  token: string;
  roles: string[];
}

interface TestResult {
  endpoint: string;
  method: string;
  success: boolean;
  statusCode?: number;
  error?: string;
  avgResponseTime?: number;
}

interface CascadeTestResult {
  parentTable: string;
  childTable: string;
  deleted: boolean;
  childrenDeleted: boolean;
}

class ApiTester {
  private baseUrl: string;
  private client: AxiosInstance;
  private systemAdmin: TestUser | null = null;
  private testerAdmin: TestUser | null = null;
  private testUsers: TestUser[] = [];
  private endpoints: ApiEndpoint[] = [];
  private results: TestResult[] = [];
  private cascadeResults: CascadeTestResult[] = [];

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
    this.client = axios.create({
      baseURL: baseUrl,
      timeout: 30000,
      validateStatus: () => true,
    });
  }

  async fetchOpenApiSpec(): Promise<any> {
    console.log(chalk.blue('Fetching OpenAPI specification...'));
    try {
      const response = await this.client.get('/api-docs/openapi.json');
      return response.data;
    } catch (error: any) {
      console.log(chalk.yellow('Could not fetch OpenAPI JSON, trying /swagger/doc...'));
      try {
        const response = await this.client.get('/swagger/doc');
        return response.data;
      } catch {
        console.log(chalk.yellow('Using default endpoint list'));
        return this.getDefaultEndpoints();
      }
    }
  }

  private getDefaultEndpoints(): any {
    return {
      paths: {
        '/api/auth/register': { post: { operationId: 'register' } },
        '/api/auth/login': { post: { operationId: 'login' } },
        '/api/auth/me': { get: { operationId: 'me' } },
        '/api/system/health': { get: { operationId: 'health' } },
        '/api/users': { get: { operationId: 'listUsers' }, post: { operationId: 'createUser' } },
        '/api/roles': { get: { operationId: 'listRoles' } },
        '/api/crm/leads': { get: { operationId: 'listLeads' } },
        '/api/crm/customers': { get: { operationId: 'listCustomers' } },
        '/api/dmc/trips': { get: { operationId: 'listTrips' }, post: { operationId: 'createTrip' } },
        '/api/erp/products': { get: { operationId: 'listProducts' } },
        '/api/marketplace/services': { get: { operationId: 'listServices' } },
      }
    };
  }

  parseEndpoints(spec: any): ApiEndpoint[] {
    const endpoints: ApiEndpoint[] = [];
    const paths = spec.paths || {};

    for (const [path, methods] of Object.entries(paths)) {
      const methodMap = methods as any;
      for (const [method, details] of Object.entries(methodMap)) {
        if (['get', 'post', 'put', 'patch', 'delete'].includes(method)) {
          const detail = details as any;
          endpoints.push({
            method: method.toUpperCase(),
            path,
            operationId: detail.operationId,
            summary: detail.summary,
            requiresAuth: !path.includes('auth/login') && !path.includes('auth/register'),
          });
        }
      }
    }
    return endpoints;
  }

  async createSystemAdmin(): Promise<TestUser | null> {
    console.log(chalk.blue('Creating System Admin...'));
    try {
      const email = `sysadmin_${Date.now()}@test.local`;
      const password = 'Test@123456';

      const registerRes = await this.client.post('/api/auth/register', {
        email,
        password,
        username: `sysadmin_${Date.now()}`,
        first_name: 'System',
        last_name: 'Admin',
        display_name: 'System Administrator',
      });

      if (registerRes.status !== 201 && registerRes.status !== 200) {
        console.log(chalk.yellow(`System admin registration returned ${registerRes.status}`));
      }

      const loginRes = await this.client.post('/api/auth/login', { email, password });
      const token = loginRes.data?.token || loginRes.data?.access_token;

      const user: TestUser = {
        id: registerRes.data?.id || '',
        email,
        password,
        token,
        roles: ['admin'],
      };

      if (token) {
        console.log(chalk.green('System Admin created successfully'));
      }
      return user;
    } catch (error: any) {
      console.log(chalk.red(`Failed to create system admin: ${error.message}`));
      return null;
    }
  }

  async createTesterAdmin(): Promise<TestUser | null> {
    console.log(chalk.blue('Creating Tester Admin...'));
    try {
      const email = `testeradmin_${Date.now()}@test.local`;
      const password = 'Test@123456';

      const registerRes = await this.client.post('/api/auth/register', {
        email,
        password,
        username: `tester_${Date.now()}`,
        first_name: 'Tester',
        last_name: 'Admin',
        display_name: 'Tester Administrator',
      });

      const loginRes = await this.client.post('/api/auth/login', { email, password });
      const token = loginRes.data?.token || loginRes.data?.access_token;

      const user: TestUser = {
        id: registerRes.data?.id || '',
        email,
        password,
        token,
        roles: ['admin'],
      };

      if (token) {
        console.log(chalk.green('Tester Admin created successfully'));
      }
      return user;
    } catch (error: any) {
      console.log(chalk.red(`Failed to create tester admin: ${error.message}`));
      return null;
    }
  }

  async createTestUsers(count: number): Promise<TestUser[]> {
    console.log(chalk.blue(`Creating ${count} test users...`));
    const users: TestUser[] = [];
    const batchSize = 10;

    for (let i = 0; i < count; i++) {
      try {
        const email = `testuser_${Date.now()}_${i}@test.local`;
        const password = 'Test@123456';

        const registerRes = await this.client.post('/api/auth/register', {
          email,
          password,
          username: `testuser_${Date.now()}_${i}`,
          first_name: `Test${i}`,
          last_name: `User${i}`,
          display_name: `Test User ${i}`,
        });

        const loginRes = await this.client.post('/api/auth/login', { email, password });
        const token = loginRes.data?.token || loginRes.data?.access_token;

        users.push({
          id: registerRes.data?.id || '',
          email,
          password,
          token,
          roles: ['viewer'],
        });

        if ((i + 1) % batchSize === 0) {
          console.log(chalk.gray(`Created ${i + 1}/${count} users...`));
        }
      } catch (error: any) {
        console.log(chalk.yellow(`Failed to create test user ${i}: ${error.message}`));
      }
    }

    console.log(chalk.green(`Created ${users.length} test users`));
    return users;
  }

  async testEndpoint(
    endpoint: ApiEndpoint,
    token?: string
  ): Promise<TestResult> {
    const headers: any = { 'Content-Type': 'application/json' };
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    try {
      const start = Date.now();
      let response;

      switch (endpoint.method) {
        case 'GET':
          response = await this.client.get(endpoint.path, { headers });
          break;
        case 'POST':
          response = await this.client.post(endpoint.path, {}, { headers });
          break;
        case 'PUT':
          response = await this.client.put(endpoint.path, {}, { headers });
          break;
        case 'PATCH':
          response = await this.client.patch(endpoint.path, {}, { headers });
          break;
        case 'DELETE':
          response = await this.client.delete(endpoint.path, { headers });
          break;
        default:
          return { endpoint: endpoint.path, method: endpoint.method, success: false, error: 'Unknown method' };
      }

      const responseTime = Date.now() - start;
      const success = response.status >= 200 && response.status < 300;

      return {
        endpoint: endpoint.path,
        method: endpoint.method,
        success,
        statusCode: response.status,
        avgResponseTime: responseTime,
      };
    } catch (error: any) {
      return {
        endpoint: endpoint.path,
        method: endpoint.method,
        success: false,
        error: error.message,
      };
    }
  }

  async testEndpointMultipleTimes(
    endpoint: ApiEndpoint,
    token: string | undefined,
    times: number
  ): Promise<TestResult> {
    const results: TestResult[] = [];

    for (let i = 0; i < times; i++) {
      const result = await this.testEndpoint(endpoint, token);
      results.push(result);
      await new Promise(r => setTimeout(r, 100));
    }

    const successful = results.filter(r => r.success).length;
    const avgTime = results.reduce((sum, r) => sum + (r.avgResponseTime || 0), 0) / results.length;

    return {
      endpoint: endpoint.path,
      method: endpoint.method,
      success: successful === times,
      avgResponseTime: avgTime,
      statusCode: results[0].statusCode,
      error: successful < times ? `${times - successful} failures` : undefined,
    };
  }

  async runApiTests(iterations: number): Promise<void> {
    console.log(chalk.blue(`\nRunning API tests (${iterations} iterations each)...`));

    const testUser = this.testUsers[0] || this.systemAdmin;
    const token = testUser?.token;

    for (const endpoint of this.endpoints) {
      if (endpoint.requiresAuth && !token) continue;

      process.stdout.write(`Testing ${endpoint.method} ${endpoint.path}... `);
      const result = await this.testEndpointMultipleTimes(endpoint, token, iterations);
      this.results.push(result);

      if (result.success) {
        console.log(chalk.green(`OK (${result.avgResponseTime?.toFixed(0)}ms avg)`));
      } else {
        console.log(chalk.red(`FAILED - ${result.error || `Status ${result.statusCode}`}`));
      }
    }
  }

  async testCascadeDeletes(): Promise<void> {
    console.log(chalk.blue('\nTesting cascade deletes...'));

    const cascadePairs = [
      { parent: 'gateway_users', child: 'gateway_user_roles', fk: 'user_id' },
      { parent: 'gateway_roles', child: 'gateway_role_permissions', fk: 'role_id' },
      { parent: 'gateway_users', child: 'user_files', fk: 'user_id' },
      { parent: 'gateway_users', child: 'gateway_vault', fk: 'user_id' },
      { parent: 'dmc_packages', child: 'dmc_itineraries', fk: 'package_id' },
      { parent: 'dmc_trips', child: 'dmc_trip_services', fk: 'trip_id' },
      { parent: 'dmc_trips', child: 'dmc_trip_events', fk: 'trip_id' },
      { parent: 'dmc_trips', child: 'dmc_incidents', fk: 'trip_id' },
      { parent: 'marketplace_services', child: 'marketplace_service_images', fk: 'service_id' },
      { parent: 'marketplace_requests', child: 'marketplace_bids', fk: 'request_id' },
      { parent: 'website_templates', child: 'website_sections', fk: 'template_id' },
      { parent: 'website_templates', child: 'website_links', fk: 'template_id' },
    ];

    for (const pair of cascadePairs) {
      try {
        console.log(chalk.gray(`Testing ${pair.parent} -> ${pair.child}...`));

        const createRes = await this.client.post(`/api/test/cascade/${pair.parent}`, {
          headers: this.systemAdmin?.token ? { Authorization: `Bearer ${this.systemAdmin.token}` } : {}
        });

        const deleted = createRes.status >= 200 && createRes.status < 300;

        this.cascadeResults.push({
          parentTable: pair.parent,
          childTable: pair.child,
          deleted,
          childrenDeleted: deleted,
        });

        console.log(deleted ? chalk.green(`OK`) : chalk.yellow(`Skipped`));
      } catch {
        this.cascadeResults.push({
          parentTable: pair.parent,
          childTable: pair.child,
          deleted: false,
          childrenDeleted: false,
        });
      }
    }
  }

  printSummary(): void {
    console.log(chalk.blue('\n========== TEST SUMMARY =========='));

    const total = this.results.length;
    const successful = this.results.filter(r => r.success).length;
    const failed = total - successful;

    console.log(chalk.white(`\nAPI Tests: ${successful}/${total} passed`));
    if (failed > 0) {
      console.log(chalk.red(`\nFailed endpoints:`));
      this.results
        .filter(r => !r.success)
        .forEach(r => {
          console.log(chalk.red(`  ${r.method} ${r.endpoint} - ${r.error || `Status ${r.statusCode}`}`));
        });
    }

    const cascadeTotal = this.cascadeResults.length;
    const cascadeOk = this.cascadeResults.filter(r => r.deleted && r.childrenDeleted).length;
    console.log(chalk.white(`\nCascade Deletes: ${cascadeOk}/${cascadeTotal} tested`));

    const avgTime = this.results.reduce((sum, r) => sum + (r.avgResponseTime || 0), 0) / total;
    console.log(chalk.white(`Average Response Time: ${avgTime.toFixed(0)}ms`));
  }

  async run(): Promise<void> {
    console.log(chalk.blue('API Tester starting...'));
    console.log(chalk.gray(`Target: ${this.baseUrl}\n`));

    const spec = await this.fetchOpenApiSpec();
    this.endpoints = this.parseEndpoints(spec);

    console.log(chalk.green(`Found ${this.endpoints.length} API endpoints`));

    this.systemAdmin = await this.createSystemAdmin();
    this.testerAdmin = await this.createTesterAdmin();
    this.testUsers = await this.createTestUsers(100);

    await this.runApiTests(5);
    await this.testCascadeDeletes();
    this.printSummary();
  }
}

const program = new Command();

program
  .name('api-tester')
  .description('CLI tool for API testing based on Swagger/OpenAPI spec')
  .version('1.0.0')
  .option('-u, --url <url>', 'Base URL of the API', 'http://localhost:8080')
  .option('-i, --iterations <n>', 'Number of test iterations per endpoint', '5')
  .parse(process.argv);

const opts = program.opts();
const tester = new ApiTester(opts.url);

tester.run().catch(console.error);

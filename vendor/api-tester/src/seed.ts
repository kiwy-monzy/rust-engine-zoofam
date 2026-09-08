#!/usr/bin/env node
import axios from 'axios';
import chalk from 'chalk';

const BASE_URL = process.env.API_URL || 'http://localhost:8080';

interface SeedUser {
  email: string;
  password: string;
  username: string;
  firstName: string;
  lastName: string;
  displayName: string;
  roles: string[];
}

interface SeedResult {
  success: boolean;
  created: number;
  failed: number;
  errors: string[];
}

class DatabaseSeeder {
  private client;
  private adminToken: string | null = null;

  constructor() {
    this.client = axios.create({
      baseURL: BASE_URL,
      timeout: 30000,
      validateStatus: () => true,
    });
  }

  async login(email: string, password: string): Promise<string | null> {
    try {
      const response = await this.client.post('/api/auth/login', { email, password });
      return response.data?.token || response.data?.access_token || null;
    } catch {
      return null;
    }
  }

  async createUser(user: SeedUser): Promise<boolean> {
    try {
      const registerResponse = await this.client.post('/api/auth/register', {
        email: user.email,
        password: user.password,
        username: user.username,
        first_name: user.firstName,
        last_name: user.lastName,
        display_name: user.displayName,
      });

      if (registerResponse.status !== 201 && registerResponse.status !== 200) {
        return false;
      }

      if (user.roles.includes('admin')) {
        await this.assignAdminRole(user.email);
      }

      return true;
    } catch {
      return false;
    }
  }

  async assignAdminRole(email: string): Promise<void> {
    try {
      const loginRes = await this.client.post('/api/auth/login', {
        email,
        password: 'Test@123456',
      });

      const token = loginRes.data?.token || loginRes.data?.access_token;
      if (!token) return;

      const headers = { Authorization: `Bearer ${token}` };

      await this.client.post('/api/rbac/users/roles', {
        email,
        roles: ['admin'],
      }, { headers });
    } catch {
      // Role assignment may not be available via API
    }
  }

  async createSystemAdmin(): Promise<SeedUser> {
    const timestamp = Date.now();
    return {
      email: `sysadmin_${timestamp}@test.local`,
      password: 'Test@123456',
      username: `sysadmin_${timestamp}`,
      firstName: 'System',
      lastName: 'Administrator',
      displayName: 'System Admin',
      roles: ['admin'],
    };
  }

  async createTesterAdmin(): Promise<SeedUser> {
    const timestamp = Date.now();
    return {
      email: `tester_admin_${timestamp}@test.local`,
      password: 'Test@123456',
      username: `tester_admin_${timestamp}`,
      firstName: 'Tester',
      lastName: 'Administrator',
      displayName: 'Tester Admin',
      roles: ['admin'],
    };
  }

  async createTestUsers(count: number): Promise<SeedUser[]> {
    const users: SeedUser[] = [];
    for (let i = 0; i < count; i++) {
      const timestamp = Date.now();
      users.push({
        email: `testuser_${timestamp}_${i}@test.local`,
        password: 'Test@123456',
        username: `testuser_${timestamp}_${i}`,
        firstName: `Test${i}`,
        lastName: `User${i}`,
        displayName: `Test User ${i}`,
        roles: ['viewer'],
      });
    }
    return users;
  }

  async seedUsers(users: SeedUser[]): Promise<SeedResult> {
    const result: SeedResult = {
      success: true,
      created: 0,
      failed: 0,
      errors: [],
    };

    const batchSize = 10;
    for (let i = 0; i < users.length; i++) {
      const user = users[i];
      const success = await this.createUser(user);

      if (success) {
        result.created++;
        process.stdout.write(chalk.green('.'));
      } else {
        result.failed++;
        result.errors.push(`Failed to create ${user.email}`);
        process.stdout.write(chalk.red('.'));
      }

      if ((i + 1) % batchSize === 0) {
        console.log(chalk.gray(` ${Math.min(i + 1, users.length)}/${users.length}`));
      }

      await new Promise(r => setTimeout(r, 50));
    }

    return result;
  }

  async seedTestData(): Promise<void> {
    console.log(chalk.blue.bold('\n========== DATABASE SEEDING ==========\n'));
    console.log(chalk.gray(`Target: ${BASE_URL}\n`));

    console.log(chalk.blue('Creating System Admin...'));
    const sysAdmin = await this.createSystemAdmin();
    const sysAdminCreated = await this.createUser(sysAdmin);
    if (sysAdminCreated) {
      console.log(chalk.green(`System Admin created: ${sysAdmin.email}`));
    } else {
      console.log(chalk.red(`Failed to create System Admin`));
    }

    console.log(chalk.blue('\nCreating Tester Admin...'));
    const testerAdmin = await this.createTesterAdmin();
    const testerCreated = await this.createUser(testerAdmin);
    if (testerCreated) {
      console.log(chalk.green(`Tester Admin created: ${testerAdmin.email}`));
    } else {
      console.log(chalk.red(`Failed to create Tester Admin`));
    }

    console.log(chalk.blue('\nCreating 100 Test Users...'));
    const testUsers = await this.createTestUsers(100);
    const result = await this.seedUsers(testUsers);

    console.log(chalk.blue.bold('\n\n========== SEED SUMMARY =========='));
    console.log(chalk.green(`Created: ${result.created}`));
    console.log(chalk.red(`Failed: ${result.failed}`));

    if (result.errors.length > 0) {
      console.log(chalk.yellow('\nErrors:'));
      result.errors.slice(0, 10).forEach(e => console.log(chalk.yellow(`  - ${e}`)));
      if (result.errors.length > 10) {
        console.log(chalk.yellow(`  ... and ${result.errors.length - 10} more`));
      }
    }
  }
}

const seeder = new DatabaseSeeder();
seeder.seedTestData().catch(console.error);

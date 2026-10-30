import assert from 'node:assert/strict';
import { generateKeyPairSync, verify } from 'node:crypto';
import test from 'node:test';

import {
  createAppStoreConnectToken,
  waitForTestFlightBuild,
} from './verify-testflight-build.mjs';

function jsonResponse(body, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

function resource(id, attributes = {}, relationships = {}) {
  return { type: 'fixture', id, attributes, relationships };
}

function createFixtureFetch({ buildStates, groups, testersByGroup = {}, expired = false }) {
  let buildIndex = 0;
  return async (url, options) => {
    assert.equal(options.headers.Authorization, 'Bearer fixture-token');
    if (url.pathname.endsWith('/apps')) {
      assert.equal(url.searchParams.get('filter[bundleId]'), 'dev.example.monitor');
      return jsonResponse({ data: [resource('app-1')] });
    }
    if (url.pathname === '/v1/builds') {
      assert.equal(url.searchParams.get('filter[app]'), 'app-1');
      assert.equal(url.searchParams.get('filter[version]'), '12345');
      assert.equal(url.searchParams.get('include'), 'betaGroups');
      const state = buildStates[Math.min(buildIndex, buildStates.length - 1)];
      buildIndex += 1;
      if (state === 'MISSING') return jsonResponse({ data: [] });
      return jsonResponse({
        data: [
          resource(
            'build-1',
            { version: '12345', processingState: state, expired },
            { betaGroups: { data: [{ type: 'betaGroups', id: 'group-1' }] } },
          ),
        ],
      });
    }
    if (url.pathname.endsWith('/apps/app-1/betaGroups')) {
      return jsonResponse({ data: groups });
    }
    const groupMatch = url.pathname.match(/\/betaGroups\/([^/]+)\/betaTesters$/);
    if (groupMatch) {
      return jsonResponse({ data: testersByGroup[groupMatch[1]] ?? [] });
    }
    throw new Error(`Unexpected fixture URL ${url}`);
  };
}

function testOptions(overrides) {
  let currentTime = 0;
  return {
    buildNumber: '12345',
    bundleId: 'dev.example.monitor',
    apiBaseUrl: 'https://example.test/v1/',
    timeoutMs: 1_000,
    intervalMs: 100,
    tokenFactory: () => 'fixture-token',
    logger: () => {},
    now: () => currentTime,
    sleep: async (milliseconds) => {
      currentTime += milliseconds;
    },
    ...overrides,
  };
}

test('creates a verifiable ES256 App Store Connect token with a 15-minute lifetime', () => {
  const { privateKey, publicKey } = generateKeyPairSync('ec', { namedCurve: 'P-256' });
  const token = createAppStoreConnectToken({
    keyId: 'KEY123',
    issuerId: 'issuer-123',
    privateKey,
    nowMs: 1_700_000_000_000,
  });
  const [header, payload, signature] = token.split('.');
  assert.deepEqual(JSON.parse(Buffer.from(header, 'base64url')), {
    alg: 'ES256', kid: 'KEY123', typ: 'JWT',
  });
  assert.deepEqual(JSON.parse(Buffer.from(payload, 'base64url')), {
    iss: 'issuer-123', iat: 1_700_000_000, exp: 1_700_000_900, aud: 'appstoreconnect-v1',
  });
  assert.equal(Buffer.from(signature, 'base64url').length, 64);
  assert.equal(verify('sha256', Buffer.from(`${header}.${payload}`), {
    key: publicKey,
    dsaEncoding: 'ieee-p1363',
  }, Buffer.from(signature, 'base64url')), true);
});

test('waits for processing and proves an internal tester can receive the build', async () => {
  const result = await waitForTestFlightBuild(
    testOptions({
      fetchImpl: createFixtureFetch({
        buildStates: ['MISSING', 'PROCESSING', 'VALID'],
        groups: [
          resource('group-1', {
            name: 'Internal Testers',
            isInternalGroup: true,
            hasAccessToAllBuilds: false,
          }),
        ],
        testersByGroup: { 'group-1': [resource('tester-1')] },
      }),
    }),
  );
  assert.deepEqual(result, { appId: 'app-1', buildId: 'build-1', groupId: 'group-1' });
});

test('fails immediately when Apple marks a build invalid', async () => {
  await assert.rejects(
    waitForTestFlightBuild(
      testOptions({
        fetchImpl: createFixtureFetch({ buildStates: ['INVALID'], groups: [] }),
      }),
    ),
    /finished processing with state INVALID/,
  );
});

test('rejects a valid build that has no receiving internal group', async () => {
  await assert.rejects(
    waitForTestFlightBuild(
      testOptions({
        fetchImpl: createFixtureFetch({
          buildStates: ['VALID'],
          groups: [
            resource('external-group', {
              name: 'External Testers',
              isInternalGroup: false,
              hasAccessToAllBuilds: true,
            }),
          ],
        }),
      }),
    ),
    /not available to an internal TestFlight group/,
  );
});

test('rejects an expired build even when processing succeeded and a tester is assigned', async () => {
  await assert.rejects(
    waitForTestFlightBuild(
      testOptions({
        fetchImpl: createFixtureFetch({
          buildStates: ['VALID'],
          expired: true,
          groups: [resource('group-1', { isInternalGroup: true })],
          testersByGroup: { 'group-1': [resource('tester-1')] },
        }),
      }),
    ),
    /Build 12345 has expired/,
  );
});

test('accepts all-build access and searches past an empty internal group for a tester', async () => {
  const result = await waitForTestFlightBuild(
    testOptions({
      fetchImpl: createFixtureFetch({
        buildStates: ['VALID'],
        groups: [
          resource('group-1', { isInternalGroup: true }),
          resource('group-2', { isInternalGroup: true, hasAccessToAllBuilds: true }),
        ],
        testersByGroup: { 'group-2': [resource('tester-1')] },
      }),
    }),
  );
  assert.deepEqual(result, { appId: 'app-1', buildId: 'build-1', groupId: 'group-2' });
});

test('rejects internal assignment when no eligible group has a tester', async () => {
  await assert.rejects(
    waitForTestFlightBuild(
      testOptions({
        fetchImpl: createFixtureFetch({
          buildStates: ['VALID'],
          groups: [resource('group-1', { isInternalGroup: true })],
        }),
      }),
    ),
    /no eligible group has a tester/,
  );
});

test('retries transient Apple failures with refreshed credentials', async () => {
  let requests = 0;
  let tokens = 0;
  const fixtureFetch = createFixtureFetch({
    buildStates: ['VALID'],
    groups: [resource('group-1', { isInternalGroup: true })],
    testersByGroup: { 'group-1': [resource('tester-1')] },
  });
  const result = await waitForTestFlightBuild(
    testOptions({
      tokenFactory: () => `token-${++tokens}`,
      fetchImpl: async (url, options) => {
        requests += 1;
        assert.equal(options.headers.Authorization, `Bearer token-${requests}`);
        if (requests === 1) throw new Error('temporary network failure');
        if (requests === 2) return jsonResponse({ errors: [{ code: 'RATE_LIMITED' }] }, 429);
        return fixtureFetch(url, {
          ...options, headers: { ...options.headers, Authorization: 'Bearer fixture-token' },
        });
      },
    }),
  );
  assert.deepEqual(result, { appId: 'app-1', buildId: 'build-1', groupId: 'group-1' });
  assert.equal(requests, 6);
});

test('rejects Apple authentication errors immediately and reports their details', async () => {
  let requests = 0;
  await assert.rejects(
    waitForTestFlightBuild(
      testOptions({
        fetchImpl: async () => {
          requests += 1;
          return jsonResponse({ errors: [{ code: 'NOT_AUTHORIZED', detail: 'Invalid token' }] }, 401);
        },
        sleep: async () => assert.fail('authentication failures must not be retried'),
      }),
    ),
    /HTTP 401: NOT_AUTHORIZED: Invalid token/,
  );
  assert.equal(requests, 1);
});

test('times out when the uploaded build never appears', async () => {
  await assert.rejects(
    waitForTestFlightBuild(
      testOptions({
        timeoutMs: 250,
        fetchImpl: createFixtureFetch({ buildStates: ['MISSING'], groups: [] }),
      }),
    ),
    /Timed out waiting for build 12345/,
  );
});

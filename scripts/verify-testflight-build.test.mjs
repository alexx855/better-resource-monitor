import assert from 'node:assert/strict';
import { generateKeyPairSync } from 'node:crypto';
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

function createFixtureFetch({ buildStates, groups, testersByGroup = {} }) {
  let buildIndex = 0;
  return async (url, options) => {
    assert.equal(options.headers.Authorization, 'Bearer fixture-token');
    if (url.pathname.endsWith('/apps')) {
      assert.equal(url.searchParams.get('filter[bundleId]'), 'dev.example.monitor');
      return jsonResponse({ data: [resource('app-1')] });
    }
    if (url.pathname.endsWith('/apps/app-1/builds')) {
      assert.equal(url.searchParams.get('filter[version]'), '12345');
      const state = buildStates[Math.min(buildIndex, buildStates.length - 1)];
      buildIndex += 1;
      if (state === 'MISSING') return jsonResponse({ data: [] });
      return jsonResponse({
        data: [
          resource(
            'build-1',
            { version: '12345', processingState: state },
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

test('creates an ES256 App Store Connect token with a raw 64-byte signature', () => {
  const { privateKey } = generateKeyPairSync('ec', { namedCurve: 'P-256' });
  const token = createAppStoreConnectToken({
    keyId: 'KEY123',
    issuerId: 'issuer-123',
    privateKey,
    nowMs: 1_700_000_000_000,
  });
  const [header, payload, signature] = token.split('.');
  assert.equal(JSON.parse(Buffer.from(header, 'base64url')).kid, 'KEY123');
  assert.equal(JSON.parse(Buffer.from(payload, 'base64url')).aud, 'appstoreconnect-v1');
  assert.equal(Buffer.from(signature, 'base64url').length, 64);
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

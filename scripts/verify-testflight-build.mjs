#!/usr/bin/env node

import { readFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { sign } from 'node:crypto';

const DEFAULT_API_BASE_URL = 'https://api.appstoreconnect.apple.com/v1/';
const DEFAULT_BUNDLE_ID = 'dev.alexpedersen.better-resource-monitor';

function requireValue(value, name) {
  if (!value) throw new Error(`${name} is required`);
  return value;
}

export function createAppStoreConnectToken({ keyId, issuerId, privateKey, nowMs = Date.now() }) {
  const encodedHeader = Buffer.from(
    JSON.stringify({ alg: 'ES256', kid: requireValue(keyId, 'keyId'), typ: 'JWT' }),
  ).toString('base64url');
  const nowSeconds = Math.floor(nowMs / 1000);
  const encodedPayload = Buffer.from(
    JSON.stringify({
      iss: requireValue(issuerId, 'issuerId'),
      iat: nowSeconds,
      exp: nowSeconds + 15 * 60,
      aud: 'appstoreconnect-v1',
    }),
  ).toString('base64url');
  const unsignedToken = `${encodedHeader}.${encodedPayload}`;
  const signature = sign('sha256', Buffer.from(unsignedToken), {
    key: requireValue(privateKey, 'privateKey'),
    dsaEncoding: 'ieee-p1363',
  }).toString('base64url');
  return `${unsignedToken}.${signature}`;
}

function appleErrorMessage(status, body) {
  const errors = Array.isArray(body?.errors) ? body.errors : [];
  const details = errors
    .map((error) => [error.code, error.title, error.detail].filter(Boolean).join(': '))
    .filter(Boolean)
    .join('; ');
  return `App Store Connect request failed with HTTP ${status}${details ? `: ${details}` : ''}`;
}

async function requestJson({
  path,
  query = {},
  apiBaseUrl,
  fetchImpl,
  tokenFactory,
  sleep,
  logger,
}) {
  const url = new URL(path.replace(/^\//, ''), apiBaseUrl);
  for (const [name, value] of Object.entries(query)) {
    if (value !== undefined && value !== null && value !== '') {
      url.searchParams.set(name, String(value));
    }
  }

  let lastError;
  for (let attempt = 1; attempt <= 5; attempt += 1) {
    try {
      const response = await fetchImpl(url, {
        headers: {
          Accept: 'application/json',
          Authorization: `Bearer ${tokenFactory()}`,
        },
      });
      const text = await response.text();
      const body = text ? JSON.parse(text) : {};
      if (response.ok) return body;

      const error = new Error(appleErrorMessage(response.status, body));
      error.status = response.status;
      if (response.status !== 429 && response.status < 500) throw error;
      lastError = error;
    } catch (error) {
      if (error?.status && error.status !== 429 && error.status < 500) throw error;
      lastError = error;
    }

    if (attempt < 5) {
      logger(`App Store Connect request attempt ${attempt} failed; retrying.`);
      await sleep(5_000);
    }
  }
  throw lastError;
}

function expectSingleResource(body, description) {
  if (!Array.isArray(body?.data) || body.data.length !== 1) {
    throw new Error(`Expected exactly one ${description}, found ${body?.data?.length ?? 0}`);
  }
  return body.data[0];
}

export async function waitForTestFlightBuild({
  buildNumber,
  bundleId = DEFAULT_BUNDLE_ID,
  apiBaseUrl = DEFAULT_API_BASE_URL,
  timeoutMs = 30 * 60 * 1000,
  intervalMs = 30 * 1000,
  fetchImpl = globalThis.fetch,
  sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds)),
  now = Date.now,
  tokenFactory,
  logger = console.log,
}) {
  requireValue(buildNumber, 'buildNumber');
  requireValue(bundleId, 'bundleId');
  requireValue(fetchImpl, 'fetchImpl');
  requireValue(tokenFactory, 'tokenFactory');

  const request = (path, query) =>
    requestJson({ path, query, apiBaseUrl, fetchImpl, tokenFactory, sleep, logger });

  const app = expectSingleResource(
    await request('apps', {
      'filter[bundleId]': bundleId,
      'fields[apps]': 'bundleId,name',
      limit: 2,
    }),
    `app matching bundle ID ${bundleId}`,
  );

  const deadline = now() + timeoutMs;
  let build;
  while (now() <= deadline) {
    const builds = await request('builds', {
      'filter[app]': app.id,
      'filter[version]': buildNumber,
      'fields[builds]': 'version,processingState,uploadedDate,expired,betaGroups',
      include: 'betaGroups',
      'fields[betaGroups]': 'name,isInternalGroup,hasAccessToAllBuilds',
      limit: 2,
    });

    if (!Array.isArray(builds?.data) || builds.data.length === 0) {
      logger(`Build ${buildNumber} is not visible in App Store Connect yet.`);
    } else {
      build = expectSingleResource(builds, `build matching ${buildNumber}`);
      const state = build.attributes?.processingState;
      if (state === 'VALID') break;
      if (state === 'FAILED' || state === 'INVALID') {
        throw new Error(`Build ${buildNumber} finished processing with state ${state}`);
      }
      if (state !== 'PROCESSING') {
        throw new Error(`Build ${buildNumber} has unexpected processing state ${state ?? 'missing'}`);
      }
      logger(`Build ${buildNumber} is still processing.`);
    }

    if (now() + intervalMs > deadline) break;
    await sleep(intervalMs);
  }

  if (!build || build.attributes?.processingState !== 'VALID') {
    throw new Error(`Timed out waiting for build ${buildNumber} to finish processing`);
  }

  const groupsResponse = await request(`apps/${encodeURIComponent(app.id)}/betaGroups`, {
    'fields[betaGroups]': 'name,isInternalGroup,hasAccessToAllBuilds',
    limit: 200,
  });
  const assignedGroupIds = new Set(
    (build.relationships?.betaGroups?.data ?? []).map((group) => group.id),
  );
  const eligibleGroups = (groupsResponse.data ?? []).filter(
    (group) =>
      group.attributes?.isInternalGroup === true &&
      (group.attributes?.hasAccessToAllBuilds === true || assignedGroupIds.has(group.id)),
  );

  if (eligibleGroups.length === 0) {
    throw new Error(
      `Build ${buildNumber} is valid but is not available to an internal TestFlight group`,
    );
  }

  for (const group of eligibleGroups) {
    const testers = await request(`betaGroups/${encodeURIComponent(group.id)}/betaTesters`, {
      limit: 1,
    });
    if ((testers.data ?? []).length > 0) {
      logger(
        `Build ${buildNumber} is VALID and available to internal group ${group.attributes?.name ?? group.id}.`,
      );
      return { appId: app.id, buildId: build.id, groupId: group.id };
    }
  }

  throw new Error(
    `Build ${buildNumber} is valid and assigned for internal testing, but no eligible group has a tester`,
  );
}

async function main() {
  const buildNumber = process.argv[2];
  const keyId = requireValue(process.env.APPLE_API_KEY_ID, 'APPLE_API_KEY_ID');
  const issuerId = requireValue(process.env.APPLE_API_ISSUER, 'APPLE_API_ISSUER');
  const keyPath =
    process.env.APPSTORE_CONNECT_API_KEY_PATH ??
    join(homedir(), '.appstoreconnect', 'private_keys', `AuthKey_${keyId}.p8`);
  const privateKey = await readFile(keyPath, 'utf8');
  const timeoutMs = Number(process.env.TESTFLIGHT_PROCESSING_TIMEOUT_SECONDS ?? 1800) * 1000;
  const intervalMs = Number(process.env.TESTFLIGHT_PROCESSING_INTERVAL_SECONDS ?? 30) * 1000;

  await waitForTestFlightBuild({
    buildNumber,
    bundleId: process.env.TESTFLIGHT_BUNDLE_ID ?? DEFAULT_BUNDLE_ID,
    timeoutMs,
    intervalMs,
    tokenFactory: () =>
      createAppStoreConnectToken({ keyId, issuerId, privateKey, nowMs: Date.now() }),
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error) => {
    console.error(`Error: ${error.message}`);
    process.exitCode = 1;
  });
}

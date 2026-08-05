#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import readline from "node:readline";
import { spawn } from "node:child_process";
import { pathToFileURL } from "node:url";

const SERVICE_NAME = "claude_code_codex_app_plugin";
const IPC_INITIAL_CLIENT_ID = "initializing-client";
const IPC_MAX_FRAME_BYTES = 256 * 1024 * 1024;
const IPC_REQUEST_VERSIONS = new Map([
  ["thread-follower-start-turn", 1],
  ["thread-follower-steer-turn", 1],
  ["thread-follower-interrupt-turn", 4]
]);
const IPC_BROADCAST_VERSIONS = new Map([
  ["thread-stream-following-changed", 1]
]);
const IPC_THREAD_STATE_VERSION = 11;
const IPC_LOCAL_HOST_ID = "local";
const IPC_SNAPSHOT_TIMEOUT_MS = 10_000;
const IPC_PATCH_OPERATIONS = new Set(["add", "replace", "remove"]);
const ACTIVE_STATUSES = new Set(["queued", "running", "steering", "interrupting"]);
const TERMINAL_TURN_STATUSES = new Set(["completed", "interrupted", "failed", "cancelled"]);
const SANDBOXES = new Map([
  ["read-only", "read-only"],
  ["workspace-write", "workspace-write"],
  ["danger-full-access", "danger-full-access"]
]);

function now() {
  return new Date().toISOString();
}

function shorten(value, limit = 72) {
  const text = String(value ?? "").trim().replace(/\s+/g, " ");
  return text.length <= limit ? text : `${text.slice(0, limit - 3)}...`;
}

function workspaceRoot(cwd = process.cwd()) {
  try {
    return fs.realpathSync(cwd);
  } catch {
    return path.resolve(cwd);
  }
}

function workspaceId(cwd) {
  return crypto.createHash("sha256").update(cwd).digest("hex").slice(0, 16);
}

function dataRoot(env = process.env) {
  return path.resolve(
    env.CODEX_APP_DATA_DIR || env.CLAUDE_PLUGIN_DATA || path.join(os.homedir(), ".codex-app")
  );
}

function jobsDirectory(cwd, env = process.env) {
  return path.join(dataRoot(env), "jobs", workspaceId(cwd));
}

function jobPath(cwd, id, env = process.env) {
  return path.join(jobsDirectory(cwd, env), `${id}.json`);
}

function writeJob(cwd, job, env = process.env) {
  const directory = jobsDirectory(cwd, env);
  fs.mkdirSync(directory, { recursive: true });
  const destination = jobPath(cwd, job.id, env);
  const temporary = `${destination}.${process.pid}.${crypto.randomBytes(3).toString("hex")}.tmp`;
  fs.writeFileSync(temporary, `${JSON.stringify({ ...job, updatedAt: now() }, null, 2)}\n`, "utf8");
  fs.renameSync(temporary, destination);
}

function readJobs(cwd, env = process.env) {
  const directory = jobsDirectory(cwd, env);
  if (!fs.existsSync(directory)) {
    return [];
  }

  return fs
    .readdirSync(directory)
    .filter((name) => name.endsWith(".json"))
    .flatMap((name) => {
      try {
        return [JSON.parse(fs.readFileSync(path.join(directory, name), "utf8"))];
      } catch {
        return [];
      }
    })
    .sort((left, right) => String(right.createdAt).localeCompare(String(left.createdAt)));
}

function patchJob(cwd, id, patch, env = process.env) {
  const file = jobPath(cwd, id, env);
  if (!fs.existsSync(file)) {
    return null;
  }
  const job = JSON.parse(fs.readFileSync(file, "utf8"));
  const updated = { ...job, ...patch };
  writeJob(cwd, updated, env);
  return updated;
}

function newJobId() {
  const stamp = new Date().toISOString().replace(/[-:TZ.]/g, "").slice(0, 14);
  return `task-${stamp}-${crypto.randomBytes(3).toString("hex")}`;
}

function controlSocketPath(jobId, env = process.env) {
  if (process.platform === "win32") {
    return `\\\\.\\pipe\\codex-app-${jobId}`;
  }
  const base = env.CODEX_APP_CONTROL_DIR || (fs.existsSync("/private/tmp") ? "/private/tmp" : os.tmpdir());
  const directory = path.join(base, `codex-app-${process.getuid?.() ?? "user"}`);
  fs.mkdirSync(directory, { recursive: true });
  return path.join(directory, `${jobId}.sock`);
}

function removeSocket(socketPath) {
  if (socketPath && process.platform !== "win32" && fs.existsSync(socketPath)) {
    fs.unlinkSync(socketPath);
  }
}

function processIsAlive(pid) {
  if (!Number.isInteger(pid) || pid <= 0) {
    return false;
  }
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error?.code === "EPERM";
  }
}

export function tokenizeArguments(raw) {
  const tokens = [];
  let token = "";
  let quote = null;
  let escaping = false;
  let started = false;

  for (const character of String(raw ?? "")) {
    if (escaping) {
      token += character;
      started = true;
      escaping = false;
      continue;
    }
    if (character === "\\" && quote !== "'") {
      escaping = true;
      started = true;
      continue;
    }
    if (quote) {
      if (character === quote) {
        quote = null;
      } else {
        token += character;
      }
      started = true;
      continue;
    }
    if (character === "'" || character === '"') {
      quote = character;
      started = true;
      continue;
    }
    if (/\s/.test(character)) {
      if (started) {
        tokens.push(token);
        token = "";
        started = false;
      }
      continue;
    }
    token += character;
    started = true;
  }

  if (escaping) {
    token += "\\";
  }
  if (quote) {
    throw new Error(`Unclosed ${quote} quote in arguments.`);
  }
  if (started) {
    tokens.push(token);
  }
  return tokens;
}

export function parseRunArguments(raw) {
  const tokens = tokenizeArguments(raw);
  const options = { model: null, effort: null, sandbox: "workspace-write" };
  let index = 0;

  while (index < tokens.length) {
    const token = tokens[index];
    if (token === "--") {
      index += 1;
      break;
    }
    if (!token.startsWith("--")) {
      break;
    }
    if (!new Set(["--model", "--effort", "--sandbox"]).has(token)) {
      throw new Error(`Unknown option ${token}.`);
    }
    const value = tokens[index + 1];
    if (!value) {
      throw new Error(`${token} requires a value.`);
    }
    if (token === "--model") {
      options.model = value;
    } else if (token === "--effort") {
      options.effort = value;
    } else {
      if (!SANDBOXES.has(value)) {
        throw new Error(`Unsupported sandbox ${value}.`);
      }
      options.sandbox = value;
    }
    index += 2;
  }

  const prompt = tokens.slice(index).join(" ").trim();
  if (!prompt) {
    throw new Error("A Codex prompt is required.");
  }
  return { prompt, options };
}

export function resolveCodexBinary(env = process.env) {
  if (env.CODEX_APP_BIN) {
    return env.CODEX_APP_BIN;
  }
  if (env.CODEX_CLI_PATH) {
    return env.CODEX_CLI_PATH;
  }
  return "codex";
}

class AppServerClient {
  constructor(cwd, env = process.env) {
    this.cwd = cwd;
    this.env = env;
    this.binary = resolveCodexBinary(env);
    this.pending = new Map();
    this.nextId = 1;
    this.stderr = "";
    this.closed = false;
    this.notificationHandler = null;
  }

  async initialize() {
    this.proc = spawn(this.binary, ["app-server"], {
      cwd: this.cwd,
      env: this.env,
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true
    });
    this.proc.stdout.setEncoding("utf8");
    this.proc.stderr.setEncoding("utf8");
    this.proc.stderr.on("data", (chunk) => {
      this.stderr += chunk;
    });
    this.proc.on("error", (error) => this.failPending(error));
    this.proc.on("exit", (code, signal) => {
      if (!this.closed && code !== 0) {
        const detail = this.stderr.trim();
        this.failPending(
          new Error(
            `codex app-server exited ${signal ? `with ${signal}` : `with code ${code}`}${detail ? `: ${detail}` : ""}`
          )
        );
      }
    });

    this.lines = readline.createInterface({ input: this.proc.stdout });
    this.lines.on("line", (line) => this.handleLine(line));

    await this.request("initialize", {
      clientInfo: {
        name: "claude_code_codex_app",
        title: "Claude Code Codex App Plugin",
        version: "0.1.1"
      },
      capabilities: {
        experimentalApi: false,
        requestAttestation: false
      }
    });
    this.notify("initialized", {});
  }

  setNotificationHandler(handler) {
    this.notificationHandler = handler;
  }

  request(method, params = {}) {
    if (this.closed || !this.proc?.stdin) {
      return Promise.reject(new Error("codex app-server is not connected."));
    }
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject, method });
      this.send({ id, method, params });
    });
  }

  notify(method, params = {}) {
    this.send({ method, params });
  }

  send(message) {
    this.proc.stdin.write(`${JSON.stringify(message)}\n`);
  }

  handleLine(line) {
    if (!line.trim()) {
      return;
    }
    let message;
    try {
      message = JSON.parse(line);
    } catch (error) {
      this.failPending(new Error(`Invalid app-server JSON: ${error.message}`));
      return;
    }

    if (message.id !== undefined && message.method) {
      this.send({
        id: message.id,
        error: {
          code: -32601,
          message: `Background codex-app worker cannot answer ${message.method}.`
        }
      });
      return;
    }
    if (message.id !== undefined) {
      const pending = this.pending.get(message.id);
      if (!pending) {
        return;
      }
      this.pending.delete(message.id);
      if (message.error) {
        const error = new Error(message.error.message || `${pending.method} failed.`);
        error.data = message.error.data;
        pending.reject(error);
      } else {
        pending.resolve(message.result ?? {});
      }
      return;
    }
    if (message.method) {
      this.notificationHandler?.(message);
    }
  }

  failPending(error) {
    for (const pending of this.pending.values()) {
      pending.reject(error);
    }
    this.pending.clear();
  }

  async close() {
    if (this.closed) {
      return;
    }
    this.closed = true;
    this.lines?.close();
    if (this.proc?.stdin && !this.proc.stdin.destroyed) {
      this.proc.stdin.end();
    }
    if (this.proc && this.proc.exitCode === null) {
      await new Promise((resolve) => {
        const timer = setTimeout(() => {
          this.proc.kill("SIGTERM");
          resolve();
        }, 1500);
        timer.unref?.();
        this.proc.once("exit", () => {
          clearTimeout(timer);
          resolve();
        });
      });
    }
  }
}

export function resolveDesktopIpcPath(env = process.env) {
  if (env.CODEX_APP_IPC_PATH) {
    return path.resolve(env.CODEX_APP_IPC_PATH);
  }
  if (process.platform === "win32") {
    return "\\\\.\\pipe\\codex-ipc";
  }
  const codexHome = env.CODEX_HOME || path.join(os.homedir(), ".codex");
  return path.join(codexHome, "ipc", "ipc.sock");
}

function validateDesktopIpcPath(socketPath, env = process.env) {
  if (process.platform === "win32" || env.CODEX_APP_IPC_PATH) {
    return;
  }
  let directory;
  let socket;
  try {
    directory = fs.lstatSync(path.dirname(socketPath));
    socket = fs.lstatSync(socketPath);
  } catch {
    throw new Error(
      `Codex Desktop IPC is unavailable at ${socketPath}. Open Codex Desktop and keep the target task visible.`
    );
  }
  const uid = process.getuid?.();
  if (
    uid == null ||
    !directory.isDirectory() ||
    directory.uid !== uid ||
    (directory.mode & 0o022) !== 0 ||
    !socket.isSocket() ||
    socket.uid !== uid
  ) {
    throw new Error(`Refusing insecure Codex Desktop IPC endpoint ${socketPath}.`);
  }
}

class DesktopIpcClient {
  constructor(env = process.env) {
    this.env = env;
    this.socketPath = resolveDesktopIpcPath(env);
    this.clientId = IPC_INITIAL_CLIENT_ID;
    this.pending = new Map();
    this.broadcastHandlers = new Map();
    this.failureHandlers = new Set();
    this.buffer = Buffer.alloc(0);
    this.closed = false;
    this.connectionError = null;
  }

  async initialize() {
    validateDesktopIpcPath(this.socketPath, this.env);
    this.socket = net.createConnection(this.socketPath);
    this.socket.on("data", (chunk) => this.handleData(chunk));
    this.socket.on("error", (error) => this.failConnection(error));
    this.socket.on("close", () => {
      if (!this.closed) {
        this.failConnection(new Error("Codex Desktop IPC connection closed."));
      }
    });
    await new Promise((resolve, reject) => {
      this.socket.once("connect", resolve);
      this.socket.once("error", reject);
    });
    const response = await this.request("initialize", {
      clientType: "claude-code-codex-app-plugin"
    });
    const clientId = response?.result?.clientId;
    if (!clientId) {
      throw new Error("Codex Desktop IPC initialization did not return a client id.");
    }
    this.clientId = clientId;
  }

  request(method, params = {}, timeoutMs = 15_000) {
    if (!this.socket?.writable) {
      return Promise.reject(new Error("Codex Desktop IPC is not connected."));
    }
    if (this.clientId === IPC_INITIAL_CLIENT_ID && method !== "initialize") {
      return Promise.reject(new Error("Codex Desktop IPC is not initialized."));
    }
    const requestId = crypto.randomUUID();
    const message = {
      type: "request",
      requestId,
      sourceClientId: this.clientId,
      version: IPC_REQUEST_VERSIONS.get(method) ?? 0,
      method,
      params,
      timeoutMs
    };
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(requestId);
        reject(new Error(`Codex Desktop IPC ${method} timed out.`));
      }, timeoutMs);
      timer.unref?.();
      this.pending.set(requestId, { method, resolve, reject, timer });
      this.send(message);
    });
  }

  broadcast(method, params = {}, targetClientIds = undefined) {
    if (!this.socket?.writable) {
      throw new Error("Codex Desktop IPC is not connected.");
    }
    if (this.clientId === IPC_INITIAL_CLIENT_ID) {
      throw new Error("Codex Desktop IPC is not initialized.");
    }
    this.send({
      type: "broadcast",
      method,
      sourceClientId: this.clientId,
      version: IPC_BROADCAST_VERSIONS.get(method) ?? 0,
      params,
      ...(targetClientIds ? { targetClientIds } : {})
    });
  }

  addBroadcastHandler(method, handler) {
    const handlers = this.broadcastHandlers.get(method) ?? new Set();
    handlers.add(handler);
    this.broadcastHandlers.set(method, handlers);
    return () => {
      handlers.delete(handler);
      if (handlers.size === 0) {
        this.broadcastHandlers.delete(method);
      }
    };
  }

  addFailureHandler(handler) {
    if (this.connectionError) {
      handler(this.connectionError);
      return () => {};
    }
    this.failureHandlers.add(handler);
    return () => this.failureHandlers.delete(handler);
  }

  send(message) {
    const payload = Buffer.from(JSON.stringify(message), "utf8");
    if (payload.length > IPC_MAX_FRAME_BYTES) {
      throw new Error("Codex Desktop IPC payload is too large.");
    }
    const header = Buffer.allocUnsafe(4);
    header.writeUInt32LE(payload.length, 0);
    this.socket.write(Buffer.concat([header, payload]));
  }

  handleData(chunk) {
    this.buffer = Buffer.concat([this.buffer, chunk]);
    while (this.buffer.length >= 4) {
      const length = this.buffer.readUInt32LE(0);
      if (length > IPC_MAX_FRAME_BYTES) {
        this.socket.destroy(new Error("Codex Desktop IPC frame is too large."));
        return;
      }
      if (this.buffer.length < 4 + length) {
        return;
      }
      const payload = this.buffer.subarray(4, 4 + length);
      this.buffer = this.buffer.subarray(4 + length);
      let message;
      try {
        message = JSON.parse(payload.toString("utf8"));
      } catch (error) {
        this.socket.destroy(new Error(`Invalid Codex Desktop IPC JSON: ${error.message}`));
        return;
      }
      this.handleMessage(message);
    }
  }

  handleMessage(message) {
    if (message.type === "client-discovery-request") {
      this.send({
        type: "client-discovery-response",
        requestId: message.requestId,
        response: { canHandle: false }
      });
      return;
    }
    if (message.type === "request") {
      this.send({
        type: "response",
        requestId: message.requestId,
        resultType: "error",
        error: "no-handler-for-request"
      });
      return;
    }
    if (message.type === "broadcast") {
      if (
        Array.isArray(message.targetClientIds) &&
        !message.targetClientIds.includes(this.clientId)
      ) {
        return;
      }
      for (const handler of this.broadcastHandlers.get(message.method) ?? []) {
        handler(message);
      }
      return;
    }
    if (message.type !== "response") {
      return;
    }
    const pending = this.pending.get(message.requestId);
    if (!pending) {
      return;
    }
    this.pending.delete(message.requestId);
    clearTimeout(pending.timer);
    if (message.resultType === "error") {
      pending.reject(
        new Error(`Codex Desktop IPC ${pending.method} failed: ${message.error || "unknown error"}`)
      );
      return;
    }
    pending.resolve(message);
  }

  failPending(error) {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.pending.clear();
  }

  failConnection(error) {
    if (this.connectionError) {
      return;
    }
    this.connectionError = error;
    this.failPending(error);
    for (const handler of this.failureHandlers) {
      handler(error);
    }
    this.failureHandlers.clear();
  }

  async close() {
    if (this.closed) {
      return;
    }
    this.closed = true;
    this.failPending(new Error("Codex Desktop IPC client closed."));
    this.broadcastHandlers.clear();
    this.failureHandlers.clear();
    this.socket?.end();
    this.socket?.destroy();
  }
}

function buildInput(prompt) {
  return [{ type: "text", text: prompt, text_elements: [] }];
}

function finalAgentMessage(turn) {
  const messages = (turn?.items ?? []).filter((item) => item.type === "agentMessage");
  return String(messages.at(-1)?.text ?? "").trim();
}

async function captureTurn(client, threadId, startTurn, onUpdate) {
  let turnId = null;
  let finalMessage = "";
  let settled = false;
  const buffered = [];
  let resolveCompletion;
  let rejectCompletion;
  const completion = new Promise((resolve, reject) => {
    resolveCompletion = resolve;
    rejectCompletion = reject;
  });

  function apply(message) {
    if (message.method === "item/started" && message.params?.threadId === threadId) {
      const type = message.params.item?.type;
      if (type) {
        onUpdate({ phase: type });
      }
      return;
    }
    if (message.method === "item/completed" && message.params?.threadId === threadId) {
      const item = message.params.item;
      if (item?.type === "agentMessage" && item.text) {
        finalMessage = String(item.text).trim();
      }
      return;
    }
    if (message.method === "error" && message.params?.threadId === threadId) {
      rejectCompletion(new Error(message.params.error?.message ?? "Codex turn failed."));
      return;
    }
    if (
      message.method === "turn/completed" &&
      message.params?.threadId === threadId &&
      (!turnId || message.params.turn?.id === turnId)
    ) {
      settled = true;
      resolveCompletion(message.params.turn);
    }
  }

  client.setNotificationHandler((message) => {
    if (!turnId) {
      buffered.push(message);
      return;
    }
    apply(message);
  });

  const response = await startTurn();
  turnId = response.turn?.id;
  if (!turnId) {
    throw new Error("turn/start did not return a turn id.");
  }
  onUpdate({ turnId, phase: "running" });
  for (const message of buffered.splice(0)) {
    apply(message);
  }
  if (!settled) {
    const turn = await completion;
    return {
      turn,
      turnId,
      finalMessage: finalMessage || finalAgentMessage(turn)
    };
  }
  const turn = await completion;
  return { turn, turnId, finalMessage: finalMessage || finalAgentMessage(turn) };
}

function candidateLabel(thread) {
  const title = thread.name || thread.preview || "untitled";
  return `${thread.id} (${shorten(title, 60)})`;
}

function normalizedTitle(value) {
  return String(value ?? "").trim().replace(/\s+/g, " ");
}

export async function resolveAskThreadByTitle(appClient, cwd, env = process.env) {
  const title = normalizedTitle(env.CODEX_APP_TITLE);
  if (!title) {
    throw new Error(
      "CODEX_APP_TITLE is unavailable. Set CODEX_APP_THREAD_ID to the target Codex task id and retry."
    );
  }

  const listed = await appClient.request("thread/list", {
    cursor: null,
    limit: 25,
    sortKey: "recency_at",
    sortDirection: "desc",
    archived: false,
    cwd
  });
  const matching = (listed.data ?? []).filter(
    (thread) => normalizedTitle(thread.name || thread.preview) === title
  );
  if (matching.length === 1) {
    return matching[0].id;
  }

  const details = matching.length
    ? ` Matches: ${matching.slice(0, 5).map(candidateLabel).join(", ")}.`
    : "";
  throw new Error(
    `CODEX_APP_TITLE did not identify exactly one unarchived Codex task in ${cwd}: ${title}.${details} ` +
      "Set CODEX_APP_THREAD_ID to the task id and retry."
  );
}

function findTurnId(value) {
  if (!value || typeof value !== "object") {
    return null;
  }
  if (typeof value.turnId === "string") {
    return value.turnId;
  }
  if (typeof value.turn?.id === "string") {
    return value.turn.id;
  }
  for (const child of Object.values(value)) {
    const found = findTurnId(child);
    if (found) {
      return found;
    }
  }
  return null;
}

function assertSafePatchPath(pathSegments) {
  for (const segment of pathSegments) {
    if (segment === "__proto__" || segment === "prototype" || segment === "constructor") {
      throw new Error(`Unsafe Desktop state patch path segment ${segment}.`);
    }
  }
}

export function applyDesktopStatePatches(state, patches) {
  let updated = state;
  for (const patch of patches) {
    if (!patch || !Array.isArray(patch.path) || !IPC_PATCH_OPERATIONS.has(patch.op)) {
      throw new Error("Invalid Desktop state patch.");
    }
    assertSafePatchPath(patch.path);
    if (patch.path.length === 0) {
      if (patch.op === "remove") {
        throw new Error("Desktop state cannot be removed by a root patch.");
      }
      updated = patch.value;
      continue;
    }

    let parent = updated;
    for (const segment of patch.path.slice(0, -1)) {
      if (parent == null || typeof parent !== "object") {
        throw new Error("Desktop state patch traversed a non-container value.");
      }
      if (!Object.prototype.hasOwnProperty.call(parent, segment)) {
        throw new Error(`Desktop state patch path is missing ${String(segment)}.`);
      }
      parent = parent[segment];
    }

    const key = patch.path.at(-1);
    if (Array.isArray(parent)) {
      if (!Number.isInteger(key)) {
        throw new Error("Desktop state array patch requires an integer index.");
      }
      if (patch.op === "add") {
        if (key < 0 || key > parent.length) {
          throw new Error("Desktop state array add index is out of bounds.");
        }
        parent.splice(key, 0, patch.value);
      } else {
        if (key < 0 || key >= parent.length) {
          throw new Error("Desktop state array patch index is out of bounds.");
        }
        if (patch.op === "remove") {
          parent.splice(key, 1);
        } else {
          parent[key] = patch.value;
        }
      }
      continue;
    }
    if (parent == null || typeof parent !== "object") {
      throw new Error("Desktop state patch parent is not an object.");
    }
    if (patch.op === "remove") {
      delete parent[key];
    } else {
      parent[key] = patch.value;
    }
  }
  return updated;
}

export function desktopTurnById(conversationState, turnId) {
  const entities = conversationState?.turnHistory?.history?.entitiesByKey;
  if (entities && typeof entities === "object") {
    const canonical = Object.values(entities).find((turn) => turn?.turnId === turnId);
    if (canonical) {
      return canonical;
    }
  }
  return (conversationState?.turns ?? []).find(
    (turn) => turn?.turnId === turnId || turn?.id === turnId
  ) ?? null;
}

export class DesktopThreadFollower {
  constructor(ipcClient, threadId, options = {}) {
    this.ipcClient = ipcClient;
    this.threadId = threadId;
    this.hostId = options.hostId ?? IPC_LOCAL_HOST_ID;
    this.snapshotTimeoutMs = options.snapshotTimeoutMs ?? IPC_SNAPSHOT_TIMEOUT_MS;
    this.state = null;
    this.revision = null;
    this.ownerClientId = null;
    this.started = false;
    this.closed = false;
    this.error = null;
    this.turnWaiter = null;
  }

  async start() {
    if (this.started) {
      throw new Error("Codex Desktop task follower is already started.");
    }
    this.started = true;
    this.removeBroadcastHandler = this.ipcClient.addBroadcastHandler(
      "thread-stream-state-changed",
      (message) => this.handleStateChanged(message)
    );
    this.removeFailureHandler = this.ipcClient.addFailureHandler((error) => this.fail(error));

    await new Promise((resolve, reject) => {
      this.readyWaiter = { resolve, reject };
      this.requestSnapshot();
    });
  }

  requestSnapshot() {
    if (this.closed || this.error) {
      return;
    }
    clearTimeout(this.snapshotTimer);
    this.awaitingSnapshot = true;
    this.snapshotTimer = setTimeout(() => {
      this.fail(
        new Error(
          `Codex Desktop did not expose task ${this.threadId} as an owned IPC stream within ${this.snapshotTimeoutMs}ms.`
        )
      );
    }, this.snapshotTimeoutMs);
    try {
      this.ipcClient.broadcast("thread-stream-following-changed", {
        conversationId: this.threadId,
        hostId: this.hostId,
        following: true
      });
    } catch (error) {
      this.fail(error);
    }
  }

  handleStateChanged(message) {
    try {
      const params = message.params;
      if (params?.conversationId !== this.threadId || params?.hostId !== this.hostId) {
        return;
      }
      if (message.version !== IPC_THREAD_STATE_VERSION) {
        throw new Error(
          `Unsupported Codex Desktop thread state protocol version ${String(message.version)}; expected ${IPC_THREAD_STATE_VERSION}.`
        );
      }
      const change = params.change;
      if (change?.type === "snapshot") {
        if (
          !change.conversationState ||
          typeof change.conversationState !== "object" ||
          !Number.isInteger(change.revision)
        ) {
          throw new Error("Codex Desktop sent an invalid task state snapshot.");
        }
        this.state = change.conversationState;
        this.revision = change.revision;
        this.ownerClientId = message.sourceClientId;
        this.awaitingSnapshot = false;
        clearTimeout(this.snapshotTimer);
        this.readyWaiter?.resolve();
        this.readyWaiter = null;
        this.settleTurnIfFinished();
        return;
      }
      if (change?.type !== "patches") {
        throw new Error(`Unsupported Codex Desktop task state change ${String(change?.type)}.`);
      }
      if (this.awaitingSnapshot) {
        return;
      }
      if (
        message.sourceClientId !== this.ownerClientId ||
        change.baseRevision !== this.revision ||
        !Number.isInteger(change.revision) ||
        change.revision <= change.baseRevision
      ) {
        this.requestSnapshot();
        return;
      }
      this.state = applyDesktopStatePatches(this.state, change.patches);
      this.revision = change.revision;
      this.settleTurnIfFinished();
    } catch (error) {
      this.fail(error);
    }
  }

  waitForTurn(turnId) {
    if (!this.started || !this.state) {
      return Promise.reject(new Error("Codex Desktop task follower is not ready."));
    }
    if (this.error) {
      return Promise.reject(this.error);
    }
    if (this.turnWaiter) {
      return Promise.reject(new Error("Codex Desktop task follower is already waiting for a turn."));
    }
    const turn = desktopTurnById(this.state, turnId);
    if (turn && TERMINAL_TURN_STATUSES.has(turn.status)) {
      return Promise.resolve({ turn, turnId, finalMessage: finalAgentMessage(turn) });
    }
    return new Promise((resolve, reject) => {
      this.turnWaiter = { turnId, resolve, reject };
    });
  }

  settleTurnIfFinished() {
    if (!this.turnWaiter) {
      return;
    }
    const turn = desktopTurnById(this.state, this.turnWaiter.turnId);
    if (!turn || !TERMINAL_TURN_STATUSES.has(turn.status)) {
      return;
    }
    const waiter = this.turnWaiter;
    this.turnWaiter = null;
    waiter.resolve({
      turn,
      turnId: waiter.turnId,
      finalMessage: finalAgentMessage(turn)
    });
  }

  fail(error) {
    if (this.error || this.closed) {
      return;
    }
    this.error = error instanceof Error ? error : new Error(String(error));
    clearTimeout(this.snapshotTimer);
    this.readyWaiter?.reject(this.error);
    this.readyWaiter = null;
    this.turnWaiter?.reject(this.error);
    this.turnWaiter = null;
  }

  async close() {
    if (this.closed) {
      return;
    }
    this.closed = true;
    clearTimeout(this.snapshotTimer);
    try {
      this.ipcClient.broadcast("thread-stream-following-changed", {
        conversationId: this.threadId,
        hostId: this.hostId,
        following: false
      });
    } catch {
      // A disconnected IPC client is already absent from Desktop's follower set.
    }
    this.removeBroadcastHandler?.();
    this.removeFailureHandler?.();
    this.turnWaiter?.reject(new Error("Codex Desktop task follower closed before completion."));
    this.turnWaiter = null;
  }
}

function buildRestoreMessage(prompt, cwd) {
  return {
    id: crypto.randomUUID(),
    text: prompt,
    context: {
      prompt,
      addedFiles: [],
      fileAttachments: [],
      ideContext: null,
      imageAttachments: [],
      commentAttachments: [],
      workspaceRoots: [cwd]
    },
    cwd,
    createdAt: Date.now()
  };
}

function openControlServer(socketPath, handler) {
  removeSocket(socketPath);
  const server = net.createServer((socket) => {
    socket.setEncoding("utf8");
    let buffer = "";
    socket.on("data", async (chunk) => {
      buffer += chunk;
      const newline = buffer.indexOf("\n");
      if (newline === -1) {
        return;
      }
      const line = buffer.slice(0, newline);
      buffer = buffer.slice(newline + 1);
      try {
        const result = await handler(JSON.parse(line));
        socket.end(`${JSON.stringify({ ok: true, result })}\n`);
      } catch (error) {
        socket.end(`${JSON.stringify({ ok: false, error: error.message })}\n`);
      }
    });
  });
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(socketPath, () => resolve(server));
  });
}

function sendControl(socketPath, request) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection(socketPath);
    socket.setEncoding("utf8");
    socket.setTimeout(3000);
    let buffer = "";
    socket.on("connect", () => socket.write(`${JSON.stringify(request)}\n`));
    socket.on("data", (chunk) => {
      buffer += chunk;
      const newline = buffer.indexOf("\n");
      if (newline === -1) {
        return;
      }
      const response = JSON.parse(buffer.slice(0, newline));
      socket.end();
      if (response.ok) {
        resolve(response.result);
      } else {
        reject(new Error(response.error || "Codex control request failed."));
      }
    });
    socket.on("timeout", () => socket.destroy(new Error("Codex control socket timed out.")));
    socket.on("error", reject);
  });
}

function closeControlServer(server, socketPath) {
  if (!server) {
    removeSocket(socketPath);
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    server.close(() => {
      removeSocket(socketPath);
      resolve();
    });
  });
}

function activeJobForThread(cwd, threadId, env) {
  return readJobs(cwd, env).find(
    (job) => job.kind === "ask" && job.threadId === threadId && ACTIVE_STATUSES.has(job.status)
  );
}

async function steerExistingAsk(cwd, threadId, prompt, env) {
  const job = activeJobForThread(cwd, threadId, env);
  if (!job) {
    return null;
  }
  try {
    await sendControl(job.controlSocket, { action: "steer", prompt });
    return job;
  } catch (error) {
    if (processIsAlive(job.pid)) {
      throw new Error(`Codex job ${job.id} is alive but cannot be steered: ${error.message}`);
    }
    patchJob(cwd, job.id, {
      status: "failed",
      phase: "stale",
      pid: null,
      error: "Background process exited before updating its job record."
    }, env);
    return null;
  }
}

async function executeAskWorker(prompt, env = process.env) {
  const cwd = workspaceRoot();
  let appClient = null;
  let ipcClient = null;
  let threadFollower = null;
  let controlServer = null;
  let socketPath = null;
  let jobId = null;
  let threadId = null;
  let activeTurnId = null;
  let update = () => null;

  try {
    threadId = String(env.CODEX_APP_THREAD_ID ?? "").trim();
    if (!threadId) {
      appClient = new AppServerClient(cwd, env);
      await appClient.initialize();
      threadId = await resolveAskThreadByTitle(appClient, cwd, env);
      await appClient.close();
      appClient = null;
    }

    const existing = await steerExistingAsk(cwd, threadId, prompt, env);
    if (existing) {
      process.stdout.write(
        `Steered Codex background job ${existing.id}. The original background task remains active.\n`
      );
      return;
    }

    ipcClient = new DesktopIpcClient(env);
    await ipcClient.initialize();
    jobId = newJobId();
    socketPath = controlSocketPath(jobId, env);
    const job = {
      id: jobId,
      kind: "ask",
      workspace: cwd,
      prompt,
      promptPreview: shorten(prompt),
      options: {},
      status: "queued",
      phase: "starting",
      threadId,
      turnId: null,
      controlSocket: socketPath,
      ipcSocket: ipcClient.socketPath,
      pid: process.pid,
      createdAt: now(),
      startedAt: null,
      completedAt: null,
      archived: false,
      finalMessage: null,
      error: null
    };
    writeJob(cwd, job, env);
    update = (patch) => patchJob(cwd, jobId, patch, env);
    update({ status: "running", startedAt: now(), transport: "desktop-ipc" });

    threadFollower = new DesktopThreadFollower(ipcClient, threadId);
    await threadFollower.start();

    controlServer = await openControlServer(socketPath, async (request) => {
      if (!activeTurnId) {
        throw new Error("Codex turn has not started yet.");
      }
      if (request.action === "steer") {
        const steerPrompt = String(request.prompt ?? "").trim();
        if (!steerPrompt) {
          throw new Error("A steer prompt is required.");
        }
        update({ status: "steering", phase: "steering" });
        const restoreMessage = buildRestoreMessage(steerPrompt, cwd);
        const response = await ipcClient.request("thread-follower-steer-turn", {
          conversationId: threadId,
          clientUserMessageId: restoreMessage.id,
          input: buildInput(steerPrompt),
          serviceTier: null,
          attachments: [],
          additionalContext: null,
          restoreMessage
        });
        update({ status: "running", phase: "running" });
        return { turnId: findTurnId(response) || activeTurnId };
      }
      if (request.action === "interrupt") {
        update({ status: "interrupting", phase: "interrupting" });
        await ipcClient.request("thread-follower-interrupt-turn", {
          conversationId: threadId,
          mode: "user",
          expectedTurnId: activeTurnId
        });
        return { turnId: activeTurnId };
      }
      throw new Error(`Unknown control action ${request.action}.`);
    });

    const startResponse = await ipcClient.request("thread-follower-start-turn", {
      conversationId: threadId,
      turnStartParams: {
        input: buildInput(prompt),
        cwd
      }
    });
    activeTurnId = findTurnId(startResponse);
    if (activeTurnId) {
      update({ turnId: activeTurnId, phase: "running" });
    }

    if (!activeTurnId) {
      throw new Error("Codex Desktop did not return a turn id.");
    }
    const result = await threadFollower.waitForTurn(activeTurnId);
    activeTurnId = result.turnId;
    const status = result.turn.status === "cancelled" ? "interrupted" : result.turn.status;
    update({
      status,
      phase: "done",
      pid: null,
      turnId: activeTurnId,
      completedAt: now(),
      finalMessage: result.finalMessage || null
    });

    process.stdout.write(
      `CODEX_APP_RESULT ${JSON.stringify({
        jobId,
        kind: "ask",
        status,
        threadId,
        turnId: activeTurnId,
        archived: false
      })}\n\n`
    );
    process.stdout.write(
      `${result.finalMessage || "Codex completed without a final agent message."}\n`
    );
  } catch (error) {
    if (jobId) {
      update({
        status: "failed",
        phase: "failed",
        pid: null,
        completedAt: now(),
        error: error.message
      });
    }
    throw error;
  } finally {
    await closeControlServer(controlServer, socketPath);
    await Promise.all([
      threadFollower?.close().catch(() => {}),
      appClient?.close().catch(() => {}),
      ipcClient?.close().catch(() => {})
    ]);
  }
}

async function executeWorker(kind, prompt, options, env = process.env) {
  if (kind === "ask") {
    await executeAskWorker(prompt, env);
    return;
  }
  const cwd = workspaceRoot();
  const id = newJobId();
  const socketPath = controlSocketPath(id, env);
  let threadId = null;
  let activeTurnId = null;
  let controlServer = null;
  let client = null;
  const job = {
    id,
    kind,
    workspace: cwd,
    prompt,
    promptPreview: shorten(prompt),
    options,
    status: "queued",
    phase: "starting",
    threadId,
    turnId: null,
    controlSocket: socketPath,
    pid: process.pid,
    createdAt: now(),
    startedAt: null,
    completedAt: null,
    archived: false,
    finalMessage: null,
    error: null
  };
  writeJob(cwd, job, env);

  const update = (patch) => patchJob(cwd, id, patch, env);

  try {
    client = new AppServerClient(cwd, env);
    await client.initialize();
    update({ status: "running", startedAt: now(), binary: client.binary });

    const response = await client.request("thread/start", {
      cwd,
      model: options.model,
      approvalPolicy: "never",
      approvalsReviewer: "auto_review",
      sandbox: SANDBOXES.get(options.sandbox),
      serviceName: SERVICE_NAME,
      ephemeral: false
    });
    threadId = response.thread.id;
    await client.request("thread/name/set", {
      threadId,
      name: `Claude worker: ${shorten(prompt, 56)}`
    }).catch(() => {});
    update({ threadId });

    controlServer = await openControlServer(socketPath, async (request) => {
      if (!activeTurnId) {
        throw new Error("Codex turn has not started yet.");
      }
      if (request.action === "steer") {
        const steerPrompt = String(request.prompt ?? "").trim();
        if (!steerPrompt) {
          throw new Error("A steer prompt is required.");
        }
        update({ status: "steering", phase: "steering" });
        const result = await client.request("turn/steer", {
          threadId,
          expectedTurnId: activeTurnId,
          input: buildInput(steerPrompt)
        });
        update({ status: "running", phase: "running" });
        return { turnId: result.turnId };
      }
      if (request.action === "interrupt") {
        update({ status: "interrupting", phase: "interrupting" });
        await client.request("turn/interrupt", { threadId, turnId: activeTurnId });
        return { turnId: activeTurnId };
      }
      throw new Error(`Unknown control action ${request.action}.`);
    });

    const turn = await captureTurn(
      client,
      threadId,
      () => {
        const params = {
          threadId,
          input: buildInput(prompt),
          model: options.model,
          effort: options.effort,
          approvalPolicy: "never",
          approvalsReviewer: "auto_review"
        };
        return client.request("turn/start", params);
      },
      (patch) => {
        if (patch.turnId) {
          activeTurnId = patch.turnId;
        }
        update(patch);
      }
    );

    const status = turn.turn.status === "completed" ? "completed" : turn.turn.status;
    let archived = false;
    let archiveError = null;
    try {
      await client.request("thread/archive", { threadId });
      archived = true;
    } catch (error) {
      archiveError = error.message;
    }

    update({
      status,
      phase: "done",
      pid: null,
      completedAt: now(),
      archived,
      archiveError,
      finalMessage: turn.finalMessage || null
    });

    const metadata = {
      jobId: id,
      kind,
      status,
      threadId,
      turnId: turn.turnId,
      archived
    };
    process.stdout.write(`CODEX_APP_RESULT ${JSON.stringify(metadata)}\n\n`);
    process.stdout.write(`${turn.finalMessage || "Codex completed without a final agent message."}\n`);
    if (archiveError) {
      process.stderr.write(`[codex-app] Thread archive failed: ${archiveError}\n`);
    }
  } catch (error) {
    let archived = false;
    let archiveError = null;
    if (client && threadId) {
      try {
        await client.request("thread/archive", { threadId });
        archived = true;
      } catch (archiveFailure) {
        archiveError = archiveFailure.message;
      }
    }
    update({
      status: "failed",
      phase: "failed",
      pid: null,
      completedAt: now(),
      archived,
      archiveError,
      error: error.message
    });
    throw error;
  } finally {
    await closeControlServer(controlServer, socketPath);
    await client?.close().catch(() => {});
  }
}

function resolveJob(jobs, reference, activeOnly = false) {
  const candidates = activeOnly ? jobs.filter((job) => ACTIVE_STATUSES.has(job.status)) : jobs;
  const normalized = String(reference ?? "").trim();
  if (!normalized) {
    if (candidates.length === 1 || !activeOnly) {
      return candidates[0] ?? null;
    }
    throw new Error("Multiple Codex jobs are active. Pass a job id.");
  }
  const matches = candidates.filter((job) => job.id === normalized || job.id.startsWith(normalized));
  if (matches.length > 1) {
    throw new Error(`Job reference ${normalized} is ambiguous.`);
  }
  return matches[0] ?? null;
}

function duration(job) {
  const start = Date.parse(job.startedAt || job.createdAt);
  const end = Date.parse(job.completedAt || now());
  if (!Number.isFinite(start) || !Number.isFinite(end)) {
    return "-";
  }
  const seconds = Math.max(0, Math.round((end - start) / 1000));
  if (seconds < 60) {
    return `${seconds}s`;
  }
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}

function showStatus(reference, env = process.env) {
  const cwd = workspaceRoot();
  const jobs = readJobs(cwd, env).map((job) => {
    if (ACTIVE_STATUSES.has(job.status) && !processIsAlive(job.pid)) {
      return patchJob(cwd, job.id, {
        status: "failed",
        phase: "stale",
        pid: null,
        completedAt: now(),
        error: "Background process exited before updating its job record."
      }, env);
    }
    return job;
  });
  const selected = String(reference ?? "").trim() ? resolveJob(jobs, reference) : null;
  if (selected) {
    process.stdout.write(`${JSON.stringify(selected, null, 2)}\n`);
    return;
  }
  if (jobs.length === 0) {
    process.stdout.write("No codex-app jobs are recorded for this project.\n");
    return;
  }
  process.stdout.write("| Job | Kind | Status | Phase | Duration | Thread | Summary |\n");
  process.stdout.write("| --- | --- | --- | --- | --- | --- | --- |\n");
  for (const job of jobs.slice(0, 20)) {
    const summary = shorten(job.finalMessage || job.error || job.promptPreview, 48).replace(/\|/g, "\\|");
    process.stdout.write(
      `| ${job.id} | ${job.kind} | ${job.status} | ${job.phase || "-"} | ${duration(job)} | ${job.threadId || "-"} | ${summary || "-"} |\n`
    );
  }
}

async function interruptJob(reference, env = process.env) {
  const cwd = workspaceRoot();
  const job = resolveJob(readJobs(cwd, env), reference, true);
  if (!job) {
    throw new Error("No active codex-app job was found for this project.");
  }
  await sendControl(job.controlSocket, { action: "interrupt" });
  process.stdout.write(`Interrupt requested for ${job.id}. Its Codex thread is preserved.\n`);
}

async function steerJob(raw, env = process.env) {
  const [reference, ...promptParts] = tokenizeArguments(raw);
  const prompt = promptParts.join(" ").trim();
  if (!reference || !prompt) {
    throw new Error("Usage: steer <job-id> <prompt>.");
  }

  const cwd = workspaceRoot();
  const job = resolveJob(readJobs(cwd, env), reference, true);
  if (!job) {
    throw new Error(`No active codex-app job matches ${reference}.`);
  }
  await sendControl(job.controlSocket, { action: "steer", prompt });
  process.stdout.write(`Steered ${job.id}. Its original background task remains active.\n`);
}

async function main() {
  const [command, ...parts] = process.argv.slice(2);
  const raw = parts.join(" ").trim();
  switch (command) {
    case "ask":
      if (!raw) {
        throw new Error("A Codex prompt is required.");
      }
      await executeWorker("ask", raw, {}, process.env);
      break;
    case "run": {
      const { prompt, options } = parseRunArguments(raw);
      await executeWorker("run", prompt, options, process.env);
      break;
    }
    case "status":
      showStatus(raw, process.env);
      break;
    case "steer":
      await steerJob(raw, process.env);
      break;
    case "interrupt":
      await interruptJob(raw, process.env);
      break;
    default:
      throw new Error("Usage: codex-app.mjs <ask|run|status|interrupt> [arguments]");
  }
}

const invokedPath = process.argv[1] ? pathToFileURL(path.resolve(process.argv[1])).href : null;
if (invokedPath === import.meta.url) {
  main().catch((error) => {
    process.stderr.write(`codex-app: ${error.message}\n`);
    process.exitCode = 1;
  });
}

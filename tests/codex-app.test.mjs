import assert from "node:assert/strict";
import test from "node:test";

import {
  DesktopThreadFollower,
  applyDesktopStatePatches,
  desktopTurnById,
  parseRunArguments,
  resolveAskThreadByTitle,
  resolveDesktopIpcPath,
  tokenizeArguments
} from "../plugins/codex-app/scripts/codex-app.mjs";

class FakeDesktopIpcClient {
  constructor() {
    this.broadcasts = [];
    this.broadcastHandlers = new Map();
    this.failureHandlers = new Set();
  }

  broadcast(method, params) {
    this.broadcasts.push({ method, params });
  }

  addBroadcastHandler(method, handler) {
    this.broadcastHandlers.set(method, handler);
    return () => this.broadcastHandlers.delete(method);
  }

  addFailureHandler(handler) {
    this.failureHandlers.add(handler);
    return () => this.failureHandlers.delete(handler);
  }

  emit(method, message) {
    this.broadcastHandlers.get(method)?.(message);
  }
}

function stateChange(change, version = 11) {
  return {
    type: "broadcast",
    method: "thread-stream-state-changed",
    sourceClientId: "desktop-owner",
    version,
    params: {
      conversationId: "thread-1",
      hostId: "local",
      change
    }
  };
}

function canonicalState(turn) {
  return {
    id: "thread-1",
    turns: [],
    turnHistory: {
      kind: "canonical",
      history: {
        entitiesByKey: {
          "tail:0:local:turn": turn
        },
        generation: 0,
        isComplete: true,
        islands: []
      }
    }
  };
}

test("argument parsing preserves quoted prompts and run options", () => {
  assert.deepEqual(tokenizeArguments('--model gpt-5.6 "inspect two files"'), [
    "--model",
    "gpt-5.6",
    "inspect two files"
  ]);
  assert.deepEqual(
    parseRunArguments('--effort high --sandbox read-only "inspect two files"'),
    {
      prompt: "inspect two files",
      options: { model: null, effort: "high", sandbox: "read-only" }
    }
  );
});

test("Desktop IPC path uses CODEX_HOME without depending on the app bundle", () => {
  assert.equal(
    resolveDesktopIpcPath({ CODEX_HOME: "/tmp/codex-home" }),
    "/tmp/codex-home/ipc/ipc.sock"
  );
});

test("title lookup asks App Server for the current unarchived cwd", async () => {
  const requests = [];
  const appClient = {
    async request(method, params) {
      requests.push({ method, params });
      return {
        data: [
          { id: "thread-1", name: "Claude handoff", cwd: "/work/project" },
          { id: "thread-2", name: "Claude handoff notes", cwd: "/work/project" }
        ]
      };
    }
  };

  assert.equal(
    await resolveAskThreadByTitle(appClient, "/work/project", {
      CODEX_APP_TITLE: " Claude   handoff "
    }),
    "thread-1"
  );
  assert.deepEqual(requests, [
    {
      method: "thread/list",
      params: {
        cursor: null,
        limit: 25,
        sortKey: "recency_at",
        sortDirection: "desc",
        archived: false,
        cwd: "/work/project",
        searchTerm: "Claude handoff"
      }
    }
  ]);
});

test("title lookup rejects ambiguity within the current cwd", async () => {
  const appClient = {
    async request() {
      return {
        data: [
          { id: "thread-1", name: "Claude handoff", cwd: "/work/project" },
          { id: "thread-2", name: "Claude handoff", cwd: "/work/project" }
        ]
      };
    }
  };

  await assert.rejects(
    resolveAskThreadByTitle(appClient, "/work/project", {
      CODEX_APP_TITLE: "Claude handoff"
    }),
    /did not identify exactly one unarchived Codex task in \/work\/project/
  );
});

test("Desktop state patches apply object and array changes", () => {
  const state = { turn: { status: "inProgress", items: [{ type: "userMessage" }] } };
  const updated = applyDesktopStatePatches(state, [
    { op: "add", path: ["turn", "items", 1], value: { type: "agentMessage", text: "done" } },
    { op: "replace", path: ["turn", "status"], value: "completed" },
    { op: "remove", path: ["turn", "items", 0] }
  ]);

  assert.deepEqual(updated, {
    turn: {
      status: "completed",
      items: [{ type: "agentMessage", text: "done" }]
    }
  });
  assert.throws(
    () => applyDesktopStatePatches({}, [{ op: "add", path: ["__proto__", "polluted"], value: true }]),
    /Unsafe Desktop state patch/
  );
});

test("turn lookup supports canonical and legacy Desktop state", () => {
  const canonicalTurn = { turnId: "turn-1", status: "completed", items: [] };
  assert.equal(desktopTurnById(canonicalState(canonicalTurn), "turn-1"), canonicalTurn);

  const legacyTurn = { id: "turn-2", status: "completed", items: [] };
  assert.equal(desktopTurnById({ turns: [legacyTurn] }, "turn-2"), legacyTurn);
});

test("Desktop follower resolves completion from authoritative state patches", async () => {
  const ipc = new FakeDesktopIpcClient();
  const follower = new DesktopThreadFollower(ipc, "thread-1", { snapshotTimeoutMs: 100 });
  const started = follower.start();

  assert.deepEqual(ipc.broadcasts[0], {
    method: "thread-stream-following-changed",
    params: { conversationId: "thread-1", hostId: "local", following: true }
  });
  ipc.emit(
    "thread-stream-state-changed",
    stateChange({
      type: "snapshot",
      revision: 1,
      conversationState: canonicalState({
        turnId: "turn-1",
        status: "inProgress",
        items: [{ type: "userMessage" }]
      })
    })
  );
  await started;

  const completion = follower.waitForTurn("turn-1");
  ipc.emit(
    "thread-stream-state-changed",
    stateChange({
      type: "patches",
      baseRevision: 1,
      revision: 2,
      patches: [
        {
          op: "add",
          path: [
            "turnHistory",
            "history",
            "entitiesByKey",
            "tail:0:local:turn",
            "items",
            1
          ],
          value: { type: "agentMessage", text: "finished" }
        },
        {
          op: "replace",
          path: [
            "turnHistory",
            "history",
            "entitiesByKey",
            "tail:0:local:turn",
            "status"
          ],
          value: "completed"
        }
      ]
    })
  );

  assert.deepEqual(await completion, {
    turn: {
      turnId: "turn-1",
      status: "completed",
      items: [
        { type: "userMessage" },
        { type: "agentMessage", text: "finished" }
      ]
    },
    turnId: "turn-1",
    finalMessage: "finished"
  });
  await follower.close();
  assert.equal(ipc.broadcasts.at(-1).params.following, false);
});

test("Desktop follower requests a fresh snapshot after a revision gap", async () => {
  const ipc = new FakeDesktopIpcClient();
  const follower = new DesktopThreadFollower(ipc, "thread-1", { snapshotTimeoutMs: 100 });
  const started = follower.start();
  ipc.emit(
    "thread-stream-state-changed",
    stateChange({
      type: "snapshot",
      revision: 4,
      conversationState: canonicalState({ turnId: "turn-1", status: "inProgress", items: [] })
    })
  );
  await started;

  const completion = follower.waitForTurn("turn-1");
  ipc.emit(
    "thread-stream-state-changed",
    stateChange({ type: "patches", baseRevision: 3, revision: 5, patches: [] })
  );
  assert.equal(ipc.broadcasts.length, 2);
  assert.equal(ipc.broadcasts.at(-1).params.following, true);

  ipc.emit(
    "thread-stream-state-changed",
    stateChange({
      type: "snapshot",
      revision: 6,
      conversationState: canonicalState({ turnId: "turn-1", status: "interrupted", items: [] })
    })
  );
  assert.equal((await completion).turn.status, "interrupted");
  await follower.close();
});

test("Desktop follower fails closed on an unknown state protocol version", async () => {
  const ipc = new FakeDesktopIpcClient();
  const follower = new DesktopThreadFollower(ipc, "thread-1", { snapshotTimeoutMs: 100 });
  const started = follower.start();
  ipc.emit(
    "thread-stream-state-changed",
    stateChange(
      {
        type: "snapshot",
        revision: 1,
        conversationState: canonicalState({ turnId: "turn-1", status: "inProgress", items: [] })
      },
      12
    )
  );

  await assert.rejects(started, /Unsupported Codex Desktop thread state protocol version 12/);
  await follower.close();
});

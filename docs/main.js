import init, { WebSim } from "./pkg/san.js";

const TARGET_FPS = 24;
const params = new URLSearchParams(window.location.search);
const DEFAULT_WORD = "codex";
const HERO_WORD = (params.get("text") || DEFAULT_WORD).toLowerCase();

const canvas = document.querySelector("#sand-canvas");
const canvasFrame = document.querySelector(".canvas-frame");
const surfaceButtons = [...document.querySelectorAll("[data-surface-text]")];
const cliHintCode = document.querySelector(".cli-hint code");
const terminalDemo = document.querySelector("#terminal-demo");
const terminalDemoProfile = document.querySelector("#terminal-demo-profile");
const terminalDemoCommand = document.querySelector("#terminal-demo-command");
const terminalDemoLog = document.querySelector("#terminal-demo-log");
const ctx = canvas.getContext("2d", {
  alpha: false,
  desynchronized: true,
});
const backBuffer = document.createElement("canvas");
const backCtx = backBuffer.getContext("2d", { alpha: false });

let sim = null;
let imageData = null;
let rafId = 0;
let lastFrameAt = 0;
let lastWidth = 0;
let lastHeight = 0;
let currentWord = HERO_WORD;
let demoRunId = 0;

function sleep(ms) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

function updateLaunchCommand(word) {
  if (cliHintCode) {
    cliHintCode.textContent = `san ${word} box`;
  }
}

function clearTerminalDemo() {
  terminalDemoCommand.textContent = "";
  terminalDemoLog.replaceChildren();
}

function appendTerminalLog(text, ready = false) {
  const line = document.createElement("div");
  line.className = "terminal-demo__log-line";
  if (ready) {
    line.classList.add("is-ready");
  }
  line.textContent = text;
  terminalDemoLog.append(line);
}

async function playTerminalDemo(word) {
  const runId = ++demoRunId;
  currentWord = word.toLowerCase();
  updateLaunchCommand(currentWord);
  syncButtons(currentWord);

  const url = new URL(window.location.href);
  url.searchParams.set("text", currentWord);
  window.history.replaceState({}, "", url);

  if (!sim) {
    return;
  }

  sim.set_text(currentWord);
  drawFrame();

  canvasFrame.classList.remove("is-live");
  terminalDemo.classList.remove("is-hidden");
  terminalDemoProfile.textContent = currentWord;
  clearTerminalDemo();

  await sleep(180);
  if (runId !== demoRunId) {
    return;
  }

  const command = `san ${currentWord} box`;
  for (let i = 1; i <= command.length; i += 1) {
    terminalDemoCommand.textContent = command.slice(0, i);
    await sleep(i < 4 ? 120 : 68);
    if (runId !== demoRunId) {
      return;
    }
  }

  await sleep(220);
  if (runId !== demoRunId) {
    return;
  }

  const logLines = [
    `[boot] resolving profile: ${currentWord}`,
    `[boot] preparing sand surface`,
    `[boot] binding renderer`,
    `[ready] animation online`,
  ];

  for (const [index, line] of logLines.entries()) {
    appendTerminalLog(line, index === logLines.length - 1);
    await sleep(index === logLines.length - 1 ? 260 : 180);
    if (runId !== demoRunId) {
      return;
    }
  }

  canvasFrame.classList.add("is-live");
  terminalDemo.classList.add("is-hidden");
}

function displaySize() {
  const rect = canvas.getBoundingClientRect();

  return {
    width: Math.max(1, Math.round(rect.width)),
    height: Math.max(1, Math.round(rect.height)),
  };
}

function ensureImageData() {
  const width = sim.width();
  const height = sim.height();

  if (backBuffer.width !== width || backBuffer.height !== height) {
    backBuffer.width = width;
    backBuffer.height = height;
    imageData = backCtx.createImageData(width, height);
  } else if (!imageData || imageData.data.length !== sim.frame_len()) {
    imageData = backCtx.createImageData(width, height);
  }
}

function syncSize(force = false) {
  const { width, height } = displaySize();
  if (!force && width === lastWidth && height === lastHeight) {
    return;
  }

  lastWidth = width;
  lastHeight = height;
  canvas.width = width;
  canvas.height = height;

  if (!sim) {
    sim = new WebSim(width, height, currentWord);
  } else {
    sim.resize(width, height);
  }

  ensureImageData();
}

function drawFrame() {
  sim.step();
  sim.render_frame();
  ensureImageData();

  const frame = sim.frame_rgba();
  imageData.data.set(frame);
  backCtx.putImageData(imageData, 0, 0);

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(backBuffer, 0, 0, canvas.width, canvas.height);
}

function animate(now) {
  rafId = window.requestAnimationFrame(animate);

  if (document.hidden || !sim) {
    return;
  }

  const frameBudget = 1000 / TARGET_FPS;
  if (now - lastFrameAt < frameBudget) {
    return;
  }

  lastFrameAt = now;
  drawFrame();
}

function boot() {
  ctx.imageSmoothingEnabled = false;
  backCtx.imageSmoothingEnabled = false;
  updateLaunchCommand(currentWord);
  syncButtons(currentWord);
  syncSize(true);
  drawFrame();
  playTerminalDemo(currentWord);

  const resizeObserver = new ResizeObserver(() => syncSize());
  resizeObserver.observe(canvas);

  window.addEventListener("resize", () => syncSize());
  document.addEventListener("visibilitychange", () => {
    if (!document.hidden) {
      lastFrameAt = 0;
      syncSize();
    }
  });

  rafId = window.requestAnimationFrame(animate);
}

function syncButtons(word) {
  for (const button of surfaceButtons) {
    const active = button.dataset.surfaceText === word;
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-pressed", String(active));
  }
}

function setSurfaceWord(word) {
  const next = word.toLowerCase();
  if (!sim) {
    currentWord = next;
    updateLaunchCommand(next);
    syncButtons(next);
    return;
  }
  playTerminalDemo(next);
}

async function main() {
  try {
    await init();
    boot();
  } catch (error) {
    window.cancelAnimationFrame(rafId);
    console.error("Failed to initialise the sand surface:", error);
    canvas.parentElement?.setAttribute("data-error", "true");
  }
}

for (const button of surfaceButtons) {
  button.addEventListener("click", () => {
    const word = button.dataset.surfaceText;
    if (word) {
      setSurfaceWord(word);
    }
  });
}

main();

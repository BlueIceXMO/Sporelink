import { chromium } from "playwright-core";

const cdp = process.env.DSH_WEBVIEW_CDP || "http://127.0.0.1:9222";
const outDir = process.env.SMOKE_OUT || "artifacts";

const browser = await chromium.connectOverCDP(cdp);
const context = browser.contexts()[0];
const page = context.pages()[0];
if (!page) throw new Error("no WebView2 page found");

await page.waitForTimeout(1200);
let text = await page.locator("body").innerText();
if (await page.locator(".onboarding-card").count()) {
  await page.locator(".onboarding-head .text-button").click();
  await page.waitForTimeout(500);
  text = await page.locator("body").innerText();
}
if (!(await page.locator(".project-view").count())) {
  await page.getByText("How to Use", { exact: true }).first().click();
  await page.waitForTimeout(1500);
}

const cards = await page.locator(".tree-node").count();
const turnsBefore = await page.locator(".turn").count();
const textarea = page.locator(".composer textarea").last();
await textarea.click();
await textarea.fill("用一个具体例子解释上下文编译");
await textarea.press("Control+Enter");
await page.waitForTimeout(1200);
await page.screenshot({ path: `${outDir}/smoke-streaming.png` });
await page.waitForTimeout(4000);
const turnsAfter = await page.locator(".turn").count();
const messages = await page.locator(".message").count();
const lastTurn = (await page.locator(".turn").last().innerText()).slice(0, 800);
const hasMockReply = lastTurn.includes("Mock 模型") || lastTurn.includes("这是 Mock");
await page.screenshot({ path: `${outDir}/smoke-after-send.png` });

// Context drawer
await page.locator(".composer-chip").first().click();
await page.waitForTimeout(800);
const contextItems = await page.locator(".context-item").count();
await page.screenshot({ path: `${outDir}/smoke-context.png` });
await page.locator(".context-backdrop").click({ position: { x: 5, y: 5 } });

const summary = {
  cards,
  turnsBefore,
  turnsAfter,
  messages,
  contextItems,
  hasMockReply,
  lastTurn,
};
console.log(JSON.stringify(summary, null, 2));

if (turnsAfter <= turnsBefore) throw new Error("no new turn after send");
if (!hasMockReply) throw new Error("mock reply missing");
if (contextItems === 0) throw new Error("context preview empty");

await browser.close();

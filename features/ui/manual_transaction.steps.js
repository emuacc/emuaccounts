const { When, Then } = require("@cucumber/cucumber");

// The shared selenium driver is owned by parse_display.steps.js; steps use it
// via the World (this.driver). "I add a manual transaction with payee ..."
// comes from there.

// Tauri's IPC transport calls the global fetch for every command (see
// search_typing.steps.js), and a response without `Tauri-Response: ok` makes
// the command fail with its body as the error. The answer comes a second
// late, so the step that saves sees the modal close before the failure
// brings it back.
const FAIL_SAVE = `
  const message = arguments[0];
  const realFetch = window.fetch;
  window.fetch = function (input, init) {
    const url = String(input instanceof Request ? input.url : input);
    if (url.split("?")[0].split("/").pop() !== "add_manual_transaction") return realFetch.call(window, input, init);
    return new Promise((resolve) => setTimeout(resolve, 1_000)).then(
      () => new Response(JSON.stringify(message), { headers: { "Content-Type": "application/json" } }),
    );
  };`;

When("saving a manual transaction fails with {string}", async function (message) {
  await this.driver.executeScript(FAIL_SAVE, message);
});

Then("the status line should say {string}", async function (expected) {
  let text = "";
  await this.driver.wait(
    async () => {
      text = await this.driver.executeScript(
        `return document.querySelector('[data-testid="parse-status"]')?.textContent.trim() ?? "";`,
      );
      return text === expected;
    },
    30_000,
    () => `the status line never said "${expected}"; it said "${text}"`,
  );
});

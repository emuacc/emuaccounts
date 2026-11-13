const { After, Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");
const assert = require("node:assert/strict");

// Session setup is owned by parse_display.steps.js BeforeAll/AfterAll; these
// steps consume the driver via the cucumber-js World (this.driver).

const FAMILY_TOKENS = ["--font-ui", "--font-mono"];
const SIZE_TOKENS = ["--text-xs", "--text-sm", "--text-md", "--text-lg", "--text-xl"];
const WEIGHT_TOKENS = ["--weight-regular", "--weight-semibold", "--weight-bold"];

// The parts of the app a scenario checks, as the selectors that contain them.
const REGIONS = {
  "main screen": [".sidebar", ".topbar", ".actionBar", ".tableCard"],
  dialog: [".modalOverlay .modal"],
  report: [".reportContent"],
};

// Every visible element in the region that paints its own text (or is a field
// showing a value or placeholder), with its computed font, and each token
// resolved to a computed value through a probe element. The wordmark (.brand)
// is the logo and is left out.
const REGION_FONTS = `
  const [selectors, families, sizes, weights] = arguments;
  const resolve = (prop, tokens) => tokens.map((token) => {
    const probe = document.createElement('span');
    probe.style.setProperty(prop, 'var(' + token + ')');
    document.body.appendChild(probe);
    const value = getComputedStyle(probe).getPropertyValue(prop);
    probe.remove();
    return value;
  });
  const texts = [];
  for (const selector of selectors) {
    document.querySelectorAll(selector + ' *').forEach((el) => {
      if (el.closest('.brand')) return;
      const isField = el.tagName === 'INPUT' || el.tagName === 'SELECT' || el.tagName === 'TEXTAREA';
      const own = Array.from(el.childNodes).filter((n) => n.nodeType === 3).map((n) => n.textContent).join('').trim();
      const text = isField ? (el.value || el.placeholder || '') : own;
      if (!text) return;
      const rect = el.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) return;
      const cs = getComputedStyle(el);
      texts.push({
        where: selector + ' ' + el.tagName.toLowerCase() + (typeof el.className === 'string' && el.className ? '.' + el.className.split(' ')[0] : ''),
        text: text.slice(0, 30),
        family: cs.fontFamily,
        size: cs.fontSize,
        weight: cs.fontWeight,
      });
    });
  }
  return {
    families: resolve('font-family', families),
    sizes: resolve('font-size', sizes),
    weights: resolve('font-weight', weights),
    texts,
  };`;

async function regionFonts(driver, region) {
  const selectors = REGIONS[region];
  assert.ok(selectors, `unknown region "${region}" (known: ${Object.keys(REGIONS).join(", ")})`);
  const fonts = await driver.executeScript(REGION_FONTS, selectors, FAMILY_TOKENS, SIZE_TOKENS, WEIGHT_TOKENS);
  assert.ok(fonts.texts.length > 5, `expected the ${region} to be populated, found ${fonts.texts.length} text elements`);
  return fonts;
}

function assertAllAllowed(region, texts, allowed, key, what) {
  const bad = texts
    .filter((t) => !allowed.includes(t[key]))
    .map((t) => `${t.where} "${t.text}": ${t[key]}`);
  assert.equal(
    bad.length,
    0,
    `${bad.length} ${region} text element(s) use a ${what} outside [${allowed.join(" | ")}]:\n  ${bad.slice(0, 12).join("\n  ")}`,
  );
}

Then("the {string} text should use only the UI and monospace font families", async function (region) {
  const fonts = await regionFonts(this.driver, region);
  assertAllAllowed(region, fonts.texts, fonts.families, "family", "font family");
});

Then("the {string} text should use only the sizes and weights of the type scale", async function (region) {
  const fonts = await regionFonts(this.driver, region);
  assertAllAllowed(region, fonts.texts, fonts.sizes, "size", "font size");
  assertAllAllowed(region, fonts.texts, fonts.weights, "weight", "font weight");
});

// A @dialog scenario leaves its dialog open; cancel it afterwards, pass or
// fail, so the next scenario starts on the main screen.
After({ tags: "@dialog" }, async function () {
  await this.driver.executeScript(
    `const cancel = Array.from(document.querySelectorAll('.modalOverlay .modal__actions button'))
       .find((b) => b.textContent.trim() === 'Cancel');
     cancel?.click();`,
  );
  await this.driver.wait(
    async () => (await this.driver.findElements(By.css(".modalOverlay"))).length === 0,
    10_000,
    "the dialog never closed",
  );
  await this.driver.wait(until.elementLocated(By.css('[data-testid="app-ready"]')), 5_000);
});

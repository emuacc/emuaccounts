import { describe, expect, it } from "vitest";
import { externaliseLinks, renderPluginReadmePanel, renderPluginReadmeSection, renderPluginSetupButton } from "./plugin-readme";

const md = (text: string) => `<p>${text}</p>`;

describe("renderPluginSetupButton", () => {
  it("renders nothing for a plugin without a README", () => {
    expect(renderPluginSetupButton({ dir_name: "hello-world", has_readme: false }, false)).toBe("");
  });

  it("renders a Setup toggle keyed on the plugin dir name", () => {
    const html = renderPluginSetupButton({ dir_name: "ibkr-flex-sync", has_readme: true }, false);
    expect(html).toContain('data-readme-plugin="ibkr-flex-sync"');
    expect(html).toContain('data-testid="plugin-setup-ibkr-flex-sync"');
    expect(html).toContain('aria-expanded="false"');
    expect(html).toContain(">Setup<");
  });

  it("labels the open state as Close setup", () => {
    const html = renderPluginSetupButton({ dir_name: "ibkr-flex-sync", has_readme: true }, true);
    expect(html).toContain('aria-expanded="true"');
    expect(html).toContain(">Close setup<");
  });

  it("escapes the dir name", () => {
    const html = renderPluginSetupButton({ dir_name: 'x"><script>', has_readme: true }, false);
    expect(html).not.toContain("<script>");
  });
});

describe("renderPluginReadmePanel", () => {
  it("shows a loading state until the README arrives", () => {
    const html = renderPluginReadmePanel("ibkr-flex-sync", undefined, md);
    expect(html).toContain('data-testid="plugin-readme-ibkr-flex-sync"');
    expect(html).toContain("Loading README");
  });

  it("says so when the README is empty", () => {
    expect(renderPluginReadmePanel("p", "  \n", md)).toContain("README is empty");
  });

  it("renders the README through the supplied markdown renderer", () => {
    const html = renderPluginReadmePanel("p", "## Setting up IBKR", (m) => `<h2>${m.slice(3)}</h2>`);
    expect(html).toContain('<div class="pluginReadme__body"><h2>Setting up IBKR</h2></div>');
  });

  it("opens README links outside the app", () => {
    const html = renderPluginReadmePanel("p", "x", () => '<p><a href="https://ibkr.example">docs</a></p>');
    expect(html).toContain('<a target="_blank" rel="noopener noreferrer" href="https://ibkr.example">docs</a>');
  });
});

describe("renderPluginReadmeSection", () => {
  const plugin = { dir_name: "ibkr-flex-sync", has_readme: true };

  it("renders nothing unless this plugin's panel is the open one", () => {
    expect(renderPluginReadmeSection(plugin, undefined, {}, md)).toBe("");
    expect(renderPluginReadmeSection(plugin, "stock-prices", { "ibkr-flex-sync": "x" }, md)).toBe("");
  });

  it("renders the open panel from the cached README, or loading when not cached yet", () => {
    expect(renderPluginReadmeSection(plugin, "ibkr-flex-sync", { "ibkr-flex-sync": "hello" }, md)).toContain("<p>hello</p>");
    expect(renderPluginReadmeSection(plugin, "ibkr-flex-sync", undefined, md)).toContain("Loading README");
  });
});

describe("externaliseLinks", () => {
  it("adds target and rel to anchors that have none, whatever their case", () => {
    expect(externaliseLinks('<a href="/x">a</a> and <A HREF="y">b</A>')).toBe(
      '<a target="_blank" rel="noopener noreferrer" href="/x">a</a> and <a target="_blank" rel="noopener noreferrer" HREF="y">b</A>',
    );
  });

  it("leaves anchors that already declare a target alone", () => {
    const html = '<a target="_self" href="/x">a</a>';
    expect(externaliseLinks(html)).toBe(html);
  });
});

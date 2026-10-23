import { describe, expect, it } from "vitest";
import { buildAddPluginsModalHtml, type AvailablePluginInfo } from "./add-plugins-modal";

const catalog: AvailablePluginInfo[] = [
  {
    dir_name: "stock-prices",
    name: "Stock Yahoo Finance Price Sync",
    version: "0.1.0",
    description: "Fetch daily equity prices.",
    daily: true,
    installed: false,
  },
  {
    dir_name: "hello-world",
    name: "Hello World",
    version: "0.0.1",
    description: null,
    daily: false,
    installed: true,
  },
];

describe("buildAddPluginsModalHtml", () => {
  it("renders name, version and description for each catalog plugin", () => {
    const html = buildAddPluginsModalHtml(catalog, undefined);
    expect(html).toContain("Stock Yahoo Finance Price Sync");
    expect(html).toContain("v0.1.0");
    expect(html).toContain("Fetch daily equity prices.");
    expect(html).toContain("Hello World");
  });

  it("shows a Daily badge only for daily plugins", () => {
    const html = buildAddPluginsModalHtml(catalog, undefined);
    const stockRow = html.slice(html.indexOf("stock-prices"), html.indexOf("hello-world"));
    expect(stockRow).toContain("Daily");
  });

  it("renders an install button for uninstalled plugins and a disabled Installed state otherwise", () => {
    const html = buildAddPluginsModalHtml(catalog, undefined);
    expect(html).toContain('data-install-plugin="stock-prices"');
    expect(html).not.toContain('data-install-plugin="hello-world"');
    expect(html).toContain("Installed");
  });

  it("disables the install button while that plugin is installing", () => {
    const html = buildAddPluginsModalHtml(catalog, "stock-prices");
    const btn = html.slice(html.indexOf('data-install-plugin="stock-prices"') - 200, html.indexOf('data-install-plugin="stock-prices"') + 100);
    expect(btn).toContain("disabled");
  });

  it("renders an empty-catalog message when there are no plugins", () => {
    expect(buildAddPluginsModalHtml([], undefined)).toContain("No plugins available");
  });

  it("escapes HTML in plugin fields", () => {
    const html = buildAddPluginsModalHtml(
      [{ dir_name: "x", name: "<script>", version: "1", description: null, daily: false, installed: false }],
      undefined,
    );
    expect(html).not.toContain("<script>");
    expect(html).toContain("&lt;script&gt;");
  });
});

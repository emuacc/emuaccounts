import { escapeText } from "./text";

/** What the Setup panel needs to know about an installed plugin (a subset
 * of the Rust `plugins::PluginInfo` payload). */
export interface PluginReadmeSource {
  dir_name: string;
  has_readme: boolean;
}

/** The "Setup" toggle for a plugin card. Only plugins whose folder carries a
 * README.md get one; the label flips while the panel is open. */
export function renderPluginSetupButton(plugin: PluginReadmeSource, open: boolean): string {
  if (!plugin.has_readme) return "";
  const name = escapeText(plugin.dir_name);
  return `<button class="btn btn--secondary pluginItem__setup" type="button" data-readme-plugin="${name}" data-testid="plugin-setup-${name}" aria-expanded="${open ? "true" : "false"}">${open ? "Close setup" : "Setup"}</button>`;
}

/** The expanded Setup panel: the plugin's README rendered through the app's
 * markdown renderer, a loading state until it arrives, or a notice when the
 * file is empty. Links open outside the app (the WebView must never navigate
 * away from the UI). */
export function renderPluginReadmePanel(
  dirName: string,
  markdown: string | undefined,
  markdownToHtml: (md: string) => string,
): string {
  let body: string;
  if (markdown === undefined) {
    body = `<div class="pluginReadme__loading">Loading README…</div>`;
  } else if (markdown.trim() === "") {
    body = `<div class="pluginReadme__empty">This plugin's README is empty.</div>`;
  } else {
    body = `<div class="pluginReadme__body">${externaliseLinks(markdownToHtml(markdown))}</div>`;
  }
  return `<div class="pluginReadme" data-testid="plugin-readme-${escapeText(dirName)}">${body}</div>`;
}

/** The Setup section under a plugin card: the README panel while that
 * plugin's panel is the open one, nothing otherwise. */
export function renderPluginReadmeSection(
  plugin: PluginReadmeSource,
  openDirName: string | undefined,
  readmes: Record<string, string> | undefined,
  markdownToHtml: (md: string) => string,
): string {
  if (openDirName !== plugin.dir_name) return "";
  return renderPluginReadmePanel(plugin.dir_name, readmes?.[plugin.dir_name], markdownToHtml);
}

/** Make every anchor open in a new window with no opener. */
export function externaliseLinks(html: string): string {
  return html.replace(/<a\s+(?![^>]*\btarget=)([^>]*)>/gi, '<a target="_blank" rel="noopener noreferrer" $1>');
}

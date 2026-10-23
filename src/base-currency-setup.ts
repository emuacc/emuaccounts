/** Base-currency setup gate — the second step of first-run setup, after the
 * data-folder picker. A vault whose `generated/config.json` has no
 * `base_currency` cannot value holdings, so the app refuses to render the main
 * page until one is chosen. Pure helpers here; Tauri wiring lives in main.ts. */

import { escapeText } from "./render";

export const BASE_CURRENCY_SETUP_TESTID = "base-currency-setup";

/** The gate: `null`/`undefined` means the vault has never been set up. */
export function needsBaseCurrencySetup(vaultBaseCurrency: string | null | undefined): boolean {
  return !vaultBaseCurrency;
}

/** Commodity codes are upper-case by convention (`AUD`, `USD`); the ledger
 * matches them case-sensitively, so normalise what the user typed. */
export function normalizeBaseCurrencyInput(raw: string): string {
  return raw.trim().toUpperCase();
}

/** What the picker should start on: the set's own override when it has one
 * (the user already told us), else the only candidate, else nothing. */
export function preselectBaseCurrency(candidates: string[], setBaseCurrency?: string): string | undefined {
  if (setBaseCurrency && candidates.includes(setBaseCurrency)) return setBaseCurrency;
  if (candidates.length === 1) return candidates[0];
  return undefined;
}

export type BaseCurrencySetupView = {
  appName: string;
  candidates: string[];
  preselected?: string;
  error?: string;
};

/** Full-screen card in the vault-picker style. The select lists the vault's
 * priced currencies; the text input takes anything else. */
export function renderBaseCurrencySetup(view: BaseCurrencySetupView): string {
  const options = view.candidates.map((c) =>
    `<option value="${escapeText(c)}"${c === view.preselected ? " selected" : ""}>${escapeText(c)}</option>`,
  ).join("");
  const hasCandidates = view.candidates.length > 0;
  return `<div class="vaultPicker" data-testid="${BASE_CURRENCY_SETUP_TESTID}">
        <div class="vaultPicker__card">
          <div class="vaultPicker__brand">${escapeText(view.appName)}</div>
          <h1 class="vaultPicker__title">Choose a base currency</h1>
          <p class="vaultPicker__desc">Every balance and report in this data folder is valued in one currency. Pick the one you think in — each account set can override it later in its own <code>config.json</code>.</p>
          ${hasCandidates ? `
            <label class="vaultPicker__field">
              <span class="vaultPicker__fieldLabel">Currencies with prices in this folder</span>
              <select id="baseCurrencySelect" class="vaultPicker__select" data-testid="base-currency-select">
                ${view.preselected ? "" : `<option value="" selected>Select…</option>`}
                ${options}
              </select>
            </label>` : ""}
          <label class="vaultPicker__field">
            <span class="vaultPicker__fieldLabel">${hasCandidates ? "Or type a currency code" : "Currency code"}</span>
            <input id="baseCurrencyInput" class="vaultPicker__input" data-testid="base-currency-input" type="text" maxlength="12" autocapitalize="characters" autocomplete="off" spellcheck="false" placeholder="e.g. AUD" />
          </label>
          ${view.error ? `<p class="vaultPicker__error" data-testid="base-currency-error">${escapeText(view.error)}</p>` : ""}
          <button id="baseCurrencyContinue" class="btn vaultPicker__btn" type="button" data-testid="base-currency-continue">Continue</button>
        </div>
      </div>`;
}

/** Resolve the user's choice from the two inputs: typed text wins over the
 * select so a code not in the list can still be entered. Empty → undefined. */
export function chosenBaseCurrency(selectValue: string | undefined, inputValue: string | undefined): string | undefined {
  const typed = normalizeBaseCurrencyInput(inputValue ?? "");
  if (typed) return typed;
  const picked = normalizeBaseCurrencyInput(selectValue ?? "");
  return picked || undefined;
}

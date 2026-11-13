// One render per frame, however many parts of the app ask for one.
//
// A full render (buildMainHtml + morphdom over the visible rows + sidebar) is
// the most expensive thing the main thread does, and the edit path used to
// run it from several places in the same tick: the optimistic click, the
// mutation queue starting and finishing, the refresh landing. Every caller
// asks the scheduler instead; the first request in a frame schedules the
// render and the rest ride on it. DOM-free so it is unit-tested; main.ts
// supplies the render and the frame.

export type RenderScheduler = {
  /** Render in the next frame (a no-op while one is already due). */
  request(): void;
  /** Render now if one is due, e.g. before measuring the DOM. */
  flush(): void;
  /** Is a render due? */
  readonly pending: boolean;
};

/** How long a render waits for a frame before running anyway. */
export const FRAME_FALLBACK_MS = 50;

/** The next animation frame, or `FRAME_FALLBACK_MS` later, whichever comes
 *  first: a window that gets no frames (hidden, minimised, or driven by
 *  WebDriver, where WebKitGTK delivers none) still renders, so its state and
 *  its DOM never drift apart. */
export function nextFrame(callback: () => void): void {
  let done = false;
  const run = () => {
    if (done) return;
    done = true;
    callback();
  };
  if (typeof requestAnimationFrame === "function") requestAnimationFrame(run);
  setTimeout(run, FRAME_FALLBACK_MS);
}

export function createRenderScheduler(
  render: () => void,
  schedule: (callback: () => void) => void = nextFrame,
): RenderScheduler {
  let pending = false;
  const run = () => {
    if (!pending) return;
    pending = false;
    render();
  };
  return {
    request() {
      if (pending) return;
      pending = true;
      schedule(run);
    },
    flush: run,
    get pending() {
      return pending;
    },
  };
}

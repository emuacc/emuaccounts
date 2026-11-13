import { afterEach, describe, expect, it, vi } from "vitest";
import { FRAME_FALLBACK_MS, createRenderScheduler, nextFrame } from "./render-scheduler";

describe("nextFrame", () => {
  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("runs the callback once, on the frame when one comes", () => {
    vi.useFakeTimers();
    const frames: FrameRequestCallback[] = [];
    vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => { frames.push(cb); return 1; });
    const callback = vi.fn();

    nextFrame(callback);
    frames[0](0);
    vi.advanceTimersByTime(FRAME_FALLBACK_MS * 2);

    expect(callback).toHaveBeenCalledTimes(1);
  });

  it("runs the callback once after the fallback when no frame comes (a hidden or driven window)", () => {
    vi.useFakeTimers();
    vi.stubGlobal("requestAnimationFrame", () => 1);
    const callback = vi.fn();

    nextFrame(callback);
    vi.advanceTimersByTime(FRAME_FALLBACK_MS - 1);
    expect(callback).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(callback).toHaveBeenCalledTimes(1);
  });
});

/** A frame the test fires by hand. */
function manualFrame() {
  const queued: (() => void)[] = [];
  return {
    schedule: (cb: () => void) => { queued.push(cb); },
    fire: () => { const cbs = queued.splice(0); cbs.forEach((cb) => cb()); },
    queued,
  };
}

describe("createRenderScheduler", () => {
  it("renders once per frame however many requests arrive before it", () => {
    const render = vi.fn();
    const frame = manualFrame();
    const scheduler = createRenderScheduler(render, frame.schedule);

    scheduler.request();
    scheduler.request();
    scheduler.request();

    expect(render).not.toHaveBeenCalled();
    expect(frame.queued).toHaveLength(1);
    frame.fire();
    expect(render).toHaveBeenCalledTimes(1);
    expect(scheduler.pending).toBe(false);
  });

  it("schedules a new frame for a request made after the render", () => {
    const render = vi.fn();
    const frame = manualFrame();
    const scheduler = createRenderScheduler(render, frame.schedule);

    scheduler.request();
    frame.fire();
    scheduler.request();
    frame.fire();

    expect(render).toHaveBeenCalledTimes(2);
  });

  it("flush renders a due render at once and is a no-op otherwise", () => {
    const render = vi.fn();
    const frame = manualFrame();
    const scheduler = createRenderScheduler(render, frame.schedule);

    scheduler.flush();
    expect(render).not.toHaveBeenCalled();

    scheduler.request();
    scheduler.flush();
    expect(render).toHaveBeenCalledTimes(1);
    // The frame that was scheduled finds nothing due.
    frame.fire();
    expect(render).toHaveBeenCalledTimes(1);
  });

  it("a request made during a render lands in the next frame", () => {
    const frame = manualFrame();
    let scheduler = createRenderScheduler(() => {}, frame.schedule);
    const render = vi.fn(() => { if (render.mock.calls.length === 1) scheduler.request(); });
    scheduler = createRenderScheduler(render, frame.schedule);

    scheduler.request();
    frame.fire();
    expect(render).toHaveBeenCalledTimes(1);
    expect(scheduler.pending).toBe(true);
    frame.fire();
    expect(render).toHaveBeenCalledTimes(2);
  });
});

// The simulated calculator: the wasm shell, its screen and keypad, sound,
// and the computer's keyboard. Pages put it where they want with
// `mountSimulator` and add their own controls around it.
//
// The wasm shell draws on the canvas with id "incredicalculator", so a page
// can hold one simulator.

import init, * as wasm from '../pkg/incredicalculator_web.js';

// Where each key sits on the shell, as [KeyboardEvent.code, left, top].
const KEY_LAYOUT = [
  ['Backspace', 214, 9],
  ['Slash', 214, 78],
  ['Digit7', 7, 147], ['Digit8', 76, 147], ['Digit9', 145, 147], ['NumpadMultiply', 214, 147],
  ['Digit4', 7, 216], ['Digit5', 76, 216], ['Digit6', 145, 216], ['Minus', 214, 216],
  ['Digit1', 7, 285], ['Digit2', 76, 285], ['Digit3', 145, 285], ['NumpadAdd', 214, 285],
  ['ShiftLeft', 7, 354], ['Digit0', 76, 354], ['ControlLeft', 145, 354], ['Equal', 214, 354],
];

function buildShell() {
  const shell = document.createElement('div');
  shell.className = 'calculator-shell';

  const slot = document.createElement('div');
  slot.className = 'display-slot';
  const canvas = document.createElement('canvas');
  canvas.id = 'incredicalculator';
  canvas.width = 320;
  canvas.height = 240;
  slot.append(canvas);

  const keypad = document.createElement('div');
  keypad.className = 'virtual-keypad';
  keypad.setAttribute('aria-label', 'Calculator keypad');
  for (const [code, left, top] of KEY_LAYOUT) {
    const key = document.createElement('button');
    key.className = 'virtual-key';
    key.type = 'button';
    key.tabIndex = -1;
    key.dataset.code = code;
    key.style.left = `${left}px`;
    key.style.top = `${top}px`;
    keypad.append(key);
  }

  shell.append(slot, keypad);
  return { shell, slot, canvas };
}

/**
 * Builds the calculator inside `container` and starts it.
 *
 * `keyboard` picks where the computer's keys go:
 * - 'document' (the default): every key press on the page reaches the
 *   calculator, for pages that are only the calculator.
 * - 'focus': only while the calculator has focus (click it, or Tab to it),
 *   so the page can have text fields. Held keys are let up when it loses
 *   focus.
 *
 * Resolves to an object with the wasm functions (`wasm`), `requestFrame()`
 * to redraw after changing something from outside, `onLog(callback)` to
 * receive each log line, and the `shell`, `displaySlot` and `canvas`
 * elements.
 */
export async function mountSimulator(container, { keyboard = 'document' } = {}) {
  const { shell, slot, canvas } = buildShell();
  container.append(shell);

  // Frames are only rendered after input, or continuously while the
  // active app asks for realtime updates, to save battery.
  let frameScheduled = false;
  let ready = false;
  const frameListeners = [];
  const logListeners = [];

  function requestFrame() {
    if (ready && !frameScheduled) {
      frameScheduled = true;
      requestAnimationFrame(frame);
    }
  }

  function frame() {
    frameScheduled = false;
    if (wasm.tick()) {
      requestFrame();
    }
    // Notes start and stop during `tick`.
    updateAudio();
    const lines = wasm.take_logs();
    for (const line of lines) {
      for (const listener of logListeners) {
        listener(line);
      }
    }
    for (const listener of frameListeners) {
      listener();
    }
  }

  // Audio is pulled from the shell at 48 kHz mono, like the PC version.
  // The context is suspended whenever the synth is silent, like the RP
  // firmware stopping I2S, so idle pages don't run the audio callback.
  let audioCtx = null;

  function startAudio() {
    if (!ready) {
      return;
    }
    if (!audioCtx) {
      audioCtx = new AudioContext({ sampleRate: 48000 });
      // ScriptProcessorNode is deprecated, but runs on the main thread
      // where the wasm shell lives, which AudioWorklet does not.
      const node = audioCtx.createScriptProcessor(1024, 1, 1);
      node.onaudioprocess = (event) => {
        wasm.fill_audio(event.outputBuffer.getChannelData(0));
        // A pending frame may be about to start a note.
        if (!frameScheduled && !wasm.has_active_audio()) {
          audioCtx.suspend();
        }
      };
      node.connect(audioCtx.destination);
    }
    // Browsers only allow audio to start from a user gesture, so resume
    // from input handlers; `frame` suspends again if nothing plays.
    audioCtx.resume();
  }

  function updateAudio() {
    if (!audioCtx) {
      return;
    }
    if (wasm.has_active_audio()) {
      audioCtx.resume();
    } else if (audioCtx.state === 'running') {
      audioCtx.suspend();
    }
  }

  // Codes of the computer's keys held on the calculator, to let them up
  // when the calculator loses focus.
  const heldCodes = new Set();

  function dispatchKey(code, type) {
    if (!ready) {
      return;
    }
    if (type === 'keydown') {
      heldCodes.add(code);
      wasm.key_down(code);
    } else {
      heldCodes.delete(code);
      wasm.key_up(code);
    }
    startAudio();
    requestFrame();
  }

  // Fills each keycap with its legends from the shared table in core.
  function labelKeys() {
    shell.querySelectorAll('.virtual-key').forEach((button) => {
      const [main = '', shift = '', sup = ''] = wasm.key_legends(button.dataset.code);
      button.replaceChildren(
        ...[['main-label', main], ['shift-label', shift], ['super-label', sup]].map(
          ([className, text]) => {
            const span = document.createElement('span');
            span.className = className;
            span.textContent = text;
            return span;
          },
        ),
      );
    });
  }

  shell.querySelectorAll('.virtual-key').forEach((button) => {
    const code = button.dataset.code;
    let down = false;

    const release = () => {
      if (!down) {
        return;
      }
      down = false;
      button.classList.remove('pressed');
      dispatchKey(code, 'keyup');
    };

    button.addEventListener('pointerdown', (event) => {
      event.preventDefault();
      if (keyboard === 'focus') {
        // preventDefault keeps the click from focusing anything.
        shell.focus({ preventScroll: true });
      }
      down = true;
      button.classList.add('pressed');
      dispatchKey(code, 'keydown');
    });

    button.addEventListener('pointerup', release);
    button.addEventListener('pointerleave', release);
    button.addEventListener('pointercancel', release);
  });

  if (keyboard === 'focus') {
    shell.tabIndex = 0;
    shell.classList.add('focus-keys');
    shell.setAttribute('aria-label', 'Calculator: click here to type on it');
    // Only take the keys the calculator has, so Tab and shortcuts still
    // work for the rest of the page.
    const isCalculatorKey = (event) => event.code && wasm.key_legends(event.code).length > 0;
    shell.addEventListener('keydown', (event) => {
      if (isCalculatorKey(event)) {
        event.preventDefault();
        dispatchKey(event.code, 'keydown');
      }
    });
    shell.addEventListener('keyup', (event) => {
      if (isCalculatorKey(event)) {
        event.preventDefault();
        dispatchKey(event.code, 'keyup');
      }
    });
    shell.addEventListener('focusout', () => {
      for (const code of [...heldCodes]) {
        dispatchKey(code, 'keyup');
      }
    });
  } else {
    document.addEventListener('keydown', (event) => {
      if (event.code) {
        event.preventDefault();
        dispatchKey(event.code, 'keydown');
      }
    });
    document.addEventListener('keyup', (event) => {
      if (event.code) {
        event.preventDefault();
        dispatchKey(event.code, 'keyup');
      }
    });
  }

  await init();
  labelKeys();
  wasm.start();
  ready = true;
  requestFrame();

  return {
    wasm,
    requestFrame,
    /** Calls `callback(line)` for each log line, such as a script's `log`. */
    onLog(callback) {
      logListeners.push(callback);
    },
    /** Calls `callback()` after each frame is drawn. */
    onFrame(callback) {
      frameListeners.push(callback);
    },
    shell,
    displaySlot: slot,
    canvas,
  };
}

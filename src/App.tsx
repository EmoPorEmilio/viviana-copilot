import { createSignal, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [pos, setPos] = createSignal({ x: 0, y: 0 });

  let mounted = true;

  onMount(() => {
    // Poll the Rust backend each animation frame for the current cursor position
    const loop = async () => {
      if (!mounted) return;
      try {
        const res = (await invoke("get_cursor")) as any;
        if (Array.isArray(res) && res.length >= 2) {
          setPos({ x: Number(res[0]), y: Number(res[1]) });
        }
      } catch {
        // swallow errors (backend not ready yet)
      }
      requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);

    // InputCommandManager: listen for H/J/K/L and call move_cursor
    const keyHandler = async (e: KeyboardEvent) => {
      const key = e.key.toLowerCase();
      let dx = 0;
      let dy = 0;
      let handled = true;
      switch (key) {
        case "h":
          dx = -8;
          break;
        case "j":
          dy = 8;
          break;
        case "k":
          dy = -8;
          break;
        case "l":
          dx = 8;
          break;
        default:
          handled = false;
      }
      if (handled) {
        e.preventDefault();
        try {
          const res = (await invoke("move_cursor", { dx, dy })) as any;
          if (Array.isArray(res) && res.length >= 2) {
            setPos({ x: Number(res[0]), y: Number(res[1]) });
          }
        } catch {
          // ignore invoke errors
        }
      }
    };
    window.addEventListener("keydown", keyHandler);
    // store removal for cleanup
    (window as any).__input_command_manager_cleanup = () => window.removeEventListener("keydown", keyHandler);
  });

  onCleanup(() => {
    mounted = false;
    const cleanup = (window as any).__input_command_manager_cleanup as (() => void) | undefined;
    if (cleanup) cleanup();
  });

  return (
    <div>
      <div class="drag-region" />
      <main class="container">
        <div class="coords">{pos().x}, {pos().y}</div>
      </main>
    </div>
  );
}

export default App;

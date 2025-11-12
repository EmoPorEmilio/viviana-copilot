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

    // Keyboard movement is handled by the Rust backend's global key thread.
    // Frontend key handling is intentionally disabled to avoid duplicate moves.
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

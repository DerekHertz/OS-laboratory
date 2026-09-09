import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import "./style.css";

function App() {
  const [left, setLeft] = useState("17");
  const [right, setRight] = useState("25");
  const [status, setStatus] = useState("Loading worker…");
  const worker = useRef<Worker | null>(null);
  useEffect(() => {
    const instance = new Worker(new URL("./worker.ts", import.meta.url), {
      type: "module",
    });
    worker.current = instance;
    instance.onmessage = ({
      data,
    }: MessageEvent<{ result?: number; error?: string }>) =>
      setStatus(
        data.error
          ? "Build probe failed: " + data.error
          : "Wasm result: " + data.result,
      );
    instance.onerror = () => setStatus("Build probe failed: worker error");
    instance.postMessage({ left: 17, right: 25 });
    return () => instance.terminate();
  }, []);
  function run() {
    const values = [left, right].map(Number);
    if (
      values.some(
        (value) => !Number.isInteger(value) || value < 0 || value > 4294967295,
      ) ||
      !left.trim() ||
      !right.trim()
    ) {
      setStatus("Enter two unsigned 32-bit integers.");
      return;
    }
    setStatus("Running build probe…");
    worker.current?.postMessage({ left: values[0], right: values[1] });
  }
  return (
    <main>
      <p className="eyebrow">OS LABORATORY · FOUNDATION</p>
      <h1>A place to explore operating systems</h1>
      <p>
        The laboratory is under construction. This build probe checks the
        browser worker and Rust WebAssembly connection.
      </p>
      <section aria-labelledby="probe-title">
        <h2 id="probe-title">Build connection</h2>
        <p>
          Add two unsigned integers modulo 2³². This is an infrastructure check;
          simulation features will follow.
        </p>
        <label>
          Left operand
          <input
            value={left}
            onChange={(event) => setLeft(event.target.value)}
            inputMode="numeric"
          />
        </label>
        <label>
          Right operand
          <input
            value={right}
            onChange={(event) => setRight(event.target.value)}
            inputMode="numeric"
          />
        </label>
        <button onClick={run}>Run build probe</button>
        <p role="status">{status}</p>
      </section>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<App />);

// Infrastructure-only messages; T02 will define simulation transport.
const worker = self as unknown as {
  onmessage:
    ((event: MessageEvent<{ left: number; right: number }>) => void) | null;
  postMessage: (value: unknown) => void;
};
const modulePromise = (async () => {
  const response = await fetch(
    new URL("./generated/sim_wasm.wasm", import.meta.url),
  );
  if (!response.ok) throw new Error("Wasm fetch failed: " + response.status);
  const { instance } = await WebAssembly.instantiateStreaming(response);
  const probe = instance.exports.build_probe;
  if (typeof probe !== "function")
    throw new Error("Missing Wasm build_probe export");
  return probe as (left: number, right: number) => number;
})();
modulePromise.catch(() => {}); // Report initialization failure through each request.
worker.onmessage = async ({ data }) => {
  try {
    const probe = await modulePromise;
    worker.postMessage({ result: probe(data.left, data.right) >>> 0 });
  } catch (error) {
    worker.postMessage({
      error: error instanceof Error ? error.message : String(error),
    });
  }
};

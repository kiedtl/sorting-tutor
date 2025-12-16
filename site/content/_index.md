+++
title = "Sorting Tutor"
+++

<script type="module">
    import init, * as bindings from '/wasm/sort.js';
    const wasm = await init({ module_or_path: '/wasm/sort_bg.wasm' });
    window.wasmBindings = bindings;
    dispatchEvent(new CustomEvent("TrunkApplicationStarted", {detail: {wasm}}));
</script>

<link crossorigin rel="modulepreload" href="/wasm/sort.js" crossorigin="anonymous">
<link rel="preload" href="/wasm/sort_bg.wasm" crossorigin="anonymous" as="fetch" type="application/wasm">

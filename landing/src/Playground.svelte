<script lang="ts">
  import { onMount } from 'svelte';
  import { Play, Loader2, AlertTriangle, CheckCircle2, XCircle, FileWarning, Copy, Check } from 'lucide-svelte';
  // @ts-ignore — local JS host driver
  import { createFromModule } from './iratxo-core.js';
  import { scenarios } from './scenarios';

  type Runner = {
    compile: (yaml: string) => Uint8Array;
    run: (ir: Uint8Array, input: string) => any;
  };

  let runner: Runner | null = $state(null);
  let engineStatus: 'loading' | 'ready' | 'error' = $state('loading');
  let engineError = $state('');

  let activeId = $state(scenarios[0].id);
  let yaml = $state(scenarios[0].yaml);
  let input = $state(scenarios[0].samples[0].input);
  let activeSampleIdx = $state(0);

  let result: any = $state(null);
  let compileError = $state('');
  let runtimeError = $state('');
  let lastIr: Uint8Array | null = null;
  let scheduled: ReturnType<typeof setTimeout> | null = null;

  let copied = $state(false);
  function copyYaml() {
    navigator.clipboard.writeText(yaml);
    copied = true;
    setTimeout(() => (copied = false), 1200);
  }

  onMount(() => {
    const handler = (e: Event) => {
      const id = (e as CustomEvent<string>).detail;
      if (typeof id === 'string') loadScenario(id);
    };
    window.addEventListener('iratxo:load-scenario', handler);
    return () => window.removeEventListener('iratxo:load-scenario', handler);
  });

  onMount(() => {
    (async () => {
      try {
        const url = new URL('iratxo_engine.wasm', document.baseURI).toString();
        const module = await WebAssembly.compileStreaming(fetch(url));
        runner = await createFromModule(module);
        engineStatus = 'ready';
        evaluate();
      } catch (e: any) {
        engineStatus = 'error';
        engineError = e?.message || String(e);
      }
    })();
  });

  function loadScenario(id: string) {
    const s = scenarios.find((x) => x.id === id);
    if (!s) return;
    activeId = id;
    yaml = s.yaml;
    activeSampleIdx = 0;
    input = s.samples[0].input;
    schedule();
  }

  function loadSample(idx: number) {
    const s = scenarios.find((x) => x.id === activeId);
    if (!s) return;
    activeSampleIdx = idx;
    input = s.samples[idx].input;
    schedule();
  }

  function schedule() {
    if (scheduled) clearTimeout(scheduled);
    scheduled = setTimeout(evaluate, 180);
  }

  function evaluate() {
    if (!runner) return;
    compileError = '';
    runtimeError = '';
    try {
      lastIr = runner.compile(yaml);
    } catch (e: any) {
      compileError = e?.message || String(e);
      result = null;
      return;
    }
    try {
      result = runner.run(lastIr!, input);
    } catch (e: any) {
      runtimeError = e?.message || String(e);
      result = null;
    }
  }

  function classOf(c: string): 'ok' | 'review' | 'blocked' | 'other' {
    if (!c) return 'other';
    if (c === 'ok') return 'ok';
    if (c.startsWith('block') || c === 'needs_lawyer_review' || c === 'rejected') return 'blocked';
    if (c.includes('review') || c.startsWith('contains_') || c.startsWith('route_') || c === 'shadow_rank' || c === 'cancelacion') return 'review';
    return 'other';
  }

  function expectMatched(expected: string): boolean | null {
    if (!result) return null;
    return result.classification === expected;
  }

  $effect(() => {
    // re-run when yaml or input change (after engine ready)
    if (runner && (yaml || input)) schedule();
  });

  let activeScenario = $derived(scenarios.find((s) => s.id === activeId)!);
</script>

<div class="bg-[#13121a] border border-white/5 rounded-xl overflow-hidden shadow-2xl">
  <!-- Scenario tabs -->
  <div class="border-b border-white/5 bg-black/20 px-2 sm:px-3 py-2 flex items-center gap-1.5 overflow-x-auto custom-scrollbar">
    {#each scenarios as s}
      <button
        onclick={() => loadScenario(s.id)}
        class="shrink-0 px-3 py-1.5 rounded-md text-xs font-mono transition-colors whitespace-nowrap {activeId === s.id ? 'bg-[#ff7a18] text-white' : 'text-[#8a8aa0] hover:text-white hover:bg-white/5'}"
      >
        {s.title}
      </button>
    {/each}
  </div>

  <!-- Engine status strip -->
  <div class="px-4 sm:px-5 py-2.5 flex items-center justify-between bg-black/10 border-b border-white/5 text-xs">
    <div class="flex items-center gap-2 font-mono text-[#9090a0]">
      {#if engineStatus === 'loading'}
        <Loader2 size={13} class="animate-spin text-[#ff7a18]" />
        <span>loading wasm engine…</span>
      {:else if engineStatus === 'ready'}
        <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
        <span>engine ready · edits compile automatically</span>
      {:else}
        <AlertTriangle size={13} class="text-red-400" />
        <span class="text-red-300">engine failed: {engineError}</span>
      {/if}
    </div>
    <div class="text-[#7a7a8a] font-mono hidden sm:block">{activeScenario.domain}</div>
  </div>

  <!-- Domain blurb -->
  <div class="px-4 sm:px-5 py-3 border-b border-white/5 text-xs sm:text-sm text-[#a0a0b0] leading-relaxed">
    {activeScenario.blurb}
  </div>

  <!-- Two-column editor + verdict -->
  <div class="grid lg:grid-cols-2 gap-px bg-white/5">
    <!-- Left: YAML + input -->
    <div class="bg-[#13121a] flex flex-col">
      <div class="px-4 sm:px-5 py-2 bg-black/20 border-b border-white/5 flex items-center justify-between">
        <span class="text-[10px] uppercase tracking-wider text-[#7a7a8a] font-semibold">Rule (YAML)</span>
        <button onclick={copyYaml} class="text-[#7a7a8a] hover:text-white text-[10px] flex items-center gap-1.5">
          {#if copied}
            <Check size={11} class="text-emerald-400" /> copied
          {:else}
            <Copy size={11} /> copy
          {/if}
        </button>
      </div>
      <textarea
        bind:value={yaml}
        spellcheck="false"
        class="flex-1 min-h-[260px] sm:min-h-[300px] bg-[#0e0d14] text-[#d6d6e1] font-mono text-[12px] leading-relaxed p-4 outline-none resize-y border-0 focus:bg-[#0c0b12] custom-scrollbar"
      ></textarea>

      <div class="px-4 sm:px-5 py-2 bg-black/20 border-y border-white/5 flex items-center justify-between gap-2 flex-wrap">
        <span class="text-[10px] uppercase tracking-wider text-[#7a7a8a] font-semibold">Input</span>
        <div class="flex flex-wrap gap-1">
          {#each activeScenario.samples as sample, i}
            <button
              onclick={() => loadSample(i)}
              class="text-[10px] px-2 py-0.5 rounded font-mono transition-colors {activeSampleIdx === i ? 'bg-white/10 text-white' : 'text-[#7a7a8a] hover:text-white hover:bg-white/5'}"
            >
              {sample.label}
            </button>
          {/each}
        </div>
      </div>
      <textarea
        bind:value={input}
        spellcheck="false"
        class="min-h-[100px] bg-[#0e0d14] text-[#d6d6e1] font-mono text-[12px] leading-relaxed p-4 outline-none resize-y border-0 focus:bg-[#0c0b12] custom-scrollbar"
      ></textarea>
    </div>

    <!-- Right: verdict -->
    <div class="bg-[#13121a] flex flex-col">
      <div class="px-4 sm:px-5 py-2 bg-black/20 border-b border-white/5 flex items-center justify-between">
        <span class="text-[10px] uppercase tracking-wider text-[#7a7a8a] font-semibold">Verdict</span>
        <button
          onclick={evaluate}
          disabled={engineStatus !== 'ready'}
          class="text-[10px] px-2 py-0.5 rounded font-mono flex items-center gap-1 text-[#ff7a18] hover:text-white hover:bg-[#ff7a18] transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
        >
          <Play size={10} /> run
        </button>
      </div>

      <div class="p-4 sm:p-5 flex-1 overflow-auto custom-scrollbar min-h-[260px]">
        {#if compileError}
          <div class="bg-red-500/10 border border-red-500/30 rounded-lg p-3 sm:p-4">
            <div class="flex items-center gap-2 text-red-300 font-semibold text-xs mb-2">
              <FileWarning size={14} /> Compile error
            </div>
            <pre class="font-mono text-[11px] text-red-200 whitespace-pre-wrap break-words">{compileError}</pre>
          </div>
        {:else if runtimeError}
          <div class="bg-red-500/10 border border-red-500/30 rounded-lg p-3 sm:p-4">
            <div class="flex items-center gap-2 text-red-300 font-semibold text-xs mb-2">
              <AlertTriangle size={14} /> Runtime error
            </div>
            <pre class="font-mono text-[11px] text-red-200 whitespace-pre-wrap break-words">{runtimeError}</pre>
          </div>
        {:else if result}
          {@const cls = classOf(result.classification)}
          <div class="flex items-center flex-wrap gap-2 mb-3">
            <span
              class="px-3 py-1 rounded-md font-mono text-xs font-bold"
              class:bg-emerald-500={cls === 'ok'}
              class:text-emerald-50={cls === 'ok'}
              class:bg-amber-500={cls === 'review'}
              class:text-amber-50={cls === 'review'}
              class:bg-red-500={cls === 'blocked'}
              class:text-red-50={cls === 'blocked'}
              class:bg-slate-500={cls === 'other'}
              class:text-slate-50={cls === 'other'}
            >
              {result.classification}
            </span>
            <span class="text-[#9090a0] font-mono text-xs">confidence {result.confidence?.toFixed?.(2) ?? result.confidence}</span>
            {#if expectMatched(activeScenario.samples[activeSampleIdx].expect) === true}
              <span class="ml-auto inline-flex items-center gap-1 text-[10px] text-emerald-400 font-mono">
                <CheckCircle2 size={11} /> matches expected
              </span>
            {:else if expectMatched(activeScenario.samples[activeSampleIdx].expect) === false}
              <span class="ml-auto inline-flex items-center gap-1 text-[10px] text-amber-400 font-mono">
                <XCircle size={11} /> expected {activeScenario.samples[activeSampleIdx].expect}
              </span>
            {/if}
          </div>

          {#if result.triggered && result.triggered.length}
            <div class="space-y-2 mb-4">
              {#each result.triggered as t}
                <div class="bg-black/20 border-l-2 border-[#ff7a18] rounded-r-md p-3">
                  <div class="font-mono text-xs text-white font-semibold">
                    {t.id}
                    <span class="text-[#9090a0] font-normal"> → {t.classification} · {t.confidence?.toFixed?.(2) ?? t.confidence}</span>
                  </div>
                  {#if t.explanation}
                    <div class="text-[11px] text-[#a0a0b0] italic mt-1">{t.explanation}</div>
                  {/if}
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-[#7a7a8a] text-xs italic">No rules triggered — falling back to default.</div>
          {/if}

          <details class="mt-4">
            <summary class="cursor-pointer text-[10px] uppercase tracking-wider text-[#7a7a8a] hover:text-white">
              Raw JSON
            </summary>
            <pre class="mt-2 bg-black/30 border border-white/5 rounded-md p-3 font-mono text-[11px] text-[#9090a0] overflow-x-auto whitespace-pre-wrap break-words">{JSON.stringify(result, null, 2)}</pre>
          </details>
        {:else if engineStatus === 'ready'}
          <div class="text-[#7a7a8a] text-xs italic">Waiting for input…</div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Why-this-scenario footer -->
  <div class="px-4 sm:px-5 py-3 border-t border-white/5 bg-black/10 text-xs text-[#8a8aa0] leading-relaxed">
    <span class="text-[#ff7a18] font-semibold mr-1">Why it matters:</span>{activeScenario.why}
  </div>
</div>

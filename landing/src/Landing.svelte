<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Zap, GitBranch, Copy, Check, ArrowRight, ChevronDown, Settings,
    TerminalSquare, Terminal, Code, FileText, Lock, Sparkles, Layers,
    ShieldCheck, Workflow, BookOpen, FileCheck2, KeyRound, Globe, Cpu,
    Boxes, FlaskConical, GitPullRequest, Bot, MessageSquare, Scale,
    Filter, Languages, Cloud, Server, MonitorSmartphone, Package
  } from 'lucide-svelte';
  import Playground from './Playground.svelte';
  import { scenarios } from './scenarios';

  // ---------------- Animated text utils ----------------
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%&*+<>{}[]_-\\/';
  function scrambleTo(target: string, progress: number) {
    return target.split('').map((ch, i) => {
      if (ch === ' ') return ' ';
      if (progress > i) return ch;
      return chars[Math.floor(Math.random() * chars.length)];
    }).join('');
  }
  function shuffleWords(text: string) {
    const words = text.split(' ');
    for (let i = words.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [words[i], words[j]] = [words[j], words[i]];
    }
    return words.join(' ');
  }

  let heroTitle = $state('');
  let heroSubtitle = $state('');
  const finalTitle = 'Rules as code,';
  const finalSubtitle = 'shipped as Wasm.';

  onMount(() => {
    let shuffles = 0;
    const maxShuffles = 8;
    const shuffleInterval = setInterval(() => {
      heroSubtitle = shuffleWords(finalSubtitle);
      shuffles++;
      if (shuffles >= maxShuffles) {
        clearInterval(shuffleInterval);
        heroSubtitle = finalSubtitle;
      }
    }, 80);

    let t = 0;
    const titleInterval = setInterval(() => {
      heroTitle = scrambleTo(finalTitle, t);
      t += 0.4;
      if (t >= finalTitle.length + 2) {
        clearInterval(titleInterval);
        heroTitle = finalTitle;
      }
    }, 40);
  });

  let visibleSections = $state<Record<string, boolean>>({});
  function observeSection(id: string) {
    const el = document.getElementById(id);
    if (!el) return;
    const io = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          visibleSections[id] = true;
          io.disconnect();
        }
      });
    }, { threshold: 0.2 });
    io.observe(el);
  }
  onMount(() => {
    ['what', 'how', 'features', 'examples', 'playground', 'use-cases', 'quickstart', 'dsl', 'docs'].forEach(observeSection);
  });

  function useScramble(finalText: string, activeFn: () => boolean) {
    let text = $state('');
    $effect(() => {
      if (!activeFn()) { text = finalText; return; }
      let t = 0;
      const interval = setInterval(() => {
        text = scrambleTo(finalText, t);
        t += 0.5;
        if (t >= finalText.length + 2) {
          clearInterval(interval);
          text = finalText;
        }
      }, 35);
      return () => clearInterval(interval);
    });
    return () => text;
  }

  const whatText      = useScramble('What is Iratxo?',           () => !!visibleSections['what']);
  const howText       = useScramble('How it works',              () => !!visibleSections['how']);
  const featuresText  = useScramble('Built for portability',     () => !!visibleSections['features']);
  const examplesText  = useScramble('Real-world rule packs',     () => !!visibleSections['examples']);
  const playText      = useScramble('Try it live',               () => !!visibleSections['playground']);
  const useCasesText  = useScramble('Where it fits',             () => !!visibleSections['use-cases']);
  const quickstartText= useScramble('Quick start',               () => !!visibleSections['quickstart']);
  const dslText       = useScramble('The DSL',                   () => !!visibleSections['dsl']);
  const docsText      = useScramble('Documentation',             () => !!visibleSections['docs']);

  // ---------------- Hero terminal animation ----------------
  let visibleLines = $state(0);
  type Line = { text: string; delay: number; success?: boolean; warn?: boolean };
  const terminalLines: Line[] = [
    { text: '> iratxo build outbound_compliance.yaml', delay: 0 },
    { text: '[iratxo] compiled 4 rules · IR v1 · 2.1 KB', delay: 700, success: true },
    { text: '> iratxo run outbound_compliance.iratxo input.txt', delay: 1500 },
    { text: '{"classification":"blocked","confidence":0.99,', delay: 2300 },
    { text: ' "triggered":[{"id":"prohibited_claim","why":"FDA approved"}]}', delay: 2700 },
    { text: '> iratxo sign outbound_compliance.iratxo --key iratxo.sk', delay: 3500 },
    { text: '[iratxo] ed25519 signature → outbound_compliance.iratxo.sig', delay: 4200, success: true },
  ];
  onMount(() => {
    terminalLines.forEach((line, index) => {
      setTimeout(() => { visibleLines = index + 1; }, line.delay);
    });
  });

  // ---------------- Background glyphs ----------------
  const glyphs = Array.from({ length: 30 }, () => ({
    char: chars[Math.floor(Math.random() * chars.length)],
    left: Math.random() * 100,
    top: Math.random() * 100,
    delay: Math.random() * 5,
    duration: 4 + Math.random() * 6,
    size: 10 + Math.floor(Math.random() * 18),
  }));

  // ---------------- Copy utilities ----------------
  let copiedCmd = $state<string | null>(null);
  function copyCmd(cmd: string) {
    navigator.clipboard.writeText(cmd);
    copiedCmd = cmd;
    setTimeout(() => (copiedCmd = null), 1500);
  }

  // ---------------- Accordion state ----------------
  let openSection = $state<string | null>('architecture');
  function toggleSection(id: string) {
    openSection = openSection === id ? null : id;
  }

  // ---------------- Hover scramble ----------------
  function hoverScramble(node: HTMLElement, finalText: string) {
    let interval: ReturnType<typeof setInterval> | null = null;
    const enter = () => {
      let t = 0;
      interval = setInterval(() => {
        node.textContent = scrambleTo(finalText, t);
        t += 0.6;
        if (t >= finalText.length + 2) {
          if (interval) clearInterval(interval);
          node.textContent = finalText;
        }
      }, 30);
    };
    const leave = () => {
      if (interval) clearInterval(interval);
      node.textContent = finalText;
    };
    node.addEventListener('mouseenter', enter);
    node.addEventListener('mouseleave', leave);
    return {
      destroy() {
        node.removeEventListener('mouseenter', enter);
        node.removeEventListener('mouseleave', leave);
        if (interval) clearInterval(interval);
      }
    };
  }

  // ---------------- Predicate table ----------------
  const predicates = [
    { name: 'contains_any / contains_all / not_contains_any', what: 'Substring membership (case toggle).' },
    { name: 'word_contains_any',           what: 'Whole-word match using Unicode word boundaries.' },
    { name: 'starts_with_any / ends_with_any', what: 'Prefix / suffix match against trimmed input.' },
    { name: 'regex',                       what: 'Cached per-evaluation Rust regex match.' },
    { name: 'min_length / max_length',     what: 'Token-count bounds.' },
    { name: 'paragraphs: { min, max }',    what: 'Paragraph-count bounds.' },
    { name: 'sentences: { min, max }',     what: 'Sentence-count bounds (split on . ! ?).' },
    { name: 'chars: { min, max }',         what: 'Unicode scalar character-count bounds.' },
    { name: 'lines: { min, max }',         what: 'Line-count bounds (split on \\n).' },
    { name: 'max_words_per_sentence',      what: 'Readability gate.' },
    { name: 'has_section: [...]',          what: 'Markdown # heading or HTML <h1..h6>.' },
    { name: 'has_entity: { kind, min_count }', what: 'email · phone · url · currency · ip_address · credit_card · iban · date_iso · hashtag · mention · emoji.' },
    { name: 'has_url_to_domain',           what: 'At least one URL host matches an allowlist; compose with not for denylist.' },
    { name: 'language_is: [...]',          what: 'Detected language ∈ codes (en, es, ca, eu).' },
    { name: 'mostly_uppercase',            what: 'Fraction of letters that are uppercase ≥ min_ratio.' },
    { name: 'digit_ratio_above',           what: 'Fraction of non-whitespace chars that are digits ≥ min_ratio.' },
    { name: 'punctuation_ratio_above',     what: 'Fraction of non-whitespace chars that are ASCII punctuation ≥ min_ratio.' },
    { name: 'token_entropy_above',         what: 'High Shannon-entropy token — catches API keys & secrets without known prefixes.' },
    { name: 'repeated_char_run',           what: 'Consecutive identical chars (soooooo, !!!!!).' },
    { name: 'repeated_token',              what: 'Same non-stopword token appears ≥ N times (spam / copy-paste).' },
    { name: 'type_token_ratio_below',      what: 'Low lexical diversity (unique / total tokens).' },
    { name: 'has_invisible_chars',         what: 'Zero-width / BOM-style chars — homoglyph / phishing defense.' },
    { name: 'has_mixed_script_token',      what: 'Token mixing Unicode scripts (e.g. Cyrillic а inside Latin PayPal).' },
    { name: 'script_is: [...]',            what: 'Input contains chars from listed scripts (latin, cyrillic, greek, han, hiragana, katakana, hangul, arabic, hebrew, devanagari, thai).' },
    { name: 'semantic_match',              what: 'Hashing-trick + per-language stem + synonym dict (en/es/ca/eu).' },
    { name: 'all / any / not / always',    what: 'Combinators with short-circuit evaluation.' },
  ];

  const ROADMAP_DONE = [
    'Source-mapped errors', 'IR versioning', 'Regex caching', 'Entity / structural / language predicates',
    'Rule chaining', 'Browser + Node + Workers hosts', 'iratxo test', 'Ed25519 signing',
    'Criterion benchmarks', 'No-panic property tests', 'Built-in es/eu synonym dicts',
    'Legal-termination domain pack', 'Web playground',
  ];
  const ROADMAP_OPEN = [
    'VS Code extension', 'Observability dashboard', 'Full domain-pack catalog',
    'Per-rule wasm compilation (if a use case emerges)',
  ];
</script>

<div class="relative min-h-screen text-[#e6e6eb] overflow-x-hidden font-sans bg-[#0b0a10]">
  <!-- Floating glyphs -->
  <div class="fixed inset-0 pointer-events-none z-0 overflow-hidden hidden sm:block">
    {#each glyphs as g, i (i)}
      <div
        class="absolute text-[#ff7a18]/10 font-mono select-none"
        style="left: {g.left}%; top: {g.top}%; font-size: {g.size}px; animation: float {g.duration}s ease-in-out infinite; animation-delay: -{g.delay}s;"
      >{g.char}</div>
    {/each}
  </div>

  <!-- Background orbs -->
  <div class="fixed inset-0 pointer-events-none z-0 overflow-hidden opacity-50">
    <div class="absolute top-[10%] left-[15%] w-[35vw] h-[35vw] rounded-full bg-[#ff7a18] blur-[120px] opacity-20 animate-float"></div>
    <div class="absolute top-[35%] right-[10%] w-[40vw] h-[40vw] rounded-full bg-[#7c3aed] blur-[140px] opacity-15 animate-float" style="animation-delay: -4s;"></div>
    <div class="absolute -bottom-[10%] left-[30%] w-[45vw] h-[45vw] rounded-full bg-[#22d3ee] blur-[120px] opacity-10 animate-float" style="animation-delay: -2s;"></div>
  </div>

  <!-- Nav -->
  <nav class="relative z-20 flex justify-between items-center px-4 sm:px-6 md:px-12 py-5 sm:py-6 bg-transparent">
    <div class="flex items-center gap-2 sm:gap-3 group cursor-pointer">
      <Zap class="text-[#ff7a18] group-hover:rotate-12 transition-transform duration-300" size={24} />
      <span class="text-lg sm:text-xl font-bold text-white tracking-tight group-hover:tracking-widest transition-all duration-300">Iratxo</span>
    </div>
    <div class="flex items-center gap-2 sm:gap-3">
      <a href="#examples"   class="hidden md:inline-flex px-3 py-2 text-sm text-[#a0a0b0] hover:text-white transition-colors">Examples</a>
      <a href="#playground" class="hidden md:inline-flex px-3 py-2 text-sm text-[#a0a0b0] hover:text-white transition-colors">Playground</a>
      <a href="#quickstart" class="hidden sm:inline-flex px-3 py-2 text-sm text-[#a0a0b0] hover:text-white transition-colors">Quick start</a>
      <a href="#docs"       class="hidden sm:inline-flex px-3 py-2 text-sm text-[#a0a0b0] hover:text-white transition-colors">Docs</a>
      <a href="https://github.com/enekos/iratxo" target="_blank" rel="noopener" class="px-3 sm:px-4 py-2 border border-white/10 rounded-full text-white hover:bg-white/10 hover:border-[#ff7a18]/50 transition-all duration-300 text-xs sm:text-sm flex items-center gap-2">
        <GitBranch size={14} /> GitHub
      </a>
    </div>
  </nav>

  <!-- Hero -->
  <main class="relative z-10 max-w-6xl mx-auto px-4 sm:px-6 md:px-12 pt-10 sm:pt-16 pb-16 sm:pb-24">
    <div class="flex flex-col lg:flex-row items-center justify-between gap-10 lg:gap-16">
      <div class="flex-1 space-y-5 sm:space-y-6 text-center lg:text-left">
        <div class="inline-flex items-center gap-2 px-3 py-1 border border-[#ff7a18]/30 rounded-full bg-[#ff7a18]/10 text-[#ff7a18] text-xs font-semibold tracking-wide uppercase">
          <Zap size={12} /> Open source · Wasm · Deterministic
        </div>
        <h1 class="text-[2.25rem] sm:text-5xl md:text-6xl font-bold leading-[1.05] text-white">
          <span class="inline-block min-w-[7ch]">{heroTitle}</span> <br/>
          <span class="bg-clip-text text-transparent bg-gradient-to-r from-[#ff7a18] to-[#7c3aed]">{heroSubtitle}</span>
        </h1>
        <p class="text-base sm:text-lg text-[#a0a0b0] max-w-xl mx-auto lg:mx-0 leading-relaxed">
          Iratxo is an executable rule engine. Author rules in YAML, compile them to a portable IR, run them deterministically inside a single 1.2&nbsp;MB Wasm module — same artifact in browsers, Node, Cloudflare Workers, and your backend.
        </p>
        <p class="text-sm text-[#7a7a8a] max-w-xl mx-auto lg:mx-0 leading-relaxed">
          Pure function. No I/O, no time, no RNG. Sign the rule pack, ship the bytes, and every host evaluates the same input identically.
        </p>
        <div class="flex flex-col sm:flex-row gap-3 sm:gap-4 pt-2 justify-center lg:justify-start">
          <a href="#playground" class="px-6 py-3 bg-[#ff7a18] text-white rounded-full font-semibold text-sm transition-all hover:shadow-[0_0_25px_rgba(255,122,24,0.4)] hover:-translate-y-0.5 flex items-center justify-center gap-2">
            Try the playground <ArrowRight size={16} />
          </a>
          <button onclick={() => copyCmd('cargo install iratxo-cli')} class="px-4 sm:px-5 py-3 border border-white/10 rounded-full text-[#a0a0b0] text-xs sm:text-sm flex items-center justify-center gap-2 bg-white/5 hover:bg-white/10 hover:border-white/20 transition-all font-mono max-w-full overflow-hidden">
            <span class="text-[#ff7a18] shrink-0">$</span>
            <span class="truncate">cargo install iratxo-cli</span>
            {#if copiedCmd === 'cargo install iratxo-cli'}
              <Check size={14} class="text-emerald-400 shrink-0" />
            {:else}
              <Copy size={14} class="shrink-0" />
            {/if}
          </button>
        </div>

        <div class="flex flex-wrap gap-x-5 gap-y-1 pt-3 text-xs text-[#7a7a8a] justify-center lg:justify-start">
          <span class="flex items-center gap-1.5"><Check size={12} class="text-emerald-400" /> 57 Rust tests</span>
          <span class="flex items-center gap-1.5"><Check size={12} class="text-emerald-400" /> 7 JS host tests</span>
          <span class="flex items-center gap-1.5"><Check size={12} class="text-emerald-400" /> 850-row Basque parity test</span>
          <span class="flex items-center gap-1.5"><Check size={12} class="text-emerald-400" /> 134 µs keyword pack</span>
        </div>
      </div>

      <div class="flex-1 w-full max-w-lg relative">
        <div class="rounded-xl bg-[#121018]/90 backdrop-blur-xl border border-white/5 overflow-hidden shadow-2xl hover:shadow-[0_0_40px_rgba(255,122,24,0.15)] transition-shadow duration-500">
          <div class="flex items-center px-4 py-3 bg-black/30 border-b border-white/5">
            <div class="flex space-x-2">
              <div class="w-3 h-3 rounded-full bg-[#ff7a18]/80"></div>
              <div class="w-3 h-3 rounded-full bg-white/20"></div>
              <div class="w-3 h-3 rounded-full bg-white/20"></div>
            </div>
            <div class="mx-auto text-xs text-[#7a7a8a] font-mono">iratxo</div>
          </div>
          <div class="p-4 sm:p-5 font-mono text-[11px] sm:text-[12px] h-72 sm:h-80 overflow-y-auto leading-relaxed custom-scrollbar">
            {#each terminalLines.slice(0, visibleLines) as line}
              <div class="mb-1.5 terminal-line">
                {#if line.text.startsWith('>')}
                  <span class="text-[#ff7a18]">~</span> <span class="text-white">{line.text.substring(2)}</span>
                {:else if line.success}
                  <span class="text-emerald-300">{line.text}</span>
                {:else if line.text.startsWith('{') || line.text.startsWith(' ')}
                  <span class="text-[#9090a0]">{line.text}</span>
                {:else}
                  <span class="text-[#7a7a8a]">{line.text}</span>
                {/if}
              </div>
            {/each}
            <div class="flex items-center text-white mt-1">
              <span class="text-[#ff7a18]">~</span>&nbsp;<span class="animate-pulse">█</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </main>

  <!-- What is Iratxo? -->
  <section id="what" class="relative z-10 py-16 sm:py-20 border-y border-white/5 bg-[#0f0e14]">
    <div class="max-w-4xl mx-auto px-4 sm:px-6">
      <div class="text-center mb-10 sm:mb-14">
        <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-3 min-h-[1.2em]">{whatText()}</h2>
        <p class="text-[#a0a0b0] max-w-2xl mx-auto text-sm sm:text-base">A short, no-jargon explanation.</p>
      </div>

      <div class="grid sm:grid-cols-2 gap-5 sm:gap-6">
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <h3 class="text-white font-semibold mb-2 flex items-center gap-2"><Filter size={18} class="text-[#ff7a18]" /> The problem</h3>
          <p class="text-[#a0a0b0] text-sm leading-relaxed">
            Every product ends up with a tangle of "if the message contains X, route to Y" logic — content moderation, compliance gates, support routing, agent guardrails. It lives in code (so changes mean a deploy), or in a SaaS rules engine (so it leaks data), or in an LLM (so it drifts).
          </p>
        </div>
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <h3 class="text-white font-semibold mb-2 flex items-center gap-2"><Sparkles size={18} class="text-[#7c3aed]" /> The fix</h3>
          <p class="text-[#a0a0b0] text-sm leading-relaxed">
            Iratxo treats rules as <strong class="text-white">data</strong>: write them in YAML, compile to a versioned binary, sign with Ed25519, ship to every runtime that can host Wasm. Pure function — same input, same verdict, every time.
          </p>
        </div>
      </div>

      <div class="mt-6 sm:mt-8 bg-gradient-to-br from-[#ff7a18]/10 to-[#7c3aed]/10 border border-white/10 rounded-xl p-5 sm:p-6 text-center">
        <p class="text-[#d0d0e0] text-sm sm:text-base leading-relaxed max-w-2xl mx-auto">
          Author <code class="text-[#ff7a18]">forbidden_phrases.yaml</code>, hand a single <code class="text-[#7c3aed]">.iratxo</code> file to your browser SDK, your API gateway, and your Cloudflare Worker. They all run the same bytes, return the same JSON verdict, and carry the same signature.
        </p>
      </div>
    </div>
  </section>

  <!-- How it works -->
  <section id="how" class="relative z-10 py-16 sm:py-20">
    <div class="max-w-5xl mx-auto px-4 sm:px-6">
      <div class="text-center mb-10 sm:mb-14">
        <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-3 min-h-[1.2em]">{howText()}</h2>
        <p class="text-[#a0a0b0] max-w-2xl mx-auto text-sm sm:text-base">YAML in. Verdict out. Three steps, no infrastructure.</p>
      </div>

      <div class="grid md:grid-cols-3 gap-4 sm:gap-6">
        <div class="relative bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <div class="absolute -top-3 -left-3 w-8 h-8 rounded-full bg-[#ff7a18] text-white font-bold text-sm flex items-center justify-center shadow-lg">1</div>
          <FileText class="text-[#ff7a18] mb-3" size={26} />
          <h3 class="text-white font-semibold mb-2">Author YAML</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Compose <code>contains_any</code>, <code>regex</code>, <code>semantic_match</code>, structural and entity predicates with <code>any/all/not</code> combinators. Chain follow-up rules with <code>then</code>.</p>
        </div>
        <div class="relative bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <div class="absolute -top-3 -left-3 w-8 h-8 rounded-full bg-[#7c3aed] text-white font-bold text-sm flex items-center justify-center shadow-lg">2</div>
          <Boxes class="text-[#7c3aed] mb-3" size={26} />
          <h3 class="text-white font-semibold mb-2">Compile to IR</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Versioned binary (<code>IRTX</code> magic + bincode payload). Sign it with Ed25519, ship it as an immutable artifact alongside your container or to a CDN.</p>
        </div>
        <div class="relative bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <div class="absolute -top-3 -left-3 w-8 h-8 rounded-full bg-[#22d3ee] text-white font-bold text-sm flex items-center justify-center shadow-lg">3</div>
          <Workflow class="text-[#22d3ee] mb-3" size={26} />
          <h3 class="text-white font-semibold mb-2">Evaluate anywhere</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">One Wasm engine. JS hosts for browser, Node, and Workers wrap the same module. Or call the ABI directly from <code>wasmtime</code>, <code>wasmer</code>, Go, Python — anything that can read linear memory.</p>
        </div>
      </div>
    </div>
  </section>

  <!-- Features -->
  <section id="features" class="relative z-10 py-16 sm:py-20 border-y border-white/5 bg-[#0f0e14]">
    <div class="max-w-6xl mx-auto px-4 sm:px-6">
      <div class="text-center mb-10 sm:mb-14">
        <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-3 min-h-[1.2em]">{featuresText()}</h2>
        <p class="text-[#a0a0b0] max-w-2xl mx-auto text-sm sm:text-base">A small, hardened core. The same engine wherever your code runs.</p>
      </div>
      <div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#ff7a18]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(255,122,24,0.1)]">
          <Cpu class="text-[#ff7a18] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Single Wasm module'}>Single Wasm module</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">~1.2 MB <code>cdylib</code>. Browser, Node, Workers, or any <code>wasmtime</code>/<code>wasmer</code> host all run the same bytes through the same ABI.</p>
        </div>
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#7c3aed]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(124,58,237,0.1)]">
          <ShieldCheck class="text-[#7c3aed] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Pure & deterministic'}>Pure &amp; deterministic</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">No I/O, no time, no RNG. Same input always produces the same verdict. No-panic property tests + 850-row parity test against the Basque stemmer reference.</p>
        </div>
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#ff7a18]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(255,122,24,0.1)]">
          <KeyRound class="text-[#ff7a18] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Signed rule packs'}>Signed rule packs</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Ed25519 signatures in a sidecar <code>.iratxo.sig</code>. Tampered or wrong-version blobs fail loudly instead of silently misbehaving.</p>
        </div>
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#7c3aed]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(124,58,237,0.1)]">
          <Languages class="text-[#7c3aed] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Semantic match'}>Semantic match</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Hashing-trick token vectors over per-language stems and a built-in synonym dictionary. Auto-detects English, Spanish, Catalan, Basque. No model file, no network.</p>
        </div>
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#ff7a18]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(255,122,24,0.1)]">
          <Zap class="text-[#ff7a18] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Fast'}>Fast</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Keyword pack 134 µs · 6-regex pack over 5&nbsp;KB doc 246 µs · semantic match 3.5 µs (M-series Mac, release build). Per-evaluation regex cache.</p>
        </div>
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#7c3aed]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(124,58,237,0.1)]">
          <Layers class="text-[#7c3aed] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Versioned IR'}>Versioned IR</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Compiled artifacts start with <code>IRTX</code> magic + a 16-bit version. Old bytes meeting a new engine fail with a clear error, never with corrupted behavior.</p>
        </div>
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#ff7a18]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(255,122,24,0.1)]">
          <FileCheck2 class="text-[#ff7a18] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Golden-file tests'}>Golden-file tests</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Express expectations as YAML cases. <code>iratxo test</code> exits 1 on any failure — drop it straight into CI alongside your other linters.</p>
        </div>
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#7c3aed]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(124,58,237,0.1)]">
          <Code class="text-[#7c3aed] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Tiny ABI'}>Tiny ABI</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Three exports: <code>iratxo_alloc</code>, <code>iratxo_dealloc</code>, <code>iratxo_execute</code>. Drive it from any host with read/write access to linear memory.</p>
        </div>
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#ff7a18]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(255,122,24,0.1)]">
          <ShieldCheck class="text-[#ff7a18] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Unicode anti-phishing'}>Unicode anti-phishing</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed"><code>has_invisible_chars</code>, <code>has_mixed_script_token</code>, and <code>script_is</code> catch zero-width obfuscation and homoglyph attacks (Cyrillic 'а' in Latin words) without external databases.</p>
        </div>
        <div class="group bg-[#13121a] border border-white/5 p-5 sm:p-6 rounded-xl hover:border-[#7c3aed]/40 transition-all hover:-translate-y-1 hover:shadow-[0_0_20px_rgba(124,58,237,0.1)]">
          <Filter class="text-[#7c3aed] mb-4 group-hover:scale-110 transition-transform" size={26} />
          <h3 class="text-base sm:text-lg font-semibold text-white mb-2" use:hoverScramble={'Heuristic spam gates'}>Heuristic spam gates</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed"><code>repeated_char_run</code>, <code>repeated_token</code>, <code>type_token_ratio_below</code>, <code>mostly_uppercase</code>, and <code>token_entropy_above</code> give you spam, shout, and secret-detection layers without regex whack-a-mole.</p>
        </div>
      </div>
    </div>
  </section>

  <!-- Real-world rule packs gallery -->
  <section id="examples" class="relative z-10 py-16 sm:py-20">
    <div class="max-w-6xl mx-auto px-4 sm:px-6">
      <div class="text-center mb-10 sm:mb-14">
        <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-3 min-h-[1.2em]">{examplesText()}</h2>
        <p class="text-[#a0a0b0] max-w-2xl mx-auto text-sm sm:text-base">
          {scenarios.length} self-contained rule packs from production-shaped problems. Each one is runnable in the
          <a href="#playground" class="text-[#ff7a18] hover:underline">playground below</a>.
        </p>
      </div>

      <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-5">
        {#each scenarios as s, i}
          {@const icons = [FileCheck2, Scale, Bot, MessageSquare, ShieldCheck, Lock, Globe, GitPullRequest]}
          {@const Icon = icons[i % icons.length]}
          <a href="#playground" onclick={() => {
            // Trigger scenario load via custom event
            window.dispatchEvent(new CustomEvent('iratxo:load-scenario', { detail: s.id }));
          }} class="group bg-[#13121a] border border-white/5 rounded-xl p-5 hover:border-[#ff7a18]/40 transition-all hover:-translate-y-0.5 hover:shadow-[0_0_20px_rgba(255,122,24,0.1)] flex flex-col">
            <div class="flex items-center justify-between mb-3">
              <Icon class="text-[#ff7a18] group-hover:scale-110 transition-transform" size={22} />
              <span class="text-[10px] uppercase tracking-wider font-mono text-[#7a7a8a]">{s.domain}</span>
            </div>
            <h3 class="text-white font-semibold mb-2 text-sm sm:text-base">{s.title}</h3>
            <p class="text-[#9090a0] text-xs sm:text-sm leading-relaxed flex-1">{s.blurb}</p>
            <div class="mt-3 pt-3 border-t border-white/5 flex flex-wrap gap-1">
              {#each s.samples as sample}
                <span class="text-[10px] px-1.5 py-0.5 rounded font-mono bg-white/5 text-[#7a7a8a] border border-white/5">{sample.expect}</span>
              {/each}
            </div>
          </a>
        {/each}
      </div>
    </div>
  </section>

  <!-- Live playground -->
  <section id="playground" class="relative z-10 py-16 sm:py-20 border-y border-white/5 bg-[#0f0e14]">
    <div class="max-w-6xl mx-auto px-4 sm:px-6">
      <div class="text-center mb-8 sm:mb-10">
        <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-3 min-h-[1.2em]">{playText()}</h2>
        <p class="text-[#a0a0b0] max-w-2xl mx-auto text-sm sm:text-base">
          The exact same Wasm engine that powers the CLI, the JS hosts, and the embedded examples — running here in your browser, in pure JavaScript.
        </p>
      </div>

      <Playground />

      <p class="text-center text-xs text-[#7a7a8a] mt-4 max-w-2xl mx-auto">
        The engine is a 1.2&nbsp;MB <code>wasm32-unknown-unknown</code> build of <code>iratxo-engine</code>. Compilation and evaluation happen entirely in your browser — nothing is sent to a server.
      </p>
    </div>
  </section>

  <!-- Use cases -->
  <section id="use-cases" class="relative z-10 py-16 sm:py-20">
    <div class="max-w-5xl mx-auto px-4 sm:px-6">
      <div class="text-center mb-10 sm:mb-14">
        <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-3 min-h-[1.2em]">{useCasesText()}</h2>
        <p class="text-[#a0a0b0] max-w-2xl mx-auto text-sm sm:text-base">If you have a rule that needs to run identically in a browser, an edge worker, and a Rust service, Iratxo fits.</p>
      </div>

      <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <Bot class="text-[#ff7a18] mb-3" size={24} />
          <h3 class="text-white font-semibold mb-1.5">LLM agent guardrails</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Pre-screen tool inputs and user messages for jailbreaks, secret exfiltration, or destructive shell commands. Auditable, deterministic, and signable.</p>
        </div>
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <FileCheck2 class="text-[#7c3aed] mb-3" size={24} />
          <h3 class="text-white font-semibold mb-1.5">Compliance gating</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Run outbound copy through a versioned policy pack before send. Same pack runs in the browser composer, the API, and the post-send audit job.</p>
        </div>
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <MessageSquare class="text-[#22d3ee] mb-3" size={24} />
          <h3 class="text-white font-semibold mb-1.5">Support routing</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Triage inbound tickets by intent — cancellations, billing disputes, lockouts — using rule chaining for nested escalations.</p>
        </div>
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <ShieldCheck class="text-[#ff7a18] mb-3" size={24} />
          <h3 class="text-white font-semibold mb-1.5">UGC moderation</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Block, hold-for-review, or shadow-rank user-generated content with layered confidence levels the calling system can interpret.</p>
        </div>
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <Lock class="text-[#7c3aed] mb-3" size={24} />
          <h3 class="text-white font-semibold mb-1.5">PII / secret screens</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Catch credit cards, API keys, and contact info before they hit your data warehouse — at submit time, in transit, and at rest.</p>
        </div>
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <Scale class="text-[#22d3ee] mb-3" size={24} />
          <h3 class="text-white font-semibold mb-1.5">Legal / contract review</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Detect termination clauses, liability caps, or jurisdiction shifts using structural <code>has_section</code> + semantic match. Route ambiguities to humans.</p>
        </div>
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <GitPullRequest class="text-[#ff7a18] mb-3" size={24} />
          <h3 class="text-white font-semibold mb-1.5">CI / PR linting</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Run a signed rule pack over diffs and commit messages. Block <code>console.log(process.env)</code>, flag skipped tests, require descriptions.</p>
        </div>
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <Globe class="text-[#7c3aed] mb-3" size={24} />
          <h3 class="text-white font-semibold mb-1.5">Multilingual triage</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Built-in stemmers + synonym dictionaries for English, Spanish, Catalan, and Basque. Auto-detect input language; per-rule language overrides for tight phrasing.</p>
        </div>
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <Cloud class="text-[#22d3ee] mb-3" size={24} />
          <h3 class="text-white font-semibold mb-1.5">Edge gateways</h3>
          <p class="text-[#9090a0] text-sm leading-relaxed">Drop the engine into a Cloudflare Worker. Reject malformed payloads at the edge, never wake your origin for traffic that was always going to be blocked.</p>
        </div>
      </div>
    </div>
  </section>

  <!-- Quick start -->
  <section id="quickstart" class="relative z-10 py-16 sm:py-20 border-y border-white/5 bg-[#0f0e14]">
    <div class="max-w-4xl mx-auto px-4 sm:px-6">
      <div class="text-center mb-10 sm:mb-14">
        <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-3 min-h-[1.2em]">{quickstartText()}</h2>
        <p class="text-[#a0a0b0] max-w-2xl mx-auto text-sm sm:text-base">From zero to a signed, executing rule pack in a minute.</p>
      </div>

      <div class="space-y-4 sm:space-y-5">
        <!-- 1 -->
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <div class="flex items-center gap-3 mb-3">
            <div class="w-7 h-7 rounded-full bg-[#ff7a18]/20 text-[#ff7a18] font-bold text-sm flex items-center justify-center">1</div>
            <h3 class="text-white font-semibold">Build the workspace</h3>
          </div>
          <p class="text-[#9090a0] text-sm mb-3">The core, the CLI, and the Wasm engine each build independently:</p>
          {#each ['cargo test -p iratxo-core', 'cargo build -p iratxo-cli', 'cd crates/iratxo-engine && cargo build --release --target wasm32-unknown-unknown'] as cmd}
            <button onclick={() => copyCmd(cmd)} class="w-full text-left bg-black/40 border border-white/5 rounded-lg p-3 sm:p-4 font-mono text-[11px] sm:text-xs text-[#d0d0e0] overflow-x-auto hover:border-white/10 transition-colors flex items-center gap-3 group mb-2 last:mb-0">
              <span class="text-[#ff7a18] shrink-0">$</span>
              <span class="flex-1 truncate">{cmd}</span>
              {#if copiedCmd === cmd}
                <Check size={14} class="text-emerald-400 shrink-0" />
              {:else}
                <Copy size={14} class="text-[#7a7a8a] group-hover:text-white transition-colors shrink-0" />
              {/if}
            </button>
          {/each}
          <p class="text-[#7a7a8a] text-xs mt-2">
            On macOS the Homebrew rustc lacks <code>wasm32</code> std — the engine line uses your rustup toolchain.
          </p>
        </div>

        <!-- 2 -->
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <div class="flex items-center gap-3 mb-3">
            <div class="w-7 h-7 rounded-full bg-[#7c3aed]/20 text-[#7c3aed] font-bold text-sm flex items-center justify-center">2</div>
            <h3 class="text-white font-semibold">Author, compile, run</h3>
          </div>
          <p class="text-[#9090a0] text-sm mb-3">Scaffold a starter pack, compile it, evaluate against an input file:</p>
          {#each ['iratxo init my_rules', 'iratxo build my_rules/forbidden_phrases.yaml', 'iratxo run my_rules/forbidden_phrases.iratxo input.txt'] as cmd}
            <button onclick={() => copyCmd(cmd)} class="w-full text-left bg-black/40 border border-white/5 rounded-lg p-3 sm:p-4 font-mono text-[11px] sm:text-xs text-[#d0d0e0] overflow-x-auto hover:border-white/10 transition-colors flex items-center gap-3 group mb-2 last:mb-0">
              <span class="text-[#ff7a18] shrink-0">$</span>
              <span class="flex-1 truncate">{cmd}</span>
              {#if copiedCmd === cmd}
                <Check size={14} class="text-emerald-400 shrink-0" />
              {:else}
                <Copy size={14} class="text-[#7a7a8a] group-hover:text-white transition-colors shrink-0" />
              {/if}
            </button>
          {/each}
        </div>

        <!-- 3 -->
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <div class="flex items-center gap-3 mb-3">
            <div class="w-7 h-7 rounded-full bg-[#22d3ee]/20 text-[#22d3ee] font-bold text-sm flex items-center justify-center">3</div>
            <h3 class="text-white font-semibold">Test &amp; sign</h3>
          </div>
          <p class="text-[#9090a0] text-sm mb-3">YAML cases pin behavior in CI; Ed25519 signatures pin the bytes shipped to production:</p>
          {#each ['iratxo test rule.yaml cases.yaml', 'iratxo keygen --secret iratxo.sk --public iratxo.pk', 'iratxo sign rule.iratxo --key iratxo.sk', 'iratxo verify rule.iratxo --pubkey iratxo.pk'] as cmd}
            <button onclick={() => copyCmd(cmd)} class="w-full text-left bg-black/40 border border-white/5 rounded-lg p-3 sm:p-4 font-mono text-[11px] sm:text-xs text-[#d0d0e0] overflow-x-auto hover:border-white/10 transition-colors flex items-center gap-3 group mb-2 last:mb-0">
              <span class="text-[#ff7a18] shrink-0">$</span>
              <span class="flex-1 truncate">{cmd}</span>
              {#if copiedCmd === cmd}
                <Check size={14} class="text-emerald-400 shrink-0" />
              {:else}
                <Copy size={14} class="text-[#7a7a8a] group-hover:text-white transition-colors shrink-0" />
              {/if}
            </button>
          {/each}
        </div>

        <!-- 4 host SDKs -->
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <div class="flex items-center gap-3 mb-3">
            <div class="w-7 h-7 rounded-full bg-[#ff7a18]/20 text-[#ff7a18] font-bold text-sm flex items-center justify-center">4</div>
            <h3 class="text-white font-semibold">Embed in JavaScript</h3>
          </div>
          <p class="text-[#9090a0] text-sm mb-3">Same module, three entry points:</p>
          <pre class="bg-black/40 border border-white/5 rounded-lg p-3 sm:p-4 font-mono text-[11px] sm:text-xs text-[#d0d0e0] overflow-x-auto leading-relaxed">{`// Node
import { load } from "@iratxo/js";
const iratxo = await load();
const r = iratxo.run(ruleBytes, "input text");

// Browser
import { load } from "@iratxo/js/browser";
const iratxo = await load("/iratxo_engine.wasm");
const ir = iratxo.compile(yamlString);
const r = iratxo.run(ir, "input text");

// Cloudflare Workers
import wasm from "./iratxo_engine.wasm";
import { loadFromModule } from "@iratxo/js/workers";
export default {
  async fetch(req, env) {
    const iratxo = loadFromModule(wasm);
    return Response.json(iratxo.run(env.RULE, await req.text()));
  }
};`}</pre>
        </div>
      </div>
    </div>
  </section>

  <!-- DSL reference -->
  <section id="dsl" class="relative z-10 py-16 sm:py-20">
    <div class="max-w-5xl mx-auto px-4 sm:px-6">
      <div class="text-center mb-10 sm:mb-14">
        <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-3 min-h-[1.2em]">{dslText()}</h2>
        <p class="text-[#a0a0b0] max-w-2xl mx-auto text-sm sm:text-base">A small set of predicates, three combinators, and rule chaining.</p>
      </div>

      <div class="grid lg:grid-cols-2 gap-6">
        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <h3 class="text-white font-semibold mb-4 flex items-center gap-2"><BookOpen size={18} class="text-[#ff7a18]" /> Predicates</h3>
          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="text-left text-[#7a7a8a] text-xs uppercase tracking-wider border-b border-white/10">
                  <th class="py-2 pr-4 font-semibold">Predicate</th>
                  <th class="py-2 font-semibold">What it checks</th>
                </tr>
              </thead>
              <tbody>
                {#each predicates as p}
                  <tr class="border-b border-white/5 last:border-b-0">
                    <td class="py-2 pr-4 font-mono text-[12px] text-[#ffae6e] whitespace-nowrap">{p.name}</td>
                    <td class="py-2 text-[#a0a0b0] text-[13px]">{p.what}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
          <p class="text-[#7a7a8a] text-xs mt-4">
            A rule may declare <code class="text-[#ffae6e]">then: [other_id, ...]</code> to chain follow-up rules whose firing is gated on this rule triggering. Cycles are detected and ignored.
          </p>
        </div>

        <div class="bg-[#13121a] border border-white/5 rounded-xl p-5 sm:p-6">
          <h3 class="text-white font-semibold mb-4 flex items-center gap-2"><FlaskConical size={18} class="text-[#7c3aed]" /> A complete pack</h3>
          <pre class="code-pre">{`name: outbound_compliance
rules:
  - id: no_refund_guarantee
    when:
      contains_any: ["guaranteed refund", "100% refund"]
    classify: review_required
    confidence: 0.95
    explanation: "Forbidden refund-guarantee language."
  - id: prohibited_claim
    when:
      regex: "\\b(cure|FDA approved)\\b"
    classify: blocked
    confidence: 0.99
  - id: cancellation_intent
    when:
      semantic_match:
        examples: ["the user wants to cancel their agreement"]
        threshold: 0.3
    classify: review_required
    confidence: 0.6
default:
  classify: ok
  confidence: 1.0`}</pre>
          <p class="text-[#7a7a8a] text-xs mt-3">
            Compiles to ~2 KB of IR. Ship it as a single artifact — every host evaluates it the same way.
          </p>
        </div>
      </div>
    </div>
  </section>

  <!-- Documentation accordion -->
  <section id="docs" class="relative z-10 py-16 sm:py-24 border-y border-white/5 bg-[#0f0e14]">
    <div class="max-w-4xl mx-auto px-4 sm:px-6">
      <div class="text-center mb-10 sm:mb-12">
        <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-3 min-h-[1.2em]">{docsText()}</h2>
        <p class="text-[#a0a0b0] text-sm sm:text-base">Under-the-hood detail for the curious.</p>
      </div>

      <!-- Architecture -->
      <div class="border border-white/5 rounded-xl overflow-hidden mb-3 sm:mb-4 bg-[#13121a]">
        <button onclick={() => toggleSection('architecture')} class="w-full flex items-center justify-between px-5 sm:px-6 py-4 sm:py-5 hover:bg-white/5 transition-colors text-left">
          <div class="flex items-center gap-3 text-white font-semibold text-sm sm:text-base">
            <TerminalSquare size={18} class="text-[#ff7a18] shrink-0" /> Architecture
          </div>
          <ChevronDown size={18} class="text-[#7a7a8a] transition-transform shrink-0 {openSection === 'architecture' ? 'rotate-180' : ''}" />
        </button>
        {#if openSection === 'architecture'}
          <div class="px-5 sm:px-6 pt-5 sm:pt-6 pb-5 sm:pb-6 text-[#b0b0c0] text-sm leading-relaxed space-y-4">
            <p>Iratxo is a four-crate workspace plus a JS host package:</p>
            <ul class="list-disc pl-5 space-y-1">
              <li><strong class="text-white">iratxo-core</strong> — IR types, YAML→IR compiler, deterministic interpreter. The only crate that defines what a "rule" means.</li>
              <li><strong class="text-white">iratxo-engine</strong> — a thin <code>cdylib</code> compiled to <code>wasm32-unknown-unknown</code> that re-exports the interpreter through a four-function ABI.</li>
              <li><strong class="text-white">iratxo-cli</strong> — <code>build · lint · run · run-native · test · sign · verify · keygen</code>.</li>
              <li><strong class="text-white">hosts/js</strong> — three entry points (Node, browser, Cloudflare Workers) over a single <code>core.js</code> driver.</li>
              <li><strong class="text-white">playground/</strong> — browser-based rule studio: write YAML, compile, evaluate in real time.</li>
            </ul>
            <p>The engine is intentionally split out of the workspace because it has a wasm-only target. Building it requires the rustup toolchain (Homebrew rustc lacks a <code>wasm32</code> std).</p>
          </div>
        {/if}
      </div>

      <!-- IR format -->
      <div class="border border-white/5 rounded-xl overflow-hidden mb-3 sm:mb-4 bg-[#13121a]">
        <button onclick={() => toggleSection('ir')} class="w-full flex items-center justify-between px-5 sm:px-6 py-4 sm:py-5 hover:bg-white/5 transition-colors text-left">
          <div class="flex items-center gap-3 text-white font-semibold text-sm sm:text-base">
            <Boxes size={18} class="text-[#7c3aed] shrink-0" /> IR format
          </div>
          <ChevronDown size={18} class="text-[#7a7a8a] transition-transform shrink-0 {openSection === 'ir' ? 'rotate-180' : ''}" />
        </button>
        {#if openSection === 'ir'}
          <div class="px-5 sm:px-6 pt-5 sm:pt-6 pb-5 sm:pb-6 text-[#b0b0c0] text-sm leading-relaxed space-y-4">
            <p>Compiled <code>.iratxo</code> files are versioned:</p>
            <pre class="code-pre">{`IRTX            (4 bytes magic)
version : u16   (little-endian)
payload : bincode-serialized IR`}</pre>
            <p>Engines refuse blobs with a different magic or version, so old artifacts produce a clear error rather than corrupted behavior. See <code>IR_VERSION</code> in <code>iratxo-core/src/lib.rs</code>.</p>
            <p>The IR itself is a tree of <code>Predicate</code> enums plus a list of <code>Rule</code> entries with <code>id</code>, <code>when</code>, <code>classify</code>, <code>confidence</code>, optional <code>explanation</code>, and optional <code>then</code> chain references.</p>
          </div>
        {/if}
      </div>

      <!-- Wasm ABI -->
      <div class="border border-white/5 rounded-xl overflow-hidden mb-3 sm:mb-4 bg-[#13121a]">
        <button onclick={() => toggleSection('abi')} class="w-full flex items-center justify-between px-5 sm:px-6 py-4 sm:py-5 hover:bg-white/5 transition-colors text-left">
          <div class="flex items-center gap-3 text-white font-semibold text-sm sm:text-base">
            <Cpu size={18} class="text-[#ff7a18] shrink-0" /> Wasm ABI
          </div>
          <ChevronDown size={18} class="text-[#7a7a8a] transition-transform shrink-0 {openSection === 'abi' ? 'rotate-180' : ''}" />
        </button>
        {#if openSection === 'abi'}
          <div class="px-5 sm:px-6 pt-5 sm:pt-6 pb-5 sm:pb-6 text-[#b0b0c0] text-sm leading-relaxed space-y-4">
            <pre class="code-pre">{`iratxo_alloc(len: u32) -> ptr: u32
iratxo_dealloc(ptr: u32, len: u32)
iratxo_compile(yaml_ptr, yaml_len) -> u64    // (ptr<<32) | len of {ok, ir|error}
iratxo_execute(rule_ptr, rule_len, input_ptr, input_len) -> u64   // (ptr<<32) | len of JSON`}</pre>
            <p>Any host that can read/write linear memory can drive the engine — browser JS, Node, Cloudflare Workers, <code>wasmtime</code>, <code>wasmer</code>, or hand-written hosts in Go, Python, .NET. There is no engine-specific glue.</p>
            <p>Memory safety contract: each call <code>alloc</code>s on the way in and <code>dealloc</code>s on the way out (see <code>hosts/js/src/core.js</code> for a 90-line reference driver).</p>
          </div>
        {/if}
      </div>

      <!-- Semantic match -->
      <div class="border border-white/5 rounded-xl overflow-hidden mb-3 sm:mb-4 bg-[#13121a]">
        <button onclick={() => toggleSection('semantic')} class="w-full flex items-center justify-between px-5 sm:px-6 py-4 sm:py-5 hover:bg-white/5 transition-colors text-left">
          <div class="flex items-center gap-3 text-white font-semibold text-sm sm:text-base">
            <Languages size={18} class="text-[#7c3aed] shrink-0" /> Semantic match
          </div>
          <ChevronDown size={18} class="text-[#7a7a8a] transition-transform shrink-0 {openSection === 'semantic' ? 'rotate-180' : ''}" />
        </button>
        {#if openSection === 'semantic'}
          <div class="px-5 sm:px-6 pt-5 sm:pt-6 pb-5 sm:pb-6 text-[#b0b0c0] text-sm leading-relaxed space-y-4">
            <p>Multilingual: English, Spanish, Catalan, Basque. Auto-detected from input by default; specify <code>language</code> to force.</p>
            <p><strong class="text-white">Pipeline:</strong> tokenize (Unicode) → drop per-language stop words → stem → synonym-normalize → signed feature hashing into 256-dim dense vector → cosine similarity.</p>
            <ul class="list-disc pl-5 space-y-1">
              <li><strong class="text-white">English / Spanish stemmers</strong> — Snowball via <code>rust-stemmers</code>.</li>
              <li><strong class="text-white">Catalan stemmer</strong> — hand-rolled light stemmer. Folds diacritics, strips high-frequency nominal/adjectival/verbal suffixes, and collapses orthographic <code>qu</code> → <code>c</code> so <code>polítiques</code> / <code>política</code> / <code>polítics</code> share a stem.</li>
              <li><strong class="text-white">Basque stemmer</strong> — direct port of <a href="https://github.com/enekos/marrow" target="_blank" rel="noopener" class="text-[#ff7a18] hover:underline">marrow</a>'s implementation, validated against marrow's reference output on 850 words (100% parity, see <code>crates/iratxo-core/tests/basque_parity.rs</code>).</li>
            </ul>
            <p><strong class="text-white">Score range:</strong> 0.0–1.0. Realistic thresholds: 0.2 for sparse texts that share a single concept token after stemming; 0.3 for typical paragraph-length inputs; 0.4–0.5 for tight phrasing matches.</p>
            <p>Pure function, no embedded model file, no network — and therefore no token cost, no rate limit, no privacy footnote.</p>
          </div>
        {/if}
      </div>

      <!-- Signing -->
      <div class="border border-white/5 rounded-xl overflow-hidden mb-3 sm:mb-4 bg-[#13121a]">
        <button onclick={() => toggleSection('signing')} class="w-full flex items-center justify-between px-5 sm:px-6 py-4 sm:py-5 hover:bg-white/5 transition-colors text-left">
          <div class="flex items-center gap-3 text-white font-semibold text-sm sm:text-base">
            <KeyRound size={18} class="text-[#ff7a18] shrink-0" /> Signing rule packs
          </div>
          <ChevronDown size={18} class="text-[#7a7a8a] transition-transform shrink-0 {openSection === 'signing' ? 'rotate-180' : ''}" />
        </button>
        {#if openSection === 'signing'}
          <div class="px-5 sm:px-6 pt-5 sm:pt-6 pb-5 sm:pb-6 text-[#b0b0c0] text-sm leading-relaxed space-y-4">
            <pre class="code-pre">{`iratxo keygen --secret iratxo.sk --public iratxo.pk   # 32 raw bytes each
iratxo sign   rule.iratxo --key iratxo.sk             # writes rule.iratxo.sig
iratxo verify rule.iratxo --pubkey iratxo.pk`}</pre>
            <p>Ed25519 signatures, 64 bytes, in a sidecar <code>.iratxo.sig</code>. Tampered blobs fail verification before the interpreter ever sees them. Pin the public key in your host code; rotate signing keys without touching the engine.</p>
          </div>
        {/if}
      </div>

      <!-- CLI -->
      <div class="border border-white/5 rounded-xl overflow-hidden mb-3 sm:mb-4 bg-[#13121a]">
        <button onclick={() => toggleSection('cli')} class="w-full flex items-center justify-between px-5 sm:px-6 py-4 sm:py-5 hover:bg-white/5 transition-colors text-left">
          <div class="flex items-center gap-3 text-white font-semibold text-sm sm:text-base">
            <Terminal size={18} class="text-[#7c3aed] shrink-0" /> CLI commands
          </div>
          <ChevronDown size={18} class="text-[#7a7a8a] transition-transform shrink-0 {openSection === 'cli' ? 'rotate-180' : ''}" />
        </button>
        {#if openSection === 'cli'}
          <div class="px-5 sm:px-6 pt-5 sm:pt-6 pb-5 sm:pb-6 text-[#b0b0c0] text-sm leading-relaxed space-y-3">
            <ul class="space-y-3">
              <li><code class="text-white bg-white/10 px-1.5 py-0.5 rounded">iratxo init NAME</code><p class="mt-1">Scaffold a starter rule pack with a YAML rule and a cases file.</p></li>
              <li><code class="text-white bg-white/10 px-1.5 py-0.5 rounded">iratxo build RULE.yaml</code><p class="mt-1">Compile YAML to a versioned <code>.iratxo</code> IR file.</p></li>
              <li><code class="text-white bg-white/10 px-1.5 py-0.5 rounded">iratxo lint RULE.yaml</code><p class="mt-1">Static check for unreachable rules, duplicate ids, malformed regex.</p></li>
              <li><code class="text-white bg-white/10 px-1.5 py-0.5 rounded">iratxo run RULE.iratxo INPUT</code><p class="mt-1">Evaluate the compiled rule against an input file. Prints JSON verdict.</p></li>
              <li><code class="text-white bg-white/10 px-1.5 py-0.5 rounded">iratxo run-native RULE.yaml INPUT</code><p class="mt-1">Skip the IR step — handy for quick iteration during authoring.</p></li>
              <li><code class="text-white bg-white/10 px-1.5 py-0.5 rounded">iratxo test RULE.yaml CASES.yaml</code><p class="mt-1">Golden-file tests. Exits 1 on any failure — drop it into CI.</p></li>
              <li><code class="text-white bg-white/10 px-1.5 py-0.5 rounded">iratxo keygen / sign / verify</code><p class="mt-1">Ed25519 key management and signing.</p></li>
            </ul>
          </div>
        {/if}
      </div>

      <!-- Hosts -->
      <div class="border border-white/5 rounded-xl overflow-hidden mb-3 sm:mb-4 bg-[#13121a]">
        <button onclick={() => toggleSection('hosts')} class="w-full flex items-center justify-between px-5 sm:px-6 py-4 sm:py-5 hover:bg-white/5 transition-colors text-left">
          <div class="flex items-center gap-3 text-white font-semibold text-sm sm:text-base">
            <MonitorSmartphone size={18} class="text-[#ff7a18] shrink-0" /> JavaScript hosts
          </div>
          <ChevronDown size={18} class="text-[#7a7a8a] transition-transform shrink-0 {openSection === 'hosts' ? 'rotate-180' : ''}" />
        </button>
        {#if openSection === 'hosts'}
          <div class="px-5 sm:px-6 pt-5 sm:pt-6 pb-5 sm:pb-6 text-[#b0b0c0] text-sm leading-relaxed space-y-4">
            <p>The <code>@iratxo/js</code> package exposes three entry points around a shared <code>core.js</code> driver:</p>
            <ul class="list-disc pl-5 space-y-2">
              <li><strong class="text-white">Node</strong> — reads <code>wasm/iratxo_engine.wasm</code> off disk on first <code>load()</code>. 7 host tests under <code>hosts/js/test/</code>.</li>
              <li><strong class="text-white">Browser</strong> — <code>load(urlOrBytes)</code> with <code>WebAssembly.compileStreaming</code>. The page you're reading right now is a browser host.</li>
              <li><strong class="text-white">Cloudflare Workers</strong> — accepts a <code>WebAssembly.Module</code> imported via Wrangler's <code>wasm</code> module syntax. Cold-start friendly (no streaming compile).</li>
            </ul>
            <p>The Wasm engine is identical across all three. JS wasm linear memory is not safe to share across concurrent calls — create one runner per worker thread / request.</p>
          </div>
        {/if}
      </div>

      <!-- Roadmap -->
      <div class="border border-white/5 rounded-xl overflow-hidden mb-3 sm:mb-4 bg-[#13121a]">
        <button onclick={() => toggleSection('roadmap')} class="w-full flex items-center justify-between px-5 sm:px-6 py-4 sm:py-5 hover:bg-white/5 transition-colors text-left">
          <div class="flex items-center gap-3 text-white font-semibold text-sm sm:text-base">
            <Settings size={18} class="text-[#7c3aed] shrink-0" /> Status &amp; roadmap
          </div>
          <ChevronDown size={18} class="text-[#7a7a8a] transition-transform shrink-0 {openSection === 'roadmap' ? 'rotate-180' : ''}" />
        </button>
        {#if openSection === 'roadmap'}
          <div class="px-5 sm:px-6 pt-5 sm:pt-6 pb-5 sm:pb-6 text-[#b0b0c0] text-sm leading-relaxed space-y-4">
            <div>
              <h4 class="text-white font-semibold mb-2 text-sm">Shipped</h4>
              <div class="flex flex-wrap gap-1.5">
                {#each ROADMAP_DONE as item}
                  <span class="text-[11px] px-2 py-0.5 rounded font-mono bg-emerald-500/10 text-emerald-300 border border-emerald-500/20">✓ {item}</span>
                {/each}
              </div>
            </div>
            <div>
              <h4 class="text-white font-semibold mb-2 text-sm">Open</h4>
              <div class="flex flex-wrap gap-1.5">
                {#each ROADMAP_OPEN as item}
                  <span class="text-[11px] px-2 py-0.5 rounded font-mono bg-white/5 text-[#9090a0] border border-white/10">{item}</span>
                {/each}
              </div>
            </div>
            <p>See <code>docs/ROADMAP.md</code> in the repo for context behind each item.</p>
          </div>
        {/if}
      </div>
    </div>
  </section>

  <!-- CTA -->
  <section class="relative z-10 py-16 sm:py-20">
    <div class="max-w-3xl mx-auto px-4 sm:px-6 text-center">
      <h2 class="text-2xl sm:text-3xl md:text-4xl font-bold text-white mb-4">Ship rule changes, not deploys.</h2>
      <p class="text-[#a0a0b0] mb-6 sm:mb-8 text-sm sm:text-base max-w-xl mx-auto">
        One YAML pack. One Wasm artifact. Same verdict in the browser, on the edge, and in your backend.
      </p>
      <div class="flex flex-col sm:flex-row gap-3 sm:gap-4 justify-center">
        <a href="#playground" class="px-6 py-3 bg-[#ff7a18] text-white rounded-full font-semibold text-sm hover:shadow-[0_0_25px_rgba(255,122,24,0.4)] hover:-translate-y-0.5 transition-all flex items-center justify-center gap-2">
          Open the playground <ArrowRight size={16} />
        </a>
        <a href="https://github.com/enekos/iratxo" target="_blank" rel="noopener" class="px-6 py-3 border border-white/10 rounded-full text-white text-sm hover:bg-white/10 hover:border-[#ff7a18]/50 transition-all flex items-center justify-center gap-2">
          <GitBranch size={14} /> Star on GitHub
        </a>
      </div>
    </div>
  </section>

  <!-- Footer -->
  <footer class="relative z-10 py-8 sm:py-10 bg-[#0b0a10] border-t border-white/5">
    <div class="max-w-6xl mx-auto px-4 sm:px-6 flex flex-col sm:flex-row justify-between items-center gap-4 text-[#707080] text-xs sm:text-sm">
      <div class="flex items-center gap-2 font-semibold text-white">
        <Zap size={18} class="text-[#ff7a18]" /> Iratxo
      </div>
      <div class="flex items-center gap-4 sm:gap-6">
        <a href="https://github.com/enekos/iratxo" target="_blank" rel="noopener" class="hover:text-white transition-colors">GitHub</a>
        <a href="https://github.com/enekos/iratxo/releases" target="_blank" rel="noopener" class="hover:text-white transition-colors">Releases</a>
        <a href="https://github.com/enekos/iratxo/issues" target="_blank" rel="noopener" class="hover:text-white transition-colors">Issues</a>
        <a href="#docs" class="hover:text-white transition-colors">Docs</a>
      </div>
      <div>
        MIT or Apache-2.0 · Built by <a href="https://github.com/enekos" target="_blank" rel="noopener" class="text-[#a0a0b0] hover:text-white transition-colors">enekos</a>
      </div>
    </div>
  </footer>
</div>

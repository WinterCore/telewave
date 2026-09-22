<script>
  /**
   * Svelte port of "Split Text" from reactbits.dev: text revealed one
   * character at a time. Re-keys on `text`, so a new string replays.
   */
  let { text, charDelay = 26, class: cls = "" } = $props();
</script>

<span class={cls}>
  <span class="sr-only">{text}</span>
  {#key text}
    <span aria-hidden="true">
      {#each [...text] as char, index}
        <span
          class="split-char inline-block"
          style:animation-delay="{index * charDelay}ms"
        >{char === " " ? " " : char}</span>
      {/each}
    </span>
  {/key}
</span>

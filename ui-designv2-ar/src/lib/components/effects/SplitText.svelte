<script>
  /**
   * Reveal whole words so Arabic glyphs remain joined within each word.
   * Re-keying on text replays the reveal when the broadcast changes.
   */
  let { text, wordDelay = 70, class: cls = "" } = $props();
</script>

<span class={cls}>
  <span class="sr-only">{text}</span>
  {#key text}
    <span aria-hidden="true">
      {#each text.split(/\s+/u) as word, index}
        {#if index > 0}{" "}{/if}<span
          class="split-word inline-block"
          style:animation-delay="{index * wordDelay}ms"
        >{word}</span>
      {/each}
    </span>
  {/key}
</span>

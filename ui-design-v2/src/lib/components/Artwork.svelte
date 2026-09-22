<script>
  import slowMornings from "../../assets/artwork/slow-mornings.svg";
  import creative from "../../assets/artwork/creative.svg";
  import tides from "../../assets/artwork/tides.svg";
  import fieldNotes from "../../assets/artwork/field-notes.svg";

  const WORK = {
    "slow-mornings": slowMornings,
    creative,
    tides,
    "field-notes": fieldNotes,
  };

  let { name, alt = "", class: cls = "" } = $props();
</script>

<img src={WORK[name] ?? slowMornings} {alt} class="block object-cover {cls}" />

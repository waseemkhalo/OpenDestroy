<script lang="ts">
  let {expanded=false}:{expanded?:boolean}=$props();
  function morph(_node:Element,direction:number){return {duration:window.matchMedia('(prefers-reduced-motion: reduce)').matches?0:240,css:(t:number)=>`transform:rotate(${expanded?t*45*direction:0}deg) translateY(${expanded?t*4*direction:0}px)`};}
  function knob(_node:Element){return {duration:window.matchMedia('(prefers-reduced-motion: reduce)').matches?0:240,css:(t:number)=>`opacity:${expanded?1-t:1}`};}
</script>
<svg class:expanded width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" aria-hidden="true">
  <g class="upper" transition:morph|global={1}><path d="M4 8H20"/><circle cx="8" cy="8" r="3" transition:knob|global/></g>
  <g class="lower" transition:morph|global={-1}><path d="M4 16H20"/><circle cx="16" cy="16" r="3" transition:knob|global/></g>
</svg>
<style>
 g{transform-origin:12px 12px;transition:transform 260ms cubic-bezier(.22,1,.36,1)}
 circle{fill:#0c0e0d;transition:opacity 180ms ease,transform 260ms ease;transform-box:fill-box;transform-origin:center}
 .expanded .upper{transform:rotate(45deg) translateY(4px)}
 .expanded .lower{transform:rotate(-45deg) translateY(-4px)}
 .expanded circle{opacity:0;transform:scale(.2)}
 @media(prefers-reduced-motion:reduce){g,circle{transition:none}}
</style>

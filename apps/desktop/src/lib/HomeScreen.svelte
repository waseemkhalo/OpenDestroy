<script lang="ts">
 import {House,History,BookOpen,PenLine,Network,Settings,ChevronRight,Copy,Search,Circle,SlidersHorizontal} from '@lucide/svelte';
 import {invoke} from '@tauri-apps/api/core';
 import {fly} from 'svelte/transition';
 import type {Snippet} from 'svelte';
 import {cubicOut} from 'svelte/easing';
 import ShortcutKeys from './ShortcutKeys.svelte';
 import InkBrush from './InkBrush.svelte';
 import WritingSettings from './WritingSettings.svelte';
 import {readWeeklyDictationSnapshot,readRecentDictations,clearRecentDictations,dictationAverageWpm,dictationTimeSavedMs,DICTATION_UPDATED_EVENT} from './dictation/dictationState';
 let {shortcut,speech,phase,microphoneLevel=0,recording=false,scope=null,onSettings,connections}: {
 shortcut:string;speech:{provider:string|null;modelReady:boolean;ready:boolean;localModelName?:string;modelName?:string;language?:string};phase:string;microphoneLevel?:number;recording?:boolean;scope?:string|null;onSettings:()=>void;connections?:Snippet;
 }=$props();
 let view=$state<'home'|'history'|'dictionary'|'writing'|'connections'>('home');
 let visited=$state<string[]>(['home']);
 let historyTab=$state<'session'|'insights'>('session'),clearConfirm=$state(false);
 let contentElement:HTMLDivElement;
 const pageTitles={history:['History','Your words, within reach.'],dictionary:['Dictionary','The words that make you, you.'],writing:['Writing','Make every word sound like you.'],connections:['Connections','Bring your tools into the conversation.']};
 let query=$state(''),copyStatus=$state('');
 let snapshot=$state({words:0,speakingMs:0,days:[] as string[],lastText:''});
 let recent=$state<ReturnType<typeof readRecentDictations>>([]);
 const filtered=$derived(recent.filter(item=>item.text.toLowerCase().includes(query.toLowerCase())));
 const modelLabel=$derived(speech.localModelName||speech.modelName||(speech.provider==='backend'?'Connected speech':speech.provider||'Speech setup'));
 const activeRecording=$derived(recording||phase==='recording');
 const workApps=['ChatGPT','Claude','Gemini','Microsoft Copilot','Perplexity','Cursor','VS Code','Slack','Microsoft Teams','Gmail','Outlook','Google Docs','Microsoft Word','Notion','Linear','Asana','Jira','Zoom','WhatsApp','GitHub'];
 let appIndex=$state(0),appsPaused=$state(false),reducedMotion=$state(false);
 $effect(()=>{
  const preference=window.matchMedia('(prefers-reduced-motion: reduce)');
  const sync=()=>{reducedMotion=preference.matches;};sync();
  preference.addEventListener('change',sync);
  const timer=window.setInterval(()=>{if(!reducedMotion&&!appsPaused&&!activeRecording&&!document.hidden)appIndex=(appIndex+1)%workApps.length;},3000);
  return()=>{window.clearInterval(timer);preference.removeEventListener('change',sync);};
 });
 const minutesSaved=$derived(Math.floor(dictationTimeSavedMs(snapshot)/60000));
 const tabs=[['home','Home',House],['history','History',History],['dictionary','Dictionary',BookOpen],['writing','Writing',PenLine],['connections','Connections',Network]] as const;
 function refresh(){snapshot=readWeeklyDictationSnapshot(scope);recent=readRecentDictations(scope);}
 $effect(()=>{scope;refresh();query='';copyStatus='';clearConfirm=false;});
 $effect(()=>{window.addEventListener(DICTATION_UPDATED_EVENT,refresh);const timer=window.setInterval(refresh,60000);return()=>{window.removeEventListener(DICTATION_UPDATED_EVENT,refresh);window.clearInterval(timer);};});
 function navigate(next:typeof view){if(next===view)return;view=next;if(!visited.includes(next))visited=[...visited,next];query='';copyStatus='';clearConfirm=false;contentElement?.scrollTo({top:0,behavior:'instant'});}
 function clearSession(){clearRecentDictations(scope);clearConfirm=false;copyStatus='Session cleared. Usage totals are unchanged.';}
 async function copy(text:string){const owner=scope;try{if('__TAURI_INTERNALS__' in window)await invoke('destroy_dictation_copy',{text});else await navigator.clipboard.writeText(text);if(owner===scope)copyStatus='Copied.';}catch{if(owner===scope)copyStatus='Could not copy. Try again.';}}
 const dateLabel=(timestamp:number)=>new Date(timestamp).toLocaleString(undefined,{month:'short',day:'numeric',hour:'numeric',minute:'2-digit'});
</script>

<section class="daily-ink" aria-label="Destroy Dictation home">
 <aside class="sidebar">
  <div class="brand" data-tauri-drag-region><span>DESTROY</span></div>
  <nav aria-label="Main navigation">
   {#each tabs as [id,label,Icon]}<button class:active={view===id} aria-current={view===id?'page':undefined} onclick={()=>navigate(id)} title={label}><Icon size={21} strokeWidth={1.65}/><span>{label}</span></button>{/each}
  </nav>
  <div class="sidebar-art" aria-hidden="true"></div>
  <button class="settings-link" onclick={onSettings} title="Settings"><Settings size={21} strokeWidth={1.65}/><span>Settings</span></button>
 </aside>
 <div class="content" class:home-fit={view==='home'} class:compact={view!=='home'} bind:this={contentElement}>
  <!-- One persistent artwork layer: it moves between layouts instead of swapping paintings. -->
  <div class="approved-art" aria-hidden="true"></div>
  <div class="page-stage">
  {#if view==='home'}
   <div class="panorama" in:fly={{y:8,duration:reducedMotion?0:320,easing:cubicOut}}>
    <div class="hero-copy"><h1>Your voice, in ink.</h1><ShortcutKeys value={shortcut} recording={activeRecording} interactive/>
     <div class="ritual"><span>Hold to dictate in any app</span><button class="rotating-app" onclick={()=>appsPaused=!appsPaused} aria-label={appsPaused?'Resume app examples':'Pause app examples'} aria-pressed={appsPaused} title="Works in editable text fields. Click to pause or resume examples.">{#key appIndex}<span class="app-stack" in:fly={{y:5,duration:reducedMotion?0:220}}>{#each [0,1,2] as offset}<span class="app-badge"><img src={`/app-icons/${(appIndex+offset)%workApps.length}.png`} alt={workApps[(appIndex+offset)%workApps.length]} width="26" height="26"/></span>{/each}</span>{/key}</button></div>
    </div>
    <div class="live-ink" class:listening={activeRecording}><InkBrush {microphoneLevel} recording={activeRecording}/></div>
   </div>
   <div class="overview">
    <section class="activity" aria-label="This week's activity"><div><p>This week</p><div class="totals"><span><strong>{scope?snapshot.words.toLocaleString():'—'}</strong> words</span><span title="Estimated against typing at 40 words per minute"><strong>{scope?minutesSaved:'—'}</strong> min saved <small>estimated</small></span></div></div><time>Today<br/>{new Date().toLocaleDateString(undefined,{month:'short',day:'numeric',year:'numeric'})}</time></section>
    <div class="home-columns">
     <section class="recent"><header><h2>Recent dictations</h2><span>THIS SESSION</span></header>
      {#if recent.length}{#each recent.slice(0,2) as item (item.id)}<article><div><p>{item.text}</p><small>{dateLabel(item.completedAt)} · {item.words} {item.words===1?'word':'words'}</small></div><button aria-label="Copy dictation" onclick={()=>void copy(item.text)}><Copy size={16}/></button></article>{/each}<button class="all-history" onclick={()=>navigate('history')}>View this session <ChevronRight size={14}/></button>{:else}<div class="empty"><h3>A little space for your words.</h3><p>Your next dictation will appear here.<br/>Transcripts stay in memory, not on disk.</p></div>{/if}
     </section>
     <section class="personalize"><h2>Make it yours</h2><button onclick={()=>navigate('dictionary')}><BookOpen size={28}/><span>Add a word to your dictionary</span><ChevronRight size={16}/></button><button onclick={()=>navigate('writing')}><SlidersHorizontal size={28}/><span>Set your writing style</span><ChevronRight size={16}/></button></section>
    </div>
   </div>
  {:else if view==='history'}
   <section class="detail-page page-enter"><header class="page-heading"><h1>History</h1><p>Your words, within reach.</p></header>
    <div class="history-tabs" role="group" aria-label="History view"><span class="tab-marker" class:second={historyTab==='insights'} aria-hidden="true"></span><button aria-pressed={historyTab==='session'} onclick={()=>historyTab='session'}>This session</button><button aria-pressed={historyTab==='insights'} onclick={()=>historyTab='insights'}>Insights</button></div>
    {#key historyTab}<div in:fly={{y:8,duration:reducedMotion?0:240,easing:cubicOut}}>
    {#if historyTab==='session'}
     <div class="history-toolbar"><label class="history-search"><Search size={21}/><input aria-label="Search dictations" placeholder="Search this session" bind:value={query}/></label><button class="clear-session" disabled={!recent.length} onclick={()=>clearConfirm=!clearConfirm}>Clear session</button></div>
     {#if clearConfirm}<div class="clear-confirm" role="group" aria-label="Confirm clearing session"><p>Clear these transcripts? This cannot be undone. Usage totals stay.</p><button onclick={()=>clearConfirm=false}>Keep transcripts</button><button class="confirm-clear" onclick={clearSession}>Clear transcripts</button></div>{/if}
     <p class="history-list-label">Recent</p>
     <div class="history-list">{#each filtered as item (item.id)}<article class="history-item"><time>{dateLabel(item.completedAt)}</time><div><p>{item.text}</p></div><button aria-label="Copy dictation" onclick={()=>void copy(item.text)}><Copy size={23}/></button><small>{item.words} {item.words===1?'word':'words'}</small></article>{:else}<div class="history-empty"><h2>{query?'No matching words.':'A little space for your words.'}</h2><p>{query?'Try a different search.':'Your next dictation will appear here.'}</p></div>{/each}</div>
     <p class="session-note">Session-only. Cleared when you close the app or change accounts.</p>
    {:else}<section class="insights" aria-label="This week's usage"><p>This week · on this device</p><div class="insight-grid"><article><strong>{scope?snapshot.words.toLocaleString():'—'}</strong><span>Words dictated</span></article><article><strong>{scope?minutesSaved:'—'} <small>min</small></strong><span>Estimated time saved</span></article><article><strong>{scope?dictationAverageWpm(snapshot):'—'} <small>wpm</small></strong><span>Average speaking speed</span></article></div><p class="session-note">Time saved compares your speaking time with typing at 40 words per minute. Only usage counts are stored, not transcripts.</p></section>{/if}
    </div>{/key}
   </section>
  {/if}
  {#each ['dictionary','writing'] as mode}{#if visited.includes(mode)}<section class="detail-page page-enter" hidden={view!==mode}><header class="page-heading"><h1>{pageTitles[mode as 'dictionary'|'writing'][0]}</h1><p>{pageTitles[mode as 'dictionary'|'writing'][1]}</p></header><WritingSettings {scope} mode={mode as 'dictionary'|'writing'} embedded styleAvailable={speech.provider==='backend'}/></section>{/if}{/each}
  {#if view==='connections'}<section class="detail-page page-enter"><header class="page-heading"><h1>Connections</h1><p>Bring your tools into the conversation.</p></header>{#if connections}{@render connections()}{:else}<p class="session-note">Open the desktop app to manage connected services.</p>{/if}</section>{/if}
  </div>
  {#if copyStatus}<p class="copy-status" role="status">{copyStatus}</p>{/if}
  <div class="model-footer">{#if speech.ready}<Circle size={10} fill="currentColor" aria-hidden="true"/>{/if}<span>{modelLabel} · {speech.language||'Auto-detect'}</span><small>{speech.provider==='local'?'On this Mac':'Connected provider'}</small></div>
 </div>
</section>

<style>
 .daily-ink{height:100%;min-height:0;container:home-shell / size;display:grid;grid-template-rows:minmax(0,1fr);grid-template-columns:clamp(200px,18vw,270px) minmax(0,1fr);background:#000;color:#f1eee7;font-size:14px}
 .sidebar{position:relative;isolation:isolate;display:flex;flex-direction:column;padding:27px 32px 38px;border-right:1px solid #ffffff10;overflow:hidden}
 .brand{display:flex;flex-direction:column;align-items:center;min-height:59px;margin:4px 0 70px;font-family:DestroyEditorial,Georgia,serif;white-space:nowrap}.brand>span{font-size:25px;letter-spacing:.29em}
 nav{display:flex;flex-direction:column;gap:24px;position:relative;z-index:1}
 nav button,.settings-link{position:relative;display:flex;align-items:center;gap:24px;min-height:45px;padding:8px 10px;border:0;background:transparent;border-radius:6px;color:#c9c8c3;text-align:left;font-size:19px}
 nav :global(svg),.settings-link :global(svg){width:25px;height:25px;flex-shrink:0}
 nav button:hover,.settings-link:hover{background:#ffffff05;color:#fff}nav button.active{color:#fff}nav button.active::after{content:'';position:absolute;left:48px;bottom:0;width:64px;height:8px;background:url('/art/selection-brush.png') center / 100% auto no-repeat;mix-blend-mode:lighten}
 .settings-link{margin-top:auto;z-index:1}.sidebar-art{position:absolute;z-index:0;bottom:65px;left:0;width:100%;aspect-ratio:270 / 340;background:url('/art/daily-ink-approved-source.png') left 90.53% / 550.7407% auto no-repeat;mix-blend-mode:lighten;pointer-events:none}
 .content{position:relative;isolation:isolate;display:flex;flex-direction:column;min-width:0;overflow:auto;scrollbar-width:thin}
 .page-stage{position:relative;flex:1;z-index:1;min-width:0}.detail-page[hidden]{display:none}
 .overview{position:relative;z-index:1}
 .sidebar-art{clip-path:inset(0 2px 0 0)}
 .sidebar-art,.approved-art{filter:contrast(1.12)}
 .panorama{position:relative;isolation:isolate;min-height:370px;height:clamp(350px,44vh,470px);flex-shrink:0;overflow:visible}
 .approved-art{position:absolute;top:-48px;left:0;width:100%;aspect-ratio:1217 / 550;background:url('/art/daily-ink-approved-source.png') right top / 122.1857% auto no-repeat;pointer-events:none;mix-blend-mode:lighten;clip-path:polygon(49% 0,100% 0,100% 98%,45% 98%,45% 91%,0 91%,0 69%,27.8% 69%,30% 60%,33% 50%,40% 40%,49% 33%)}
 .hero-copy{position:relative;z-index:1;padding:52px 48px 0}.hero-copy h1{font-family:DestroyEditorial,Georgia,serif;font-weight:500;font-size:clamp(34px,5.55vw,82px);line-height:1.05;letter-spacing:-.025em;margin:0 0 19px;white-space:nowrap}
 .hero-copy :global(.shortcut-keys){margin-left:18px;gap:18px}
 .hero-copy :global(.shortcut-keys kbd){min-width:120px;height:106px;font-size:43px}
 .ritual{display:flex;align-items:baseline;flex-wrap:wrap;gap:7px;margin:24px 0 0 18px;font-family:-apple-system,BlinkMacSystemFont,sans-serif;font-size:19px;letter-spacing:.01em;color:#b9b9b5}
 .ritual{align-items:center;gap:14px;font-size:17px;color:#fffaf2;width:fit-content;max-width:calc(100% - 18px);padding:9px 12px;background:#000;border-radius:10px;letter-spacing:.01em;text-shadow:0 1px 2px #000}
 .rotating-app{display:inline-flex;align-items:center;padding:0;border:0;background:transparent;color:#f1eee7;font:inherit;cursor:pointer;border-radius:8px;flex-shrink:0}.rotating-app:focus-visible{outline:1px solid #d94c40;outline-offset:5px}.app-stack{display:inline-flex;align-items:center;gap:7px}.app-badge{display:grid;place-items:center;width:42px;height:42px;border:1px solid #dededb;background:#fafaf8;border-radius:8px;box-shadow:0 3px 8px #0003}.app-badge img{width:28px;height:28px;object-fit:contain}
 .live-ink{position:absolute;right:0;bottom:-10px;width:48%;opacity:0;transition:opacity 160ms}.live-ink.listening{opacity:1}
 .overview{padding:0 42px 26px}.activity{display:flex;justify-content:space-between;align-items:center;gap:20px;margin:0 0 42px}.activity p{margin:0 0 10px;color:#aaa9a2}.totals{display:flex;align-items:baseline;gap:30px;font-family:DestroyEditorial,Georgia,serif;font-size:20px}.totals strong{font-size:43px;font-weight:400;margin-right:5px}.totals>span+span{padding-left:28px;border-left:1px solid #ffffff12}.totals small{display:block;font:10px -apple-system,BlinkMacSystemFont,sans-serif;color:#888c85;text-align:right}time{font-size:12px;line-height:1.65;text-align:right;color:#aaa9a2}
 .home-columns{display:grid;grid-template-columns:minmax(0,1.6fr) minmax(220px,1fr);gap:32px}h2{font:500 28px/1.2 DestroyEditorial,Georgia,serif;margin:0 0 22px}.recent header{display:flex;justify-content:space-between;align-items:baseline;gap:15px}.recent header>span{color:#91938c;font-size:9px;letter-spacing:.15em;white-space:nowrap}.recent article{display:flex;gap:12px;align-items:start;margin-bottom:24px}.recent article>div{min-width:0}.recent h3{font:500 21px/1.2 DestroyEditorial,Georgia,serif;margin:0 0 7px}.recent article p{display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:2;line-clamp:2;overflow:hidden;overflow-wrap:anywhere;color:#f1eee7;margin:0 0 10px;font:500 21px/1.35 DestroyEditorial,Georgia,serif}.recent small{color:#90938c;font-size:11px}.recent article button,.history-item button{padding:6px;border:0;background:transparent;color:#aaa9a2;margin-left:auto}.personalize{border-left:1px solid #ffffff12;padding-left:28px}.personalize button{display:flex;align-items:center;gap:13px;text-align:left;border:0;background:transparent;padding:10px 0 18px;width:100%;font-size:13px;line-height:1.4}.personalize button span{flex:1}.personalize button:hover{color:#e46551}.all-history{display:flex;align-items:center;gap:6px;padding:4px 0;background:transparent;border:0;color:#aaa9a2;font-size:11px}.empty{color:#aaa9a2;font-size:13px;line-height:1.7;padding:8px 0 20px}.empty h3{color:#e0dbd1}.empty p{margin:8px 0}
 .model-footer{display:flex;align-items:center;gap:10px;margin-top:auto;padding:20px 42px 26px;color:#d1d0c8;font-size:12px}.model-footer small{color:#93968f;font-size:11px}.detail-page{padding:36px 42px;flex:1;min-width:0}.detail-page h1{font:500 46px/1.1 DestroyEditorial,Georgia,serif;margin:0 0 12px}.detail-page header>p{font-size:13px;color:#aaa9a2;line-height:1.6}.history-search{display:flex;flex-direction:row;align-items:center;gap:10px;max-width:460px;background:#ffffff05;border-radius:8px;padding:10px 12px;margin:26px 0}.history-search input{background:transparent;border:0;color:#f1eee7;min-width:0;width:100%;font-size:14px}.history-item{display:flex;align-items:start;gap:20px;padding:18px 0}.history-item p{white-space:pre-wrap;overflow-wrap:anywhere;line-height:1.7;max-width:65ch}.history-item small{color:#aaa9a2}.copy-status{padding:0 42px;font-size:12px;color:#d1d0c8}
 @media(min-width:1051px){.totals>span:first-child{min-width:194px}.home-columns{min-height:277px}.overview .activity{margin-bottom:34px}}
 /* Reference composition: 270px rail, 54px body inset, editorial activity area. */
 .overview{padding-inline:54px}.activity{margin-bottom:50px}.activity p{font-size:19px;margin-bottom:8px}.totals{font-size:24px;gap:40px}.totals strong{font-size:50px}.totals>span+span{padding-left:38px}.totals small{font-size:10px}.home-columns{grid-template-columns:minmax(0,1.4fr) minmax(260px,1fr);gap:44px}h2{font-size:30px}.personalize{padding-left:34px}.personalize button{font-size:18px;gap:32px;padding:12px 0 26px}.personalize :global(svg){flex-shrink:0}.recent article p{font-size:25px}.empty{font-size:18px}.recent h3{font-size:25px}.recent small{font-size:15px}.recent header>span{font-size:12px}.model-footer{padding:20px 54px 44px;font-size:17px}.model-footer small{font-size:15px}time{font-size:17px}
 @media(max-width:1050px){.daily-ink{grid-template-columns:180px minmax(0,1fr)}.sidebar{padding-inline:16px}.brand>span{font-size:19px}nav button,.settings-link{font-size:15px;gap:14px}.hero-copy,.detail-page{padding-inline:28px}.overview{padding-inline:28px}.model-footer{padding-inline:28px}.home-columns{gap:20px;grid-template-columns:minmax(0,1.2fr) minmax(190px,1fr)}.personalize{padding-left:20px}.personalize button{font-size:14px;gap:14px}.hero-copy h1{font-size:42px}.totals{gap:18px}.totals strong{font-size:36px}.totals>span+span{padding-left:18px}.ritual{font-size:15px;gap:10px;letter-spacing:.08em}.approved-art{top:0}.recent header>span{font-size:9px}}
 @media(max-width:760px){.daily-ink{grid-template-columns:66px minmax(0,1fr)}.sidebar{padding:24px 8px}.brand{display:none}nav{gap:12px}nav button,.settings-link{justify-content:center;padding:10px;gap:0}nav button span,.settings-link span{display:none}nav button.active::after{left:10px;width:28px}.sidebar-art{width:250px;height:300px;opacity:.7}.hero-copy h1{font-size:36px;white-space:normal;max-width:300px}.panorama{min-height:340px;height:340px;overflow:hidden}.approved-art{width:760px;left:auto;right:0;opacity:.7}.home-columns{grid-template-columns:1fr}.personalize{border-left:0;padding:0}.personalize button{max-width:330px}.activity{margin-bottom:28px}.totals strong{font-size:30px}.totals{font-size:16px}time{display:none}.model-footer{flex-wrap:wrap}.recent header>span{font-size:8px}.detail-page{padding:26px 22px}}
 @media(max-width:760px){.sidebar-art{display:none}}
 .approved-art{z-index:0;transform-origin:top right;transition:transform 620ms cubic-bezier(.22,1,.36,1),opacity 380ms ease;will-change:transform}
 .compact .approved-art{transform:translate(-70px,4px) scale(.55)}
 .detail-page{padding:26px 48px 36px}
 .page-heading{position:relative;min-height:150px;max-width:65%;margin-bottom:0}
 .detail-page .page-heading h1{font:500 clamp(44px,5vw,70px)/1.05 DestroyEditorial,Georgia,serif;letter-spacing:-.02em;margin:0 0 14px}
 .detail-page .page-heading p{font-size:21px;line-height:1.45;margin:0;color:#b6b5b0}
 .page-enter{animation:page-settle 340ms cubic-bezier(.22,1,.36,1) both}
 @keyframes page-settle{from{opacity:0;transform:translateY(10px)}to{opacity:1;transform:translateY(0)}}
 .history-tabs{position:relative;display:flex;width:min(375px,100%);padding:4px;background:#131313;border:1px solid #ffffff0c;border-radius:999px;margin:18px 0 28px}
 .history-tabs button{flex:1;z-index:1;background:transparent;border:0;padding:10px 15px;border-radius:999px;color:#aaa;font-size:18px;transition:color 180ms}
 .history-tabs button[aria-pressed=true]{color:#fff}.tab-marker{position:absolute;inset:4px auto 4px 4px;width:calc(50% - 4px);background:#303030;border-radius:999px;transition:transform 320ms cubic-bezier(.22,1,.36,1)}.tab-marker.second{transform:translateX(100%)}
 .history-toolbar{display:flex;align-items:center;gap:20px;margin-bottom:36px}.history-search{flex:1;max-width:750px;margin:0;padding:15px 20px;border:1px solid #ffffff12;background:#141414;border-radius:999px;color:#aaa}.history-search input{font-size:18px;outline:0}.history-search:focus-within{border-color:#d95749}.clear-session{margin-left:auto;background:transparent;border:0;color:#aaa;text-decoration:underline;font-size:16px;padding:12px}.clear-session:disabled{opacity:.35;cursor:default}
 .history-item{border:1px solid #ffffff08;border-radius:20px;padding:28px;gap:28px;background:#111;margin-bottom:18px;display:grid;grid-template-columns:130px minmax(0,1fr) 38px 75px}.history-item time{font-size:15px;text-align:left;padding-top:8px}.history-item p{font-size:19px;line-height:1.5;margin:0;color:#b5b4af}.history-item small{font-size:14px;padding-top:10px;white-space:nowrap}.history-item button{margin:0;border-radius:8px}.history-item button:hover{color:#fff;background:#ffffff0a}.history-empty{padding:46px 28px;background:#101010;border-radius:20px}.history-empty h2{margin-bottom:10px}.history-empty p,.session-note{font-size:15px;line-height:1.6;color:#8e8e89}.session-note{margin:34px 0 0}.clear-confirm{background:#241513;padding:18px;border-radius:16px;margin-bottom:22px}.clear-confirm p{font-size:15px}.clear-confirm button{border:0;padding:10px 14px;margin-right:8px;border-radius:10px}.confirm-clear{color:#ff9586}
 .insights>p{font-size:18px;color:#aaa}.insight-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}.insight-grid article{display:flex;flex-direction:column;gap:12px;padding:28px;background:#111;border-radius:20px}.insight-grid strong{font:500 46px DestroyEditorial,Georgia,serif}.insight-grid strong small{font-size:22px}.insight-grid span{color:#aaa;font-size:16px}
 .history-search input{padding:0;height:26px}.history-list-label{font-size:20px;color:#aaa;margin:0 0 16px}.history-item time{display:block}
 .daily-ink button:focus-visible{outline:2px solid #dc6655;outline-offset:4px}
 @media(max-width:1050px){.detail-page{padding-inline:28px}.page-heading{min-height:150px;max-width:70%}.detail-page .page-heading p{font-size:17px}.history-item{grid-template-columns:minmax(0,1fr) 38px;gap:12px}.history-item time{grid-column:1 / -1}.history-item small{grid-column:1}.history-item p{font-size:16px}.history-tabs button{font-size:16px}}
 @media(max-width:760px){.compact .approved-art{transform:translateY(0) scale(.38);opacity:.8}.page-heading{max-width:100%;padding-top:90px;min-height:220px}.detail-page{padding:24px 22px}.detail-page .page-heading h1{font-size:44px}.history-toolbar{gap:8px;flex-wrap:wrap}.history-search{min-width:100%}.history-item{padding:22px}.model-footer{font-size:14px}.history-tabs{margin-top:0}}
 .insight-grid{grid-template-columns:repeat(3,minmax(0,1fr));gap:clamp(8px,1.5vw,16px)}.insight-grid article{min-width:0;padding:clamp(14px,2vw,28px)}.insight-grid strong{font-size:clamp(28px,4vw,46px);overflow-wrap:anywhere}.insight-grid span{font-size:clamp(12px,1.5vw,16px);line-height:1.4}.insight-grid strong small{font-size:clamp(14px,2vw,22px)}
 /* Home is a viewport composition, not a document. Reserve the overview and
    footer first, then let the artwork/shortcut area use the remaining height.
    Container height also accounts for native titlebar and error banners. */
 .home-fit{overflow:clip;min-height:0}
 .home-fit .page-stage{flex:1;min-height:0;display:grid;grid-template-rows:minmax(0,1fr) auto}
 .home-fit .panorama{height:auto;min-height:0}
 .home-fit .hero-copy{padding-top:clamp(12px,4cqh,52px)}
 .home-fit .hero-copy h1{font-size:clamp(30px,min(5.55vw,8cqh),82px);margin-bottom:clamp(12px,2cqh,19px)}
 .home-fit .hero-copy :global(.shortcut-keys kbd){min-width:clamp(64px,13cqh,120px);height:clamp(58px,12cqh,106px);font-size:clamp(28px,5cqh,43px)}
 .home-fit .ritual{margin-top:clamp(16px,3cqh,24px);font-size:clamp(13px,2cqh,17px)}
 .home-fit .overview{padding-bottom:0}
 .home-fit .activity{margin-bottom:clamp(12px,3cqh,34px)}
 .home-fit .activity p{font-size:clamp(13px,2cqh,19px);margin-bottom:4px}
 .home-fit .totals{font-size:clamp(16px,2.5cqh,24px)}
 .home-fit .totals strong{font-size:clamp(28px,5cqh,50px)}
 .home-fit .home-columns{min-height:0}
 .home-fit h2{font-size:clamp(21px,3.2cqh,30px);margin-bottom:clamp(10px,2cqh,22px)}
 .home-fit .recent article{margin-bottom:clamp(10px,1.7cqh,24px)}
 .home-fit .recent article p{font-size:clamp(17px,2.6cqh,25px);margin-bottom:4px}
 .home-fit .recent small{font-size:clamp(11px,1.6cqh,15px)}
 .home-fit .empty{font-size:clamp(13px,1.9cqh,18px);padding:0}
 .home-fit .empty h3{font-size:clamp(18px,2.6cqh,25px)}
 .home-fit .personalize button{font-size:clamp(13px,1.9cqh,18px);gap:14px;padding:8px 0 12px}
 .home-fit .model-footer{flex:none;padding-block:clamp(10px,2cqh,20px);font-size:clamp(12px,1.8cqh,17px)}
 .home-fit .model-footer small{font-size:inherit}
 .home-fit .copy-status{position:absolute;bottom:44px;right:24px;z-index:3;padding:8px 12px;border-radius:8px;background:#191919}
 @container home-shell (max-height:650px){
  .sidebar{padding-top:16px;padding-bottom:16px}.brand{min-height:36px;margin-bottom:clamp(12px,4cqh,26px)}nav{gap:clamp(4px,2cqh,12px)}
  .home-fit .hero-copy{padding-top:12px}
  .home-fit .approved-art{width:min(100%,135cqh);left:auto;right:0;top:-20px}
  .home-fit .recent article p{-webkit-line-clamp:1;line-clamp:1}
  .home-fit .recent header>span{font-size:8px}
  .home-fit .ritual{padding:5px 8px;gap:8px;margin-left:0}
  .home-fit .app-badge{width:30px;height:30px}.home-fit .app-badge img{width:22px;height:22px}
  .home-fit .personalize :global(svg){width:20px;height:20px}
  .home-fit time{font-size:13px}
 }
 @media(max-width:760px){
  .home-fit .home-columns{grid-template-columns:minmax(0,1.2fr) minmax(0,1fr);gap:16px}
  .home-fit .recent header>span{display:none}
  .home-fit .hero-copy,.home-fit .overview,.home-fit .model-footer{padding-inline:18px}
  .home-fit .hero-copy h1{max-width:none;white-space:nowrap;font-size:clamp(28px,6vw,36px)}
  .home-fit .personalize button{gap:8px;font-size:12px;padding:6px 0 10px}
  .home-fit .personalize :global(svg){width:18px;height:18px}
  .home-fit .ritual{letter-spacing:0;gap:8px;max-width:100%;margin-left:0;font-size:13px}
  .home-fit .app-badge{width:30px;height:30px}.home-fit .app-badge img{width:22px;height:22px}
  .home-fit .model-footer{gap:6px;font-size:12px}
  .home-fit .totals>span:first-child{min-width:0}
 }
 @media(prefers-reduced-motion:reduce){.live-ink,.approved-art,.tab-marker,.history-tabs button{transition:none}.page-enter{animation:none}}
</style>

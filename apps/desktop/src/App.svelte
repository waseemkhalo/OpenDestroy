<script lang="ts">
 import {onMount} from 'svelte';
 import {initialHud,sanitizeHudSnapshot,runOwnedHudAction,currentHudAction,type HudAction} from './lib/notch/hud';
 const native=typeof window!=='undefined'&&'__TAURI_INTERNALS__' in window;
 import {check,type Update} from '@tauri-apps/plugin-updater';
 import {invoke} from '@tauri-apps/api/core';
 import {listen} from '@tauri-apps/api/event';
 import {setAccount} from './lib/account';
 import {appendMediaResults} from './lib/mediaPages';
 import {Capture,bytesBase64} from './lib/capture';
 import Dashboard from './lib/dictation/DictationDashboard.svelte';
 import {loadDictationPreferences,transformDictation,isUndoDictationCommand,isPreviousDictationCorrection,extractTerminalRewriteCommand,resetDictationComposerCache} from './lib/dictation/dictationComposer';
 import {recordDictation,clearDictationMemory} from './lib/dictation/dictationState';
 import {extractDictationEmojiIntent,emojiChoices,type DictationEmojiChoice} from './lib/dictation/emoji';
 import {extractDictationMediaIntent,parseDictationMediaVoiceChoice,searchDictationMedia,saveDictationMediaFavorite,loadDictationMediaFavorites,favoriteAsMediaResult,refreshDictationMediaAvailability,type DictationMediaResult,type DictationMediaKind} from './lib/dictation/giphy';
 import {extractDictationAttachmentIntent} from './lib/dictation/attachment';
 import {isVoiceNoteCommand} from './lib/dictation/voiceNote';
 import MediaPicker from './lib/dictation/DictationMediaPicker.svelte';
 import EmojiPicker from './lib/dictation/DictationEmojiPicker.svelte';
 import {dictationDeliveryFeedback} from './lib/dictation/accessibilityFeedback';
 type Target={canPaste:boolean;appName:string;bundleId:string;appKind:string;appIconDataUrl?:string};
 type Start={sessionId:string;target:Target;needsAccessibility:boolean};
 type Link={name:string;url:string;keywords?:string};
 let recovery=$state(''),recoveryTimer:ReturnType<typeof setTimeout>|undefined;
 function clearRecovery(){clearTimeout(recoveryTimer);recovery='';}
 function retainRecovery(text:string){clearRecovery();recovery=text;recoveryTimer=setTimeout(clearRecovery,30000);}
 let camera=$state({width:210,height:34});
 let user=$state<string|null>(null),url=$state('http://127.0.0.1:8787'),token=$state(''),error=$state(''),message=$state('Connect your backend to begin.'),expanded=$state(true),tab=$state<'dictation'|'connection'>('connection');
 let phase=$state<'idle'|'recording'|'processing'|'picker'>('idle'),target=$state<Target|null>(null),needsAccessibility=$state(false),level=$state(0),shortcut=$state('CommandOrControl+Backquote'),mic=$state(localStorage.getItem('destroy.microphone')||''),devices=$state<MediaDeviceInfo[]>([]),permissions=$state({microphone:false,accessibility:false});
 let connectedServer=$state(''),connectionRevision=0,connectionAction=0,appDisposed=false;
 const scope=$derived(user?connectedServer+"|"+user:null);
 const account=()=>({expectedUserId:user,expectedBackendUrl:connectedServer,revision:connectionRevision});
 const owns=(owner:ReturnType<typeof account>)=>!appDisposed&&owner.revision===connectionRevision&&owner.expectedUserId===user&&owner.expectedBackendUrl===connectedServer;
 let capture:Capture|null=null,session:string|null=null,generation=0,startedAt=0;
 let emoji=$state<DictationEmojiChoice[]>([]),media=$state<DictationMediaResult[]>([]),links=$state<Link[]>([]),leading=$state(''),query=$state(''),kind=$state<DictationMediaKind>('gif'),page=$state(0),busy=$state(false),voiceReady=$state(false),mediaOpen=$state(false);
 let updatesEnabled=$state(false);
 let favorites=$state<{id:string;title:string}[]>([]),update=$state<Update|null>(null),updateMessage=$state('No update feed configured for this community build.');
 let vars=$state('{}'),linksJson=$state('[]'),deleteConfirm=$state(false),connecting=$state(false);
 const api=<T,>(method:string,path:string,body:unknown=null,owner=account())=>{if(!owner.expectedUserId||!owns(owner))return Promise.reject(new Error('Connection changed. Try again.'));return invoke<T>('backend_api',{method,path,body,expectedUserId:owner.expectedUserId,expectedBackendUrl:owner.expectedBackendUrl});};
 function fail(e:unknown){error=e instanceof Error?e.message:String(e);}
 let mediaPool:DictationMediaResult[]=[],mediaOffset=0,mediaEnded=false,mediaRequest=0,mediaInitialized=false;
 function resetMedia(){mediaRequest++;mediaPool=[];mediaOffset=0;mediaEnded=false;mediaInitialized=false;page=0;media=[];}
 function clearPicker(){resetMedia();emoji=[];media=[];links=[];leading='';query='';page=0;mediaOpen=false;}
 async function panel(open:boolean,focus=true){if(native)await invoke('show_panel',{expanded:open,focus});}
 async function cancel(){hudVisible=false;clearRecovery();const cancelledGeneration=++generation;capture?.cancel();capture=null;const id=session;session=null;phase='idle';voiceReady=false;busy=false;clearPicker();if(id)await invoke('destroy_dictation_cancel',{sessionId:id}).catch(()=>{});if(cancelledGeneration===generation&&!appDisposed)message='Cancelled';}
 let hudVisible=$state(false),hudFailure=$state(''),hudRevision=Date.now(),currentHud=initialHud();
 $effect(()=>{
  if(!hudVisible||phase!=='idle')return;
  const timer=setTimeout(()=>hudVisible=false,recovery||needsAccessibility||error?30000:4500);
  return()=>clearTimeout(timer);
 });
 $effect(()=>{
  currentHud=sanitizeHudSnapshot({revision:++hudRevision,visible:hudVisible,phase,camera,target,message,error,needsAccessibility,recovery:!!recovery,voiceReady,emoji,media,links,leading,query,kind,page,busy,mediaOpen});
  if(native)void invoke('update_hud',{snapshot:currentHud}).catch(e=>{hudFailure=String(e);void panel(true,false);});
 });
 async function hudAction(action:HudAction){
  if(!hudVisible||!currentHudAction(action,currentHud))return;
  const gen=generation,owner=user,server=connectedServer,accountRevision=connectionRevision,id=session;
  const current=()=>!appDisposed&&hudVisible&&gen===generation&&owner===user&&server===connectedServer&&accountRevision===connectionRevision&&id===session;
  try{
   if(action.action==='cancel')await cancel();
   else if(action.action==='open'){tab=user?'dictation':'connection';await panel(true);}
   else if(action.action==='copy'&&recovery){const text=recovery;await runOwnedHudAction(()=>invoke('destroy_dictation_copy',{text}),current,()=>{clearRecovery();message='Recognized text copied. Review it before pasting.';},fail);}
   else if(action.action==='accessibility'){await runOwnedHudAction(()=>invoke<boolean>('request_accessibility_access'),current,allowed=>{permissions.accessibility=allowed;needsAccessibility=!allowed;},fail);}
   else if(action.action==='choose')await choose(action.index!);
   else if(action.action==='favorite')await favorite(action.index!);
   else if(action.action==='rotate')await rotate();
   else if(action.action==='kind'&&action.kind)await changeKind(action.kind);
  }catch(e){if(current())fail(e);}
 }
 async function boundary(){connectionRevision++;setAccount(null);connectedServer='';user=null;target=null;needsAccessibility=false;error='';vars='{}';linksJson='[]';favorites=[];token='';deleteConfirm=false;clearDictationMemory();resetDictationComposerCache();await cancel();}
 async function connect(){
  if(connecting||!native)return;
  connecting=true;error='';const submittedUrl=url,submittedToken=token,attempt=++connectionAction;
  await cancel();
  try{
   const result=await invoke<{user_id:string}>('configure_backend',{backendUrl:submittedUrl,token:submittedToken});
   if(appDisposed||attempt!==connectionAction)return;
   token='';connectionRevision++;connectedServer=new URL(submittedUrl).origin;user=result.user_id;setAccount(user,connectedServer);
   message='Ready. Hold your shortcut in another app.';tab='dictation';await loadExtras();
  }catch(e){if(!appDisposed&&attempt===connectionAction)fail(e)}finally{if(attempt===connectionAction)connecting=false;}
 }
 async function loadExtras(){
  const owner=account();if(!owner.expectedUserId)return;
  const [v,l,f]=await Promise.all([api('GET','/v1/variables',null,owner),api('GET','/v1/links',null,owner),api<{favorites:{id:string;title:string}[]}>('GET','/v1/dictation/media/favorites',null,owner)]);
  if(!owns(owner))return;
  vars=JSON.stringify(v,null,2);linksJson=JSON.stringify(l,null,2);favorites=f.favorites;
 }
 async function refreshDevices(){devices=(await navigator.mediaDevices.enumerateDevices()).filter(d=>d.kind==='audioinput');}
 async function allowMic(){try{await invoke('request_microphone_access');permissions=await invoke('get_permission_status');if(permissions.microphone){const stream=await navigator.mediaDevices.getUserMedia({audio:true});stream.getTracks().forEach(t=>t.stop());await refreshDevices();}}catch(e){fail(e)}}
 async function start(value:Start){
 if(phase==='recording'||phase==='processing')return;
 if(!user){await invoke('destroy_dictation_cancel',{sessionId:value.sessionId});message='Connect your backend in Settings first.';hudVisible=true;return;}
 clearRecovery();const gen=++generation;session=value.sessionId;target=value.target;needsAccessibility=value.needsAccessibility;error='';phase='recording';startedAt=Date.now();
 hudFailure='';hudVisible=true;const current=new Capture(v=>{if(gen===generation)level=v;},()=>{void stop()});capture=current;
 try{await current.start(mic||null);if(gen!==generation)current.cancel();}catch(e){if(gen!==generation)return;current.cancel();const cancelled=cancel();const cancellationGeneration=generation;await cancelled;if(appDisposed||generation!==cancellationGeneration)return;fail(e);hudVisible=true;}
 }
 async function stop(){
 if(phase!=='recording'||!capture)return;const current=capture,gen=generation,id=session;phase='processing';const note=voiceReady;let recognized='';
 try{const result=await current.finish();if(gen!==generation||!id)return;capture=null;const text=result.text.trim();recognized=text;if(!text)throw new Error('No speech was recognized. Check the microphone and try again.');
 if(note){voiceReady=false;const audioBase64=bytesBase64(new Uint8Array(await result.blob.arrayBuffer()));if(gen!==generation)return;const delivery=await invoke<any>('destroy_dictation_deliver_voice_note',{sessionId:id,audioBase64,contentType:result.blob.type,transcript:text});if(gen!==generation)return;done(delivery,text);return;}
 if(emoji.length||mediaOpen||links.length){phase='picker';const choice=parseDictationMediaVoiceChoice(text);if(choice?.type==='cancel'){await cancel();return;}if(choice?.type==='kind'&&mediaOpen){await changeKind(choice.kind);return;}if(choice?.type==='rotate'){await rotate();return;}if(choice?.type==='select'){await choose(choice.index);return;}throw new Error('Say one, two, three, more, or cancel.');}
 if(isVoiceNoteCommand(text)){voiceReady=true;phase='picker';message='Voice note ready. Hold again to record the note.';return;}
 if(isUndoDictationCommand(text)){const ok=await invoke<boolean>('destroy_dictation_undo',{sessionId:id});if(gen!==generation)return;message=ok?'Undid the last insertion.':'No recent insertion in this exact field to undo.';await invoke('destroy_dictation_cancel',{sessionId:id});if(gen!==generation)return;session=null;phase='idle';return;}
 const prefs=await loadDictationPreferences(scope);if(gen!==generation)return;
 const selection=prefs.selected_text_editing?await invoke<string|null>('destroy_dictation_selected_text',{sessionId:id}):null;if(gen!==generation)return;
 if(selection){await compose(text,'edit_selected',selection);return;}
 if(isPreviousDictationCorrection(text)){const previous=await invoke<string|null>('destroy_dictation_previous_text',{sessionId:id});if(gen!==generation)return;if(!previous)throw new Error('No recent dictation in this exact field to correct.');await compose(text,'correct_previous',null,previous);return;}
 const ei=extractDictationEmojiIntent(text);if(ei){leading=ei.leadingText;query=ei.query;emoji=emojiChoices(ei.query);phase='picker';message='Choose an emoji';return;}
 const mi=extractDictationMediaIntent(text);if(mi){leading=mi.leadingText;query=mi.query;kind=mi.kind;mediaOpen=true;phase='picker';await searchMedia();return;}
 const file=extractDictationAttachmentIntent(text);if(file){leading=file.messageText;query=file.query;const result=await api<{results:Link[]}>('POST','/v1/links/search',{query});if(gen!==generation)return;links=result.results;phase='picker';message='Choose a saved link. Recipient access is unchanged.';if(!links.length)throw new Error('No saved link matched. Add a named HTTPS link in Connection settings.');return;}
 const rewrite=extractTerminalRewriteCommand(text);await compose(rewrite??text,rewrite!==null?'rewrite':'dictate');
 }catch(e){if(gen===generation){current.cancel();phase=(emoji.length||mediaOpen||links.length)?'picker':'idle';if(phase==='idle'){await invoke('destroy_dictation_cancel',{sessionId:id}).catch(()=>{});if(gen!==generation)return;session=null;voiceReady=false;if(recognized&&!note)retainRecovery(recognized);}fail(e)}}
 }
 async function compose(text:string,operation:'dictate'|'rewrite'|'edit_selected'|'correct_previous',selectedText:string|null=null,previousText:string|null=null){const gen=generation,id=session;if(!id)return;const result=await transformDictation({text,appKind:target?.appKind||'generic',operation,selectedText,previousText});if(gen!==generation)return;const command=operation==='correct_previous'?'destroy_dictation_replace_last':'destroy_dictation_deliver';const delivery=await invoke<any>(command,{sessionId:id,text:result.text});if(gen!==generation)return;if(delivery===false)throw new Error('The recent field changed; correction was not inserted.');done(typeof delivery==='boolean'?{delivery:'inline'}:delivery,result.text);}
 function done(delivery:any,text:string){const feedback=dictationDeliveryFeedback(delivery.reason??null,false);message=delivery.delivery==='inline'?'Inserted into '+(target?.appName||'the original field')+'.':feedback.hint==='accessibility'?'Copied. Allow Accessibility, then press Command-V.':'Copied. The original field was unavailable; press Command-V to paste.';needsAccessibility=delivery.reason==='needs_accessibility';phase='idle';session=null;clearPicker();recordDictation({teamId:scope,text,elapsedMs:Date.now()-startedAt});}
 async function searchMedia(){
 const gen=generation,request=++mediaRequest,wanted=(page+1)*3;busy=true;error='';
 const current=()=>gen===generation&&request===mediaRequest;
 try{
  if(!mediaInitialized){
   if(!await refreshDictationMediaAvailability())throw new Error('Media is unavailable. The backend operator must configure an approved GIPHY integration.');
   if(!current())return;
   const saved=await loadDictationMediaFavorites();if(!current())return;
   mediaPool=saved.filter(f=>f.kind===kind&&f.query.toLowerCase()===query.toLowerCase()).map(favoriteAsMediaResult);mediaInitialized=true;
  }
  while(mediaPool.length<wanted&&!mediaEnded&&mediaOffset<60){
   const result=await searchDictationMedia(query,kind,mediaOffset);if(!current())return;
   mediaOffset+=3;mediaEnded=result.results.length<3;mediaPool=appendMediaResults(mediaPool,result.results);
  }
  if(!current())return;
  if(page*3>=mediaPool.length)page=0;
  media=mediaPool.slice(page*3,page*3+3);
 }catch(e){if(current())fail(e)}finally{if(current())busy=false;}
 }
 async function changeKind(value:DictationMediaKind){kind=value;resetMedia();await searchMedia();}
 async function rotate(){if(!mediaOpen||busy)return;page=(page+1)%10;await searchMedia();}
 async function choose(index:number){if(busy||!session)return;const gen=generation,id=session;busy=true;error='';try{let delivery:any,text='';if(emoji[index]){text=[leading,emoji[index].emoji].filter(Boolean).join(' ');delivery=await invoke('destroy_dictation_deliver_picker_text',{sessionId:id,text});}else if(links[index]){text=[leading,links[index].url].filter(Boolean).join('\n');delivery=await invoke('destroy_dictation_deliver_picker_text',{sessionId:id,text});}else if(media[index]){const r=media[index];text=leading;delivery=await invoke('destroy_dictation_deliver_media',{sessionId:id,leadingText:leading,contentUrl:r.content_url,sourceUrl:r.source_url,altText:r.alt_text});}else{return;}if(gen===generation)done(delivery,text);}catch(e){if(gen===generation)fail(e)}finally{if(gen===generation)busy=false;}}
 async function favorite(index:number){const r=media[index];if(!r)return;const gen=generation;try{const saved=await saveDictationMediaFavorite(query,r);if(gen!==generation)return;media=media.map(v=>v.id===r.id?{...v,favorite_id:saved.id}:v);mediaPool=mediaPool.map(v=>v.id===r.id?{...v,favorite_id:saved.id}:v);message='Favorite saved.';}catch(e){if(gen===generation&&!appDisposed)fail(e)}}
 function keys(e:KeyboardEvent){if(e.key==='Escape'){void cancel();return;}if((e.target as HTMLElement)?.matches('input,textarea,select'))return;if(phase==='picker'){if(['1','2','3'].includes(e.key))void choose(Number(e.key)-1);if(e.key==='r'||e.key==='ArrowDown')void rotate();}}
 async function saveExtras(){const owner=account();try{const variables=JSON.parse(vars),savedLinks=JSON.parse(linksJson);await api('PUT','/v1/variables',variables,owner);if(!owns(owner))return;await api('PUT','/v1/links',savedLinks,owner);if(owns(owner))message='Variables and links saved.';}catch(e){if(owns(owner))fail(e)}}
 async function exportData(){const owner=account();try{const path=await invoke<string>('export_personal_data');if(owns(owner))message='Personal data saved to '+path;}catch(e){if(owns(owner))fail(e)}}
 async function disconnect(){const attempt=++connectionAction;await boundary();try{await invoke('disconnect_backend');if(!appDisposed&&attempt===connectionAction){tab='connection';message='Disconnected.';return true;}}catch(e){if(!appDisposed&&attempt===connectionAction)fail(e)}return false;}
 async function deleteData(){const owner=account();try{await cancel();if(!owns(owner))return;await api('DELETE','/v1/account/data',null,owner);if(!owns(owner))return;if(!await disconnect())return;for(const key of Object.keys(localStorage))if(key.startsWith('destroy'))localStorage.removeItem(key);message='Personal data deleted. Backups follow your server retention policy.';}catch(e){if(owns(owner))fail(e)}}
 async function removeFavorite(id:string){const owner=account();try{await api('DELETE','/v1/dictation/media/favorites/'+encodeURIComponent(id),null,owner);if(owns(owner))favorites=favorites.filter(f=>f.id!==id);}catch(e){if(owns(owner))fail(e)}}
 async function restoreConnection(){
  const revision=++connectionRevision;
  try{const c=await invoke<{user_id:string;backend_url:string}>('restore_connection');if(appDisposed||revision!==connectionRevision)return;user=c.user_id;connectedServer=c.backend_url;setAccount(user,connectedServer);url=c.backend_url;tab='dictation';message='Ready. Hold your shortcut in another app.';await loadExtras();}
  catch(e){if(!appDisposed&&revision===connectionRevision)message='Connect your backend to begin.';}
 }
 onMount(()=>{
  appDisposed=false;
  if(!native){message='Interface preview. Run npm run dev to use the native Mac app.';return()=>{appDisposed=true;};}
  const off:(()=>void)[]=[];
  void(async()=>{try{
   for(const [name,handler]of [
    ['destroy://dictation-start',(e:any)=>void start(e.payload)],
    ['destroy://dictation-stop',()=>void stop()],
    ['destroy://hud-action',(e:any)=>void hudAction(e.payload)],
    ['destroy://dictation-cancel',()=>{if(hudVisible)void cancel();}],
    ['destroy://account-changed',()=>void boundary()]
   ] as const){const fn=await listen(name,handler);if(appDisposed){fn();return;}off.push(fn);}
   camera=await invoke('notch_geometry');shortcut=await invoke('get_shortcut');await invoke('shortcut_status').catch(fail);permissions=await invoke('get_permission_status');updatesEnabled=await invoke('updater_available');if(appDisposed)return;await restoreConnection();await refreshDevices();
  }catch(e){if(!appDisposed)fail(e)}})();
  return()=>{appDisposed=true;connectionRevision++;connectionAction++;off.forEach(fn=>fn());void cancel();};
 });
</script>
<svelte:window onkeydown={keys}/>
<main class:compact={!expanded} style={`--camera-width:${camera.width}px;--camera-height:${camera.height}px`}>
 <div class="topline" data-tauri-drag-region><span data-tauri-drag-region>DESTROY <small data-tauri-drag-region>Dictation</small></span><button aria-label="Hide settings" onclick={()=>void panel(false)}>Hide</button></div>
 <div class="status" role="status"><span class:live={phase==='recording'} class="dot"></span><p>{phase==='recording'?'Listening · release to finish':phase==='processing'?'Transcribing…':message}</p>{#if phase!=='idle'||voiceReady}<button onclick={()=>void cancel()}>Cancel <kbd>Esc</kbd></button>{/if}</div>
 {#if hudFailure}<p class="error" role="alert">{hudFailure}</p>{/if}
 {#if error}<p class="error" role="alert">{error}</p>{/if}
 {#if recovery}<button onclick={async()=>{try{await invoke('destroy_dictation_copy',{text:recovery});clearRecovery();message='Recognized text copied. Review it before pasting.';}catch(e){fail(e)}}}>Copy recognized text <small>· clears after 30 seconds</small></button>{/if}
 {#if needsAccessibility}<button class="permission" onclick={async()=>{permissions.accessibility=await invoke('request_accessibility_access');needsAccessibility=!permissions.accessibility;}}>Allow Accessibility for direct insertion</button>{/if}
 {#if emoji.length}<EmojiPicker choices={emoji} selectedIndex={0} {query} leadingText={leading} onSelect={i=>void choose(i)} onCancel={()=>void cancel()}/>{/if}
 {#if mediaOpen}<MediaPicker {kind} {query} leadingText={leading} {page} results={media} selectedIndex={0} loading={busy} delivering={false} {error} attribution="Powered by GIPHY" onKindChange={v=>void changeKind(v)} onSelect={i=>void choose(i)} onFavorite={i=>void favorite(i)} onRotate={()=>void rotate()} onCancel={()=>void cancel()}/>{/if}
 {#if links.length}<div class="link-picker"><p>{leading}</p>{#each links as link,i}<button disabled={busy} onclick={()=>void choose(i)}><kbd>{i+1}</kbd><strong>{link.name}</strong><small>{link.url}</small></button>{/each}<p>Inserts a link. Check access in the file provider before sending.</p></div>{/if}
 {#if expanded}
 <nav><button class:active={tab==='dictation'} onclick={()=>tab='dictation'}>Dictation</button><button class:active={tab==='connection'} onclick={()=>tab='connection'}>Connection & device</button></nav>
 <section class="workspace">
 {#if tab==='dictation'&&user}<Dashboard teamId={scope} permissionRole={null}/>{:else}
 <h1>Your voice.<br><span>Your setup.</span></h1><p class="intro">Connect a self-hosted dictation service. Audio goes to your configured speech provider; provider charges apply.</p>
 <form onsubmit={e=>{e.preventDefault();void connect()}}><label>Backend URL<input bind:value={url} type="url" required/></label><label>Backend access token<input bind:value={token} type="password" autocomplete="off" placeholder={user?'Enter a token to change connection':'From your backend operator'} required/></label><button class="primary" disabled={connecting||!native}>{connecting?'Connecting…':'Connect backend'}</button></form>
 {#if user}<p>Connected as {user}</p><button onclick={()=>void disconnect()}>Disconnect</button>{/if}
 <div class="settings-group"><h2>Microphone & shortcut</h2><div class="row"><span>Microphone {permissions.microphone?'allowed':'required'}</span><button onclick={()=>void allowMic()}>Allow / refresh</button></div><label>Input device<select bind:value={mic} onchange={()=>localStorage.setItem('destroy.microphone',mic)}><option value="">System default</option>{#each devices as device}<option value={device.deviceId}>{device.label||'Microphone'}</option>{/each}</select></label><p class="hint">A pinned microphone must be available. System default follows macOS at the next hold.</p><label>Hold-to-dictate shortcut<input bind:value={shortcut}/></label><button onclick={async()=>{try{await invoke('set_shortcut',{value:shortcut});message='Shortcut saved.';}catch(e){fail(e)}}}>Save shortcut</button><button onclick={async()=>{permissions.accessibility=await invoke('request_accessibility_access');}}>Allow Accessibility</button><p class="hint">Microphone captures your voice. Accessibility enables insertion into the original field. Screen Recording is not required.</p></div>
 {#if user}<div class="settings-group"><h2>Snippet variables & saved links</h2><label>Variables (JSON)<textarea bind:value={vars} spellcheck="false" rows="4"></textarea></label><p class="hint">For example, {`{"my_company":"Your company"}`}. Only variables you define are used.</p><label>Named links (JSON)<textarea bind:value={linksJson} spellcheck="false" rows="5"></textarea></label><p class="hint">{`[{"name":"demo video","url":"https://example.com/demo","keywords":"product walkthrough"}]`}. These are links, not uploaded attachments.</p><button onclick={()=>void saveExtras()}>Save variables and links</button></div><div class="settings-group"><h2>Saved media favorites</h2>{#each favorites as favorite}<div class="row"><span>{favorite.title||'Saved media'}</span><button onclick={()=>void removeFavorite(favorite.id)}>Remove</button></div>{/each}<h2>Your data</h2><button onclick={()=>void exportData()}>Export personal data</button><button onclick={()=>deleteConfirm=true}>Delete personal data…</button>{#if deleteConfirm}<p>Deletes your saved preferences, snippets, favorites, links, vocabulary and usage from this backend and clears this app’s cache.</p><button class="danger" onclick={()=>void deleteData()}>Confirm deletion</button><button onclick={()=>deleteConfirm=false}>Keep data</button>{/if}</div>{/if}
 {/if}</section><div class="updates"><button disabled={!updatesEnabled} onclick={async()=>{try{update=await check();updateMessage=update?'Version '+update.version+' is available.':'No newer version is available.';}catch{updateMessage='Update feed is not configured or could not be reached.';}}}>Check for updates</button>{#if update}<button onclick={async()=>{try{await cancel();await update?.downloadAndInstall();updateMessage='Update installed. Quit and reopen Destroy Dictation.';}catch(e){fail(e)}}}>Install {update.version}</button>{/if}<span>{updateMessage}</span></div><footer><span>0.1.0 · Development build</span><button onclick={()=>void invoke('quit_app')}>Quit Destroy Dictation</button></footer>
 {/if}
</main>

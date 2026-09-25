<script lang="ts">
 import {onMount} from 'svelte';
 import {fly,fade} from 'svelte/transition';
 import {cubicOut} from 'svelte/easing';
 function panelMotion(node:Element){return fly(node,{x:node.getBoundingClientRect().width,duration:window.matchMedia('(prefers-reduced-motion: reduce)').matches?0:360,easing:cubicOut});}
 import {initialHud,sanitizeHudSnapshot,runOwnedHudAction,currentHudAction,deliveryHudFeedback,type HudAction} from './lib/notch/hud';
 const native=typeof window!=='undefined'&&'__TAURI_INTERNALS__' in window;
 import {check,type Update} from '@tauri-apps/plugin-updater';
 import {invoke} from '@tauri-apps/api/core';
 import {listen} from '@tauri-apps/api/event';
 import {setAccount} from './lib/account';
 import {openMicrophoneStream} from './lib/audioDevices';
 import {appendMediaResults} from './lib/mediaPages';
 import {NativeCapture,bytesBase64,isNoSpeechTranscript} from './lib/capture';
 import Dashboard from './lib/dictation/DictationDashboard.svelte';
 import {currentDictationPreferences,loadDictationPreferences,transformDictation,isUndoDictationCommand,isPreviousDictationCorrection,extractTerminalRewriteCommand,resetDictationComposerCache,type DictationPreferences} from './lib/dictation/dictationComposer';
 import {recordDictation,clearDictationMemory} from './lib/dictation/dictationState';
 import {extractDictationEmojiIntent,emojiChoices,type DictationEmojiChoice} from './lib/dictation/emoji';
 import {extractDictationMediaIntent,parseDictationMediaVoiceChoice,searchDictationMedia,saveDictationMediaFavorite,loadDictationMediaFavorites,favoriteAsMediaResult,refreshDictationMediaAvailability,mediaDeliveryRequest,type DictationMediaResult,type DictationMediaKind} from './lib/dictation/giphy';
 import {extractDictationAttachmentIntent,extractDictationDriveIntent,searchGoogleDriveLinks} from './lib/dictation/attachment';
 import {isVoiceNoteCommand} from './lib/dictation/voiceNote';
 import MediaPicker from './lib/dictation/DictationMediaPicker.svelte';
 import EmojiPicker from './lib/dictation/DictationEmojiPicker.svelte';
 import Onboarding from './lib/Onboarding.svelte';
 import LocalModelPicker from './lib/LocalModelPicker.svelte';
 import HomeScreen from './lib/HomeScreen.svelte';
 import Settings from './lib/Settings.svelte';
 import SettingsConnections from './lib/SettingsConnections.svelte';
 import {searchDirectGiphy} from './lib/dictation/giphy';
 import {isBasicDictationReady,isSpeechProviderReady,usesNativeBatch,type PublicSpeechStatus} from './lib/publicSetup';
 import {emptyPublicLocalModels,type PublicLocalModels} from './lib/localModels';
 import {clearScopedLocalStorage,localResetSucceeded} from './lib/appIntegration';
 type Target={canPaste:boolean;appName:string;bundleId:string;appKind:string;appIconDataUrl?:string};
 type Start={sessionId:string;target:Target;needsAccessibility:boolean};
 type Link={name:string;url:string;keywords?:string|string[]};
 type SpeechProvider='local'|'openai'|'gemini'|'backend';
 type SpeechStatus=PublicSpeechStatus;
 type IntegrationStatus={composioKeyPresent:boolean;driveConnected:boolean;driveAccountLabel:string;giphyKeyPresent:boolean};
 const localIdentity='local-user',localBackend='https://local.destroy.invalid';
 let recovery=$state(''),recoveryTimer:ReturnType<typeof setTimeout>|undefined;
 function clearRecovery(){clearTimeout(recoveryTimer);recovery='';}
 function retainRecovery(text:string){clearRecovery();recovery=text;recoveryTimer=setTimeout(clearRecovery,30000);}
 async function copyRecovery(){
  const text=recovery,gen=generation,owner=account();if(!text)return;
  await runOwnedHudAction(()=>invoke('destroy_dictation_copy',{text}),()=>!appDisposed&&gen===generation&&owns(owner)&&recovery===text,()=>{clearRecovery();error='';clipboardCopied=true;hudVisible=true;message='Your text is on the clipboard. Paste it into your chosen app.';},fail);
 }
 let camera=$state({width:210,height:34});
 let user=$state<string|null>(null),url=$state('http://127.0.0.1:8787'),token=$state(''),error=$state(''),message=$state('Choose a speech provider to begin.'),expanded=$state(true),tab=$state<'dictation'|'connection'>('connection');
 let phase=$state<'idle'|'recording'|'processing'|'picker'>('idle'),target=$state<Target|null>(null),needsAccessibility=$state(false),level=$state(0),shortcut=$state('CommandOrControl+Backquote'),mic=$state(localStorage.getItem('destroy.microphone')||''),devices=$state<MediaDeviceInfo[]>([]),permissions=$state({microphone:false,accessibility:false});
 let connectedServer=$state(''),connectionRevision=0,connectionAction=0,appDisposed=false;
 const scope=$derived(user?connectedServer+"|"+user:null);
 const account=()=>({expectedUserId:user,expectedBackendUrl:connectedServer,revision:connectionRevision});
 const owns=(owner:ReturnType<typeof account>)=>!appDisposed&&owner.revision===connectionRevision&&owner.expectedUserId===user&&owner.expectedBackendUrl===connectedServer;
 let capture:NativeCapture|null=null,session:string|null=null,generation=0,startedAt=0;
 let microphoneQuietWarning=$state(''),microphoneLimitWarning=$state('');
 const recordingWarning=$derived(microphoneLimitWarning||microphoneQuietWarning);
 let micTestLevel=$state(0),micTesting=$state(false),micTestStream:MediaStream|null=null,micTestContext:AudioContext|null=null,micTestSource:MediaStreamAudioSourceNode|null=null,micTestTimer:ReturnType<typeof setTimeout>|undefined,micTestGeneration=0;
 let emoji=$state<DictationEmojiChoice[]>([]),media=$state<DictationMediaResult[]>([]),links=$state<Link[]>([]),leading=$state(''),query=$state(''),kind=$state<DictationMediaKind>('gif'),page=$state(0),busy=$state(false),voiceReady=$state(false),mediaOpen=$state(false),driveSearch=$state(false),giphyDirect=$state(false);
 let updatesEnabled=$state(false);
 let favorites=$state<{id:string;title:string}[]>([]),update=$state<Update|null>(null),updateMessage=$state('No update feed configured for this community build.');
 let vars=$state('{}'),linksJson=$state('[]'),deleteConfirm=$state(false),remoteDeleteConfirm=$state(false),removeInstalledModels=$state(false),resetBusy=$state(false),resetAction=0,pendingLocalReset=false,connecting=$state(false);
 let onboarding=$state(true),onboardingStep=$state(0),settingsOpen=$state(false),onboardingForced=false,selectedProvider=$state<SpeechProvider>('local'),speechKey=$state(''),composioKey=$state(''),giphyKey=$state('');
 let speech=$state<SpeechStatus>({provider:null,ready:false,modelReady:false,openaiKeyPresent:false,geminiKeyPresent:false,user_id:null,backend_url:''});
 let dictationPrefs=$state<DictationPreferences>(currentDictationPreferences());
 let localModels=$state<PublicLocalModels>(emptyPublicLocalModels()),localModelBusy=$state(false);
 let integrations=$state<IntegrationStatus>({composioKeyPresent:false,driveConnected:false,driveAccountLabel:'',giphyKeyPresent:false});
 const basicReady=$derived(isBasicDictationReady(speech,permissions.microphone));
 const localCaptureBusy=$derived(phase!=='idle'||Boolean(capture)||Boolean(session)||micTesting);
 const selectedLocalModel=$derived(localModels.models.find(model=>model.id===localModels.selectedModelId));
 const languageNames:Record<string,string>={en:'English',fr:'French',de:'German',es:'Spanish',it:'Italian',pt:'Portuguese',nl:'Dutch',ja:'Japanese',ko:'Korean',zh:'Chinese'};
 function readableLanguage(value:string|undefined){
  const normalized=value?.trim().toLowerCase();
  if(!normalized||normalized==='auto')return normalized==='auto'?'Auto-detect':undefined;
  if(languageNames[normalized])return languageNames[normalized];
  if(normalized==='multilingual')return 'Multilingual';
  return value?.trim()||undefined;
 }
 function modelLanguage(model:typeof selectedLocalModel){
  const declared=readableLanguage(model?.language);
  if(declared)return declared;
  const suffix=model?.name.match(/·\s*(English|multilingual)\s*$/i)?.[1];
  return readableLanguage(suffix);
 }
 function modelBaseName(name:string|undefined){return name?.replace(/\s*·\s*(English|multilingual)\s*$/i,'').trim()||undefined;}
 function speechSummaryLanguage(model:typeof selectedLocalModel,preference:string|undefined){
  const modelLabel=modelLanguage(model),preferenceLabel=readableLanguage(preference);
  if(modelLabel&&preferenceLabel&&modelLabel===preferenceLabel)return modelLabel;
  return [modelLabel,preferenceLabel].filter(Boolean).join(' · ')||undefined;
 }
 const homeSpeech=$derived({...speech,localModelName:speech.provider==='local'?modelBaseName(selectedLocalModel?.name):undefined,modelName:speech.provider==='local'?modelBaseName(selectedLocalModel?.name):undefined,language:speech.provider==='local'?speechSummaryLanguage(selectedLocalModel,dictationPrefs.language):readableLanguage(dictationPrefs.language)});
 let publicStatusLoaded=false,modelPoll:ReturnType<typeof setTimeout>|undefined,localModelPoll:ReturnType<typeof setTimeout>|undefined;
 const api=<T,>(method:string,path:string,body:unknown=null,owner=account())=>{if(!owner.expectedUserId||!owns(owner))return Promise.reject(new Error('Connection changed. Try again.'));return invoke<T>('backend_api',{method,path,body,expectedUserId:owner.expectedUserId,expectedBackendUrl:owner.expectedBackendUrl});};
 function fail(e:unknown){error=e instanceof Error?e.message:String(e);}
 function stripCommandScaffolding(value:string){let text=value.trim().replace(/\s+/gu,' '),previous='';while(text!==previous){previous=text;text=text.replace(/(?:^|[,;:]\s*|\s)(?:(?:can|could|would|will)\s+you(?:\s+please)?|i\s+(?:need|want)\s+(?:you\s+)?to|go\s+ahead\s+and|please)$/iu,'').replace(/[,;:\u2014-]+$/u,'').trim();}return text;}
 let mediaPool:DictationMediaResult[]=[],mediaOffset=0,mediaEnded=false,mediaRequest=0,mediaInitialized=false;
 function resetMedia(){mediaRequest++;mediaPool=[];mediaOffset=0;mediaEnded=false;mediaInitialized=false;page=0;media=[];}
 function clearPicker(){resetMedia();emoji=[];media=[];links=[];leading='';query='';page=0;mediaOpen=false;driveSearch=false;}
 async function panel(open:boolean,focus=true){if(native)await invoke('show_panel',{expanded:open,focus});}
 async function cancel(){hudVisible=false;clipboardCopied=false;error='';message='';microphoneQuietWarning='';microphoneLimitWarning='';needsAccessibility=false;clearRecovery();++generation;capture?.cancel();capture=null;const id=session;session=null;phase='idle';voiceReady=false;busy=false;clearPicker();if(id)await invoke('destroy_dictation_cancel',{sessionId:id}).catch(()=>{});}
 function stopMicTest(){++micTestGeneration;clearTimeout(micTestTimer);micTestTimer=undefined;micTestSource?.disconnect();micTestSource=null;micTestStream?.getTracks().forEach(track=>track.stop());micTestStream=null;void micTestContext?.close().catch(()=>{});micTestContext=null;micTesting=false;micTestLevel=0;}
 async function testMicrophone(){
  if(!native)return nativeOnly();
  if(micTesting){stopMicTest();return;}
  stopMicTest();const testGeneration=micTestGeneration;
  let stream:MediaStream|null=null,context:AudioContext|null=null;
  try{
   stream=await openMicrophoneStream(mic||null);
   if(appDisposed||testGeneration!==micTestGeneration){stream.getTracks().forEach(track=>track.stop());return;}
   context=new AudioContext();const analyser=context.createAnalyser();analyser.fftSize=512;const source=context.createMediaStreamSource(stream);source.connect(analyser);
   micTestStream=stream;micTestContext=context;micTestSource=source;micTesting=true;const samples=new Uint8Array(analyser.fftSize);const deadline=Date.now()+5000;
   const sample=()=>{if(appDisposed||testGeneration!==micTestGeneration||!micTestStream)return;analyser.getByteTimeDomainData(samples);let sum=0;for(const value of samples){const centered=(value-128)/128;sum+=centered*centered;}micTestLevel=Math.sqrt(sum/samples.length);if(Date.now()>=deadline){stopMicTest();return;}micTestTimer=setTimeout(sample,80);};sample();
  }catch(e){stream?.getTracks().forEach(track=>track.stop());void context?.close().catch(()=>{});if(testGeneration===micTestGeneration&&!appDisposed){stopMicTest();fail(e);}}
 }
 let hudVisible=$state(false),clipboardCopied=$state(false),hudFailure=$state(''),hudRevision=Date.now(),currentHud=initialHud();
 $effect(()=>{
  if(!hudVisible||phase!=='idle')return;
  const timer=setTimeout(()=>hudVisible=false,recovery||needsAccessibility||error?30000:clipboardCopied?12000:4500);
  return()=>clearTimeout(timer);
 });
 $effect(()=>{
  currentHud=sanitizeHudSnapshot({revision:++hudRevision,visible:hudVisible,phase,camera,target,message:phase==='recording'&&recordingWarning?recordingWarning:message,error,needsAccessibility,clipboardCopied,recovery:!!recovery,voiceReady,emoji,media,links,leading,query,kind,page,busy,mediaOpen,mediaFavoritesEnabled:!giphyDirect});
  if(native)void invoke('update_hud',{snapshot:currentHud}).catch(e=>{hudFailure=String(e);void panel(true,false);});
 });
 async function hudAction(action:HudAction){
  if(!hudVisible||!currentHudAction(action,currentHud))return;
  const gen=generation,owner=user,server=connectedServer,accountRevision=connectionRevision,id=session;
  const current=()=>!appDisposed&&hudVisible&&gen===generation&&owner===user&&server===connectedServer&&accountRevision===connectionRevision&&id===session;
  try{
   if(action.action==='cancel')await cancel();
   else if(action.action==='open'){settingsOpen=true;onboarding=false;tab='dictation';await panel(true);}
   else if(action.action==='copy'&&recovery)await copyRecovery();
   else if(action.action==='accessibility'){await runOwnedHudAction(()=>invoke<boolean>('request_accessibility_access'),current,allowed=>{permissions.accessibility=allowed;needsAccessibility=!allowed;},fail);}
   else if(action.action==='choose')await choose(action.index!);
   else if(action.action==='favorite')await favorite(action.index!);
   else if(action.action==='rotate')await rotate();
   else if(action.action==='kind'&&action.kind)await changeKind(action.kind);
  }catch(e){if(current())fail(e);}
 }
 async function boundary(){connectionRevision++;setAccount(null);connectedServer='';user=null;target=null;needsAccessibility=false;error='';vars='{}';linksJson='[]';favorites=[];token='';deleteConfirm=false;remoteDeleteConfirm=false;giphyDirect=false;stopMicTest();clearDictationMemory();resetDictationComposerCache();await cancel();}
 function nativeOnly(){message='This is a browser preview. Native setup is available in the macOS app.';}
 function applySpeechStatus(next:SpeechStatus){speech=next;if(next.provider)selectedProvider=next.provider;if(next.user_id&&next.ready){user=next.user_id;connectedServer=next.backend_url||localBackend;setAccount(user,connectedServer);giphyDirect=integrations.giphyKeyPresent;}else if(!next.provider){user=null;connectedServer='';setAccount(null);}if(next.error)error=next.error;}
 async function refreshPublicStatus(){
  if(!native)return;
  try{publicStatusLoaded=true;applySpeechStatus(await invoke<SpeechStatus>('public_setup_status'));await refreshLocalModels();}catch(e){fail(e)}
 }
 function scheduleLocalModelPoll(){clearTimeout(localModelPoll);if(localModels.downloadingModelId)localModelPoll=setTimeout(()=>void refreshLocalModels(),1200);}
 async function refreshLocalModels(){
  if(!native)return;
  try{localModels=await invoke<PublicLocalModels>('public_local_models');scheduleLocalModelPoll();}
  catch(e){localModels={...emptyPublicLocalModels(),error:e instanceof Error?e.message:String(e)};clearTimeout(localModelPoll);}
 }
 async function selectProvider(provider:SpeechProvider){selectedProvider=provider;error='';if(!native||provider==='backend')return;const saved=(provider==='openai'&&speech.openaiKeyPresent)||(provider==='gemini'&&speech.geminiKeyPresent);if(provider==='local'||saved){try{applySpeechStatus(await invoke<SpeechStatus>('public_configure_speech',{provider}));}catch(e){fail(e)}}}
 async function configureSpeech(){
  if(!native)return nativeOnly();
  const provider=selectedProvider,key=speechKey;speechKey='';error='';
  try{const next=await invoke<SpeechStatus>('public_configure_speech',key?{provider,apiKey:key}:{provider});applySpeechStatus(next);message=isSpeechProviderReady(next)?'Speech is ready. Continue with your Mac permissions.':next.error||'Speech setup needs another step.';}catch(e){fail(e)}
 }
 async function downloadModel(modelId?:string){
  if(!native)return nativeOnly();
  if(localCaptureBusy||localModelBusy)return;
  localModelBusy=true;
  error='';
  try{
   const next=modelId?await invoke<SpeechStatus>('public_download_model',{modelId}):await invoke<SpeechStatus>('public_download_model');
   applySpeechStatus(next);await refreshLocalModels();
   const selected=localModels.models.find(model=>model.id===modelId);
   message=localModels.downloadingModelId?`Downloading ${selected?.name||'the local model'}…`:isSpeechProviderReady(next)?'Local speech model is ready.':next.error||'Local speech model is not ready yet.';
   if(next.modelDownloading){clearTimeout(modelPoll);modelPoll=setTimeout(()=>void pollModelStatus(),1500);}
  }catch(e){fail(e)}finally{localModelBusy=false;}
 }
 async function selectLocalModel(modelId:string){
  if(!native)return nativeOnly();
  if(localCaptureBusy||localModelBusy)return;
  localModelBusy=true;error='';
  try{await invoke('public_select_local_model',{modelId});await refreshPublicStatus();const model=localModels.models.find(value=>value.id===modelId);message=`${model?.name||'Local model'} selected.`;}
  catch(e){fail(e)}finally{localModelBusy=false;}
 }
 async function removeLocalModel(modelId:string){
  if(!native)return nativeOnly();
  if(localCaptureBusy||localModelBusy)return;
  const model=localModels.models.find(value=>value.id===modelId);
  localModelBusy=true;error='';
  try{await invoke('public_remove_local_model',{modelId});await refreshPublicStatus();message=`${model?.name||'Local model'} removed.`;}
  catch(e){fail(e)}finally{localModelBusy=false;}
 }
 async function pollModelStatus(){if(!native||speech.provider!=='local'||!speech.modelDownloading)return;try{const next=await invoke<SpeechStatus>('public_setup_status');applySpeechStatus(next);await refreshLocalModels();if(next.modelDownloading)modelPoll=setTimeout(()=>void pollModelStatus(),1500);else if(isSpeechProviderReady(next))message='Local speech model is ready.';}catch(e){fail(e)}}
 async function refreshIntegrations(){
  if(!native)return;
  try{integrations=await invoke<IntegrationStatus>('integration_status');giphyDirect=integrations.giphyKeyPresent;}catch(e){fail(e)}
 }
 async function saveComposio(){
  if(!native)return nativeOnly();
  const key=composioKey;composioKey='';error='';
  try{integrations=await invoke<IntegrationStatus>('save_composio_key',{apiKey:key});message='Composio project key saved in Keychain.';}catch(e){fail(e)}
 }
 async function connectDrive(){
  if(!native)return nativeOnly();
  try{const result=await invoke<{pending:boolean}>('connect_google_drive');message=result.pending?'Finish Google approval in your browser, then return and refresh.':'Google Drive is connected.';}catch(e){fail(e)}
 }
 async function refreshDrive(){if(!native)return nativeOnly();try{integrations={...integrations,...await invoke<IntegrationStatus>('refresh_google_drive')};}catch(e){fail(e)}}
 async function disconnectDrive(){if(!native)return nativeOnly();try{integrations={...integrations,driveConnected:false,driveAccountLabel:''};await invoke('disconnect_google_drive');}catch(e){fail(e)}}
 async function saveGiphy(){
  if(!native)return nativeOnly();
  const key=giphyKey;giphyKey='';error='';
  try{integrations=await invoke<IntegrationStatus>('save_giphy_key',{apiKey:key});giphyDirect=true;message='GIPHY key saved in Keychain.';}catch(e){fail(e)}
 }
 async function disconnectGiphy(){if(!native)return nativeOnly();try{integrations={...integrations,giphyKeyPresent:false};giphyDirect=false;await invoke('disconnect_giphy');}catch(e){fail(e)}}
 async function refreshSetupReadiness(){
  const next=await invoke<SpeechStatus>('public_setup_status');
  const grants=await invoke<{microphone:boolean;accessibility:boolean}>('get_permission_status');
  if(appDisposed)return false;
  applySpeechStatus(next);permissions=grants;return true;
 }
 async function continueOnboarding(){
  if(!native){onboardingStep=Math.min(2,onboardingStep+1);return;}
  try{if(!await refreshSetupReadiness())return;}catch(e){fail(e);return;}
  if(onboardingStep===0&&!isSpeechProviderReady(speech)){message='Choose a provider and finish its setup before continuing.';return;}
  if(onboardingStep===1&&!speech.ready){message='Finish speech setup first.';onboardingStep=0;return;}
  if(onboardingStep===1&&!permissions.microphone){message='Allow the microphone before continuing.';return;}
  onboardingStep=Math.min(2,onboardingStep+1);
 }
 async function finishOnboarding(){if(!native)return nativeOnly();try{if(!await refreshSetupReadiness())return;}catch(e){fail(e);return;}if(!basicReady){message='Speech and microphone setup are required before basic dictation is ready.';return;}onboardingForced=false;onboarding=false;settingsOpen=false;tab='dictation';message='';await loadExtras().catch(fail);}
 async function connect(){
  if(connecting||!native)return;
  connecting=true;error='';const submittedUrl=url,submittedToken=token,attempt=++connectionAction;
  await cancel();
  try{
   const result=await invoke<{user_id:string}>('configure_backend',{backendUrl:submittedUrl,token:submittedToken});
   if(appDisposed||attempt!==connectionAction)return;
   token='';connectionRevision++;connectedServer=new URL(submittedUrl).origin;user=result.user_id;setAccount(user,connectedServer);
   speech={...speech,provider:'backend',ready:true,modelReady:true,user_id:user,backend_url:connectedServer};selectedProvider='backend';onboarding=false;settingsOpen=false;
   message='';tab='dictation';await loadExtras();
  }catch(e){if(!appDisposed&&attempt===connectionAction)fail(e)}finally{if(attempt===connectionAction)connecting=false;}
 }
 async function loadExtras(){
  const owner=account();if(!owner.expectedUserId)return;
  const [v,l,f,p]=await Promise.all([api('GET','/v1/variables',null,owner),api('GET','/v1/links',null,owner),api<{favorites:{id:string;title:string}[]}>('GET','/v1/dictation/media/favorites',null,owner),loadDictationPreferences(scope)]);
  if(!owns(owner))return;
  vars=JSON.stringify(v,null,2);linksJson=JSON.stringify(l,null,2);favorites=f.favorites;dictationPrefs=p;
 }
 async function refreshDevices(){devices=(await navigator.mediaDevices.enumerateDevices()).filter(d=>d.kind==='audioinput');}
 async function allowMic(){if(!native)return nativeOnly();try{await invoke('request_microphone_access');permissions=await invoke('get_permission_status');if(permissions.microphone){const stream=await navigator.mediaDevices.getUserMedia({audio:true});stream.getTracks().forEach(t=>t.stop());await refreshDevices();}}catch(e){fail(e)}}
 async function start(value:Start){
 if(phase==='recording'||phase==='processing')return;
 stopMicTest();
 if(!user){await invoke('destroy_dictation_cancel',{sessionId:value.sessionId});message='Connect your backend in Settings first.';hudVisible=true;return;}
 clearRecovery();clipboardCopied=false;message='';const gen=++generation;session=value.sessionId;target=value.target;needsAccessibility=value.needsAccessibility;error='';phase='recording';startedAt=Date.now();
 hudFailure='';microphoneQuietWarning='';microphoneLimitWarning='';hudVisible=true;const current=new NativeCapture(value.sessionId,v=>{if(gen===generation)level=v;},()=>{void stop()},text=>{if(gen!==generation)return;void invoke('destroy_dictation_cancel',{sessionId:value.sessionId}).catch(()=>{});capture=null;session=null;phase='idle';voiceReady=false;error=text;hudVisible=true;},(text,kind)=>{if(gen!==generation)return;if(kind==='quiet')microphoneQuietWarning=text||'';else if(text)microphoneLimitWarning=text;});capture=current;
 try{await current.start(mic||null);if(gen!==generation)current.cancel();}catch(e){if(gen!==generation)return;current.cancel();const cancelled=cancel();const cancellationGeneration=generation;await cancelled;if(appDisposed||generation!==cancellationGeneration)return;fail(e);hudVisible=true;}
 }
 async function stop(){
 if(phase!=='recording'||!capture)return;const current=capture,gen=generation,id=session;phase='processing';const note=voiceReady;let recognized='';
 try{const result=await current.finish();if(gen!==generation||!id)return;capture=null;const text=result.text.trim();if(isNoSpeechTranscript(text))throw new Error('No speech was recognized. Check the microphone and try again.');recognized=text;
 if(note){voiceReady=false;const audioBase64=bytesBase64(new Uint8Array(await result.blob.arrayBuffer()));if(gen!==generation)return;const delivery=await invoke<any>('destroy_dictation_deliver_voice_note',{sessionId:id,audioBase64,contentType:result.blob.type,transcript:text});if(gen!==generation)return;done(delivery,text);return;}
 if(emoji.length||mediaOpen||links.length){phase='picker';const choice=parseDictationMediaVoiceChoice(text);if(choice?.type==='cancel'){await cancel();return;}if(choice?.type==='kind'&&mediaOpen){await changeKind(choice.kind);return;}if(choice?.type==='rotate'){await rotate();return;}if(choice?.type==='select'){await choose(choice.index);return;}throw new Error('Say one, two, three, more, or cancel.');}
 if(isVoiceNoteCommand(text)){voiceReady=true;phase='picker';message='Voice note ready. Hold again to record the note.';return;}
 if(isUndoDictationCommand(text)){const ok=await invoke<boolean>('destroy_dictation_undo',{sessionId:id});if(gen!==generation)return;message=ok?'':'No recent insertion in this exact field to undo.';await invoke('destroy_dictation_cancel',{sessionId:id});if(gen!==generation)return;session=null;phase='idle';hudVisible=!ok;return;}
 const prefs=speech.provider==='backend'?await loadDictationPreferences(scope):null;if(gen!==generation)return;
 const selection=prefs?.selected_text_editing?await invoke<string|null>('destroy_dictation_selected_text',{sessionId:id}):null;if(gen!==generation)return;
 if(selection){await compose(text,'edit_selected',selection);return;}
 if(isPreviousDictationCorrection(text)){const previous=await invoke<string|null>('destroy_dictation_previous_text',{sessionId:id});if(gen!==generation)return;if(!previous)throw new Error('No recent dictation in this exact field to correct.');await compose(text,'correct_previous',null,previous);return;}
 const ei=extractDictationEmojiIntent(text);if(ei){leading=stripCommandScaffolding(ei.leadingText);query=ei.query;emoji=emojiChoices(ei.query);phase='picker';message='Choose an emoji';return;}
 const mi=extractDictationMediaIntent(text);if(mi){leading=mi.leadingText;query=mi.query;kind=mi.kind;mediaOpen=true;phase='picker';await searchMedia();return;}
 const drive=extractDictationDriveIntent(text);if(drive){if(!native)throw new Error('Drive search is available in the native app only.');leading=drive.messageText;query=drive.query;const result=await searchGoogleDriveLinks(query);if(gen!==generation)return;links=result;driveSearch=true;phase='picker';message='Choose a Drive link. Sharing permissions stay unchanged.';if(!links.length)throw new Error('No Drive file matched that request.');return;}
 const file=extractDictationAttachmentIntent(text);if(file){driveSearch=false;leading=file.messageText;query=file.query;const result=await api<{results:Link[]}>('POST','/v1/links/search',{query});if(gen!==generation)return;links=result.results;phase='picker';message='Choose a saved link. Recipient access is unchanged.';if(!links.length)throw new Error('No saved link matched. Add a named HTTPS link in Connection settings.');return;}
 const rewrite=extractTerminalRewriteCommand(text);await compose(rewrite??text,rewrite!==null?'rewrite':'dictate');
 }catch(e){if(gen===generation){current.cancel();phase=(emoji.length||mediaOpen||links.length)?'picker':'idle';if(phase==='idle'){await invoke('destroy_dictation_cancel',{sessionId:id}).catch(()=>{});if(gen!==generation)return;session=null;voiceReady=false;if(recognized&&!note)retainRecovery(recognized);}fail(e)}}
 }
 async function compose(text:string,operation:'dictate'|'rewrite'|'edit_selected'|'correct_previous',selectedText:string|null=null,previousText:string|null=null){const gen=generation,id=session;if(!id)return;if(speech.provider!=='backend'&&operation!=='dictate')throw new Error('Local speech currently supports direct dictation only; rewriting selected text is unavailable.');const result=await transformDictation({text,appKind:target?.appKind||'generic',operation,selectedText,previousText});if(gen!==generation)return;const command=operation==='correct_previous'?'destroy_dictation_replace_last':'destroy_dictation_deliver';const delivery=await invoke<any>(command,{sessionId:id,text:result.text});if(gen!==generation)return;if(delivery===false)throw new Error('The recent field changed; correction was not inserted.');done(typeof delivery==='boolean'?{delivery:'inline'}:delivery,result.text);}
 function done(delivery:any,text:string){const feedback=deliveryHudFeedback(delivery,target?.appName);message=feedback.message;needsAccessibility=feedback.needsAccessibility;clipboardCopied=feedback.clipboardCopied;error='';clearRecovery();phase='idle';session=null;voiceReady=false;busy=false;clearPicker();hudVisible=feedback.visible;recordDictation({teamId:scope,text,elapsedMs:Date.now()-startedAt});}
 async function searchMedia(){
 const gen=generation,request=++mediaRequest,wanted=(page+1)*3;busy=true;error='';
 const current=()=>gen===generation&&request===mediaRequest;
 try{
  if(giphyDirect){
   const result=await searchDirectGiphy(query,kind,page*3);if(!current())return;media=result.results.slice(0,3);mediaPool=[];mediaInitialized=true;mediaEnded=result.results.length<3;return;
  }
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
 async function choose(index:number){if(busy||!session)return;const gen=generation,id=session;busy=true;error='';try{let delivery:any,text='';if(emoji[index]){text=[leading,emoji[index].emoji].filter(Boolean).join(' ');delivery=await invoke('destroy_dictation_deliver_picker_text',{sessionId:id,text});}else if(links[index]){text=[leading,links[index].url].filter(Boolean).join('\n');delivery=await invoke('destroy_dictation_deliver_picker_text',{sessionId:id,text});}else if(media[index]){const r=media[index];text=leading;delivery=await invoke('destroy_dictation_deliver_media',{sessionId:id,...mediaDeliveryRequest(r,leading)});}else{return;}if(gen===generation)done(delivery,text);}catch(e){if(gen===generation)fail(e)}finally{if(gen===generation)busy=false;}}
 async function favorite(index:number){if(giphyDirect)return;const r=media[index];if(!r)return;const gen=generation;try{const saved=await saveDictationMediaFavorite(query,r);if(gen!==generation)return;media=media.map(v=>v.id===r.id?{...v,favorite_id:saved.id}:v);mediaPool=mediaPool.map(v=>v.id===r.id?{...v,favorite_id:saved.id}:v);message='Favorite saved.';}catch(e){if(gen===generation&&!appDisposed)fail(e)}}
 function keys(e:KeyboardEvent){if(e.key==='Escape'){if(deleteConfirm){if(!resetBusy)deleteConfirm=false;return;}if(remoteDeleteConfirm){remoteDeleteConfirm=false;return;}if(settingsOpen){backToHome();return;}stopMicTest();void cancel();return;}if((e.target as HTMLElement)?.matches('input,textarea,select'))return;if(phase==='picker'){if(['1','2','3'].includes(e.key))void choose(Number(e.key)-1);if(e.key==='r'||e.key==='ArrowDown')void rotate();}}
 async function saveExtras(){const owner=account();try{const variables=JSON.parse(vars),savedLinks=JSON.parse(linksJson);await api('PUT','/v1/variables',variables,owner);if(!owns(owner))return;await api('PUT','/v1/links',savedLinks,owner);if(owns(owner))message='Variables and links saved.';}catch(e){if(owns(owner))fail(e)}}
 async function exportData(){const owner=account();try{const path=await invoke<string>('export_personal_data');if(owns(owner))message='Personal data saved to '+path;}catch(e){if(owns(owner))fail(e)}}
 async function disconnect(){const attempt=++connectionAction;await boundary();try{await invoke('disconnect_backend');if(!appDisposed&&attempt===connectionAction){tab='connection';message='Disconnected.';return true;}}catch(e){if(!appDisposed&&attempt===connectionAction)fail(e)}return false;}
 async function deleteData(){const owner=account();try{await cancel();if(!owns(owner))return;await api('DELETE','/v1/account/data',null,owner);if(!owns(owner))return;if(!await disconnect())return;clearScopedLocalStorage(localStorage);message='Remote personal data deleted. Backups follow your server retention policy.';}catch(e){if(owns(owner))fail(e)}}
 function promptLocalReset(){deleteConfirm=true;removeInstalledModels=false;error='';}
 async function restartOnboarding(){
  if(resetBusy)return;
  const operation=++resetAction;stopMicTest();await cancel();
  if(!native){onboardingForced=true;onboarding=true;settingsOpen=false;onboardingStep=0;message='Onboarding restarted.';return;}
  onboardingForced=true;
  connectionRevision++;setAccount(null);
  try{
   await invoke('public_restart_onboarding');
   if(appDisposed||operation!==resetAction)return;
   onboarding=true;settingsOpen=false;onboardingStep=0;
   await refreshSetupReadiness();
   if(appDisposed||operation!==resetAction)return;
   onboarding=true;settingsOpen=false;onboardingStep=0;message='Onboarding restarted. Your local settings are still saved.';
  }catch(e){if(!appDisposed&&operation===resetAction)fail(e);}
 }
 function clearLocalResetState(){
  setAccount(null);user=null;connectedServer='';target=null;needsAccessibility=false;token='';url='http://127.0.0.1:8787';vars='{}';linksJson='[]';favorites=[];giphyDirect=false;
  clearDictationMemory();resetDictationComposerCache();speech={provider:null,ready:false,modelReady:false,openaiKeyPresent:false,geminiKeyPresent:false,user_id:null,backend_url:''};dictationPrefs=currentDictationPreferences();localModels=emptyPublicLocalModels();integrations={composioKeyPresent:false,driveConnected:false,driveAccountLabel:'',giphyKeyPresent:false};
  mic='';shortcut='CommandOrControl+Backquote';selectedProvider='local';speechKey='';composioKey='';giphyKey='';clearTimeout(modelPoll);clearTimeout(localModelPoll);
 }
 async function resetLocalData(){
  if(!native||resetBusy)return;
  const operation=++resetAction;resetBusy=true;pendingLocalReset=true;error='';stopMicTest();await cancel();connectionRevision++;setAccount(null);
  try{
   const result=await invoke<{reset:boolean;modelsRemoved:boolean}>('public_reset_local_data',{confirmation:true,removeInstalledModels});
   if(appDisposed||operation!==resetAction)return;
   if(!localResetSucceeded(result))throw new Error('Local reset did not complete.');
   clearScopedLocalStorage(localStorage);clearLocalResetState();onboardingForced=true;onboarding=true;settingsOpen=false;onboardingStep=0;deleteConfirm=false;removeInstalledModels=false;
   await refreshSetupReadiness();await refreshLocalModels();
   if(appDisposed||operation!==resetAction)return;
   onboarding=true;settingsOpen=false;onboardingStep=0;message=result.modelsRemoved?'Local data reset, including installed models. Choose a speech provider to begin.':'Local data reset. Choose a speech provider to begin.';
  }catch(e){if(!appDisposed&&operation===resetAction){const resetError=e instanceof Error?e.message:String(e);fail(e);await refreshSetupReadiness().catch(()=>{});await refreshLocalModels().catch(()=>{});if(!appDisposed&&operation===resetAction)error=resetError;}}
  finally{if(operation===resetAction){pendingLocalReset=false;resetBusy=false;}}
 }
 async function removeFavorite(id:string){const owner=account();try{await api('DELETE','/v1/dictation/media/favorites/'+encodeURIComponent(id),null,owner);if(owns(owner))favorites=favorites.filter(f=>f.id!==id);}catch(e){if(owns(owner))fail(e)}}
 function drawerFocus(node:HTMLElement){
  const previous=document.activeElement as HTMLElement|null;
  const focusable=()=>Array.from(node.querySelectorAll<HTMLElement>('button:not(:disabled),input:not(:disabled),select:not(:disabled),textarea:not(:disabled),a[href],[tabindex="0"]')).filter(el=>el.getClientRects().length>0);
  (focusable()[0]??node).focus();
  const trap=(event:KeyboardEvent)=>{if(event.key!=='Tab'||deleteConfirm)return;const items=focusable();const first=items[0],last=items[items.length-1];if(!first){event.preventDefault();node.focus();return;}if(event.shiftKey&&(document.activeElement===first||document.activeElement===node)){event.preventDefault();last.focus();}else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first.focus();}};
  node.addEventListener('keydown',trap);
  return {destroy(){node.removeEventListener('keydown',trap);if(previous?.isConnected)previous.focus();}};
 }
 let settingsTab=$state<'speech'|'writing'|'connections'|'data'>('speech');
 function openSettings(tab:'speech'|'writing'|'connections'|'data'='speech'){stopMicTest();settingsTab=tab;settingsOpen=true;onboarding=false;}
 function backToHome(){stopMicTest();settingsOpen=false;}
 function openOnboarding(){stopMicTest();onboarding=true;settingsOpen=false;onboardingStep=0;}
 async function restoreConnection(){
  const revision=++connectionRevision;
  if(publicStatusLoaded&&speech.provider&&speech.provider!=='backend')return;
  try{const c=await invoke<{user_id:string;backend_url:string;provider?:SpeechProvider;ready?:boolean;modelReady?:boolean;modelDownloading?:boolean;openaiKeyPresent?:boolean;geminiKeyPresent?:boolean}>('restore_connection');if(appDisposed||revision!==connectionRevision)return;user=c.user_id;connectedServer=c.backend_url;setAccount(user,connectedServer);url=c.backend_url;if(c.provider){applySpeechStatus({...speech,...c,user_id:c.user_id,backend_url:c.backend_url} as SpeechStatus);}else if(speech.provider){speech={...speech,user_id:c.user_id,backend_url:c.backend_url};}await refreshPublicStatus();if(isSpeechProviderReady(speech)&&!onboardingForced){onboarding=!permissions.microphone;onboardingStep=permissions.microphone?2:1;settingsOpen=false;tab='dictation';message='';await loadExtras();}}
  catch(e){if(!appDisposed&&revision===connectionRevision&&(!speech.provider||speech.provider==='backend'))message='Connect your backend to begin.';}
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
    ['destroy://account-changed',()=>{if(pendingLocalReset||resetBusy){connectionRevision++;setAccount(null);void cancel();}else void boundary();}]
   ] as const){const fn=await listen(name,handler);if(appDisposed){fn();return;}off.push(fn);}
   await refreshPublicStatus();
   permissions=await invoke('get_permission_status');
   await Promise.all([
    invoke<typeof camera>('notch_geometry').then(value=>camera=value).catch(fail),
    invoke<string>('get_shortcut').then(value=>shortcut=value).catch(fail),
    invoke('shortcut_status').catch(fail),
    invoke<boolean>('updater_available').then(value=>updatesEnabled=value).catch(()=>updatesEnabled=false),
    refreshIntegrations()
   ]);
   if(appDisposed)return;
   await restoreConnection();await refreshDevices();if(user&&isSpeechProviderReady(speech)&&!onboardingForced){onboardingStep=permissions.microphone?2:1;if(permissions.microphone)onboarding=false;}
  }catch(e){if(!appDisposed)fail(e)}})();
  return()=>{appDisposed=true;connectionRevision++;connectionAction++;clearTimeout(modelPoll);clearTimeout(localModelPoll);stopMicTest();off.forEach(fn=>fn());void cancel();};
 });
</script>
<svelte:window onkeydown={keys}/>
<main class:home-active={!onboarding} style={`--camera-width:${camera.width}px;--camera-height:${camera.height}px`}>
 <div class="topline" data-tauri-drag-region></div>
 {#if phase!=='idle'||voiceReady||error||hudFailure||recovery||needsAccessibility}<div class="status" role="status"><p>{phase==='recording'?(recordingWarning||'Listening — release to finish'):phase==='processing'?'Transcribing…':message}</p>{#if phase!=='idle'||voiceReady}<button onclick={()=>void cancel()}>Cancel <kbd>Esc</kbd></button>{/if}</div>{/if}
 {#if hudFailure}<p class="error" role="alert">{hudFailure}</p>{/if}
 {#if error}<p class="error" role="alert">{error}</p>{/if}
 {#if recovery}<button onclick={()=>void copyRecovery()}>Copy recognized text <small>· clears after 30 seconds</small></button>{/if}
 {#if needsAccessibility}<button class="permission" onclick={async()=>{permissions.accessibility=await invoke('request_accessibility_access');needsAccessibility=!permissions.accessibility;}}>Allow Accessibility for direct insertion</button>{/if}
 {#if emoji.length}<EmojiPicker choices={emoji} selectedIndex={0} {query} leadingText={leading} onSelect={i=>void choose(i)} onCancel={()=>void cancel()}/>{/if}
 {#if mediaOpen}<MediaPicker {kind} {query} leadingText={leading} {page} results={media} selectedIndex={0} loading={busy} delivering={false} {error} allowFavorites={!giphyDirect} attribution="Powered by GIPHY" onKindChange={v=>void changeKind(v)} onSelect={i=>void choose(i)} onFavorite={i=>void favorite(i)} onRotate={()=>void rotate()} onCancel={()=>void cancel()}/>{/if}
 {#if links.length}<div class="link-picker"><p>{leading}</p>{#each links as link,i}<button disabled={busy} onclick={()=>void choose(i)}><kbd>{i+1}</kbd><strong>{link.name}</strong><small>{link.url}</small></button>{/each}<p>{driveSearch?'Inserts the existing Drive link. Sharing permissions stay unchanged.':'Inserts a named HTTPS link. Check access before sending.'}</p></div>{/if}
 {#if onboarding}
  <Onboarding {native} step={onboardingStep} speech={homeSpeech} {selectedProvider} {speechKey} {permissions} {shortcut} {integrations} {composioKey} {giphyKey} {localModels} modelBusy={localCaptureBusy||localModelBusy} backendUrl={url} backendToken={token} {connecting} onProvider={value=>void selectProvider(value)} onSpeechKey={value=>speechKey=value} onConfigureSpeech={()=>void configureSpeech()} onDownloadModel={modelId=>void downloadModel(modelId)} onSelectModel={modelId=>void selectLocalModel(modelId)} onRemoveModel={modelId=>void removeLocalModel(modelId)} onMic={()=>void allowMic()} onAccessibility={async()=>{if(!native)return nativeOnly();try{permissions.accessibility=await invoke('request_accessibility_access');}catch(e){fail(e)}}} onShortcutChange={value=>shortcut=value} onShortcutSave={async()=>{if(!native)return nativeOnly();try{await invoke('set_shortcut',{value:shortcut});message='Shortcut saved.';}catch(e){fail(e)}}} onIntegration={()=>{}} onComposioKey={value=>composioKey=value} onSaveComposio={()=>void saveComposio()} onConnectDrive={()=>void connectDrive()} onRefreshDrive={()=>void refreshDrive()} onDisconnectDrive={()=>void disconnectDrive()} onGiphyKey={value=>giphyKey=value} onSaveGiphy={()=>void saveGiphy()} onDisconnectGiphy={()=>void disconnectGiphy()} onBackendUrl={value=>url=value} onBackendToken={value=>token=value} onConnectBackend={()=>void connect()} onContinue={continueOnboarding} onBack={()=>onboardingStep=Math.max(0,onboardingStep-1)} onFinish={finishOnboarding} onNativeOnly={nativeOnly} />
 {:else}
  <section class="workspace home-view" inert={settingsOpen}><HomeScreen {shortcut} {scope} speech={homeSpeech} {phase} microphoneLevel={phase==='recording'?level:0} recording={phase==='recording'} onSettings={()=>openSettings()}>
   {#snippet connections()}<SettingsConnections embedded {integrations} {native} {connecting} {composioKey} {giphyKey} backendUrl={url} backendToken={token} onComposioKey={(value:string)=>composioKey=value} onGiphyKey={(value:string)=>giphyKey=value} onSaveComposio={()=>void saveComposio()} onConnectDrive={()=>void connectDrive()} onRefreshDrive={()=>void refreshDrive()} onDisconnectDrive={()=>void disconnectDrive()} onSaveGiphy={()=>void saveGiphy()} onDisconnectGiphy={()=>void disconnectGiphy()} onBackendUrl={(value:string)=>url=value} onBackendToken={(value:string)=>token=value} onConnectBackend={()=>void connect()}/>{/snippet}
  </HomeScreen></section>
 {/if}
 {#if settingsOpen&&!onboarding}
  <button class="settings-scrim" aria-label="Close Settings" tabindex="-1" onclick={backToHome} transition:fade={{duration:window.matchMedia('(prefers-reduced-motion: reduce)').matches?0:180}}></button>
  <div class="settings-drawer" role="dialog" aria-modal="true" aria-label="Settings" tabindex="-1" use:drawerFocus transition:panelMotion>
  <Settings initialTab={settingsTab} speech={homeSpeech} {permissions} {shortcut} {devices} {mic} micTestLevel={micTestLevel} micTesting={micTesting} {localModels} {integrations} {native} localBusy={localCaptureBusy||localModelBusy||resetBusy} {user} scope={scope} {vars} {linksJson} deleteConfirm={remoteDeleteConfirm} backendUrl={url} backendToken={token} {connecting} onBack={backToHome} onDownload={(modelId:string)=>void downloadModel(modelId)} onSelectModel={(modelId:string)=>void selectLocalModel(modelId)} onRemoveModel={(modelId:string)=>void removeLocalModel(modelId)} onMic={()=>void testMicrophone()} onShortcutSave={async()=>{try{await invoke('set_shortcut',{value:shortcut});message='Shortcut saved.';}catch(e){fail(e)}}} onShortcutChange={(value:string)=>shortcut=value} onMicChange={(event:Event)=>{mic=(event.currentTarget as HTMLSelectElement).value;localStorage.setItem('destroy.microphone',mic)}} onSaveExtras={()=>void saveExtras()} onVarsChange={(value:string)=>vars=value} onLinksChange={(value:string)=>linksJson=value} onExport={()=>void exportData()} onDeletePrompt={()=>remoteDeleteConfirm=true} onDelete={()=>void deleteData()} onKeepData={()=>remoteDeleteConfirm=false} onResetPrompt={promptLocalReset} onRestartOnboarding={()=>void restartOnboarding()} onModelChange={()=>{if(speech.provider!=='local')openOnboarding();}} onBackendUrl={(value:string)=>url=value} onBackendToken={(value:string)=>token=value} onConnectBackend={()=>void connect()} onConnectDrive={()=>void connectDrive()} onRefreshDrive={()=>void refreshDrive()} onDisconnectDrive={()=>void disconnectDrive()} onSaveComposio={()=>void saveComposio()} onSaveGiphy={()=>void saveGiphy()} onDisconnectGiphy={()=>void disconnectGiphy()} {composioKey} {giphyKey} onComposioKey={(value:string)=>composioKey=value} onGiphyKey={(value:string)=>giphyKey=value} onOpenOnboarding={openOnboarding} />
  </div>
 {/if}
 {#if deleteConfirm}
  <div role="dialog" aria-modal="true" aria-labelledby="local-reset-title" style="position:fixed;inset:0;z-index:20;display:grid;place-items:center;padding:24px;background:#090a09cc;backdrop-filter:blur(6px)">
   <section style="width:min(420px,100%);padding:22px;border:1px solid #ffffff35;border-radius:10px;background:#1a1c1bdc;box-shadow:0 18px 60px #000b">
    <h2 id="local-reset-title" style="margin:0 0 8px;color:#f4f0e7">Delete local data &amp; reset?</h2>
    <p style="margin:0 0 16px;color:#c9c6bf;font-size:12px;line-height:1.6">This removes this installation’s saved setup, snippets, favorites, links, speech credentials, and model selection. It does not delete remote backend data.</p>
    {#if error}<p role="alert" style="margin:0 0 14px;border-left:2px solid #d83a30;background:#d83a3010;color:#ffd3d3;padding:8px 12px;font-size:12px;line-height:1.5">{error}</p>{/if}
    <label style="display:flex;flex-direction:row;align-items:center;gap:8px;margin:0 0 18px;color:#d8d5cf;font-size:12px"><input type="checkbox" bind:checked={removeInstalledModels} style="width:auto"/> Also remove installed speech models</label>
    <div style="display:flex;justify-content:flex-end;gap:8px"><button disabled={resetBusy} onclick={()=>deleteConfirm=false}>Cancel</button><button class="danger" disabled={resetBusy||!native} onclick={()=>void resetLocalData()}>{resetBusy?'Resetting…':'Delete local data & reset'}</button></div>
   </section>
  </div>
 {/if}
</main>

<style>
 .settings-scrim{position:fixed;inset:0;z-index:9;padding:0;border:0;border-radius:0;background:#0007;cursor:default}
 .settings-scrim:hover{background:#0007}
 .settings-drawer{position:fixed;inset:0 0 0 auto;z-index:10;width:min(730px,100%);height:100%;display:flex;flex-direction:column;padding:24px 38px 14px;background:#000;border:0;border-left:1px solid #ffffff15;box-shadow:-16px 0 60px #0005;overflow:hidden}
 @media(max-width:620px){.settings-drawer{padding:24px 20px 12px}}
 @media(prefers-reduced-motion:reduce){.settings-drawer{animation:none}}
</style>

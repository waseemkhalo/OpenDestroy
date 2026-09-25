import { invoke } from '@tauri-apps/api/core';
import {captureAccount, invokeAccount} from './account';
import { openMicrophoneStream } from './audioDevices';
import { keepAudioContextRunning, recoverDictationTranscription, transcribeDictationRecording } from './dictation/dictationCapture';
export function captureUsesLiveStream(provider: string | null): boolean { return provider === 'backend'; }
export function isNoSpeechTranscript(text: string): boolean {
 return !text.trim() || /^(?:(?:\[BLANK_AUDIO\]|\[NO_SPEECH\]|<\|nospeech\|>)\s*)+$/iu.test(text.trim());
}
export function bytesBase64(bytes: Uint8Array): string {
 let text='';for(let i=0;i<bytes.length;i+=8192)text+=String.fromCharCode(...bytes.subarray(i,i+8192));return btoa(text);
}

/** Global dictation must not wait for a background WebView to grant capture.
 * Native audio is owned by the same key-down session as the eventual paste. */
export class NativeCapture {
 private owner=captureAccount();
 private cancelled=false;
 private startPromise:Promise<void>|null=null;
 private levelTimer:ReturnType<typeof setInterval>|undefined;
 private healthTimer:ReturnType<typeof setInterval>|undefined;
 private warningTimer:ReturnType<typeof setTimeout>|undefined;
 private limitTimer:ReturnType<typeof setTimeout>|undefined;
 private polling=false;
 private healthPolling=false;
 private active=true;
 private quietSince=Date.now();
 private quietWarned=false;
 constructor(private sessionId:string,private meter:(v:number)=>void,private timeout:()=>void,private onError:(message:string)=>void=()=>{},private onWarning:(message:string|null,kind:'quiet'|'limit')=>void=()=>{}){}
 start(mic:string|null){
  this.startPromise=this.startInner(mic);
  return this.startPromise;
 }
 private async startInner(mic:string|null){
  const pending=invokeAccount('native_audio_start',{sessionId:this.sessionId,deviceId:mic},this.owner);
  // If startup resolves after cancellation or a timeout, dispose its late native queue.
  void pending.then(()=>{if(this.cancelled)void invoke('native_audio_cancel',{sessionId:this.sessionId}).catch(()=>{});}).catch(()=>{});
  let startupTimer:ReturnType<typeof setTimeout>|undefined;
  try{
   await Promise.race([pending,new Promise<never>((_,reject)=>{
    startupTimer=setTimeout(()=>reject(new Error('The microphone did not start within 5 seconds. Check macOS Microphone permission and your input device, then try again.')),5000);
   })]);
  }catch(error){this.cancel();throw error;}finally{clearTimeout(startupTimer);}
  if(this.cancelled){await invoke('native_audio_cancel',{sessionId:this.sessionId});return;}
  this.levelTimer=setInterval(()=>{
   if(this.polling||this.cancelled)return;
   this.polling=true;
   void invoke<number>('native_audio_level',{sessionId:this.sessionId})
    .then(level=>{if(!this.cancelled&&this.levelTimer){this.meter(level);if(level>=0.002){this.quietSince=Date.now();if(this.quietWarned){this.quietWarned=false;this.onWarning(null,'quiet');}}else if(!this.quietWarned&&Date.now()-this.quietSince>=8000){this.quietWarned=true;this.onWarning('No sound detected. Check your microphone.','quiet');}}})
    .catch(()=>{}).finally(()=>{this.polling=false;});
  },100);
  this.healthTimer=setInterval(()=>{
   if(!this.active||this.healthPolling)return;
   this.healthPolling=true;
   void invoke<string|null>('native_audio_health',{sessionId:this.sessionId}).then(message=>{
    if(message&&this.active){this.cancel();this.onError(message);}
   }).catch(()=>{}).finally(()=>{this.healthPolling=false;});
  },250);
  this.warningTimer=setTimeout(()=>this.onWarning('About 15 seconds left. Release your shortcut to finish dictation.','limit'),105000);
  // Finish before the native hard deadline, which is independent of WebView timers.
  this.limitTimer=setTimeout(this.timeout,119000);
 }
 private release(){clearInterval(this.levelTimer);this.levelTimer=undefined;clearInterval(this.healthTimer);this.healthTimer=undefined;clearTimeout(this.warningTimer);clearTimeout(this.limitTimer);this.meter(0);}
 async finish(){
  this.active=false;
  await this.startPromise;
  this.release();
  if(this.cancelled)throw new Error('Dictation was cancelled.');
  const audio=await invokeAccount<{audioBase64:string;mimeType:string}>('native_audio_finish',{sessionId:this.sessionId},this.owner);
  if(!audio.audioBase64)throw new Error('No microphone audio was captured. Check macOS Microphone permission and your input device, then try again.');
  const result=await transcribeDictationRecording({
   encodeAudio:async()=>audio.audioBase64,isCancelled:()=>this.cancelled,
   transcribe:audioBase64=>invokeAccount< {text:string;path:'local'|'cloud'} >('destroy_transcribe_dictation',{audioBase64,mimeType:audio.mimeType},this.owner),
  });
  if(this.cancelled)throw new Error('Dictation was cancelled.');
  const bytes=Uint8Array.from(atob(audio.audioBase64),character=>character.charCodeAt(0));
  return {text:result.text,blob:new Blob([bytes],{type:audio.mimeType})};
 }
 cancel(){this.active=false;this.cancelled=true;this.release();void invoke('native_audio_cancel',{sessionId:this.sessionId}).catch(()=>{});}
}
export class Capture {
 private owner=captureAccount();
 private stream:MediaStream|null=null;private context:AudioContext|null=null;private node:ScriptProcessorNode|null=null;private recorder:MediaRecorder|null=null;
 private parts:Blob[]=[];private closeResume=()=>{};private cancelled=false;private stopRequested=false;private startPromise:Promise<void>|null=null;private useLiveStream=true;
 private live:Promise<string|null>|null=null;private chain:Promise<void>=Promise.resolve();private liveError=false;private timer:ReturnType<typeof setTimeout>|undefined;
 constructor(private meter:(v:number)=>void,private timeout:()=>void){}
 start(mic:string|null,useLiveStream=true){this.useLiveStream=useLiveStream;this.startPromise=this.startInner(mic,useLiveStream);return this.startPromise;}
 private async startInner(mic:string|null,useLiveStream:boolean){
 const stream=await openMicrophoneStream(mic);if(this.cancelled||this.stopRequested){stream.getTracks().forEach(t=>t.stop());return;}
 this.stream=stream;const mime=['audio/mp4','audio/webm'].find(m=>MediaRecorder.isTypeSupported(m));if(!mime)throw new Error('This WebKit version cannot encode dictation audio.');
 this.recorder=new MediaRecorder(stream,{mimeType:mime,audioBitsPerSecond:64000});this.recorder.ondataavailable=e=>{if(!this.cancelled&&e.data.size)this.parts.push(e.data);};this.recorder.start(200);
 const ctx=new AudioContext({sampleRate:16000});this.context=ctx;this.closeResume=keepAudioContextRunning(ctx);
 const source=ctx.createMediaStreamSource(stream);const node=ctx.createScriptProcessor(2048,1,1);this.node=node;const mute=ctx.createGain();mute.gain.value=0;source.connect(node);node.connect(mute);mute.connect(ctx.destination);
 this.live=useLiveStream
  ? invokeAccount<{sessionId:string}>('destroy_dictation_stream_start',{sampleRateHz:ctx.sampleRate},this.owner).then(async r=>{if(this.cancelled){await invoke('destroy_dictation_stream_cancel',{sessionId:r.sessionId});return null;}return r.sessionId;}).catch(()=>null)
  : Promise.resolve(null);
 let queued=0;
 node.onaudioprocess=e=>{if(this.cancelled||this.stopRequested)return;const input=e.inputBuffer.getChannelData(0);const bytes=new Uint8Array(input.length*2);const view=new DataView(bytes.buffer);let sum=0;for(let i=0;i<input.length;i++){sum+=input[i]*input[i];view.setInt16(i*2,Math.max(-1,Math.min(1,input[i]))*32767,true);}this.meter(Math.sqrt(sum/input.length));
 if(queued>=64){this.liveError=true;return;}queued++;this.chain=this.chain.then(async()=>{const id=await this.live;if(useLiveStream&&id&&!this.cancelled&&!this.liveError)await invoke('destroy_dictation_stream_send_audio',{sessionId:id,audioBase64:bytesBase64(bytes)});}).catch(()=>{this.liveError=true;}).finally(()=>queued--);};
 this.timer=setTimeout(this.timeout,120000);
 }
 private release(){clearTimeout(this.timer);this.closeResume();if(this.node)this.node.onaudioprocess=null;this.node?.disconnect();this.stream?.getTracks().forEach(t=>t.stop());this.stream=null;void this.context?.close().catch(()=>{});this.context=null;this.meter(0);}
 async finish(){this.stopRequested=true;await this.startPromise;if(this.cancelled)throw new Error('Cancelled');
 const recorder=this.recorder;if(!recorder){this.release();throw new Error('Hold the shortcut until the microphone is ready, then speak.');}
 await new Promise<void>((resolve,reject)=>{const timer=setTimeout(()=>reject(new Error('Recording could not finish. Please try again.')),5000);recorder.onerror=()=>{clearTimeout(timer);reject(new Error('Recording failed.'));};recorder.onstop=()=>{clearTimeout(timer);resolve();};if(recorder.state==='inactive'){clearTimeout(timer);resolve();}else recorder.stop();});this.release();const blob=new Blob(this.parts,{type:recorder.mimeType});this.parts=[];this.recorder=null;
 await this.chain;const id=await this.live;let live:null|{text:string;path:'cloud'}=null;
 if(this.useLiveStream&&id&&!this.cancelled){if(this.liveError){await invoke('destroy_dictation_stream_cancel',{sessionId:id}).catch(()=>{});}else{live=await invoke<{text:string;path:'cloud'}>('destroy_dictation_stream_finish',{sessionId:id}).catch(()=>null);}}
 const result=await recoverDictationTranscription(live,()=>transcribeDictationRecording({encodeAudio:async()=>bytesBase64(new Uint8Array(await blob.arrayBuffer())),isCancelled:()=>this.cancelled,transcribe:audioBase64=>invokeAccount('destroy_transcribe_dictation',{audioBase64,mimeType:blob.type},this.owner)}));
 if(this.cancelled)throw new Error('Cancelled');return {text:result.text,blob};
 }
 cancel(){this.cancelled=true;this.stopRequested=true;if(this.recorder&&this.recorder.state!=='inactive')this.recorder.stop();this.parts=[];this.release();void this.live?.then(id=>id&&invoke('destroy_dictation_stream_cancel',{sessionId:id})).catch(()=>{});}
}

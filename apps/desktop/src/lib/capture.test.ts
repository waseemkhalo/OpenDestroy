import {describe,it,expect,vi,beforeEach} from 'vitest';
import {Capture,NativeCapture,bytesBase64,captureUsesLiveStream,isNoSpeechTranscript} from './capture';
import {invoke} from '@tauri-apps/api/core';
import {setAccount} from './account';
import {openMicrophoneStream} from './audioDevices';
vi.mock('./audioDevices',()=>({openMicrophoneStream:vi.fn()}));
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}));
describe('capture ownership',()=>{
 it('recognizes empty speech markers without dropping actual words',()=>{
  for(const text of ['', '  ', '[BLANK_AUDIO]', '[NO_SPEECH]', '<|nospeech|>', '[BLANK_AUDIO] [BLANK_AUDIO]'])expect(isNoSpeechTranscript(text)).toBe(true);
  for(const text of ['Hello world', 'Explain [BLANK_AUDIO] to me', 'blank audio'])expect(isNoSpeechTranscript(text)).toBe(false);
 });
 beforeEach(()=>vi.clearAllMocks());
 it('uses the live stream only for the backend provider',()=>{
  expect(captureUsesLiveStream('backend')).toBe(true);
  expect(captureUsesLiveStream('local')).toBe(false);
  expect(captureUsesLiveStream('openai')).toBe(false);
  expect(captureUsesLiveStream(null)).toBe(false);
 });
 it('stops a late microphone grant after cancellation, without starting an encoder',async()=>{
  let release!:(s:MediaStream)=>void;
  vi.mocked(openMicrophoneStream).mockReturnValue(new Promise(resolve=>{release=resolve}));
  const stop=vi.fn();const capture=new Capture(()=>{},()=>{});const start=capture.start(null);capture.cancel();release({getTracks:()=>[{stop}]} as unknown as MediaStream);await start;expect(stop).toHaveBeenCalledTimes(1);
 });
 it('does not record if key-up wins the microphone permission race',async()=>{
  let release!:(s:MediaStream)=>void;
  vi.mocked(openMicrophoneStream).mockReturnValue(new Promise(resolve=>{release=resolve}));
  const stop=vi.fn();const capture=new Capture(()=>{},()=>{});const start=capture.start('pinned');const finished=capture.finish();release({getTracks:()=>[{stop}]} as unknown as MediaStream);await start;await expect(finished).rejects.toThrow('Hold the shortcut');expect(stop).toHaveBeenCalledTimes(1);expect(openMicrophoneStream).toHaveBeenCalledWith('pinned');
 });
 it('encodes arbitrary PCM bytes without sign or chunk-boundary loss',()=>{const input=Uint8Array.from({length:70000},(_,i)=>i%256);const encoded=bytesBase64(input);expect(Uint8Array.from(atob(encoded),c=>c.charCodeAt(0))).toEqual(input);});
});

describe('native global capture',()=>{
 beforeEach(()=>{vi.clearAllMocks();setAccount('local-user','https://local.destroy.invalid');});
 it('records and transcribes without asking a background WebView for its microphone',async()=>{
  vi.mocked(invoke).mockImplementation(async(command)=>{
   if(command==='native_audio_finish')return {audioBase64:btoa('audio'),mimeType:'audio/wav'};
   if(command==='destroy_transcribe_dictation')return {text:'Native dictation',path:'local'};
   return undefined;
  });
  const capture=new NativeCapture('hold-1',()=>{},()=>{});
  await capture.start(null);
  const result=await capture.finish();
  expect(result.text).toBe('Native dictation');
  expect(result.blob.type).toBe('audio/wav');
  expect(openMicrophoneStream).not.toHaveBeenCalled();
  expect(invoke).toHaveBeenCalledWith('native_audio_finish',expect.objectContaining({sessionId:'hold-1'}));
 });
 it('key-up waits for native startup then finishes the same session',async()=>{
  let ready!:()=>void;
  vi.mocked(invoke).mockImplementation(async(command)=>{
   if(command==='native_audio_start')return new Promise<void>(resolve=>ready=resolve);
   if(command==='native_audio_finish')return {audioBase64:btoa('audio'),mimeType:'audio/wav'};
   if(command==='destroy_transcribe_dictation')return {text:'Kept the recording',path:'local'};
  });
  const capture=new NativeCapture('hold-2',()=>{},()=>{});
  const started=capture.start(null),finished=capture.finish();
  expect(invoke).not.toHaveBeenCalledWith('native_audio_finish',expect.anything());
  ready();await started;
  expect((await finished).text).toBe('Kept the recording');
 });
 it('cancellation during startup stops native audio and never transcribes',async()=>{
  let ready!:()=>void;
  vi.mocked(invoke).mockImplementation(async(command)=>{
   if(command==='native_audio_start')return new Promise<void>(resolve=>ready=resolve);
  });
  const capture=new NativeCapture('hold-3',()=>{},()=>{});
  const started=capture.start(null);capture.cancel();ready();await started;
  await expect(capture.finish()).rejects.toThrow('cancelled');
  expect(invoke).toHaveBeenCalledWith('native_audio_cancel',{sessionId:'hold-3'});
  expect(vi.mocked(invoke).mock.calls.some(([command])=>command==='destroy_transcribe_dictation')).toBe(false);
 });
 it('bounds native startup and cancels a queue that resolves after the deadline',async()=>{
  vi.useFakeTimers();
  let ready!:()=>void;
  vi.mocked(invoke).mockImplementation(async(command)=>{
   if(command==='native_audio_start')return new Promise<void>(resolve=>ready=resolve);
  });
  const capture=new NativeCapture('hold-start-timeout',()=>{},()=>{});
  const started=capture.start(null);
  const failed=expect(started).rejects.toThrow('did not start within 5 seconds');
  await vi.advanceTimersByTimeAsync(5000);
  await failed;
  expect(invoke).toHaveBeenCalledWith('native_audio_cancel',{sessionId:'hold-start-timeout'});
  ready();
  await vi.advanceTimersByTimeAsync(0);
  expect(vi.mocked(invoke).mock.calls.filter(([command])=>command==='native_audio_cancel')).toHaveLength(2);
  vi.useRealTimers();
 });
 it('surfaces a stalled native input while the key is still held and cancels capture',async()=>{
  vi.useFakeTimers();
  const onError=vi.fn();
  vi.mocked(invoke).mockImplementation(async(command)=>command==='native_audio_health'
   ? 'Microphone audio stopped arriving. Check that your input device is connected, then try again.'
   : undefined);
  const capture=new NativeCapture('hold-health',()=>{},()=>{},onError);
  await capture.start(null);
  await vi.advanceTimersByTimeAsync(250);
  expect(onError).toHaveBeenCalledWith(expect.stringContaining('input device'));
  expect(invoke).toHaveBeenCalledWith('native_audio_cancel',{sessionId:'hold-health'});
  expect(invoke).not.toHaveBeenCalledWith('native_audio_finish',expect.anything());
  vi.useRealTimers();
 });
 it('finishes at the 119-second safety timer without cancelling the retained recording',async()=>{
  vi.useFakeTimers();
  vi.mocked(invoke).mockImplementation(async(command)=>{
   if(command==='native_audio_finish')return {audioBase64:btoa('audio'),mimeType:'audio/wav'};
   if(command==='destroy_transcribe_dictation')return {text:'The full recording',path:'local'};
   return command==='native_audio_health'?null:0;
  });
  let capture!:NativeCapture;
  const timeout=vi.fn(()=>void capture.finish());
  capture=new NativeCapture('hold-limit',()=>{},timeout);
  await capture.start(null);
  await vi.advanceTimersByTimeAsync(119000);
  expect(timeout).toHaveBeenCalledTimes(1);
  expect(invoke).toHaveBeenCalledWith('native_audio_finish',expect.objectContaining({sessionId:'hold-limit'}));
  expect(invoke).not.toHaveBeenCalledWith('native_audio_cancel',{sessionId:'hold-limit'});
  vi.useRealTimers();
 });
 it('clears a quiet advisory when sound resumes without clearing the near-limit warning',async()=>{
  vi.useFakeTimers();
  let level=0;
  vi.mocked(invoke).mockImplementation(async(command)=>command==='native_audio_level'?level:command==='native_audio_health'?null:undefined);
  const warnings:Array<[string|null,'quiet'|'limit']>=[];
  const capture=new NativeCapture('hold-warnings',()=>{},()=>{},()=>{},(text,kind)=>warnings.push([text,kind]));
  await capture.start(null);
  await vi.advanceTimersByTimeAsync(8100);
  expect(warnings).toContainEqual(['No sound detected. Check your microphone.','quiet']);
  level=0.5;
  await vi.advanceTimersByTimeAsync(150);
  expect(warnings).toContainEqual([null,'quiet']);
  await vi.advanceTimersByTimeAsync(105000-8250);
  expect(warnings).toContainEqual(['About 15 seconds left. Release your shortcut to finish dictation.','limit']);
  level=0;
  await vi.advanceTimersByTimeAsync(8500);
  expect(warnings).toContainEqual(['About 15 seconds left. Release your shortcut to finish dictation.','limit']);
  expect(warnings.filter(([,kind])=>kind==='limit')).toHaveLength(1);
  capture.cancel();
  vi.useRealTimers();
 });
 it('ignores a deferred health failure that arrives after finish begins',async()=>{
  let reportHealth!:(value:string)=>void;
  vi.mocked(invoke).mockImplementation(async(command)=>{
   if(command==='native_audio_health')return new Promise<string>(resolve=>reportHealth=resolve);
   if(command==='native_audio_finish')return {audioBase64:btoa('audio'),mimeType:'audio/wav'};
   if(command==='destroy_transcribe_dictation')return {text:'Finished safely',path:'local'};
  });
  const onError=vi.fn();
  vi.useFakeTimers();
  const capture=new NativeCapture('hold-health-race',()=>{},()=>{},onError);
  await capture.start(null);
  await vi.advanceTimersByTimeAsync(250);
  const finished=capture.finish();
  reportHealth('Microphone audio stopped arriving. Check your input device.');
  expect((await finished).text).toBe('Finished safely');
  await Promise.resolve();
  expect(onError).not.toHaveBeenCalled();
  expect(invoke).not.toHaveBeenCalledWith('native_audio_cancel',{sessionId:'hold-health-race'});
  vi.useRealTimers();
 });
 it('does not send captured audio after the connection owner changes',async()=>{
  vi.mocked(invoke).mockImplementation(async(command)=>{
   if(command==='native_audio_finish'){
    setAccount('different-owner','https://other.invalid');
    return {audioBase64:btoa('audio'),mimeType:'audio/wav'};
   }
  });
  const capture=new NativeCapture('hold-4',()=>{},()=>{});
  await capture.start(null);
  await expect(capture.finish()).rejects.toThrow('Connection changed');
  expect(vi.mocked(invoke).mock.calls.some(([command])=>command==='destroy_transcribe_dictation')).toBe(false);
 });
});

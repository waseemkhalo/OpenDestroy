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

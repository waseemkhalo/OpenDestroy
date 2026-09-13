import {describe,it,expect,vi,beforeEach} from 'vitest';
import {Capture,bytesBase64} from './capture';
import {openMicrophoneStream} from './audioDevices';
vi.mock('./audioDevices',()=>({openMicrophoneStream:vi.fn()}));
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}));
describe('capture ownership',()=>{
 beforeEach(()=>vi.clearAllMocks());
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

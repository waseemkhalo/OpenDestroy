import {describe, expect, it} from 'vitest';
import {render} from 'svelte/server';
import HomeScreen from './HomeScreen.svelte';
import {shortcutFeedback, type HeldShortcutKeys} from './shortcutFeedback';
const idle: HeldShortcutKeys = {meta:false,ctrl:false,alt:false,shift:false,codes:[]};
describe('shortcut key feedback', () => {
  it('keeps Home focused on the shortcut without duplicate practice controls', () => {
    const {body} = render(HomeScreen, {props:{shortcut:'CommandOrControl+Backquote',speech:{provider:'local',ready:true,modelReady:true,localModelName:'Whisper Base',language:'English'},phase:'recording',onSettings:()=>{}}});
    expect(body).not.toContain('Try your voice');
    expect(body).not.toContain('Place your cursor');
    expect(body).toContain('Hold to dictate in');
    expect(body).toContain('ChatGPT');
    expect(body).toContain('/app-icons/0.png');
    expect(body).toContain('/app-icons/1.png');
    expect(body).toContain('/app-icons/2.png');
    expect(body).toContain('Pause app examples');
    expect(body).not.toContain('Hold, speak, release');
    expect(body).not.toContain('<textarea');
    expect(body).not.toContain('Test microphone');
    expect(body).toContain('Main navigation');
    expect(body).toContain('Dictionary');
    expect(body).toContain('History');
    expect(body).toContain('Transcripts stay in memory');
    expect(body).toContain('Whisper Base');
    expect(body.match(/<kbd[^>]*pressed/g)).toHaveLength(2);
  });
  it('depresses only the actual held keys', () => {
    expect(shortcutFeedback('CommandOrControl+Backquote',{...idle,meta:true},true)).toEqual([true,false]);
    expect(shortcutFeedback('CommandOrControl+Backquote',{...idle,meta:true,codes:['Backquote']},true)).toEqual([true,true]);
  });
  it('clears released keys and blur state', () => {
    expect(shortcutFeedback('CommandOrControl+Backquote',idle,true)).toEqual([false,false]);
    expect(shortcutFeedback('CommandOrControl+Backquote',{...idle,meta:true,codes:[]},true)).toEqual([true,false]);
  });
  it('honors custom shortcuts and non-Mac control', () => {
    expect(shortcutFeedback('Ctrl+Shift+K',{...idle,ctrl:true,shift:true,codes:['KeyK']},false)).toEqual([true,true,true]);
    expect(shortcutFeedback('CommandOrControl+Space',{...idle,ctrl:true,codes:['Space']},false)).toEqual([true,true]);
    expect(shortcutFeedback('CommandOrControl+Backquote',{...idle,ctrl:true},true)).toEqual([false,false]);
  });
});

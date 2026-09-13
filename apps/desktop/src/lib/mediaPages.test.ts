import {describe,it,expect} from 'vitest';
import {appendMediaResults} from './mediaPages';
import type {DictationMediaResult} from './dictation/giphy';
const row=(id:string,kind:'gif'|'sticker'='gif')=>({id,kind}) as DictationMediaResult;
describe('media rotation',()=>{
 it('keeps displaced provider results after favorites for the next window',()=>{
 const all=appendMediaResults([row('favorite')],[row('a'),row('b'),row('c')]);
 const next=appendMediaResults(all,[row('d'),row('e'),row('f')]);
 expect(next.slice(0,3).map(v=>v.id)).toEqual(['favorite','a','b']);
 expect(next.slice(3,6).map(v=>v.id)).toEqual(['c','d','e']);
 });
 it('deduplicates overlapping pages while preserving provider order',()=>{
 expect(appendMediaResults([row('a')],[row('a'),row('b'),row('b'),row('c')]).map(v=>v.id)).toEqual(['a','b','c']);
 });
 it('keeps distinct kinds of the same provider identifier',()=>{
 expect(appendMediaResults([row('a')],[row('a','sticker')])).toHaveLength(2);
 });
});

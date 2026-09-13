import type {DictationMediaResult} from './dictation/giphy';
/** Keep approved favorite order, then provider order without skipping or repeating hits. */
export function appendMediaResults(existing:DictationMediaResult[],incoming:DictationMediaResult[]):DictationMediaResult[]{
 const seen=new Set(existing.map(v=>v.kind+':'+v.id));
 return [...existing,...incoming.filter(v=>{const key=v.kind+':'+v.id;if(seen.has(key))return false;seen.add(key);return true;})];
}

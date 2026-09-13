import {mount} from 'svelte';
import {getCurrentWindow} from '@tauri-apps/api/window';
import App from './App.svelte';
import Hud from './Hud.svelte';
import './style.css';
const hud = '__TAURI_INTERNALS__' in window && getCurrentWindow().label === 'dictation-hud';
mount(hud ? Hud : App, {target: document.getElementById('app')!});

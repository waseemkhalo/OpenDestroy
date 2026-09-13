#!/usr/bin/env python3
"""Run the built server against throwaway encrypted data; never use paid providers."""
import base64,hashlib,json,os,secrets,socket,subprocess,tempfile,time,urllib.request,urllib.error
from pathlib import Path
root=Path(__file__).resolve().parent.parent
target=Path(os.environ.get('CARGO_TARGET_DIR', str(root/'target')))
if not target.is_absolute():target=root/target
binary=target/'debug/dictation-backend'
if not binary.exists():raise SystemExit('Build first: cargo build --locked -p dictation-backend')
with tempfile.TemporaryDirectory(prefix='destroy-backend-smoke-') as temporary:
 with socket.socket() as sock:
  sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
 tokens=[secrets.token_urlsafe(32) for _ in range(2)]
 users={hashlib.sha256(t.encode()).hexdigest():u for t,u in zip(tokens,['alice','bob'])}
 env={k:v for k,v in os.environ.items() if not k.startswith(('OPENAI_','GEMINI_','GIPHY_','DESTROY_'))}
 env.update(DESTROY_LISTEN=f'127.0.0.1:{port}',DESTROY_DB=temporary+'/data.sqlite',DESTROY_DATA_KEY=base64.b64encode(secrets.token_bytes(32)).decode(),DESTROY_USERS=json.dumps(users))
 process=subprocess.Popen([str(binary)],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 def call(method,path,token=None,body=None):
  headers={'Content-Type':'application/json'} if body is not None else {}
  if token:headers['Authorization']='Bearer '+token
  request=urllib.request.Request(f'http://127.0.0.1:{port}'+path,data=json.dumps(body).encode() if body is not None else None,headers=headers,method=method)
  try:
   with urllib.request.urlopen(request,timeout=5) as response:return response.status,json.load(response)
  except urllib.error.HTTPError as error:return error.code,json.load(error)
 try:
  for attempt in range(100):
   if process.poll() is not None:raise RuntimeError('Backend startup failed')
   try:
    if call('GET','/health')[0]==200:break
   except OSError:time.sleep(.05)
  else:raise RuntimeError('Backend did not start')
  a,b=tokens
  assert call('GET','/v1/account')[0]==401
  assert call('GET','/v1/account','invalid-token')[0]==401
  assert call('GET','/v1/account',a)==(200,{'user_id':'alice'})
  assert call('GET','/v1/me?user_id=bob',a)==(200,{'user_id':'alice'})
  assert call('POST','/v1/dictation/transcribe',a,{'audio_base64':'AAAA','mime_type':'audio/wav'})[0]==503
  marker='private-smoke-vocabulary-'+secrets.token_hex(8)
  assert call('PUT','/v1/dictation/vocabulary',a,{'terms':[marker]})[0]==200
  assert call('GET','/v1/dictation/vocabulary',b)[1]['terms']==[]
  assert call('PUT','/v1/links',a,[{'name':'demo file','url':'https://example.com/demo','keywords':'product'}])[0]==200
  assert call('POST','/v1/links/search',a,{'query':'demo'})[1]['results'][0]['url']=='https://example.com/demo'
  assert call('POST','/v1/links/search',b,{'query':'demo'})[1]['results']==[]
  assert call('GET','/v1/dictation/media/status',a)[1]['enabled'] is False
  assert call('POST','/v1/dictation/media/search',a,{'query':'happy','kind':'gif'})[0]==503
  assert call('GET','/v1/account/export',a)[1]['terms']==[marker]
  for file in Path(temporary).iterdir():
   if file.is_file():assert marker.encode() not in file.read_bytes()
  # Even a damaged encrypted record must remain deletable by its owner.
  import sqlite3
  with sqlite3.connect(temporary+'/data.sqlite') as db:
   db.execute("UPDATE personal_data SET payload=? WHERE user_id='alice'", (b'corrupt',))
  assert call('GET','/v1/account/export',a)[0]==500
  assert call('GET','/v1/account',a)==(200,{'user_id':'alice'})
  assert call('DELETE','/v1/account/data',a)[0]==200
  assert call('GET','/v1/account/export',a)[1]=={}
  print('HTTP smoke passed: startup, auth, user isolation, vocabulary, link resolution, provider gates, encrypted files, export and corrupt-record deletion.')
 finally:
  process.terminate()
  try:process.wait(timeout=5)
  except subprocess.TimeoutExpired:process.kill();process.wait()

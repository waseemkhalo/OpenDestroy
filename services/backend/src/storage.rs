use aes_gcm::{
    aead::{Aead, Payload},
    Aes256Gcm, KeyInit,
};
use rand::RngCore;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::Value;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Semaphore;
use zeroize::Zeroizing;

pub struct Store {
    db: Arc<Mutex<Connection>>,
    permits: Arc<Semaphore>,
    master: Zeroizing<[u8; 32]>,
}
fn seal(key: &[u8], aad: &[u8], bytes: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "Invalid key")?;
    let mut nonce = [0; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    let mut output = nonce.to_vec();
    output.extend(
        cipher
            .encrypt((&nonce).into(), Payload { msg: bytes, aad })
            .map_err(|_| "Encryption failed")?,
    );
    Ok(output)
}
fn open(key: &[u8], aad: &[u8], bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, String> {
    if bytes.len() < 28 {
        return Err("Invalid encrypted record".into());
    }
    Aes256Gcm::new_from_slice(key)
        .map_err(|_| "Invalid key")?
        .decrypt(
            bytes[..12].into(),
            Payload {
                msg: &bytes[12..],
                aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| "Record authentication failed".into())
}
impl Store {
    pub fn new(path: &str, master: Zeroizing<[u8; 32]>) -> Result<Self, String> {
        let mut db = Connection::open(path).map_err(|_| "Cannot open database")?;
        db.busy_timeout(Duration::from_secs(5))
            .map_err(|_| "Cannot configure database")?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA secure_delete=ON;")
            .map_err(|_| "Cannot configure database")?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| "Storage busy")?;
        let version: i64 = tx
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|_| "Cannot read schema version")?;
        if !(0..=1).contains(&version) {
            return Err("Unsupported SQLite schema version".into());
        }
        tx.execute_batch("CREATE TABLE IF NOT EXISTS personal_data (user_id TEXT PRIMARY KEY, wrapped_key BLOB NOT NULL, payload BLOB NOT NULL); PRAGMA user_version=1;").map_err(|_| "Cannot migrate database")?;
        tx.commit().map_err(|_| "Cannot migrate database")?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            permits: Arc::new(Semaphore::new(16)),
            master,
        })
    }
    // SQLite and encryption run off the async executor, with bounded pending work.
    async fn run<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut Connection, &[u8; 32]) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        let permit = self
            .permits
            .clone()
            .try_acquire_owned()
            .map_err(|_| "Storage busy")?;
        let db = self.db.clone();
        let master = self.master.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let mut db = db.lock().map_err(|_| "Storage unavailable")?;
            f(&mut db, &master)
        })
        .await
        .map_err(|_| "Storage task failed")?
    }
    fn read_with(db: &Connection, master: &[u8; 32], user: &str) -> Result<Value, String> {
        let row: Option<(Vec<u8>, Vec<u8>)> = db
            .query_row(
                "SELECT wrapped_key,payload FROM personal_data WHERE user_id=?1",
                [user],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|_| "Cannot read personal data")?;
        match row {
            None => Ok(serde_json::json!({})),
            Some((key, data)) => {
                let key = open(master, format!("destroy:v1:key:{user}").as_bytes(), &*key)?;
                let bytes = open(&key, format!("destroy:v1:data:{user}").as_bytes(), &data)?;
                serde_json::from_slice(&bytes).map_err(|_| "Invalid personal data".into())
            }
        }
    }
    pub async fn read(&self, user: &str) -> Result<Value, String> {
        let user = user.to_owned();
        self.run(move |db, master| Self::read_with(db, master, &user))
            .await
    }
    pub async fn update(
        &self,
        user: &str,
        f: impl FnOnce(&mut Value) -> Result<Value, String> + Send + 'static,
    ) -> Result<Value, String> {
        let user = user.to_owned();
        self.run(move |db, master| {
            let tx = db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|_| "Storage busy")?;
            let mut value = Self::read_with(&tx, master, &user)?;
            let result = f(&mut value)?;
            let bytes = Zeroizing::new(serde_json::to_vec(&value).map_err(|_| "Invalid personal data")?);
            if bytes.len() > 1024 * 1024 { return Err("Personal data exceeds 1 MiB".into()); }
            let mut key = Zeroizing::new([0; 32]);
            rand::thread_rng().fill_bytes(&mut *key);
            let wrapped = seal(master, format!("destroy:v1:key:{user}").as_bytes(), &*key)?;
            let encrypted = seal(&*key, format!("destroy:v1:data:{user}").as_bytes(), &bytes)?;
            tx.execute("INSERT INTO personal_data VALUES (?1,?2,?3) ON CONFLICT(user_id) DO UPDATE SET wrapped_key=excluded.wrapped_key,payload=excluded.payload",params![user,wrapped,encrypted]).map_err(|_| "Cannot save personal data")?;
            tx.commit().map_err(|_| "Cannot save personal data")?;
            Ok(result)
        }).await
    }
    pub async fn delete(&self, user: &str) -> Result<(), String> {
        let user = user.to_owned();
        self.run(move |db, _| {
            db.execute("DELETE FROM personal_data WHERE user_id=?1", [user])
                .map_err(|_| "Cannot delete personal data")?;
            Ok(())
        })
        .await
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[tokio::test]
    async fn isolated_encrypted_and_deletable() {
        let s = Store::new(":memory:", Zeroizing::new([7; 32])).unwrap();
        s.update("alice", |v| {
            v["secret"] = "private phrase".into();
            Ok(v.clone())
        })
        .await
        .unwrap();
        assert_eq!(s.read("alice").await.unwrap()["secret"], "private phrase");
        assert!(s.read("bob").await.unwrap()["secret"].is_null());
        let db = s.db.lock().unwrap();
        let ciphertext: Vec<u8> = db
            .query_row("SELECT payload FROM personal_data", [], |r| r.get(0))
            .unwrap();
        assert!(!String::from_utf8_lossy(&ciphertext).contains("private phrase"));
        db.execute("UPDATE personal_data SET user_id='bob'", [])
            .unwrap();
        drop(db);
        assert!(s.read("bob").await.is_err());
        s.delete("bob").await.unwrap();
        assert_eq!(s.read("bob").await.unwrap(), serde_json::json!({}));
    }
    #[test]
    fn tamper_fails() {
        let mut b = seal(&[4; 32], b"alice", b"secret").unwrap();
        b[15] ^= 1;
        assert!(open(&[4; 32], b"alice", &b).is_err());
    }
    #[tokio::test]
    async fn rejected_update_leaves_previous_record_intact() {
        let store = Store::new(":memory:", Zeroizing::new([9; 32])).unwrap();
        store
            .update("alice", |v| {
                v["name"] = json!("original");
                Ok(Value::Null)
            })
            .await
            .unwrap();
        let result = store
            .update("alice", |v| {
                v["name"] = json!("changed");
                Err("validation rejected".into())
            })
            .await;
        assert!(result.is_err());
        assert_eq!(store.read("alice").await.unwrap()["name"], "original");
        let result = store
            .update("alice", |v| {
                v["large"] = json!("x".repeat(1024 * 1024));
                Ok(Value::Null)
            })
            .await;
        assert!(result.is_err());
        assert!(store.read("alice").await.unwrap()["large"].is_null());
    }
    #[test]
    fn encrypted_data_rejects_wrong_key_and_owner() {
        let encrypted = seal(&[4; 32], b"alice", b"private").unwrap();
        assert!(open(&[5; 32], b"alice", &encrypted).is_err());
        assert!(open(&[4; 32], b"bob", &encrypted).is_err());
        assert!(open(&[4; 32], b"alice", b"too short").is_err());
    }
}

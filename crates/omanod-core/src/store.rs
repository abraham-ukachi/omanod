use crate::{Error, Message, Result, Role};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct Thread {
  pub id: String,
  pub title: String,
  pub updated_at: String,
}

pub struct Store {
  conn: Connection,
}

impl Store {
  pub fn open() -> Result<Store> {
    let data_home = std::env::var("XDG_DATA_HOME")
      .map(PathBuf::from)
      .unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/abrahamukachi".into());
        PathBuf::from(home).join(".local").join("share")
      });
    let dir = data_home.join("omanod");
    std::fs::create_dir_all(&dir)?;
    let conn = Connection::open(dir.join("omanod.db"))?;
    conn.execute_batch(
      "CREATE TABLE IF NOT EXISTS threads (
         id TEXT PRIMARY KEY,
         title TEXT NOT NULL,
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL
       );
       CREATE TABLE IF NOT EXISTS messages (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         thread_id TEXT NOT NULL,
         role TEXT NOT NULL,
         content TEXT NOT NULL,
         created_at TEXT NOT NULL
       );
       CREATE INDEX IF NOT EXISTS messages_thread ON messages(thread_id, id);
       CREATE TABLE IF NOT EXISTS settings (
         key TEXT PRIMARY KEY,
         value TEXT NOT NULL
       );",
    )?;
    Ok(Store { conn })
  }

  pub fn threads(&self) -> Result<Vec<Thread>> {
    let mut stmt = self
      .conn
      .prepare("SELECT id, title, updated_at FROM threads ORDER BY updated_at DESC")?;
    let rows = stmt.query_map([], |row| {
      Ok(Thread {
        id: row.get(0)?,
        title: row.get(1)?,
        updated_at: row.get(2)?,
      })
    })?;
    rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Error::from)
  }

  pub fn create_thread(&self, title: &str) -> Result<String> {
    let id = format!(
      "t{}-{}",
      Utc::now().timestamp_millis(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    );
    let now = Utc::now().to_rfc3339();
    self.conn.execute(
      "INSERT INTO threads (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
      params![id, title, now],
    )?;
    Ok(id)
  }

  pub fn rename_thread(&self, id: &str, title: &str) -> Result<()> {
    self
      .conn
      .execute("UPDATE threads SET title = ?1 WHERE id = ?2", params![title, id])?;
    Ok(())
  }

  pub fn delete_thread(&self, id: &str) -> Result<()> {
    self
      .conn
      .execute("DELETE FROM messages WHERE thread_id = ?1", params![id])?;
    self.conn.execute("DELETE FROM threads WHERE id = ?1", params![id])?;
    Ok(())
  }

  pub fn messages(&self, thread_id: &str) -> Result<Vec<Message>> {
    let mut stmt = self
      .conn
      .prepare("SELECT role, content FROM messages WHERE thread_id = ?1 ORDER BY id")?;
    let rows = stmt.query_map(params![thread_id], |row| {
      let role: String = row.get(0)?;
      let role = match role.as_str() {
        "assistant" => Role::Assistant,
        "system" => Role::System,
        _ => Role::User,
      };
      Ok(Message {
        role,
        content: row.get(1)?,
      })
    })?;
    rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Error::from)
  }

  pub fn add_message(&self, thread_id: &str, role: Role, content: &str) -> Result<()> {
    let role = match role {
      Role::User => "user",
      Role::Assistant => "assistant",
      Role::System => "system",
    };
    let now = Utc::now().to_rfc3339();
    self.conn.execute(
      "INSERT INTO messages (thread_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4)",
      params![thread_id, role, content, now],
    )?;
    self.conn.execute(
      "UPDATE threads SET updated_at = ?1 WHERE id = ?2",
      params![now, thread_id],
    )?;
    Ok(())
  }

  pub fn setting(&self, key: &str) -> Result<Option<String>> {
    let mut stmt = self.conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    match rows.next()? {
      Some(row) => Ok(Some(row.get(0)?)),
      None => Ok(None),
    }
  }

  pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
    self.conn.execute(
      "INSERT INTO settings (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![key, value],
    )?;
    Ok(())
  }
}

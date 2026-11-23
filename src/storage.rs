#![forbid(unsafe_code)]

//! Bundled SQLite WAL Workset Persistence (`F5` / `F9`) and Portable `.pworkset.json` SHA-256 Envelopes.

use crate::render3d::Camera3D;
use crate::ExplorerParams;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksetEnvelope {
    pub schema_version: String,
    pub slot_name: String,
    pub saved_utc: String,
    pub params: ExplorerParams,
    pub camera: Camera3D,
    pub sha256_hex: String,
}

impl WorksetEnvelope {
    pub fn new(slot_name: impl Into<String>, params: ExplorerParams, camera: Camera3D) -> Self {
        let slot_name = slot_name.into();
        let saved_utc = "2026-11-22T22:00:00Z".to_string();
        let mut env = Self {
            schema_version: "principia.workset.v1".to_string(),
            slot_name,
            saved_utc,
            params,
            camera,
            sha256_hex: String::new(),
        };
        env.sha256_hex = env.compute_digest();
        env
    }

    pub fn compute_digest(&self) -> String {
        #[derive(Serialize)]
        struct Canonical<'a> {
            schema_version: &'a str,
            slot_name: &'a str,
            params: &'a ExplorerParams,
            camera: &'a Camera3D,
        }
        let bytes = serde_json::to_vec(&Canonical {
            schema_version: &self.schema_version,
            slot_name: &self.slot_name,
            params: &self.params,
            camera: &self.camera,
        })
        .unwrap_or_default();
        let digest = Sha256::digest(&bytes);
        let mut hex = String::with_capacity(64);
        for b in digest {
            hex.push_str(&format!("{:02x}", b));
        }
        hex
    }

    pub fn verify_checksum(&self) -> bool {
        self.sha256_hex == self.compute_digest()
    }
}

pub struct WorksetVault {
    conn: Connection,
}

impl WorksetVault {
    pub fn open(db_path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS workset_slots (
                slot_id INTEGER PRIMARY KEY,
                slot_name TEXT NOT NULL,
                module_code TEXT NOT NULL,
                envelope_json TEXT NOT NULL,
                sha256_hex TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS workset_slots (
                slot_id INTEGER PRIMARY KEY,
                slot_name TEXT NOT NULL,
                module_code TEXT NOT NULL,
                envelope_json TEXT NOT NULL,
                sha256_hex TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn save_slot(&self, slot_id: i64, envelope: &WorksetEnvelope) -> SqlResult<()> {
        let json = serde_json::to_string_pretty(envelope).unwrap_or_default();
        self.conn.execute(
            "INSERT INTO workset_slots (slot_id, slot_name, module_code, envelope_json, sha256_hex)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(slot_id) DO UPDATE SET
               slot_name = excluded.slot_name,
               module_code = excluded.module_code,
               envelope_json = excluded.envelope_json,
               sha256_hex = excluded.sha256_hex;",
            params![
                slot_id,
                envelope.slot_name,
                envelope.params.module.code(),
                json,
                envelope.sha256_hex
            ],
        )?;
        Ok(())
    }

    pub fn load_slot(&self, slot_id: i64) -> SqlResult<Option<WorksetEnvelope>> {
        let mut stmt = self
            .conn
            .prepare("SELECT envelope_json FROM workset_slots WHERE slot_id = ?1")?;
        let mut rows = stmt.query(params![slot_id])?;
        if let Some(row) = rows.next()? {
            let json: String = row.get(0)?;
            let parsed: Option<WorksetEnvelope> = serde_json::from_str(&json).ok();
            Ok(parsed.filter(|e| e.verify_checksum()))
        } else {
            Ok(None)
        }
    }
}

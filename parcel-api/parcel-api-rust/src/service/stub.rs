use chrono::Utc;
use rand::seq::SliceRandom;
use std::fs;
use std::path::Path;

use crate::domain::parcel::Parcel;
use crate::service::features::compute_features;

/// Stores parcel JSON as raw bytes in memory; re-deserialises on every request.
/// Puts the JSON parse cost on the request path so the bench measures the same
/// kind of work a real service does when decoding data from a downstream.
pub struct StubParcelService {
    parcels_json: Vec<Vec<u8>>,
}

impl StubParcelService {
    pub fn load(dir: &Path) -> Self {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("read parcel data dir {dir:?}: {e}"))
            .filter_map(|r| r.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .collect();
        entries.sort_by_key(|e| e.path());

        let parcels_json: Vec<Vec<u8>> = entries
            .iter()
            .map(|e| {
                let path = e.path();
                fs::read(&path).unwrap_or_else(|err| panic!("read {path:?}: {err}"))
            })
            .collect();

        Self { parcels_json }
    }

    pub fn list_parcels(&self) -> Vec<Parcel> {
        let now = Utc::now();
        let mut order: Vec<&Vec<u8>> = self.parcels_json.iter().collect();
        order.shuffle(&mut rand::thread_rng());
        order
            .into_iter()
            .map(|bytes| {
                let mut parcel: Parcel =
                    serde_json::from_slice(bytes).expect("parcel json should parse");
                parcel.features = compute_features(&parcel, now);
                parcel
            })
            .collect()
    }
}

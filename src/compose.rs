use ::serde_json::*;

use crate::{parse::record::Record, storage::Storage};

pub fn compose_push_record_message(record: &Record) -> Value {
    json!({
         "method": "pushRecord",
         "params": {
             "record": record,
         }
    })
}

pub fn compose_init_message(storage: &Storage) -> Value {
    if storage.is_empty() {
        json!({
             "method": "initStorage",
             "params": {
                 "data": {},
             }
        })
    } else {
        json!({
             "method": "initStorage",
             "params": {
                 "data": storage,
             }
        })
    }
}

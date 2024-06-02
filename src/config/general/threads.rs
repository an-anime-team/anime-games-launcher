use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Threads {
    /// Amount of threads in the pool
    pub number: u64,

    /// Max amount of tasks that can be put into the threads pool queue at the same time
    pub max_queue_size: u64
}

impl Default for Threads {
    #[inline]
    fn default() -> Self {
        // let cores = num_cpus::get() as u64;
        let cores = 4;

        Self {
            number: cores,
            max_queue_size: cores * 8
        }
    }
}

impl From<&Json> for Threads {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            number: value.get("number")
                .and_then(Json::as_u64)
                .unwrap_or(default.number),

            max_queue_size: value.get("max_queue_size")
                .and_then(Json::as_u64)
                .unwrap_or(default.max_queue_size)
        }
    }
}

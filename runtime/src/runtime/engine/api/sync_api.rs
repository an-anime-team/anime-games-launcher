use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mlua::prelude::*;

use super::*;

const MUTEX_LOCK_TIMEOUT: Duration = Duration::from_millis(100);

// Workaround for lifetimes fuckery.
#[derive(Debug, Clone)]
enum ChannelMessage {
    Table(Vec<(Self, Self)>),
    String(String),
    Double(f64),
    Integer(i32),
    Boolean(bool),
    Nil
}

impl ChannelMessage {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaValue, LuaError> {
        match self {
            Self::String(value) => lua.create_string(value)
                .map(LuaValue::String),

            Self::Double(value)  => Ok(LuaValue::Number(*value)),
            Self::Integer(value) => Ok(LuaValue::Integer(*value)),
            Self::Boolean(value) => Ok(LuaValue::Boolean(*value)),
            Self::Nil            => Ok(LuaNil),

            Self::Table(table) => {
                let result = lua.create_table_with_capacity(0, table.len())?;

                for (key, value) in table {
                    result.set(
                        key.to_lua(lua)?,
                        value.to_lua(lua)?
                    )?;
                }

                Ok(LuaValue::Table(result))
            }
        }
    }

    pub fn from_lua(value: &LuaValue) -> Result<Self, LuaError> {
        match value {
            LuaValue::String(value)  => Ok(Self::String(value.to_string_lossy().to_string())),
            LuaValue::Number(value)  => Ok(Self::Double(*value)),
            LuaValue::Integer(value) => Ok(Self::Integer(*value)),
            LuaValue::Boolean(value) => Ok(Self::Boolean(*value)),
            LuaValue::Nil            => Ok(Self::Nil),

            LuaValue::Table(table) => {
                let mut result = Vec::with_capacity(table.raw_len());

                for pair in table.clone().pairs::<LuaValue, LuaValue>() {
                    let (key, value) = pair?;

                    result.push((
                        Self::from_lua(&key)?,
                        Self::from_lua(&value)?
                    ));
                }

                Ok(Self::Table(result))
            }

            _ => Err(LuaError::external("can't coerce given value type"))
        }
    }
}

pub struct SyncAPI {
    lua: Lua,

    sync_channel_open: LuaFunction,
    sync_channel_send: LuaFunction,
    sync_channel_recv: LuaFunction,
    sync_channel_close: LuaFunction,

    sync_mutex_open: LuaFunction,
    sync_mutex_lock: LuaFunction,
    sync_mutex_unlock: LuaFunction,
    sync_mutex_close: LuaFunction
}

impl SyncAPI {
    pub fn new(lua: Lua) -> Result<Self, PackagesEngineError> {
        let sync_channels_consumers = Arc::new(Mutex::new(HashMap::new())); // key => handle
        let sync_channels_data = Arc::new(Mutex::new(HashMap::new())); // handle => (key, data)

        let sync_mutex_consumers = Arc::new(Mutex::new(HashMap::<i32, Hash>::new())); // handle => key
        let sync_mutex_locks = Arc::new(Mutex::new(HashMap::<Hash, Option<i32>>::new())); // key => curr_lock_handle

        Ok(Self {
            sync_channel_open: {
                let sync_channels_consumers = sync_channels_consumers.clone();
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |_, key: LuaString| {
                    let mut listeners = sync_channels_data.lock()
                        .map_err(|err| LuaError::external(format!("failed to register channel listeners: {err}")))?;

                    let key = Hash::for_slice(key.as_bytes());
                    let mut handle = rand::random::<i32>();

                    while listeners.contains_key(&handle) {
                        handle = rand::random::<i32>();
                    }

                    let mut consumers = sync_channels_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to register channel consumers: {err}")))?;

                    consumers.entry(key).or_insert_with(HashSet::new);

                    if let Some(listeners) = consumers.get_mut(&key) {
                        listeners.insert(handle);
                    }

                    listeners.insert(handle, (key, VecDeque::new()));

                    Ok(handle)
                })?
            },

            sync_channel_send: {
                let sync_channels_consumers = sync_channels_consumers.clone();
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |_, (handle, message): (i32, LuaValue)| {
                    let message = ChannelMessage::from_lua(&message)?;

                    let mut listeners = sync_channels_data.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel listeners: {err}")))?;

                    let Some((key, _)) = listeners.get(&handle) else {
                        return Err(LuaError::external("invalid channel handle"));
                    };

                    let consumers = sync_channels_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel consumers: {err}")))?;

                    let Some(consumers) = consumers.get(key) else {
                        return Err(LuaError::external("invalid channel handle"));
                    };

                    for consumer in consumers {
                        if consumer != &handle {
                            if let Some((_, data)) = listeners.get_mut(consumer) {
                                data.push_back(message.clone());
                            }
                        }
                    }

                    Ok(())
                })?
            },

            sync_channel_recv: {
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |lua, handle: i32| {
                    let mut listeners = sync_channels_data.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel listeners: {err}")))?;

                    let Some((_, data)) = listeners.get_mut(&handle) else {
                        return Err(LuaError::external("invalid channel handle"));
                    };

                    match data.pop_front() {
                        Some(message) => Ok((message.to_lua(lua)?, true)),
                        None => Ok((LuaNil, false))
                    }
                })?
            },

            sync_channel_close: {
                let sync_channels_consumers = sync_channels_consumers.clone();
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut consumers = sync_channels_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel consumers: {err}")))?;

                    let mut listeners = sync_channels_data.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel listeners: {err}")))?;

                    if let Some((hash, _)) = listeners.remove(&handle) {
                        let mut empty = false;

                        if let Some(listeners) = consumers.get_mut(&hash) {
                            listeners.remove(&handle);

                            empty = listeners.is_empty();
                        }

                        if empty {
                            consumers.remove(&hash);
                        }
                    }

                    Ok(())
                })?
            },

            sync_mutex_open: {
                let sync_mutex_consumers = sync_mutex_consumers.clone();

                lua.create_function(move |_, key: LuaString| {
                    let key = Hash::for_slice(key.as_bytes());

                    let mut consumers = sync_mutex_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to register mutex consumers: {err}")))?;

                    let mut handle = rand::random::<i32>();

                    while consumers.contains_key(&handle) {
                        handle = rand::random::<i32>();
                    }

                    consumers.insert(handle, key);

                    Ok(handle)
                })?
            },

            sync_mutex_lock: {
                let sync_mutex_consumers = sync_mutex_consumers.clone();
                let sync_mutex_locks = sync_mutex_locks.clone();

                lua.create_function(move |_, handle: i32| {
                    let key = sync_mutex_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read mutex consumers: {err}")))?
                        .get(&handle)
                        .copied()
                        .ok_or_else(|| LuaError::external("invalid mutex handle"))?;

                    loop {
                        let mut locks = sync_mutex_locks.lock()
                            .map_err(|err| LuaError::external(format!("failed to read mutex locks: {err}")))?;

                        match locks.get_mut(&key) {
                            Some(lock) => {
                                if lock.is_none() {
                                    *lock = Some(handle);

                                    return Ok(());
                                }
                            }

                            None => {
                                locks.insert(key, Some(handle));

                                return Ok(());
                            }
                        }

                        drop(locks);

                        std::thread::sleep(MUTEX_LOCK_TIMEOUT);
                    }
                })?
            },

            sync_mutex_unlock: {
                let sync_mutex_consumers = sync_mutex_consumers.clone();
                let sync_mutex_locks = sync_mutex_locks.clone();

                lua.create_function(move |_, handle: i32| {
                    let key = sync_mutex_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read mutex consumers: {err}")))?
                        .get(&handle)
                        .copied()
                        .ok_or_else(|| LuaError::external("invalid mutex handle"))?;

                    let mut locks = sync_mutex_locks.lock()
                        .map_err(|err| LuaError::external(format!("failed to read mutex locks: {err}")))?;

                    if let Some(lock) = locks.get_mut(&key) {
                        if let Some(lock_handle) = lock {
                            if *lock_handle != handle {
                                return Err(LuaError::external("can't unlock mutex locked by another handle"));
                            }

                            *lock = None;
                        }
                    }

                    Ok(())
                })?
            },

            sync_mutex_close: {
                let sync_mutex_consumers = sync_mutex_consumers.clone();
                let sync_mutex_locks = sync_mutex_locks.clone();

                lua.create_function(move |_, handle: i32| {
                    let key = sync_mutex_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read mutex consumers: {err}")))?
                        .remove(&handle);

                    if let Some(key) = key {
                        let mut locks = sync_mutex_locks.lock()
                            .map_err(|err| LuaError::external(format!("failed to read mutex locks: {err}")))?;

                        if let Some(lock) = locks.get_mut(&key) {
                            if let Some(lock_handle) = lock {
                                if *lock_handle == handle {
                                    *lock = None;
                                }
                            }
                        }
                    }

                    Ok(())
                })?
            },

            lua
        })
    }

    #[inline(always)]
    pub const fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 2)?;

        let sync_channel = self.lua.create_table_with_capacity(0, 4)?;
        let sync_mutex = self.lua.create_table_with_capacity(0, 4)?;

        env.raw_set("channel", sync_channel.clone())?;
        env.raw_set("mutex", sync_mutex.clone())?;

        // Channel

        sync_channel.raw_set("open", self.sync_channel_open.clone())?;
        sync_channel.raw_set("send", self.sync_channel_send.clone())?;
        sync_channel.raw_set("recv", self.sync_channel_recv.clone())?;
        sync_channel.raw_set("close", self.sync_channel_close.clone())?;

        // Mutex

        sync_mutex.raw_set("open", self.sync_mutex_open.clone())?;
        sync_mutex.raw_set("lock", self.sync_mutex_lock.clone())?;
        sync_mutex.raw_set("unlock", self.sync_mutex_unlock.clone())?;
        sync_mutex.raw_set("close", self.sync_mutex_close.clone())?;

        Ok(env)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn sync_channels() -> anyhow::Result<()> {
//         let api = SyncAPI::new(Lua::new())?;

//         assert!(api.sync_channel_send.call::<()>((0, String::new())).is_err());
//         assert!(api.sync_channel_recv.call::<Option<String>>(0).is_err());

//         let a = api.sync_channel_open.call::<i32>("test")?;
//         let b = api.sync_channel_open.call::<i32>("test")?;

//         assert_eq!(api.sync_channel_recv.call::<Option<String>>(a)?, None);
//         assert_eq!(api.sync_channel_recv.call::<Option<String>>(b)?, None);

//         api.sync_channel_send.call::<()>((a, String::from("Message 1")))?;
//         api.sync_channel_send.call::<()>((a, String::from("Message 2")))?;

//         let c = api.sync_channel_open.call::<i32>("test")?;

//         assert_eq!(api.sync_channel_recv.call::<Option<String>>(a)?, None);
//         assert_eq!(api.sync_channel_recv.call::<Option<String>>(c)?, None);
//         assert_eq!(api.sync_channel_recv.call::<String>(b)?, "Message 1");
//         assert_eq!(api.sync_channel_recv.call::<String>(b)?, "Message 2");
//         assert_eq!(api.sync_channel_recv.call::<Option<String>>(b)?, None);

//         api.sync_channel_send.call::<()>((a, String::from("Message 3")))?;

//         assert_eq!(api.sync_channel_recv.call::<Option<String>>(a)?, None);
//         assert_eq!(api.sync_channel_recv.call::<String>(b)?, "Message 3");
//         assert_eq!(api.sync_channel_recv.call::<String>(c)?, "Message 3");
//         assert_eq!(api.sync_channel_recv.call::<Option<String>>(b)?, None);
//         assert_eq!(api.sync_channel_recv.call::<Option<String>>(c)?, None);

//         api.sync_channel_send.call::<()>((a, true))?;
//         api.sync_channel_send.call::<()>((a, 0.5))?;
//         api.sync_channel_send.call::<()>((a, -17))?;
//         api.sync_channel_send.call::<()>((a, vec![1, 2, 3]))?;
//         api.sync_channel_send.call::<()>((a, vec!["Hello", "World"]))?;
//         api.sync_channel_send.call::<()>((a, vec![vec![1, 2], vec![3, 4]]))?;

//         assert_eq!(api.sync_channel_recv.call::<Option<_>>(b)?, Some(true));
//         assert_eq!(api.sync_channel_recv.call::<Option<_>>(b)?, Some(0.5));
//         assert_eq!(api.sync_channel_recv.call::<Option<_>>(b)?, Some(-17));
//         assert_eq!(api.sync_channel_recv.call::<Option<_>>(b)?, Some(vec![1, 2, 3]));
//         assert_eq!(api.sync_channel_recv.call::<Option<_>>(b)?, Some(vec![String::from("Hello"), String::from("World")]));
//         assert_eq!(api.sync_channel_recv.call::<Option<_>>(b)?, Some(vec![vec![1, 2], vec![3, 4]]));
//         assert_eq!(api.sync_channel_recv.call::<Option<String>>(b)?, None);

//         api.sync_channel_close.call::<()>(a)?;
//         api.sync_channel_close.call::<()>(b)?;
//         api.sync_channel_close.call::<()>(c)?;

//         assert!(api.sync_channel_send.call::<()>((a, String::new())).is_err());
//         assert!(api.sync_channel_recv.call::<Option<String>>(a).is_err());

//         assert!(api.sync_channel_send.call::<()>((b, String::new())).is_err());
//         assert!(api.sync_channel_recv.call::<Option<String>>(b).is_err());

//         assert!(api.sync_channel_send.call::<()>((c, String::new())).is_err());
//         assert!(api.sync_channel_recv.call::<Option<String>>(c).is_err());

//         Ok(())
//     }
// }

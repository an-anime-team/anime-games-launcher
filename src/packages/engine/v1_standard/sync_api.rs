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
    pub fn from_lua(value: LuaValue) -> Result<Self, LuaError> {
        match value {
            LuaValue::String(value) => Ok(Self::String(value.to_string_lossy().to_string())),
            LuaValue::Number(value) => Ok(Self::Double(value)),
            LuaValue::Integer(value) => Ok(Self::Integer(value)),
            LuaValue::Boolean(value) => Ok(Self::Boolean(value)),
            LuaValue::Nil => Ok(Self::Nil),

            LuaValue::Table(table) => {
                let mut result = Vec::with_capacity(table.raw_len());

                for pair in table.pairs::<LuaValue, LuaValue>() {
                    let (key, value) = pair?;

                    result.push((
                        Self::from_lua(key)?,
                        Self::from_lua(value)?
                    ));
                }

                Ok(Self::Table(result))
            }

            _ => Err(LuaError::external("can't coerce given value type"))
        }
    }

    pub fn to_lua<'lua>(&self, lua: &'lua Lua) -> Result<LuaValue<'lua>, LuaError> {
        match self {
            Self::String(value) => lua.create_string(value)
                .map(LuaValue::String),

            Self::Double(value) => Ok(LuaValue::Number(*value)),
            Self::Integer(value) => Ok(LuaValue::Integer(*value)),
            Self::Boolean(value) => Ok(LuaValue::Boolean(*value)),
            Self::Nil => Ok(LuaNil),

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
}

pub struct SyncAPI<'lua> {
    lua: &'lua Lua,

    sync_channel_open: LuaFunction<'lua>,
    sync_channel_send: LuaFunction<'lua>,
    sync_channel_recv: LuaFunction<'lua>,
    sync_channel_close: LuaFunction<'lua>,

    sync_mutex_open: LuaFunction<'lua>,
    sync_mutex_lock: LuaFunction<'lua>,
    sync_mutex_unlock: LuaFunction<'lua>,
    sync_mutex_close: LuaFunction<'lua>
}

impl<'lua> SyncAPI<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, EngineError> {
        let sync_channels_consumers = Arc::new(Mutex::new(HashMap::new())); // key => handle
        let sync_channels_data = Arc::new(Mutex::new(HashMap::new())); // handle => (key, data)

        let sync_mutex_consumers = Arc::new(Mutex::new(HashMap::<u32, Hash>::new())); // handle => key
        let sync_mutex_locks = Arc::new(Mutex::new(HashMap::<Hash, Option<u32>>::new())); // key => curr_lock_handle

        Ok(Self {
            lua,

            sync_channel_open: {
                let sync_channels_consumers = sync_channels_consumers.clone();
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |_, key: LuaString| {
                    let mut listeners = sync_channels_data.lock()
                        .map_err(|err| LuaError::external(format!("failed to register channel listeners: {err}")))?;

                    let key = Hash::for_slice(key.as_bytes());
                    let mut handle = rand::random::<u32>();

                    while listeners.contains_key(&handle) {
                        handle = rand::random::<u32>();
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

                lua.create_function(move |_, (handle, message): (u32, LuaValue<'lua>)| {
                    let message = ChannelMessage::from_lua(message)?;

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
                            if let Some((_, ref mut data)) = listeners.get_mut(consumer) {
                                data.push_back(message.clone());
                            }
                        }
                    }

                    Ok(())
                })?
            },

            sync_channel_recv: {
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |lua, handle: u32| {
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

                lua.create_function(move |_, handle: u32| {
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

                    let mut handle = rand::random::<u32>();

                    while consumers.contains_key(&handle) {
                        handle = rand::random::<u32>();
                    }

                    consumers.insert(handle, key);

                    Ok(handle)
                })?
            },

            sync_mutex_lock: {
                let sync_mutex_consumers = sync_mutex_consumers.clone();
                let sync_mutex_locks = sync_mutex_locks.clone();

                lua.create_function(move |_, handle: u32| {
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

                lua.create_function(move |_, handle: u32| {
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

                lua.create_function(move |_, handle: u32| {
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
            }
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable<'lua>, EngineError> {
        let env = self.lua.create_table_with_capacity(0, 2)?;

        let sync_channel = self.lua.create_table_with_capacity(0, 4)?;
        let sync_mutex = self.lua.create_table_with_capacity(0, 4)?;

        env.set("channel", sync_channel.clone())?;
        env.set("mutex", sync_mutex.clone())?;

        // Channel

        sync_channel.set("open", self.sync_channel_open.clone())?;
        sync_channel.set("send", self.sync_channel_send.clone())?;
        sync_channel.set("recv", self.sync_channel_recv.clone())?;
        sync_channel.set("close", self.sync_channel_close.clone())?;

        // Mutex

        sync_mutex.set("open", self.sync_mutex_open.clone())?;
        sync_mutex.set("lock", self.sync_mutex_lock.clone())?;
        sync_mutex.set("unlock", self.sync_mutex_unlock.clone())?;
        sync_mutex.set("close", self.sync_mutex_close.clone())?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_channels() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = SyncAPI::new(&lua)?;

        assert!(api.sync_channel_send.call::<_, ()>((0, String::new())).is_err());
        assert!(api.sync_channel_recv.call::<_, Option<String>>(0).is_err());

        let a = api.sync_channel_open.call::<_, u32>("test")?;
        let b = api.sync_channel_open.call::<_, u32>("test")?;

        assert_eq!(api.sync_channel_recv.call::<_, Option<String>>(a)?, None);
        assert_eq!(api.sync_channel_recv.call::<_, Option<String>>(b)?, None);

        api.sync_channel_send.call::<_, ()>((a, String::from("Message 1")))?;
        api.sync_channel_send.call::<_, ()>((a, String::from("Message 2")))?;

        let c = api.sync_channel_open.call::<_, u32>("test")?;

        assert_eq!(api.sync_channel_recv.call::<_, Option<String>>(a)?, None);
        assert_eq!(api.sync_channel_recv.call::<_, Option<String>>(c)?, None);
        assert_eq!(api.sync_channel_recv.call::<_, String>(b)?, "Message 1");
        assert_eq!(api.sync_channel_recv.call::<_, String>(b)?, "Message 2");
        assert_eq!(api.sync_channel_recv.call::<_, Option<String>>(b)?, None);

        api.sync_channel_send.call::<_, ()>((a, String::from("Message 3")))?;

        assert_eq!(api.sync_channel_recv.call::<_, Option<String>>(a)?, None);
        assert_eq!(api.sync_channel_recv.call::<_, String>(b)?, "Message 3");
        assert_eq!(api.sync_channel_recv.call::<_, String>(c)?, "Message 3");
        assert_eq!(api.sync_channel_recv.call::<_, Option<String>>(b)?, None);
        assert_eq!(api.sync_channel_recv.call::<_, Option<String>>(c)?, None);

        api.sync_channel_send.call::<_, ()>((a, true))?;
        api.sync_channel_send.call::<_, ()>((a, 0.5))?;
        api.sync_channel_send.call::<_, ()>((a, -17))?;
        api.sync_channel_send.call::<_, ()>((a, vec![1, 2, 3]))?;
        api.sync_channel_send.call::<_, ()>((a, vec!["Hello", "World"]))?;
        api.sync_channel_send.call::<_, ()>((a, vec![vec![1, 2], vec![3, 4]]))?;

        assert_eq!(api.sync_channel_recv.call::<_, Option<_>>(b)?, Some(true));
        assert_eq!(api.sync_channel_recv.call::<_, Option<_>>(b)?, Some(0.5));
        assert_eq!(api.sync_channel_recv.call::<_, Option<_>>(b)?, Some(-17));
        assert_eq!(api.sync_channel_recv.call::<_, Option<_>>(b)?, Some(vec![1, 2, 3]));
        assert_eq!(api.sync_channel_recv.call::<_, Option<_>>(b)?, Some(vec![String::from("Hello"), String::from("World")]));
        assert_eq!(api.sync_channel_recv.call::<_, Option<_>>(b)?, Some(vec![vec![1, 2], vec![3, 4]]));
        assert_eq!(api.sync_channel_recv.call::<_, Option<String>>(b)?, None);

        api.sync_channel_close.call::<_, ()>(a)?;
        api.sync_channel_close.call::<_, ()>(b)?;
        api.sync_channel_close.call::<_, ()>(c)?;

        assert!(api.sync_channel_send.call::<_, ()>((a, String::new())).is_err());
        assert!(api.sync_channel_recv.call::<_, Option<String>>(a).is_err());

        assert!(api.sync_channel_send.call::<_, ()>((b, String::new())).is_err());
        assert!(api.sync_channel_recv.call::<_, Option<String>>(b).is_err());

        assert!(api.sync_channel_send.call::<_, ()>((c, String::new())).is_err());
        assert!(api.sync_channel_recv.call::<_, Option<String>>(c).is_err());

        Ok(())
    }
}

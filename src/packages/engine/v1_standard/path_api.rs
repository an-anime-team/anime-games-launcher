use mlua::prelude::*;

use super::*;

fn normalize_path_parts(parts: &[impl AsRef<str>]) -> Option<Vec<String>> {
    let mut normal_parts = Vec::with_capacity(parts.len());

    let mut i = 0;
    let n = parts.len();

    while i < n {
        let part = parts[i].as_ref();

        if part == "." {
            i += 1;
        }

        else if part == ".." {
            normal_parts.pop()?;

            i += 1;
        }

        else {
            if !["", "/", "\\"].contains(&part) {
                normal_parts.push(part.to_string());
            }

            i += 1;
        }
    }

    if normal_parts.is_empty() {
        None
    } else {
        Some(normal_parts)
    }
}

fn split_path(path: impl AsRef<str>) -> Option<Vec<String>> {
    let path = path.as_ref()
        .replace('\\', "/");

    let raw_parts = path
        .split('/')
        .collect::<Vec<_>>();

    normalize_path_parts(&raw_parts)
}

pub struct PathAPI<'lua> {
    lua: &'lua Lua,

    path_temp_dir: LuaFunctionBuilder<'lua>,
    path_module_dir: LuaFunctionBuilder<'lua>,
    path_persist_dir: LuaFunctionBuilder<'lua>,
    path_normalize: LuaFunction<'lua>,
    path_join: LuaFunction<'lua>,
    path_parts: LuaFunction<'lua>,
    path_parent: LuaFunction<'lua>,
    path_file_name: LuaFunction<'lua>,
    path_exists: LuaFunction<'lua>,
    path_accessible: LuaFunctionBuilder<'lua>
}

impl<'lua> PathAPI<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, PackagesEngineError> {
        Ok(Self {
            lua,

            path_temp_dir: Box::new(|lua: &'lua Lua, context: &Context| {
                let path = context.temp_folder
                    .to_string_lossy()
                    .to_string();

                lua.create_function(move |_, ()| Ok(path.clone()))
            }),

            path_module_dir: Box::new(|lua: &'lua Lua, context: &Context| {
                let path = context.module_folder
                    .to_string_lossy()
                    .to_string();

                lua.create_function(move |_, ()| Ok(path.clone()))
            }),

            path_persist_dir: Box::new(|lua: &'lua Lua, context: &Context| {
                let path = context.persistent_folder.clone();

                lua.create_function(move |_, key: LuaString| {
                    let path = path.join(Hash::for_slice(key.as_bytes()).to_base32())
                        .to_string_lossy()
                        .to_string();

                    Ok(path)
                })
            }),

            path_normalize: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                let (path, is_absolute) = match path.strip_prefix("/") {
                    Some(path) => (path, true),
                    None => (path.as_str(), false)
                };

                match split_path(path) {
                    Some(parts) => {
                        let mut path = parts.join("/");

                        if is_absolute {
                            path = format!("/{path}");
                        }

                        lua.create_string(path)
                            .map(LuaValue::String)
                    }

                    None if is_absolute => lua.create_string("/")
                        .map(LuaValue::String),

                    None => Ok(LuaNil)
                }
            })?,

            path_join: lua.create_function(|lua, parts: LuaMultiValue| {
                if parts.is_empty() {
                    return Ok(LuaNil);
                }

                let parts = parts.iter()
                    .flat_map(|part| part.to_string())
                    .filter(|part| !part.as_bytes().is_empty())
                    .collect::<Vec<_>>();

                let (parts, is_absolute) = match parts.first() {
                    None => return Ok(LuaNil),

                    Some(v) if v == "/" || v == "\\" => (&parts[1..], true),
                    Some(_) => (parts.as_slice(), false)
                };

                let Some(parts) = normalize_path_parts(parts) else {
                    if is_absolute {
                        return lua.create_string("/")
                            .map(LuaValue::String);
                    } else {
                        return Ok(LuaNil);
                    }
                };

                let mut path = parts.join("/");

                if is_absolute {
                    path = format!("/{path}");
                }

                lua.create_string(path)
                    .map(LuaValue::String)
            })?,

            path_parts: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                let path = path.strip_prefix("/")
                    .unwrap_or(&path);

                let Some(parts) = split_path(path) else {
                    return Ok(LuaNil);
                };

                let result = lua.create_table_with_capacity(parts.len(), 0)?;

                for part in parts {
                    result.push(part)?;
                }

                Ok(LuaValue::Table(result))
            })?,

            path_parent: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                let (path, is_absolute) = match path.strip_prefix("/") {
                    Some(path) => (path, true),
                    None => (path.as_str(), false)
                };

                let Some(parts) = split_path(path) else {
                    return Ok(LuaNil);
                };

                if parts.len() > 1 {
                    let mut path = parts[..parts.len() - 1].join("/");

                    if is_absolute {
                        path = format!("/{path}");
                    }

                    lua.create_string(path)
                        .map(LuaValue::String)
                }

                else {
                    Ok(LuaNil)
                }
            })?,

            path_file_name: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                if path == "/" {
                    return lua.create_string("/")
                        .map(LuaValue::String);
                }

                let path = path.strip_prefix("/")
                    .unwrap_or(&path);

                let Some(mut parts) = split_path(path) else {
                    return Ok(LuaNil);
                };

                let Some(file_name) = parts.pop() else {
                    return Ok(LuaNil);
                };

                lua.create_string(file_name)
                    .map(LuaValue::String)
            })?,

            path_exists: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                Ok(path.exists())
            })?,

            path_accessible: Box::new(|lua: &'lua Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |_, path: LuaString| {
                    let path = resolve_path(path.to_string_lossy())?;

                    Ok(context.is_accessible(path))
                })
            })
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable<'lua>, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 10)?;

        env.set("temp_dir", (self.path_temp_dir)(self.lua, context)?)?;
        env.set("module_dir", (self.path_module_dir)(self.lua, context)?)?;
        env.set("persist_dir", (self.path_persist_dir)(self.lua, context)?)?;
        env.set("normalize", self.path_normalize.clone())?;
        env.set("join", self.path_join.clone())?;
        env.set("parts", self.path_parts.clone())?;
        env.set("parent", self.path_parent.clone())?;
        env.set("file_name", self.path_file_name.clone())?;
        env.set("exists", self.path_exists.clone())?;
        env.set("accessible", (self.path_accessible)(self.lua, context)?)?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_actions() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = PathAPI::new(&lua)?;

        assert_eq!(api.path_normalize.call::<_, String>("/")?, "/");
        assert_eq!(api.path_normalize.call::<_, String>("a/b/c")?, "a/b/c");
        assert_eq!(api.path_normalize.call::<_, String>("/a/b/c")?, "/a/b/c");
        assert_eq!(api.path_normalize.call::<_, String>("a/./c")?, "a/c");
        assert_eq!(api.path_normalize.call::<_, String>("a/../c")?, "c");
        assert_eq!(api.path_normalize.call::<_, String>("a/../c/./")?, "c");
        assert_eq!(api.path_normalize.call::<_, String>("./a//\\./../b")?, "b");
        assert_eq!(api.path_normalize.call::<_, String>(" ")?, " "); // space is a correct entry name
        assert_eq!(api.path_normalize.call::<_, Option<String>>("")?, None); // entry name cannot be empty
        assert_eq!(api.path_normalize.call::<_, Option<String>>(".")?, None); // we do not support relative paths
        assert_eq!(api.path_normalize.call::<_, Option<String>>("..")?, None);
        assert_eq!(api.path_normalize.call::<_, Option<String>>("./..")?, None);
        assert_eq!(api.path_normalize.call::<_, Option<String>>("a/..")?, None);

        assert_eq!(api.path_join.call::<_, String>(("a", "b", "c"))?, "a/b/c");
        assert_eq!(api.path_join.call::<_, String>(("/", "a", "b", "c"))?, "/a/b/c");
        assert_eq!(api.path_join.call::<_, String>(("a", "..", "b"))?, "b");
        assert_eq!(api.path_join.call::<_, String>((".", "a", ".", "b"))?, "a/b");
        assert_eq!(api.path_join.call::<_, Option<String>>("")?, None);
        assert_eq!(api.path_join.call::<_, Option<String>>(".")?, None);
        assert_eq!(api.path_join.call::<_, Option<String>>("..")?, None);
        assert_eq!(api.path_join.call::<_, Option<String>>((".", ".."))?, None);
        assert_eq!(api.path_join.call::<_, Option<String>>(("a", ".."))?, None);

        assert_eq!(api.path_parts.call::<_, Vec<String>>("a/b/c")?, &["a", "b", "c"]);
        assert_eq!(api.path_parts.call::<_, Vec<String>>("a/./c")?, &["a", "c"]);
        assert_eq!(api.path_parts.call::<_, Vec<String>>("a/./c/..")?, &["a"]);
        assert_eq!(api.path_parts.call::<_, Vec<String>>("\\a/b/// /c")?, &["a", "b", " ", "c"]);
        assert_eq!(api.path_parts.call::<_, Option<Vec<String>>>("")?, None);
        assert_eq!(api.path_parts.call::<_, Option<Vec<String>>>(".")?, None);
        assert_eq!(api.path_parts.call::<_, Option<Vec<String>>>("..")?, None);
        assert_eq!(api.path_parts.call::<_, Option<Vec<String>>>("./..")?, None);
        assert_eq!(api.path_parts.call::<_, Option<Vec<String>>>("a/..")?, None);

        assert_eq!(api.path_parent.call::<_, String>("a/b/c")?, "a/b");
        assert_eq!(api.path_parent.call::<_, String>("/a/b/c")?, "/a/b");
        assert_eq!(api.path_parent.call::<_, String>("a\\./b")?, "a");
        assert_eq!(api.path_parent.call::<_, Option<Vec<String>>>("a")?, None);
        assert_eq!(api.path_parent.call::<_, Option<Vec<String>>>("a/.")?, None);
        assert_eq!(api.path_parent.call::<_, Option<Vec<String>>>("a/../b")?, None);

        assert_eq!(api.path_file_name.call::<_, String>("/")?, "/");
        assert_eq!(api.path_file_name.call::<_, String>("a")?, "a");
        assert_eq!(api.path_file_name.call::<_, String>("a/b/c")?, "c");
        assert_eq!(api.path_file_name.call::<_, String>("/a/b/c")?, "c");
        assert_eq!(api.path_file_name.call::<_, String>("a\\./b")?, "b");
        assert_eq!(api.path_file_name.call::<_, Option<Vec<String>>>(".")?, None);
        assert_eq!(api.path_file_name.call::<_, Option<Vec<String>>>("a/..")?, None);

        Ok(())
    }
}

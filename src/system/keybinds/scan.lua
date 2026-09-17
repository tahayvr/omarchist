-- Omarchist keybind scanner.
--
-- Evaluates the user's ~/.config/hypr/hyprland.lua (path in arg[1]) with a
-- stubbed `hl` table, the same technique Omarchy's own
-- `omarchy-menu-keybindings` uses, and prints one tab-separated record per
-- hl.bind / hl.unbind call in registration order:
--
--   bind   <seq> <source> <keys> <description> <kind> <arg> <flags>
--   unbind <seq> <source> <keys>
--   error  <seq> <message>
--   done   <seq>
--
-- Fields are escaped (\\ \t \n \r). `kind` is `exec` (shell command), `lua`
-- (an hl.dsp.* expression reconstructed as source text) or `fn` (an opaque
-- Lua function). `source` is the Lua file that declared the bind, skipping
-- Omarchy's helpers.lua wrapper frames. `flags` is a comma-separated list of
-- the boolean bind options that are set.

local config_path = arg and arg[1] or ((os.getenv("HOME") or "") .. "/.config/hypr/hyprland.lua")
local seq = 0

local option_names = {
  "locked",
  "repeating",
  "release",
  "mouse",
  "non_consuming",
  "transparent",
  "ignore_mods",
  "long_press",
  "submap_universal",
}

local function esc(value)
  local text = tostring(value == nil and "" or value)
  text = text:gsub("\\", "\\\\"):gsub("\t", "\\t"):gsub("\n", "\\n"):gsub("\r", "\\r")
  return text
end

local function emit(fields)
  local out = {}
  for index, field in ipairs(fields) do
    out[index] = esc(field)
  end
  print(table.concat(out, "\t"))
end

-- The file whose code called hl.bind/hl.unbind. Level 1 is the stub itself;
-- Omarchy's o.bind/o.rebind live in helpers.lua and are skipped so the bind
-- is attributed to the bindings file that invoked them.
local function caller_source()
  for level = 2, 40 do
    local info = debug.getinfo(level, "S")
    if not info then
      break
    end
    local source = info.source or ""
    if source:sub(1, 1) == "@" and not source:find("/default/hypr/helpers.lua", 1, true) then
      return source:sub(2)
    end
  end
  return ""
end

local function lua_literal(value)
  local value_type = type(value)

  if value_type == "string" then
    return string.format("%q", value)
  elseif value_type == "number" or value_type == "boolean" then
    return tostring(value)
  elseif value_type == "table" then
    local parts = {}
    local keys = {}
    local array_length = #value

    for index = 1, array_length do
      parts[#parts + 1] = lua_literal(value[index])
    end

    for key in pairs(value) do
      if not (type(key) == "number" and key >= 1 and key <= array_length and math.floor(key) == key) then
        keys[#keys + 1] = key
      end
    end

    table.sort(keys, function(left, right)
      return tostring(left) < tostring(right)
    end)

    for _, key in ipairs(keys) do
      local key_prefix
      if type(key) == "string" and key:match("^[%a_][%w_]*$") then
        key_prefix = key .. " = "
      else
        key_prefix = "[" .. lua_literal(key) .. "] = "
      end

      parts[#parts + 1] = key_prefix .. lua_literal(value[key])
    end

    return "{ " .. table.concat(parts, ", ") .. " }"
  end

  return "nil"
end

local function call_expression(path, ...)
  local args = {}

  for index = 1, select("#", ...) do
    args[index] = lua_literal(select(index, ...))
  end

  return path .. "(" .. table.concat(args, ", ") .. ")"
end

local function dispatcher(kind, arg, expr)
  return {
    __omarchist_dispatcher = true,
    kind = kind or "",
    arg = arg or "",
    expr = expr or "",
  }
end

-- `hl.dsp.window.close()` and friends are reconstructed as source text so
-- they can be re-emitted verbatim under a different chord.
local function dsp_proxy(path)
  return setmetatable({ path = path }, {
    __index = function(self, key)
      return dsp_proxy(self.path .. "." .. tostring(key))
    end,
    __call = function(self, ...)
      local first_arg = ...
      local expr = call_expression(self.path, ...)

      if self.path == "hl.dsp.exec_cmd" and type(first_arg) == "string" and select("#", ...) == 1 then
        return dispatcher("exec", first_arg, expr)
      end

      return dispatcher("lua", expr, expr)
    end,
  })
end

local noop
noop = setmetatable({}, {
  __index = function()
    return noop
  end,
  __call = function()
    return noop
  end,
})

local function classify(bind_dispatcher)
  local kind = type(bind_dispatcher)

  if kind == "string" then
    return "exec", bind_dispatcher
  elseif kind == "table" and bind_dispatcher.__omarchist_dispatcher then
    return bind_dispatcher.kind, bind_dispatcher.arg
  elseif kind == "function" then
    return "fn", ""
  end

  return "lua", tostring(bind_dispatcher)
end

hl = setmetatable({
  dsp = dsp_proxy("hl.dsp"),
  bind = function(keys, bind_dispatcher, opts)
    seq = seq + 1
    opts = opts or {}

    local kind, arg = classify(bind_dispatcher)
    local flags = {}
    for _, name in ipairs(option_names) do
      if opts[name] then
        flags[#flags + 1] = name
      end
    end

    emit({
      "bind",
      seq,
      caller_source(),
      keys,
      opts.description or opts.desc or "",
      kind,
      arg,
      table.concat(flags, ","),
    })

    return noop
  end,
  unbind = function(keys)
    seq = seq + 1
    emit({ "unbind", seq, caller_source(), keys })
  end,
}, {
  -- Getters (hl.get_config, hl.get_active_monitor, ...) answer nil, as they
  -- do in Hyprland when nothing matches, so config code that inspects the
  -- result takes its "absent" branch instead of comparing a table with a
  -- number. Everything else (hl.config, hl.on, hl.env, hl.dsp...) is a noop.
  __index = function(_, key)
    if type(key) == "string" and key:sub(1, 4) == "get_" then
      return function()
        return nil
      end
    end
    return noop
  end,
})

local file = io.open(config_path, "r")
if file then
  file:close()
  local ok, err = pcall(dofile, config_path)
  if not ok then
    emit({ "error", seq, tostring(err) })
  end
else
  emit({ "error", seq, "Cannot open " .. tostring(config_path) })
end

emit({ "done", seq })

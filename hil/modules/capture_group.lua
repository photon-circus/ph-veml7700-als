local M = {
  meta = {
    name = "capture_group",
    claims = { "dut", "logic" },
    timeout = "45s",
    params = {
      command = { type = "string" },
      capability = { type = "string" },
      expected_cases = { type = "string_list" },
      artifact = { type = "string" },
      samplerate = { type = "quantity", dimension = "frequency" },
      channels = { type = "int_list" },
      duration = { type = "duration" },
    },
  },
}

function M.run(ctx)
  local capture = logic.start_capture({
    samplerate = q(ctx.params.samplerate),
    channels = ctx.params.channels,
  }, ctx.params.duration)
  local reply = dut.cmd(ctx.params.command, "40s")
  local retained = capture:finish()
  ctx.store_file(ctx.params.artifact, retained.bytes, retained.meta)
  if not reply.ok then error("captured DUT command failed: " .. (reply.code or "unknown")) end
  if reply.payload == nil or string.sub(reply.payload, 1, 5) ~= "json:" then
    error("captured DUT command returned malformed evidence")
  end
  local evidence = json.decode(string.sub(reply.payload, 6))
  if evidence.schema ~= 1 or evidence.capability ~= ctx.params.capability then
    error("captured evidence did not match schema/capability")
  end
  if #evidence.cases ~= #ctx.params.expected_cases then
    error("captured case inventory length did not match plan")
  end
  for index, expected in ipairs(ctx.params.expected_cases) do
    local seen = evidence.cases[index]
    if type(seen) ~= "table" or seen.case ~= expected then
      error("captured case inventory did not match plan")
    end
    if seen.outcome == "fail" then
      ctx.store_json("evidence", evidence)
      return verdict.fail(seen.case .. " failed")
    elseif seen.outcome ~= "pass" and seen.outcome ~= "skip" then
      error("captured evidence used invalid outcome")
    end
  end
  ctx.store_json("evidence", evidence)
  return verdict.pass()
end

return M

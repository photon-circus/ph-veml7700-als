local M = {
  meta = {
    name = "run_group",
    claims = { "dut" },
    timeout = "45s",
    params = {
      command = { type = "string" },
      capability = { type = "string" },
      expected_cases = { type = "string_list" },
    },
  },
}

local function decode_reply(reply)
  if not reply.ok then error("DUT command failed: " .. (reply.code or "unknown")) end
  if reply.payload == nil or string.sub(reply.payload, 1, 5) ~= "json:" then
    error("DUT command returned a malformed terminal payload")
  end
  local evidence = json.decode(string.sub(reply.payload, 6))
  if evidence.schema ~= 1 or type(evidence.capability) ~= "string" or type(evidence.cases) ~= "table" then
    error("DUT command returned malformed schema-1 evidence")
  end
  return evidence
end

function M.run(ctx)
  local evidence = decode_reply(dut.cmd(ctx.params.command, "40s"))
  if evidence.capability ~= ctx.params.capability then
    error("DUT evidence capability did not match requested command")
  end
  if #evidence.cases ~= #ctx.params.expected_cases then
    error("DUT evidence case inventory length did not match plan")
  end
  local all_skipped = true
  for index, expected in ipairs(ctx.params.expected_cases) do
    local seen = evidence.cases[index]
    if type(seen) ~= "table" or seen.case ~= expected then
      error("DUT evidence case inventory did not match plan")
    end
    if seen.outcome == "fail" then
      ctx.store_json("evidence", evidence)
      return verdict.fail(seen.case .. " failed")
    elseif seen.outcome ~= "skip" and seen.outcome ~= "pass" then
      error("DUT evidence used an invalid outcome")
    end
    if seen.outcome ~= "skip" then all_skipped = false end
  end
  ctx.store_json("evidence", evidence)
  if all_skipped then return verdict.skip("all cases were skipped") end
  return verdict.pass()
end

return M

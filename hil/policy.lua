-- VEML7700-owned offline policy. Firmware and operators never assign confidence.
return function(ctx)
  local groups = {
    { "fixture", "run_group", "veml7700.fixture@1" },
    { "identity", "capture_group", "veml7700.identity@1" },
    { "config", "capture_group", "veml7700.config@1" },
    { "measurement", "capture_group", "veml7700.measurement@1" },
    { "scaling", "run_group", "veml7700.scaling@1" },
    { "power", "capture_group", "veml7700.power@1" },
    { "cadence", "capture_group", "veml7700.cadence@1" },
    { "threshold", "capture_group", "veml7700.threshold@1" },
    { "white", "run_group", "veml7700.white@1" },
    { "recovery_interrupt", "run_group", "veml7700.recovery@1" },
    { "recovery_verify", "run_group", "veml7700.recovery@1" },
  }

  local provenance = ctx.manifest.adapter_provenance or {}
  local dut_provenance = provenance.dut or {}
  local logic_provenance = provenance.logic or {}
  local is_mock = dut_provenance.transcript_sha256 ~= nil
  local assessed = {}
  local unsupportable = {}
  local aggregate = "pass"

  ctx.decode_i2c("identity", "identity-capture")
  ctx.decode_i2c("config", "config-capture")
  ctx.decode_i2c("measurement", "measurement-capture")
  ctx.decode_i2c("power", "power-capture")
  ctx.decode_i2c("cadence", "cadence-capture")
  ctx.decode_i2c("threshold", "threshold-capture")

  for _, group in ipairs(groups) do
    local evidence = ctx.artifact(group[1], group[2], "evidence.json")
    if evidence.schema ~= 1 or evidence.capability ~= group[3] or type(evidence.cases) ~= "table" then
      error("malformed evidence artifact for " .. group[1])
    end
    for _, item in ipairs(evidence.cases) do
      local outcome = item.outcome
      if outcome ~= "pass" and outcome ~= "fail" and outcome ~= "skip" then outcome = "error" end
      if outcome == "fail" then aggregate = "fail" end
      if outcome == "error" then aggregate = "error" end

      local confidence = "direct"
      local reason = nil
      if is_mock then
        confidence = "void"
        reason = "deterministic mock evidence cannot support a hardware capability"
      elseif item.source == "attested" then
        confidence = "qualified"
        reason = "operator-attested fixture condition"
      elseif item.source == "instrument" and
          (logic_provenance.identity == nil or logic_provenance.calibration == nil) then
        confidence = "qualified"
        reason = "instrument identity or calibration state was not retained"
      elseif item.witness == "repeatable-light-only" then
        confidence = "qualified"
        reason = "relative optical source was not a calibrated lux reference"
      elseif item.witness == "none" then
        confidence = "void"
        reason = "case has no retained witness"
      end

      local case_id = string.gsub(item.case, "[^a-z0-9_-]", "-")
      local supported = {}
      if outcome == "pass" and confidence ~= "void" then supported = { evidence.capability } end
      if confidence == "void" then table.insert(unsupportable, case_id) end
      table.insert(assessed, {
        case = case_id,
        outcome = outcome,
        confidence = confidence,
        qualification_reason = reason,
        capabilities = supported,
      })
    end
  end

  return {
    schema = 1,
    assessment = aggregate,
    cases = assessed,
    unsupportable_cases = unsupportable,
  }
end

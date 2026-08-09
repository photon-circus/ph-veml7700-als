-- TEMPLATE: finalize after source and calibrated-reference evidence schemas are frozen.
-- Required rules:
-- * mock evidence is void;
-- * missing reference identity or calibration state is void;
-- * missing fixture geometry/window/diffuser/source metadata is void;
-- * commanded LED level without paired reference lux is void;
-- * unstable source or unrecorded dark baseline is qualified or void;
-- * only pass + non-void cases support veml7700.optical@1.
return function(_ctx)
  error("optical policy is a template until the calibrated fixture is characterized")
end

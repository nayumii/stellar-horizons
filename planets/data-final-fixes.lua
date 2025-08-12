local Tech = require('__kry_stdlib__/stdlib/data/technology')

-- wood_ammo is true if lignumis enabled and ammo progression is on
local wood_ammo = mods["lignumis"] and settings.startup["lignumis-ammo-progression"].value

-- remove astroponics from critical tech path if wood is not needed for ammo
if mods["astroponics"] and not wood_ammo then
    if mods["planet-muluna"] then
        -- Asteroid Collector is the Muluna planet discovery prerequisite
        -- unlink from astroponics, then link directly to space science
        Tech("asteroid-collector"):remove_prereq("astroponics")
        Tech("asteroid-collector"):add_prereq("space-science-pack")
    else
        -- Space Platform Thruster is the vanilla planet discovery prerequisite
        -- unlink from astroponics, then link directly to space science
        Tech("space-platform-thruster"):remove_prereq("astroponics")
        Tech("space-platform-thruster"):add_prereq("space-science-pack")
    end
    -- gleba astronponics is the only sensible future tech to require astroponics
    Tech("gleba-astroponics"):add_prereq("astroponics")
end

//! A deliberately narrow, validated adapter for embedded Pasu-style scenarios.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io::Read, path::Path};

pub const LIMIT: usize = 2 * 1024 * 1024;
pub const UNIT: f32 = 0.00375;
pub const LIMITS: &str = "Approximation: local physics, 2D dodge, gravity base 980, continuous Knocker forces and safe bounds. Map brushes, source visuals/sounds, FOV scale and spawn exclusion are not reproduced. No KovaaK leaderboard compatibility.";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub motion: Motion,
    pub spawns: Vec<[f32; 3]>,
    pub knockers: Vec<Knocker>,
    pub radius: f32,
    pub shot_interval: f32,
    pub respawn: f32,
    pub kill_score: f32,
    pub sqrt_accuracy: bool,
    pub accuracy_multiplier: bool,
    pub fov: [f32; 2],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Motion {
    pub speed: f32,
    pub acceleration: f32,
    pub jump: f32,
    pub air_jump: f32,
    pub air_jumps: u32,
    pub gravity: f32,
    pub terminal: f32,
    pub friction: f32,
    pub turn: [f32; 2],
    pub jump_time: [f32; 2],
    pub jump_chance: f32,
    pub swap_pause: [f32; 2],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Knocker {
    pub position: [f32; 3],
    pub radius: f32,
    pub horizontal: f32,
    pub vertical: f32,
    pub interval: f32,
}
fn bounded(v: f32, lo: f32, hi: f32, label: &str) -> Result<f32, String> {
    if v.is_finite() && (lo..=hi).contains(&v) {
        Ok(v)
    } else {
        Err(format!("Invalid {label}: expected {lo}..{hi}"))
    }
}
impl Scenario {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty()
            || self.name.len() > 80
            || self.name.chars().any(char::is_control)
            || self.id.len() != 32
            || !self.id.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("Invalid scenario identity".into());
        }
        bounded(self.radius, 0.05, 1.0, "radius")?;
        bounded(self.shot_interval, 0.02, 1.0, "shot interval")?;
        bounded(self.respawn, 0.0, 2.0, "respawn delay")?;
        bounded(self.kill_score, 0.0, 100.0, "kill score")?;
        bounded(self.fov[0], 60.0, 140.0, "minimum FOV")?;
        bounded(self.fov[1], self.fov[0], 140.0, "maximum FOV")?;
        let m = &self.motion;
        for (v, label) in [
            (m.speed, "speed"),
            (m.acceleration, "acceleration"),
            (m.jump, "jump"),
            (m.air_jump, "air jump"),
            (m.gravity, "gravity"),
            (m.terminal, "terminal velocity"),
        ] {
            bounded(v, 0.001, 500.0, label)?;
        }
        bounded(m.friction, 0.0, 100.0, "air friction")?;
        bounded(m.jump_chance, 0.0, 1.0, "jump chance")?;
        if m.air_jumps > 10000 {
            return Err("Too many air jumps".into());
        }
        for (range, lo, label) in [
            (m.turn, 0.05, "dodge interval"),
            (m.jump_time, 0.05, "jump interval"),
            (m.swap_pause, 0.0, "strafe pause"),
        ] {
            bounded(range[0], lo, 10.0, label)?;
            bounded(range[1], range[0], 10.0, label)?;
        }
        if self.spawns.len() < 4 || self.spawns.len() > 1024 || self.knockers.len() != 8 {
            return Err("Requires 4+ spawns and 8 Knocker placements".into());
        }
        for p in &self.spawns {
            bounded(p[0], -16.0, 16.0, "spawn x")?;
            bounded(p[1], -6.0, 12.0, "spawn height")?;
            bounded(p[2], -20.0, -4.0, "spawn depth")?;
        }
        for k in &self.knockers {
            for v in k.position {
                bounded(v, -100.0, 100.0, "Knocker position")?;
            }
            bounded(k.radius, 0.1, 50.0, "Knocker radius")?;
            bounded(k.horizontal, 0.0, 10.0, "horizontal impulse")?;
            bounded(k.vertical, -10.0, 10.0, "vertical impulse")?;
            bounded(k.interval, 0.01, 1.0, "Knocker interval")?;
        }
        Ok(())
    }
    pub fn score(&self, hits: u32, shots: u32) -> f64 {
        let accuracy = if shots == 0 {
            0.0
        } else {
            f64::from(hits) / f64::from(shots)
        };
        let multiplier = if !self.accuracy_multiplier {
            1.0
        } else if self.sqrt_accuracy {
            accuracy.sqrt()
        } else {
            accuracy
        };
        f64::from(hits) * f64::from(self.kill_score) * multiplier
    }
}

#[derive(Default)]
struct Section {
    kind: String,
    fields: BTreeMap<String, String>,
}
impl Section {
    fn get(&self, key: &str) -> Result<&str, String> {
        self.fields
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("Missing {}.{key}", self.kind))
    }
    fn number(&self, key: &str, lo: f32, hi: f32) -> Result<f32, String> {
        let value = self
            .get(key)?
            .parse::<f32>()
            .map_err(|_| format!("Invalid number: {key}"))?;
        bounded(value, lo, hi, key)
    }
    fn flag(&self, key: &str) -> Result<bool, String> {
        self.get(key)?
            .parse()
            .map_err(|_| format!("Invalid boolean: {key}"))
    }
    fn require(&self, key: &str, value: &str) -> Result<(), String> {
        if self.get(key)? == value {
            Ok(())
        } else {
            Err(format!("Unsupported {}.{key}; requires {value}", self.kind))
        }
    }
    fn zero_if_present(&self, key: &str) -> Result<(), String> {
        if self.fields.contains_key(key) {
            self.number(key, 0.0, 0.0)?;
        }
        Ok(())
    }
    fn false_if_present(&self, key: &str) -> Result<(), String> {
        if self.fields.contains_key(key) {
            self.require(key, "false")?;
        }
        Ok(())
    }
}
fn reference<'a>(sections: &'a [Section], kind: &str, name: &str) -> Result<&'a Section, String> {
    let name = name.trim_end_matches(".bot").trim_end_matches(".abilmelee");
    sections
        .iter()
        .find(|s| {
            s.kind == kind
                && s.fields
                    .get("Name")
                    .is_some_and(|v| v.eq_ignore_ascii_case(name))
        })
        .ok_or_else(|| format!("Unresolved {kind}: {name}"))
}
fn vector(text: &str) -> Result<[f32; 3], String> {
    let values = text
        .split(',')
        .map(|v| {
            v.trim()
                .parse::<f32>()
                .map_err(|_| "Invalid map coordinate".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result: [f32; 3] = values
        .try_into()
        .map_err(|_| "Map coordinate needs three values")?;
    for v in result {
        bounded(v, -100000.0, 100000.0, "map coordinate")?;
    }
    Ok(result)
}
#[derive(Deserialize)]
struct Map {
    objects: Vec<MapObject>,
}
#[derive(Deserialize)]
struct MapObject {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    location: String,
    #[serde(default)]
    properties: Vec<Property>,
}
#[derive(Deserialize)]
struct Property {
    name: String,
    value: serde_json::Value,
}
impl MapObject {
    fn property(&self, key: &str) -> Result<&serde_json::Value, String> {
        let mut found = self.properties.iter().filter(|p| p.name == key);
        let value = &found
            .next()
            .ok_or_else(|| format!("Missing map property {key}"))?
            .value;
        if found.next().is_some() {
            return Err(format!("Duplicate map property {key}"));
        }
        Ok(value)
    }
    fn text(&self, key: &str) -> Result<&str, String> {
        self.property(key)?
            .as_str()
            .ok_or_else(|| format!("Invalid map text {key}"))
    }
}

pub fn load(path: &Path) -> Result<Scenario, String> {
    if path.extension().and_then(|s| s.to_str()) != Some("sce") {
        return Err("Choose a .sce file".into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| format!("Cannot open {}: {e}", path.display()))?
        .take(2_097_153)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "Scenario must be UTF-8 text")?;
    parse(text)
}
pub fn parse(text: &str) -> Result<Scenario, String> {
    if text.len() > LIMIT {
        return Err("Scenario exceeds 2 MiB".into());
    }
    let (profiles, map) = text
        .split_once("[Map Data]")
        .ok_or("Missing embedded Map Data")?;
    let mut sections = vec![Section {
        kind: "Scenario".into(),
        ..Section::default()
    }];
    for (i, line) in profiles.lines().enumerate() {
        let line = line.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let kind = &line[1..line.len() - 1];
            if ![
                "Aim Profile",
                "Bot Profile",
                "Character Profile",
                "Dodge Profile",
                "Weapon Profile",
                "Melee Ability Profile",
            ]
            .contains(&kind)
            {
                return Err(format!("Unsupported profile {kind}"));
            }
            if sections.len() >= 64 {
                return Err("Too many profiles".into());
            }
            sections.push(Section {
                kind: kind.into(),
                ..Section::default()
            });
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("Invalid line {}", i + 1))?;
        let section = sections.last_mut().ok_or("Missing section")?;
        if section
            .fields
            .insert(key.trim().into(), value.trim().into())
            .is_some()
        {
            return Err(format!("Duplicate field {key}"));
        }
    }
    for (i, s) in sections.iter().enumerate().skip(1) {
        let name = s.get("Name")?;
        if sections[1..i].iter().any(|other| {
            other.kind == s.kind
                && other
                    .fields
                    .get("Name")
                    .is_some_and(|n| n.eq_ignore_ascii_case(name))
        }) {
            return Err(format!("Duplicate profile {name}"));
        }
    }
    let root = &sections[0];
    root.number("Timelimit", 60.0, 60.0)?;
    root.number("Timescale", 1.0, 1.0)?;
    root.require("IsChallenge", "true")?;
    root.require("InvinciblePlayer", "true")?;
    root.require("InvincibleBots", "false")?;
    root.require("MapName", "Wall-less Wall.json")?;
    for (k, _) in &root.fields {
        if (k.starts_with("ScorePer") && k != "ScorePerKill")
            || k.starts_with("ScoreLoss")
            || [
                "TimeRefilledByKill",
                "EndChallengeAfterKills",
                "EndChallengeAfterDamage",
            ]
            .contains(&k.as_str())
        {
            root.zero_if_present(k)?;
        }
    }
    for key in [
        "ScoreMultDamageEfficiency",
        "ScoreMultKillEfficiency",
        "MBSEnable",
        "IsTimeDilationActive",
        "IsTargetSizeActive",
        "IsHeightLocked",
        "EnableOverDamage",
    ] {
        root.false_if_present(key)?;
    }
    root.require("PlayerTeam", "1")?;
    let player = reference(&sections, "Character Profile", root.get("PlayerProfile")?)?;
    player.number("MaxSpeed", 0.0, 0.0)?;
    let weapon_names: Vec<_> = player
        .get("WeaponProfileNames")?
        .split(';')
        .filter(|v| !v.is_empty())
        .collect();
    if weapon_names.len() != 1 {
        return Err("Requires one player weapon".into());
    }
    let weapon = reference(&sections, "Weapon Profile", weapon_names[0])?;
    for (key, value) in [
        ("Type", "Hitscan"),
        ("Category", "SemiAuto"),
        ("ShotsPerClick", "1"),
        ("CooldownType", "InfiniteUse"),
    ] {
        weapon.require(key, value)?;
    }
    for key in [
        "Explosive",
        "Pierces",
        "FullyAutomatic",
        "IsChargeWeapon",
        "IsBurstWeapon",
        "TriggerBotEnabled",
        "CanAimDownSight",
        "HeadshotCapable",
        "UsePerShotRecoil",
        "UsePerBulletSpread",
    ] {
        weapon.false_if_present(key)?;
    }
    for key in [
        "DelayBeforeShot",
        "MaxRecoilUp",
        "MinRecoilUp",
        "MinRecoilHoriz",
        "MaxRecoilHoriz",
        "AAMode",
        "HitscanRadius",
    ] {
        weapon.zero_if_present(key)?;
    }
    let bots: Vec<_> = root.get("AddedBots")?.split(';').collect();
    let teams: Vec<_> = root.get("BotTeams")?.split(';').collect();
    let lives: Vec<_> = root.get("BotMaxLives")?.split(';').collect();
    if bots.len() != 12 || teams.len() != 12 || lives.len() != 12 || lives.iter().any(|v| *v != "0")
    {
        return Err("Requires four targets and eight unlimited-life Knockers".into());
    }
    let mut target = None;
    let mut count = 0;
    let mut knocker_profiles = BTreeMap::new();
    for (name, team) in bots.iter().zip(teams) {
        let bot = reference(&sections, "Bot Profile", name)?;
        let character = reference(&sections, "Character Profile", bot.get("CharacterProfile")?)?;
        if team == "2" && !bot.flag("Untargetable")? {
            bot.require("UseWeapons", "false")?;
            bot.require("NoDodging", "false")?;
            bot.false_if_present("DisableScoring")?;
            if target.is_some_and(|(previous, _)| previous != character.get("Name").unwrap_or("")) {
                return Err("Mixed target characters are unsupported".into());
            }
            target = Some((character.get("Name")?, bot));
            count += 1;
        } else if team == "1" && bot.flag("Untargetable")? {
            bot.require("NoDodging", "true")?;
            character.number("MaxSpeed", 0.0, 0.0)?;
            let abilities: Vec<_> = character
                .get("AbilityProfileNames")?
                .split(';')
                .filter(|v| !v.is_empty())
                .collect();
            if abilities.len() != 1 {
                return Err("Each Knocker needs one melee ability".into());
            }
            let ability = reference(&sections, "Melee Ability Profile", abilities[0])?;
            ability.number("HurtboxDamage", 0.0, 0.0)?;
            let entry = knocker_profiles
                .entry(character.get("Name")?.to_string())
                .or_insert((ability, 0usize));
            entry.1 += 1;
        } else {
            return Err("Unsupported bot team or target role".into());
        }
    }
    if count != 4 {
        return Err("Requires four hittable targets".into());
    }
    let (name, bot) = target.ok_or("Missing target")?;
    let character = reference(&sections, "Character Profile", name)?;
    for key in [
        "HeadshotOnly",
        "MainBBHasHead",
        "HasJetpack",
        "IsFlyer",
        "MeshHitDetection",
        "InvincibleBots",
        "EnableQuakeMovement",
    ] {
        character.false_if_present(key)?;
    }
    character.require("MainBBType", "Spheroid")?;
    character.require("CanPogoJump", "true")?;
    character.require("DisableCharacterCollision", "true")?;
    character.number("AirControl", 1.0, 1.0)?;
    if character
        .get("AbilityProfileNames")?
        .split(';')
        .any(|s| !s.is_empty())
    {
        return Err("Target abilities unsupported".into());
    }
    let radius = character.number("MainBBRadius", 1.0, 300.0)?;
    character.number("MainBBHeight", radius * 2.0, radius * 2.0)?;
    let health = character.number("MaxHealth", 0.01, 100.0)?;
    weapon.number("DamagePerShot", health, health)?;
    let dodge = reference(&sections, "Dodge Profile", bot.get("DodgeProfileNames")?)?;
    dodge.require("ToggleLeftRight", "true")?;
    dodge.require("WaypointLogic", "Ignore")?;
    for key in [
        "DamageReactionChangesDirection",
        "DamageReactionChangesFB",
        "DamageReactionTriggersProfileChange",
        "LOSReactKillBot",
    ] {
        dodge.false_if_present(key)?;
    }
    for key in ["CrouchInAirFrequency", "CrouchOnGroundFrequency"] {
        dodge.zero_if_present(key)?;
    }
    let interval = |s: &Section, min: &str, max: &str| -> Result<[f32; 2], String> {
        let lo = s.number(min, 0.0, 10.0)?;
        Ok([lo, s.number(max, lo, 10.0)?])
    };
    let respawn = character.number("MinRespawnDelay", 0.0, 2.0)?;
    character.number("MaxRespawnDelay", respawn, respawn)?;
    let map: Map = serde_json::from_str(map).map_err(|e| format!("Invalid map JSON: {e}"))?;
    if map.objects.len() > 4096 {
        return Err("Too many map objects".into());
    }
    let spawn_objects: Vec<_> = map
        .objects
        .iter()
        .filter(|o| o.kind == "gameObject" && o.name == "SpawnPoint")
        .collect();
    if map
        .objects
        .iter()
        .any(|o| o.kind != "brush" && !(o.kind == "gameObject" && o.name == "SpawnPoint"))
    {
        return Err("Unsupported map object".into());
    }
    let player_spawns = spawn_objects
        .iter()
        .filter(|o| {
            o.text("PermittedCharacterProfiles")
                .is_ok_and(|v| v == player.get("Name").unwrap_or(""))
        })
        .collect::<Vec<_>>();
    if player_spawns.len() != 1 {
        return Err("Requires one player spawn".into());
    }
    let origin = vector(&player_spawns[0].location)?;
    let scale = root.number("MapScale", 0.1, 10.0)? * UNIT;
    let convert = |v: [f32; 3], offset: f32| -> [f32; 3] {
        [
            (v[1] - origin[1]) * scale,
            (v[2] - origin[2]) * scale + 2.6 + offset,
            (origin[0] - v[0]) * scale + 2.0,
        ]
    };
    let mut spawns = Vec::new();
    let mut knockers = Vec::new();
    for o in spawn_objects {
        let p = vector(&o.location)?;
        if o.text("Path")? != "" {
            return Err("Spawn paths unsupported".into());
        }
        let permitted = o.text("PermittedCharacterProfiles")?;
        let team = o
            .property("TeamMask")?
            .as_u64()
            .ok_or("Invalid spawn team")?;
        if team == 2 && (permitted.is_empty() || permitted.eq_ignore_ascii_case(name)) {
            spawns.push(convert(p, -31.0 * UNIT));
        } else if let Some((ability, expected)) = knocker_profiles.get_mut(permitted) {
            if team != 1 || *expected == 0 {
                return Err("Invalid Knocker placement count/team".into());
            }
            *expected -= 1;
            knockers.push(Knocker {
                position: convert(p, 0.0),
                radius: ability.number("HurtboxRadius", 1.0, 13000.0)? * UNIT,
                horizontal: ability.number("FlatKnockbackHorizontal", 0.0, 2600.0)? * UNIT,
                vertical: ability.number("FlatKnockbackVertical", -2600.0, 2600.0)? * UNIT,
                interval: ability.number("ChargeTimer", 0.01, 1.0)?,
            });
        } else if permitted != player.get("Name")? {
            return Err(format!("Unresolved spawn character: {permitted}"));
        }
    }
    if knocker_profiles
        .values()
        .any(|(_, remaining)| *remaining != 0)
    {
        return Err("Missing Knocker placements".into());
    }
    // Stable source identity, independent of file name/location. Not a security digest.
    let hash = text
        .bytes()
        .fold(0x6c62272e07bb014262b821756295c58du128, |h, b| {
            (h ^ u128::from(b)).wrapping_mul(0x1000000000000000000013b)
        });
    let result = Scenario {
        id: format!("{hash:032x}"),
        name: root.get("Name")?.into(),
        radius: radius * UNIT,
        spawns,
        knockers,
        respawn,
        shot_interval: weapon.number("TimeBetweenShots", 0.02, 1.0)?,
        kill_score: root.number("ScorePerKill", 0.0, 100.0)?,
        sqrt_accuracy: root.flag("MultSqrtAcc")?,
        accuracy_multiplier: root.flag("ScoreMultAccuracy")?,
        fov: [
            root.number("LockedFOVMin", 60.0, 140.0)?,
            root.number("LockedFOVMax", 60.0, 140.0)?,
        ],
        motion: Motion {
            speed: character.number("MaxSpeed", 1.0, 10000.0)? * UNIT,
            acceleration: character.number("Acceleration", 1.0, 100000.0)? * UNIT,
            jump: character.number("JumpVelocity", 1.0, 10000.0)? * UNIT,
            air_jump: character.number("AirJumpVelocity", 1.0, 10000.0)? * UNIT,
            air_jumps: character
                .get("AirJumpCount")?
                .parse()
                .map_err(|_| "Invalid air jump count")?,
            gravity: character.number("Gravity", 0.001, 10.0)? * 980.0 * UNIT,
            terminal: character.number("TerminalVelocity", 1.0, 10000.0)? * UNIT,
            friction: character.number("AerialFriction", 0.0, 100.0)?,
            turn: interval(dodge, "MinLRTimeChange", "MaxLRTimeChange")?,
            jump_time: interval(dodge, "MinJumpTime", "MaxJumpTime")?,
            jump_chance: dodge.number("JumpFrequency", 0.0, 1.0)?,
            swap_pause: interval(dodge, "StrafeSwapMinPause", "StrafeSwapMaxPause")?,
        },
    };
    result.validate()?;
    Ok(result)
}

Name=Synthetic Pasu
PlayerCharacters=Player
BotCharacters=Ball.bot;Edge.bot;Ceiling.bot;Floor.bot
IsChallenge=true
Timelimit=60.0
Timescale=1.0
PlayerProfile=Player
AddedBots=Ball.bot;Ball.bot;Ball.bot;Ball.bot;Edge.bot;Edge.bot;Edge.bot;Edge.bot;Ceiling.bot;Ceiling.bot;Floor.bot;Floor.bot
BotTeams=2;2;2;2;1;1;1;1;1;1;1;1
BotMaxLives=0;0;0;0;0;0;0;0;0;0;0;0
PlayerTeam=1
ScorePerKill=10.0
ScoreMultAccuracy=true
MultSqrtAcc=true
InvinciblePlayer=true
InvincibleBots=false
MapName=Wall-less Wall.json
MapScale=4.0
LockFOVRange=true
LockedFOVMin=103.0
LockedFOVMax=140.0
[Character Profile]
Name=Player
MaxSpeed=0.0
WeaponProfileNames=Click
[Weapon Profile]
Name=Click
Type=Hitscan
Category=SemiAuto
ShotsPerClick=1
CooldownType=InfiniteUse
DamagePerShot=1.0
TimeBetweenShots=0.1
[Bot Profile]
Name=Ball
CharacterProfile=Ball
Untargetable=false
UseWeapons=false
NoDodging=false
DodgeProfileNames=Float
[Character Profile]
Name=Ball
MainBBType=Spheroid
MainBBRadius=60.0
MainBBHeight=120.0
CanPogoJump=true
DisableCharacterCollision=true
AirControl=1.0
AbilityProfileNames=
MaxHealth=1.0
MaxSpeed=900.0
Acceleration=9000.0
JumpVelocity=1100.0
AirJumpVelocity=1100.0
AirJumpCount=1000
Gravity=0.5
TerminalVelocity=1200.0
AerialFriction=4.0
MinRespawnDelay=0.1
MaxRespawnDelay=0.1
[Dodge Profile]
Name=Float
ToggleLeftRight=true
WaypointLogic=Ignore
MinLRTimeChange=1.0
MaxLRTimeChange=1.8
MinJumpTime=0.5
MaxJumpTime=1.0
JumpFrequency=0.4
StrafeSwapMinPause=0.0
StrafeSwapMaxPause=0.5
[Bot Profile]
Name=Edge
CharacterProfile=Edge
Untargetable=true
UseAbilityFrequency=1.0
NoDodging=true
[Bot Profile]
Name=Ceiling
CharacterProfile=Ceiling
Untargetable=true
UseAbilityFrequency=1.0
NoDodging=true
[Bot Profile]
Name=Floor
CharacterProfile=Floor
Untargetable=true
UseAbilityFrequency=1.0
NoDodging=true
[Character Profile]
Name=Edge
MaxSpeed=0.0
AbilityProfileNames=Push.abilmelee
[Character Profile]
Name=Ceiling
MaxSpeed=0.0
AbilityProfileNames=Down.abilmelee
[Character Profile]
Name=Floor
MaxSpeed=0.0
AbilityProfileNames=Up.abilmelee
[Melee Ability Profile]
Name=Push
HurtboxDamage=0.0
HurtboxRadius=4000.0
FlatKnockbackHorizontal=760.0
FlatKnockbackVertical=0.0
ChargeTimer=0.01
[Melee Ability Profile]
Name=Down
HurtboxDamage=0.0
HurtboxRadius=4000.0
FlatKnockbackHorizontal=0.0
FlatKnockbackVertical=-40.0
ChargeTimer=0.02
[Melee Ability Profile]
Name=Up
HurtboxDamage=0.0
HurtboxRadius=4000.0
FlatKnockbackHorizontal=0.0
FlatKnockbackVertical=40.0
ChargeTimer=0.02
[Map Data]
{
  "objects": [
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "0, 0, 400",
      "properties": [
        {
          "name": "TeamMask",
          "value": 1
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": "Player"
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, -1000, 800",
      "properties": [
        {
          "name": "TeamMask",
          "value": 1
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": "Edge"
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, 1000, 800",
      "properties": [
        {
          "name": "TeamMask",
          "value": 1
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": "Edge"
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, -1000, 0",
      "properties": [
        {
          "name": "TeamMask",
          "value": 1
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": "Edge"
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, 1000, 0",
      "properties": [
        {
          "name": "TeamMask",
          "value": 1
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": "Edge"
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, -1050, 900",
      "properties": [
        {
          "name": "TeamMask",
          "value": 1
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": "Ceiling"
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, 1050, 900",
      "properties": [
        {
          "name": "TeamMask",
          "value": 1
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": "Ceiling"
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, -1050, -100",
      "properties": [
        {
          "name": "TeamMask",
          "value": 1
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": "Floor"
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, 1050, -100",
      "properties": [
        {
          "name": "TeamMask",
          "value": 1
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": "Floor"
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, -300, 300",
      "properties": [
        {
          "name": "TeamMask",
          "value": 2
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": ""
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, -300, 500",
      "properties": [
        {
          "name": "TeamMask",
          "value": 2
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": ""
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, -100, 300",
      "properties": [
        {
          "name": "TeamMask",
          "value": 2
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": ""
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, -100, 500",
      "properties": [
        {
          "name": "TeamMask",
          "value": 2
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": ""
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, 100, 300",
      "properties": [
        {
          "name": "TeamMask",
          "value": 2
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": ""
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, 100, 500",
      "properties": [
        {
          "name": "TeamMask",
          "value": 2
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": ""
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, 300, 300",
      "properties": [
        {
          "name": "TeamMask",
          "value": 2
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": ""
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    },
    {
      "name": "SpawnPoint",
      "type": "gameObject",
      "location": "800, 300, 500",
      "properties": [
        {
          "name": "TeamMask",
          "value": 2
        },
        {
          "name": "PermittedCharacterProfiles",
          "value": ""
        },
        {
          "name": "Path",
          "value": ""
        },
        {
          "name": "Weight",
          "value": 1.0
        }
      ]
    }
  ]
}

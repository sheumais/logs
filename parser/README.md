## Unofficial encounter log version 15 documentation:

All lines begin with the time in milliseconds since logging began and the line type. Every field is comma separated. All strings are annotated with "surrounding" inverted commas. All texturePaths and iconPaths are filepaths to BC3 DXT5 DirectDraw Surface files which can be extracted using [EsoExtractData](https://en.uesp.net/wiki/ESO_Mod:EsoExtractData). Timestamps are Unix time in milliseconds. Booleans are represented using T or F characters.

`<unitState>` refers to the following fields for a unit: unitId, health/max, magicka/max, stamina/max, ultimate/max, werewolf/max, shield, mapNormalisedX, mapNormalisedY, headingRadians.

`<targetUnitState>` is replaced with an asterisk if the source and target are the same.

`<equipmentInfo>` refers to the following fields for a piece of equipment: slot, id, isCP, level, trait, displayQuality, setId, enchantType, isEnchantCP, enchantLevel, enchantQuality.

`<scribingInfo>` refers to the following fields for an ability: focusScript, signatureScript, affixScript.

## Line types

BEGIN_LOG, timeSinceEpochMS, logVersion, realmName, language, gameVersion

END_LOG

BEGIN_COMBAT

END_COMBAT

PLAYER_INFO, unitId, [longTermEffectAbilityId,...], [longTermEffectStackCounts,...], [`<equipmentInfo>`,...], [primaryAbilityId,...], [backupAbilityId,...]

BEGIN_CAST, durationMS, channeled, castTrackId, abilityId, `<sourceUnitState>`, `<targetUnitState>`

END_CAST, endReason, castTrackId, interruptedAbilityId, interruptingAbilityId:optional, interruptingUnitId:optional

COMBAT_EVENT, actionResult, damageType, powerType, hitValue, overflow, castTrackId, abilityId, `<sourceUnitState>`, `<targetUnitState>`

HEALTH_REGEN, effectiveRegen, `<unitState>`

UNIT_ADDED, unitId, unitType, isLocalPlayer, playerPerSessionId, monsterId, isBoss, classId, raceId, name, displayName, characterId, level, championPoints, ownerUnitId, reaction, isGroupedWithLocalPlayer

UNIT_CHANGED, unitId, classId, raceId, name, displayName, characterId, level, championPoints, ownerUnitId, reaction, isGroupedWithLocalPlayer

UNIT_REMOVED, unitId

EFFECT_CHANGED, changeType, stackCount, castTrackId, abilityId, `<sourceUnitState>`, `<targetUnitState>`, playerInitiatedRemoveCastTrackId:optional

ABILITY_INFO, abilityId, name, iconPath, interruptible, blockable, `<scribingInfo>`:optional

EFFECT_INFO, abilityId, effectType, statusEffectType, effectBarDisplayBehaviour, grantsSynergyAbilityId:optional

MAP_CHANGED, id, name, texturePath

ZONE_CHANGED, id, name, dungeonDifficulty

TRIAL_INIT, id, inProgress, completed, startTimeMS, durationMS, success, finalScore

BEGIN_TRIAL, id, startTimeMS

END_TRIAL, id, durationMS, success, finalScore, finalVitalityBonus

ENDLESS_DUNGEON_BEGIN, id, startTimeMS, unknownBoolean

ENDLESS_DUNGEON_END, id, durationMS, finalScore, unknownBoolean

ENDLESS_DUNGEON_STAGE_END, id, dungeonBeginStartTimeMS

ENDLESS_DUNGEON_BUFF_ADDED, id, abilityId

ENDLESS_DUNGEON_BUFF_REMOVED, id, abilityId

## Undocumented/unknown line types
ENDLESS_DUNGEON_INIT
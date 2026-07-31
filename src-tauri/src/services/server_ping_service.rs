//! Port of `src/main/services/serverPingService.ts`.

use byteorder::BigEndian;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub featured: bool,
    pub online: bool,
    pub favicon: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "playersOnline")]
    pub players_online: u32,
    #[serde(rename = "playersMax")]
    pub players_max: u32,
    pub motd: Option<String>,
    pub ping: Option<u64>,
}

struct FeaturedServer {
    id: &'static str,
    name: &'static str,
    host: &'static str,
}

// Ported verbatim from serverPingService.ts's FEATURED_SERVERS (all default to port 25565).
const FEATURED_SERVERS: &[FeaturedServer] = &[
    FeaturedServer { id: "donutsmp", name: "DonutSMP", host: "donutsmp.net" },
    FeaturedServer { id: "hugosmp", name: "HugoSMP", host: "hugosmp.net" },
    FeaturedServer { id: "hypixel", name: "Hypixel", host: "mc.hypixel.net" },
    FeaturedServer { id: "complexgaming", name: "Complex Gaming", host: "hub.mc-complex.com" },
    FeaturedServer { id: "manacube", name: "ManaCube", host: "buzz.manacube.com" },
    FeaturedServer { id: "blossomcraft", name: "BlossomCraft", host: "buzz.blossomcraft.org" },
    FeaturedServer { id: "mysticmc", name: "MysticMC", host: "buzz.mysticmc.co" },
    FeaturedServer { id: "smilemorecraft", name: "SmileMoreCraft", host: "buzz.smilemorecraft.com" },
    FeaturedServer { id: "akumamc", name: "AkumaMC", host: "akumamc.net" },
    FeaturedServer { id: "gommehd", name: "GommeHD", host: "gommehd.net" },
    FeaturedServer { id: "griefergames", name: "GrieferGames", host: "griefergames.net" },
    FeaturedServer { id: "2b2t", name: "2b2t", host: "connect.2b2t.org" },
    FeaturedServer { id: "wynncraft", name: "Wynncraft", host: "play.wynncraft.com" },
    FeaturedServer { id: "minepiece", name: "MinePiece", host: "adventure.minepiece.com" },
    FeaturedServer { id: "chunklock", name: "ChunkLock", host: "survival.chunklock.com" },
    FeaturedServer { id: "mythria", name: "Mythria", host: "play.mythria.gg" },
    FeaturedServer { id: "blockdrop", name: "BlockDrop Network", host: "playbd.games" },
    FeaturedServer { id: "opblocks", name: "OPBlocks", host: "play.opblocks.com" },
    FeaturedServer { id: "purpleprison", name: "Purple Prison", host: "purpleprison.org" },
    FeaturedServer { id: "extremecraft", name: "ExtremeCraft", host: "play.extremecraft.net" },
    FeaturedServer { id: "cubecraft", name: "CubeCraft", host: "play.cubecraft.net" },
    FeaturedServer { id: "pikanetwork", name: "PikaNetwork", host: "play.pika-network.net" },
    FeaturedServer { id: "jartexnetwork", name: "JartexNetwork", host: "top.jartex.fun" },
    FeaturedServer { id: "loka", name: "Loka", host: "play.lokamc.com" },
    FeaturedServer { id: "earthmc", name: "EarthMC", host: "play.earthmc.net" },
    FeaturedServer { id: "herobrine", name: "Herobrine.org", host: "herobrine.org" },
    FeaturedServer { id: "minemenclub", name: "Minemen Club", host: "minemen.club" },
    FeaturedServer { id: "vipermc", name: "ViperMC", host: "play.vipermc.net" },
    FeaturedServer { id: "oneblockmc", name: "OneBlock MC", host: "play.oneblockmc.com" },
    FeaturedServer { id: "loverfella", name: "LoverFella", host: "play.loverfella.com" },
    FeaturedServer { id: "insanitycraft", name: "InsanityCraft", host: "play.insanitycraft.net" },
    FeaturedServer { id: "advancius", name: "Advancius Network", host: "mc.advancius.net" },
    FeaturedServer { id: "simplesurvival", name: "SimpleSurvival", host: "play.simplesurvival.gg" },
    FeaturedServer { id: "mixelpixel", name: "MixelPixel", host: "mixelpixel.net" },
    FeaturedServer { id: "landania", name: "Landania", host: "landania.net" },
    FeaturedServer { id: "opsucht", name: "Opsucht", host: "opsucht.net" },
    FeaturedServer { id: "twerion", name: "Twerion", host: "ms.twerion.net" },
    FeaturedServer { id: "cubeside", name: "Cubeside", host: "cubeside.de" },
    FeaturedServer { id: "mcplayhd", name: "McPlayHD", host: "mcplayhd.net" },
    FeaturedServer { id: "hexagonmc", name: "HexagonMC", host: "hexagonmc.eu" },
    FeaturedServer { id: "lemoncloud", name: "LemonCloud", host: "play.lemoncloud.org" },
    FeaturedServer { id: "originrealms", name: "Origin Realms", host: "play.originrealms.com" },
    FeaturedServer { id: "wildercraft", name: "WilderCraft", host: "play.wildercraft.net" },
    FeaturedServer { id: "vulengate", name: "Vulengate", host: "play.vulengate.com" },
    FeaturedServer { id: "skyblocknet", name: "Skyblock Net", host: "skyblock.net" },
    FeaturedServer { id: "tulipsurvival", name: "TulipSurvival", host: "play.tulipsurvival.com" },
    FeaturedServer { id: "datblock", name: "Datblock", host: "play.datblock.com" },
    FeaturedServer { id: "potterworld", name: "Potterworld", host: "play.potterworldmc.com" },
    FeaturedServer { id: "piratecraft", name: "PirateCraft", host: "mc.piratemc.com" },
    FeaturedServer { id: "pokefind", name: "PokeFind", host: "play.pokefind.co" },
    FeaturedServer { id: "stoneworks", name: "Stoneworks", host: "play.stoneworks.gg" },
    FeaturedServer { id: "oplegends", name: "OPLegends", host: "play.oplegends.com" },
    FeaturedServer { id: "fadecloud", name: "FadeCloud", host: "fadecloud.com" },
    FeaturedServer { id: "craftyourtown", name: "CraftYourTown", host: "play.craftyourtown.com" },
    FeaturedServer { id: "fruitservers", name: "FruitServers", host: "mc.fruitservers.net" },
    FeaturedServer { id: "autocraft", name: "Autocraft", host: "play.autocraftmc.org" },
    FeaturedServer { id: "suniverse", name: "Suniverse", host: "play.suniverse.or.id" },
    FeaturedServer { id: "netherite", name: "Netherite", host: "play.netherite.gg" },
    FeaturedServer { id: "medievallords", name: "Medieval Lords", host: "play.ml-mc.com" },
    FeaturedServer { id: "skyblocksquad", name: "Skyblock Squad", host: "play.skyblocksquad.net" },
    FeaturedServer { id: "cosmicsky", name: "Cosmic Sky", host: "play.cosmicsky.com" },
    FeaturedServer { id: "destinymc", name: "DestinyMC", host: "play.thedestinymc.com" },
    FeaturedServer { id: "supercraft", name: "SuperCraft", host: "play.supercraft.club" },
    FeaturedServer { id: "minespire", name: "MineSpire", host: "play.minespire.net" },
    FeaturedServer { id: "craftrise", name: "CraftRise", host: "play.craftrise.tc" },
    FeaturedServer { id: "sonoyuncu", name: "Sonoyuncu", host: "play.sonoyuncu.network" },
    FeaturedServer { id: "shadowverse", name: "Shadowverse", host: "play.shadowverse.net" },
    FeaturedServer { id: "roxbot", name: "RoxBot", host: "play.roxbot.com" },
    FeaturedServer { id: "mcprison", name: "MCPrison", host: "play.mcprison.com" },
    FeaturedServer { id: "aerocraft", name: "AeroCraft", host: "play.aerocraft.net" },
    FeaturedServer { id: "primemc", name: "PrimeMC", host: "play.primemc.org" },
    FeaturedServer { id: "saiyancraft", name: "SaiyanCraft", host: "play.saiyancraft.net" },
    FeaturedServer { id: "totalfreedom", name: "Total Freedom", host: "play.totalfreedom.me" },
    FeaturedServer { id: "purityvanilla", name: "PurityVanilla", host: "purityvanilla.com" },
    FeaturedServer { id: "vanillahigh", name: "Vanilla High", host: "vanillahigh.net" },
    FeaturedServer { id: "simplyvanilla", name: "Simply Vanilla", host: "simplyvanilla.org" },
    FeaturedServer { id: "auracraft", name: "AuraCraft", host: "play.auracraft.net" },
    FeaturedServer { id: "titanmc", name: "TitanMC", host: "play.titanmc.net" },
    FeaturedServer { id: "ultranetwork", name: "Ultra Network", host: "play.ultramc.net" },
    FeaturedServer { id: "blazegaming", name: "BlazeGaming", host: "play.blazegaming.co" },
    FeaturedServer { id: "pixelmontogo", name: "Pixelmon To Go", host: "play.pixelmontogo.com" },
    FeaturedServer { id: "pokeplanet", name: "PokéPlanet", host: "play.pokeplanet.net" },
    FeaturedServer { id: "journeygaming", name: "Journey Gaming", host: "play.journeygaming.com" },
    FeaturedServer { id: "roverssmp", name: "Rovers SMP", host: "play.roverssmp.com" },
    FeaturedServer { id: "peacecraft", name: "Peace Craft", host: "play.peacecraft.net" },
    FeaturedServer { id: "harmonysmp", name: "Harmony SMP", host: "play.harmonysmp.net" },
    FeaturedServer { id: "unitymc", name: "UnityMC", host: "play.unitymc.net" },
    FeaturedServer { id: "townyclassic", name: "TownyClassic", host: "play.townyclassic.net" },
    FeaturedServer { id: "aliquam", name: "Aliquam", host: "play.aliquam.org" },
    FeaturedServer { id: "creativecentral", name: "Creative Central", host: "play.creativecentral.net" },
    FeaturedServer { id: "buildersrefuge", name: "Builders Refuge", host: "buildersrefuge.com" },
    FeaturedServer { id: "pocketpixels", name: "PocketPixels", host: "play.pocketpixels.net" },
    FeaturedServer { id: "miragecraft", name: "MirageCraft", host: "play.miragecraft.net" },
    FeaturedServer { id: "pixelmongens", name: "Pixelmon Generations", host: "play.pixelmongenerations.com" },
    FeaturedServer { id: "smashmc", name: "SmashMC", host: "play.smashmc.co" },
    FeaturedServer { id: "hylex", name: "Hylex", host: "play.hylex.io" },
    FeaturedServer { id: "bedwarsgames", name: "BedWars.games", host: "bedwars.games" },
    FeaturedServer { id: "lunarnetwork", name: "Lunar Network", host: "lunar.gg" },
    FeaturedServer { id: "straynetwork", name: "Stray Network", host: "stray.gg" },
    FeaturedServer { id: "syblock", name: "Syblock", host: "play.syblock.net" },
    FeaturedServer { id: "skyblockx", name: "SkyblockX", host: "play.skyblockx.net" },
    FeaturedServer { id: "castia", name: "Castia", host: "play.castiamc.com" },
    FeaturedServer { id: "phrenzy", name: "Phrenzy", host: "play.phrenzymc.net" },
    FeaturedServer { id: "desteria", name: "Desteria", host: "pvp.desteria.com" },
    FeaturedServer { id: "saicopvp", name: "SaicoPvP", host: "play.saicopvp.com" },
    FeaturedServer { id: "originmc", name: "OriginMC", host: "play.originmc.org" },
    FeaturedServer { id: "thearchon", name: "The Archon", host: "play.thearchon.net" },
    FeaturedServer { id: "sbhub", name: "Sb-hub", host: "play.sb-hub.com" },
    FeaturedServer { id: "foxcraft", name: "FoxCraft", host: "play.foxcraft.net" },
    FeaturedServer { id: "minetexas", name: "MineTexas", host: "minetexas.com" },
    FeaturedServer { id: "minr", name: "Minr", host: "zero.minr.org" },
    FeaturedServer { id: "jumpcraft", name: "JumpCraft", host: "play.jumpcraft.org" },
    FeaturedServer { id: "parkourcraft", name: "ParkourCraft", host: "play.parkourcraft.com" },
    FeaturedServer { id: "happyhg", name: "Happy-HG", host: "happy-hg.com" },
    FeaturedServer { id: "mcwars", name: "MC-Wars", host: "play.mc-wars.org" },
    FeaturedServer { id: "minestrike", name: "MineStrike", host: "play.minestrike.com" },
    FeaturedServer { id: "mcmagic", name: "McMagic", host: "mcmagic.us" },
    FeaturedServer { id: "palacenetwork", name: "Palace Network", host: "play.palacenetwork.net" },
    FeaturedServer { id: "dwo", name: "DWO", host: "play.doctorwhoonline.co.uk" },
    FeaturedServer { id: "craftingforchrist", name: "Crafting For Christ", host: "play.craftingforchrist.net" },
    FeaturedServer { id: "safecraft", name: "SafeCraft", host: "play.safecraft.org" },
    FeaturedServer { id: "intercraftmc", name: "InterCraft", host: "play.intercraftmc.com" },
    FeaturedServer { id: "blockverse", name: "BlockVerse", host: "play.blockverse.net" },
    FeaturedServer { id: "neocraftmc", name: "NeoCraft", host: "play.neocraftmc.com" },
    FeaturedServer { id: "starcraft", name: "StarCraft", host: "play.starcraft.net" },
    FeaturedServer { id: "galaxynetwork", name: "Galaxy Network", host: "play.galaxynetwork.gg" },
    FeaturedServer { id: "novasmp", name: "Nova SMP", host: "play.novasmp.com" },
    FeaturedServer { id: "nebulamc", name: "Nebula MC", host: "play.nebulamc.net" },
    FeaturedServer { id: "zenithsmp", name: "Zenith SMP", host: "play.zenithsmp.org" },
    FeaturedServer { id: "apexsurvival", name: "Apex Survival", host: "play.apexsurvival.com" },
    FeaturedServer { id: "titansurvival", name: "Titan Survival", host: "play.titansurvival.net" },
    FeaturedServer { id: "mythiccraft", name: "Mythic Craft", host: "play.mythiccraft.org" },
    FeaturedServer { id: "legendarymc", name: "Legendary MC", host: "play.legendarymc.net" },
    FeaturedServer { id: "ancientworld", name: "Ancient World", host: "play.ancientworld.net" },
    FeaturedServer { id: "kingdomcraft", name: "Kingdom Craft", host: "play.kingdomcraft.org" },
    FeaturedServer { id: "empireminecraft", name: "Empire Minecraft", host: "play.emc.gs" },
    FeaturedServer { id: "ecocitycraft", name: "EcoCityCraft", host: "play.ecocitycraft.com" },
    FeaturedServer { id: "pottercraft", name: "Pottercraft", host: "play.pottercraft.net" },
    FeaturedServer { id: "swconquest", name: "Star Wars Conquest", host: "play.swconquest.net" },
    FeaturedServer { id: "minecraftonline", name: "MineCraftOnline", host: "minecraftonline.com" },
    FeaturedServer { id: "9b9t", name: "9b9t", host: "9b9t.org" },
    FeaturedServer { id: "constantiam", name: "Constantiam", host: "constantiam.net" },
    FeaturedServer { id: "8b8t", name: "8b8t", host: "8b8t.me" },
    FeaturedServer { id: "4b4t", name: "4b4t", host: "4b4t.org" },
    FeaturedServer { id: "1b1t", name: "1b1t", host: "1b1t.org" },
    FeaturedServer { id: "oldschoolmc", name: "OldSchoolMinecraft", host: "oldschoolminecraft.com" },
    FeaturedServer { id: "betacraft", name: "BetaCraft", host: "betacraft.uk" },
    FeaturedServer { id: "retrocraft", name: "RetroCraft", host: "play.retrocraft.net" },
    FeaturedServer { id: "classiccraft", name: "ClassicCraft", host: "play.classiccraft.org" },
    FeaturedServer { id: "alphaserver", name: "AlphaServer", host: "play.alphaserver.net" },
    FeaturedServer { id: "nostalgiamc", name: "NostalgiaMC", host: "play.nostalgiamc.com" },
    FeaturedServer { id: "vanillaanarchy", name: "Vanilla Anarchy", host: "vanillaanarchy.net" },
    FeaturedServer { id: "pureanarchy", name: "Pure Anarchy", host: "pureanarchy.org" },
    FeaturedServer { id: "trueanarchy", name: "True Anarchy", host: "trueanarchy.net" },
    FeaturedServer { id: "darkanarchy", name: "Dark Anarchy", host: "darkanarchy.org" },
    FeaturedServer { id: "chaosmc", name: "Chaos MC", host: "play.chaosmc.net" },
    FeaturedServer { id: "riotnetwork", name: "Riot Network", host: "play.riotnetwork.com" },
    FeaturedServer { id: "havocgames", name: "Havoc Games", host: "play.havocgames.net" },
    FeaturedServer { id: "miningdead", name: "Mining Dead", host: "play.miningdead.com" },
    FeaturedServer { id: "zombiesmp", name: "ZombieSMP", host: "play.zombiesmp.com" },
    // Minehut-hosted
    FeaturedServer { id: "solarskies", name: "SolarSkies", host: "solarskies.minehut.gg" },
    FeaturedServer { id: "runedmc", name: "RunedMC", host: "runedmc.minehut.gg" },
    FeaturedServer { id: "unstablehq", name: "UnstableHQ", host: "unstablehq.minehut.gg" },
    FeaturedServer { id: "alpmc", name: "AlpMc", host: "alpmc.minehut.gg" },
    // mcsh.io community
    FeaturedServer { id: "medievalsmp", name: "MedievalSMP", host: "smpmedieval.mcsh.io" },
    FeaturedServer { id: "viresmc", name: "ViresMC", host: "viresmc.mcsh.io" },
    FeaturedServer { id: "astralmc", name: "AstralMC", host: "elementalsmp670.mcsh.io" },
    FeaturedServer { id: "vanillashake", name: "VanillaShake+", host: "vanillashakeplus.mcsh.io" },
    FeaturedServer { id: "vanitysmp", name: "VanitySMP", host: "vanitysmpv2.mcsh.io" },
    FeaturedServer { id: "megasmp", name: "MegaSMP", host: "megasmp.mcsh.io" },
    FeaturedServer { id: "cellsmp", name: "CellSmp", host: "cellsmp.mcsh.io" },
    FeaturedServer { id: "netheritesmp", name: "NetheriteSMP", host: "netheritesmp368.mcsh.io" },
    FeaturedServer { id: "macessmp", name: "MaceSmp", host: "macessmp.mcsh.io" },
    FeaturedServer { id: "aspectsmp", name: "Aspect SMP", host: "aspectsmp.mcsh.io" },
    FeaturedServer { id: "bigberrysmp", name: "BigberrySMP", host: "bigberrysmp.mcsh.io" },
    FeaturedServer { id: "peacesbp", name: "PeaceSBP", host: "peacesbp497.mcsh.io" },
    FeaturedServer { id: "atomffa", name: "Atom FFA", host: "atomffagg.mcsh.io" },
    FeaturedServer { id: "amethystsmp", name: "Amethystsmp", host: "amethystsmp396.mcsh.io" },
    FeaturedServer { id: "abyssrevenge", name: "AbyssRevenge", host: "tough.mcsh.io" },
    FeaturedServer { id: "ritesmprec", name: "RiteSMPrec", host: "ritesmprec.mcsh.io" },
    // German servers
    FeaturedServer { id: "meloncity", name: "MelonCity", host: "meloncity.de" },
    FeaturedServer { id: "mcspielplatz", name: "Minecraft-Spielplatz", host: "mc.minecraft-spielplatz.net" },
    FeaturedServer { id: "secretcraft", name: "SecretCraft", host: "mc.secretcraft.de" },
    FeaturedServer { id: "rangemc", name: "RangeMC", host: "rangemc.net" },
    FeaturedServer { id: "kadcon", name: "Kadcon", host: "kadcon.de" },
    FeaturedServer { id: "minebench", name: "MineBench", host: "minebench.de" },
    FeaturedServer { id: "skyblockserver", name: "SkyBlock-Server", host: "skyblock-server.de" },
    FeaturedServer { id: "blockminers", name: "Blockminers", host: "blockminers.de" },
    FeaturedServer { id: "terraconia", name: "Terraconia", host: "terraconia.de" },
    FeaturedServer { id: "minecaria", name: "Minecaria", host: "minecaria.de" },
    FeaturedServer { id: "freecraft", name: "FreeCraft", host: "freecraft.de" },
    FeaturedServer { id: "bausucht", name: "Bausucht", host: "bausucht.net" },
    FeaturedServer { id: "mycraft", name: "MyCraft", host: "mycraft.com" },
    FeaturedServer { id: "unitedmc", name: "UnitedMC", host: "unitedmc.de" },
    FeaturedServer { id: "crafttopia", name: "Crafttopia", host: "play.crafttopia.org" },
    FeaturedServer { id: "minesuperior", name: "MineSuperior", host: "hub.minesuperior.com" },
    FeaturedServer { id: "primeblocks", name: "PrimeBlocks", host: "primeblocks.net" },
];

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CustomServer {
    id: String,
    name: String,
    host: String,
    port: u16,
}

fn load_custom_servers() -> Vec<CustomServer> {
    std::fs::read_to_string(paths::servers_path())
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_custom_servers(servers: &[CustomServer]) {
    if let Ok(json) = serde_json::to_string_pretty(servers) {
        let _ = std::fs::write(paths::servers_path(), json);
    }
}

pub fn get_static_list() -> Vec<ServerStatus> {
    let mut out: Vec<ServerStatus> = FEATURED_SERVERS
        .iter()
        .map(|s| offline_entry(s.id, s.name, s.host, 25565, true))
        .collect();
    out.extend(load_custom_servers().iter().map(|s| offline_entry(&s.id, &s.name, &s.host, s.port, false)));
    out
}

fn offline_entry(id: &str, name: &str, host: &str, port: u16, featured: bool) -> ServerStatus {
    ServerStatus {
        id: id.to_string(),
        name: name.to_string(),
        host: host.to_string(),
        port,
        featured,
        online: false,
        favicon: None,
        version: None,
        players_online: 0,
        players_max: 0,
        motd: None,
        ping: None,
    }
}

pub fn add_server(host: &str, port: u16, name: &str) -> String {
    let mut custom = load_custom_servers();
    let id = format!("custom-{}", chrono::Utc::now().timestamp_millis());
    custom.push(CustomServer { id: id.clone(), name: if name.is_empty() { host.to_string() } else { name.to_string() }, host: host.to_string(), port });
    save_custom_servers(&custom);
    id
}

pub fn remove_server(id: &str) {
    let custom: Vec<CustomServer> = load_custom_servers().into_iter().filter(|s| s.id != id).collect();
    save_custom_servers(&custom);
}

// ── VarInt ──────────────────────────────────────────────────────────────────

fn write_var_int(mut n: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut b = (n & 0x7f) as u8;
        n >>= 7;
        if n != 0 {
            b |= 0x80;
        }
        bytes.push(b);
        if n == 0 {
            break;
        }
    }
    bytes
}

fn read_var_int(buf: &[u8], offset: usize) -> Option<(u32, usize)> {
    let mut value: u32 = 0;
    let mut size = 0usize;
    loop {
        let b = *buf.get(offset + size)?;
        value |= ((b & 0x7f) as u32) << (7 * size);
        size += 1;
        if size > 5 {
            return None;
        }
        if b & 0x80 == 0 {
            break;
        }
    }
    Some((value, size))
}

fn mc_str(s: &str) -> Vec<u8> {
    let data = s.as_bytes();
    let mut out = write_var_int(data.len() as u32);
    out.extend_from_slice(data);
    out
}

fn mc_packet(id: u32, parts: &[Vec<u8>]) -> Vec<u8> {
    let mut body = write_var_int(id);
    for p in parts {
        body.extend_from_slice(p);
    }
    let mut out = write_var_int(body.len() as u32);
    out.extend_from_slice(&body);
    out
}

pub struct PingResult {
    pub favicon: Option<String>,
    pub version: Option<String>,
    pub players_online: u32,
    pub players_max: u32,
    pub motd: Option<String>,
    pub ping: u64,
}

/// connect_host/connect_port is what the TCP socket actually dials; virtual_host/virtual_port is
/// what's sent inside the handshake packet's "server address" field — these differ once an SRV
/// record redirects the connection (BungeeCord/Velocity virtual-host backend routing).
async fn ping_server_direct(connect_host: &str, connect_port: u16, virtual_host: &str, virtual_port: u16) -> Result<PingResult, String> {
    let connect_fut = TcpStream::connect((connect_host, connect_port));
    let mut socket = timeout(Duration::from_secs(5), connect_fut)
        .await
        .map_err(|_| "Timeout".to_string())?
        .map_err(|e| e.to_string())?;

    let connect_time = Instant::now();

    let mut port_buf = Vec::new();
    byteorder::WriteBytesExt::write_u16::<BigEndian>(&mut port_buf, virtual_port).map_err(|e| e.to_string())?;
    let handshake = mc_packet(0x00, &[write_var_int(765), mc_str(virtual_host), port_buf, write_var_int(1)]);
    let status_request = mc_packet(0x00, &[]);

    timeout(Duration::from_secs(5), async {
        socket.write_all(&handshake).await?;
        socket.write_all(&status_request).await?;
        Ok::<(), std::io::Error>(())
    })
    .await
    .map_err(|_| "Timeout".to_string())?
    .map_err(|e| e.to_string())?;

    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];

    loop {
        let n = timeout(Duration::from_secs(5), socket.read(&mut chunk))
            .await
            .map_err(|_| "Timeout".to_string())?
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Connection closed".to_string());
        }
        buf.extend_from_slice(&chunk[..n]);

        while !buf.is_empty() {
            let Some((len, len_size)) = read_var_int(&buf, 0) else { break };
            let total_len = len_size + len as usize;
            if buf.len() < total_len {
                break;
            }
            let pkt_data = buf[len_size..total_len].to_vec();
            buf.drain(0..total_len);

            let Some((packet_id, id_size)) = read_var_int(&pkt_data, 0) else { continue };
            if packet_id != 0x00 {
                continue;
            }
            let Some((str_len, str_len_size)) = read_var_int(&pkt_data, id_size) else { continue };
            let start = id_size + str_len_size;
            let end = start + str_len as usize;
            if end > pkt_data.len() {
                continue;
            }
            let json_str = String::from_utf8_lossy(&pkt_data[start..end]).to_string();
            let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) else { continue };

            let raw_motd = match &json["description"] {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Object(o) => o.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()),
                _ => None,
            };
            let motd = raw_motd.map(|m| strip_color_codes(&m));

            return Ok(PingResult {
                favicon: json["favicon"].as_str().map(|s| s.to_string()),
                version: json["version"]["name"].as_str().map(|s| s.to_string()),
                players_online: json["players"]["online"].as_u64().unwrap_or(0) as u32,
                players_max: json["players"]["max"].as_u64().unwrap_or(0) as u32,
                motd,
                ping: connect_time.elapsed().as_millis() as u64,
            });
        }
    }
}

fn strip_color_codes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

/// Many BungeeCord/Velocity networks don't listen on the bare domain at 25565 — they publish a
/// `_minecraft._tcp.<host>` SRV record pointing at the real backend instead. Only attempted when
/// port is the default 25565, matching the vanilla client's own behaviour.
pub async fn ping_server(host: &str, port: u16) -> Result<PingResult, String> {
    let mut connect_host = host.to_string();
    let mut connect_port = port;

    if port == 25565 {
        let resolver = hickory_resolver::TokioAsyncResolver::tokio_from_system_conf();
        if let Ok(resolver) = resolver {
            if let Ok(lookup) = resolver.srv_lookup(format!("_minecraft._tcp.{host}")).await {
                if let Some(best) = lookup.iter().min_by_key(|r| r.priority()) {
                    connect_host = best.target().to_string().trim_end_matches('.').to_string();
                    connect_port = best.port();
                }
            }
        }
    }

    ping_server_direct(&connect_host, connect_port, host, port).await
}

// ── servers.dat NBT ─────────────────────────────────────────────────────────

const TAG_END: u8 = 0;
const TAG_BYTE: u8 = 1;
const TAG_STRING: u8 = 8;
const TAG_LIST: u8 = 9;
const TAG_COMPOUND: u8 = 10;

fn nbt_str16(w: &mut Vec<u8>, s: &str) {
    let utf8 = s.as_bytes();
    byteorder::WriteBytesExt::write_u16::<BigEndian>(w, utf8.len() as u16).unwrap();
    w.extend_from_slice(utf8);
}

fn nbt_entry_string(w: &mut Vec<u8>, name: &str, value: &str) {
    w.push(TAG_STRING);
    nbt_str16(w, name);
    nbt_str16(w, value);
}

fn nbt_entry_byte(w: &mut Vec<u8>, name: &str, value: u8) {
    w.push(TAG_BYTE);
    nbt_str16(w, name);
    w.push(value);
}

struct DatServerEntry {
    name: String,
    ip: String,
    icon: Option<String>,
}

fn build_servers_dat(servers: &[DatServerEntry]) -> Vec<u8> {
    let mut entries = Vec::new();
    for s in servers {
        nbt_entry_string(&mut entries, "ip", &s.ip);
        nbt_entry_string(&mut entries, "name", &s.name);
        if let Some(icon) = &s.icon {
            nbt_entry_string(&mut entries, "icon", icon);
        }
        nbt_entry_byte(&mut entries, "hideAddress", 0);
        nbt_entry_byte(&mut entries, "acceptTexturePackStatus", 0);
        entries.push(TAG_END);
    }

    let mut list_val = Vec::new();
    list_val.push(TAG_COMPOUND);
    byteorder::WriteBytesExt::write_i32::<BigEndian>(&mut list_val, servers.len() as i32).unwrap();
    list_val.extend_from_slice(&entries);

    let mut root_body = Vec::new();
    root_body.push(TAG_LIST);
    nbt_str16(&mut root_body, "servers");
    root_body.extend_from_slice(&list_val);
    root_body.push(TAG_END);

    let mut root = Vec::new();
    root.push(TAG_COMPOUND);
    nbt_str16(&mut root, "");
    root.extend_from_slice(&root_body);

    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&root).unwrap();
    encoder.finish().unwrap_or_default()
}

fn skip_nbt_tag(raw: &[u8], mut pos: usize, tag_type: u8) -> usize {
    let mut cursor = Cursor::new(raw);
    match tag_type {
        1 => pos + 1,
        2 => pos + 2,
        3 => pos + 4,
        4 => pos + 8,
        5 => pos + 4,
        6 => pos + 8,
        7 => {
            cursor.set_position(pos as u64);
            let len = byteorder::ReadBytesExt::read_i32::<BigEndian>(&mut cursor).unwrap_or(0);
            pos + 4 + len.max(0) as usize
        }
        8 => {
            cursor.set_position(pos as u64);
            let len = byteorder::ReadBytesExt::read_u16::<BigEndian>(&mut cursor).unwrap_or(0);
            pos + 2 + len as usize
        }
        9 => {
            let elem_type = raw.get(pos).copied().unwrap_or(0);
            pos += 1;
            cursor.set_position(pos as u64);
            let count = byteorder::ReadBytesExt::read_i32::<BigEndian>(&mut cursor).unwrap_or(0).max(0);
            pos += 4;
            for _ in 0..count {
                pos = skip_nbt_tag(raw, pos, elem_type);
            }
            pos
        }
        10 => {
            while pos < raw.len() {
                let t = raw[pos];
                pos += 1;
                if t == TAG_END {
                    break;
                }
                cursor.set_position(pos as u64);
                let nl = byteorder::ReadBytesExt::read_u16::<BigEndian>(&mut cursor).unwrap_or(0) as usize;
                pos += 2 + nl;
                pos = skip_nbt_tag(raw, pos, t);
            }
            pos
        }
        11 => {
            cursor.set_position(pos as u64);
            let len = byteorder::ReadBytesExt::read_i32::<BigEndian>(&mut cursor).unwrap_or(0).max(0);
            pos + 4 + len as usize * 4
        }
        12 => {
            cursor.set_position(pos as u64);
            let len = byteorder::ReadBytesExt::read_i32::<BigEndian>(&mut cursor).unwrap_or(0).max(0);
            pos + 4 + len as usize * 8
        }
        _ => pos,
    }
}

fn read_servers_dat(data: &[u8]) -> Vec<DatServerEntry> {
    let Ok(mut decoder) = Ok::<_, ()>(flate2::read::GzDecoder::new(data)) else { return Vec::new() };
    let mut raw = Vec::new();
    if decoder.read_to_end(&mut raw).is_err() {
        return Vec::new();
    }

    let mut pos = 1usize; // skip root TAG_COMPOUND byte
    let Some(name_len) = raw.get(pos..pos + 2).map(|b| u16::from_be_bytes([b[0], b[1]])) else { return Vec::new() };
    pos += 2 + name_len as usize;

    while pos < raw.len() {
        let tag_type = raw[pos];
        pos += 1;
        if tag_type == TAG_END {
            break;
        }
        let Some(name_len) = raw.get(pos..pos + 2).map(|b| u16::from_be_bytes([b[0], b[1]]) as usize) else { break };
        pos += 2;
        let Some(name_bytes) = raw.get(pos..pos + name_len) else { break };
        let tag_name = String::from_utf8_lossy(name_bytes).to_string();
        pos += name_len;

        if tag_type == TAG_LIST && tag_name == "servers" {
            let elem_type = raw.get(pos).copied().unwrap_or(0);
            pos += 1;
            let Some(count) = raw.get(pos..pos + 4).map(|b| i32::from_be_bytes([b[0], b[1], b[2], b[3]]).max(0)) else {
                return Vec::new();
            };
            pos += 4;

            let mut result = Vec::new();
            if elem_type == TAG_COMPOUND {
                for _ in 0..count {
                    let mut fields = std::collections::HashMap::new();
                    while pos < raw.len() {
                        let ft = raw[pos];
                        pos += 1;
                        if ft == TAG_END {
                            break;
                        }
                        let Some(fl) = raw.get(pos..pos + 2).map(|b| u16::from_be_bytes([b[0], b[1]]) as usize) else { break };
                        pos += 2;
                        let Some(fname_bytes) = raw.get(pos..pos + fl) else { break };
                        let fname = String::from_utf8_lossy(fname_bytes).to_string();
                        pos += fl;
                        if ft == TAG_STRING {
                            let Some(vl) = raw.get(pos..pos + 2).map(|b| u16::from_be_bytes([b[0], b[1]]) as usize) else { break };
                            pos += 2;
                            let Some(vbytes) = raw.get(pos..pos + vl) else { break };
                            fields.insert(fname, String::from_utf8_lossy(vbytes).to_string());
                            pos += vl;
                        } else if ft == TAG_BYTE {
                            pos += 1;
                        }
                    }
                    if let Some(ip) = fields.get("ip") {
                        result.push(DatServerEntry {
                            name: fields.get("name").cloned().unwrap_or_default(),
                            ip: ip.clone(),
                            icon: fields.get("icon").cloned(),
                        });
                    }
                }
            }
            return result;
        }
        pos = skip_nbt_tag(&raw, pos, tag_type);
    }
    Vec::new()
}

pub fn add_server_to_profile(host: &str, port: u16, name: &str, favicon: Option<&str>, game_dir: &Path) {
    let dat_file = game_dir.join("servers.dat");

    let mut existing = if dat_file.exists() {
        std::fs::read(&dat_file).ok().map(|d| read_servers_dat(&d)).unwrap_or_default()
    } else {
        Vec::new()
    };

    let ip = if port == 25565 { host.to_string() } else { format!("{host}:{port}") };
    if !existing.iter().any(|s| s.ip == ip) {
        existing.push(DatServerEntry { name: name.to_string(), ip, icon: favicon.map(|s| s.to_string()) });
        let dat = build_servers_dat(&existing);
        if let Some(parent) = dat_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&dat_file, dat);
    }
}

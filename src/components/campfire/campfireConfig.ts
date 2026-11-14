import { Vector3 } from 'three'

// All coordinates are in the Campfire_Scene.gltf's own space: 1 unit = 1 block.
// Read straight from the exported model - logs are 1 block tall, ground around
// the fire sits at y=35, log tops at y=36.

export const FIRE = new Vector3(-17.5, 35.5, -1.5)
export const SEAT_TOP = 36

// Solid obstacles on the fire plateau (block footprints, xz) - routes must stay clear of them:
//   row log     x -21..-15, z -6..-5   (5 stripped logs + an oak log at the east end)
//   column log  x -14..-13, z -4..2    (4 stripped logs + an oak log at each end)
//   campfire    x -18..-17, z -2..-1
// The plateau around the fire is a flat y=35. Terrain one block higher (a hill, single raised blocks)
// starts a few blocks away, and a character crossing such a step would clip through the block face.
// The routes below were checked against the model's block heights: every sample stays on y=35 ground
// with at least 0.5 blocks of clearance to the logs.

/** Where walkers enter the scene (xz). Both are off-screen or far away on the flat plateau. */
const ENTRANCE_S: [number, number] = [-16.5, 6.5]
const ENTRANCE_N: [number, number] = [-8.5, -6.4]

const VIA_S: [number, number][] = [[-16.0, 3.0], [-15.7, 1.6]]
const VIA_N: [number, number][] = [[-14.5, -6.4], [-14.5, -3.3]]

export interface Seat {
  /** Centre of the seat on top of the log (feet-level of a sitting hip). */
  pos: Vector3
  /** Horizontal direction the sitter faces (towards the fire). */
  facing: Vector3
  /** Walk-in route: entrance → via points → lane → stand spot. */
  path: Vector3[]
  /** Spot on the fire side of the log where the character turns around and sits down backwards. */
  stand: Vector3
  /** Where this character walks to when it gets up for a visit to the fire (see VISITS_ENABLED). */
  visit: {
    /** Waypoints between the lane (the point 2.2 blocks in front of the log) and the spot. */
    via: Vector3[]
    spot: Vector3
  }
}

function makeSeat(pos: [number, number], facing: [number, number], entrance: [number, number], via: [number, number][], standDist: number, laneDist: number,
  visit: { via: [number, number][]; spot: [number, number] }): Seat {
  const p = new Vector3(pos[0], SEAT_TOP, pos[1])
  const f = new Vector3(facing[0], 0, facing[1])
  const stand = p.clone().addScaledVector(f, standDist).setY(0)
  const lane = p.clone().addScaledVector(f, laneDist).setY(0)
  const v = (xz: [number, number]) => new Vector3(xz[0], 0, xz[1])
  return { pos: p, facing: f, stand, path: [v(entrance), ...via.map(v), lane, stand], visit: { via: visit.via.map(v), spot: v(visit.spot) } }
}

// Standing spots for visits (one per seat so two figures never share one). All of them are at least 1.2 blocks from
// the fire, and the routes (lane -> via -> spot) were checked against the glTF block heights like the seat routes:
// flat y=35 ground, >= 0.5 blocks from the logs, the fire and the lying dog (centre -15.3, -1.5, body along x).
// The dog's lying spot is why the column-log seats go round the north / south side of the fire, never along x=-15.7
// past z=-1.5.

// Seat 0 is always the local player; the others are filled first-come.
// Column log (x=-13.5, runs along z) faces the camera; row log (z=-5.5) sits behind the fire.
export const SEATS: Seat[] = [
  makeSeat([-13.5, -2.5], [-1, 0], ENTRANCE_S, VIA_S, 0.75, 2.2, { via: [[-15.7, -3.2]], spot: [-17.3, -3.2] }),
  makeSeat([-13.5, -0.5], [-1, 0], ENTRANCE_S, VIA_S, 0.75, 2.2, { via: [[-15.7, 0.4]], spot: [-17.4, 0.4] }),
  makeSeat([-16.5, -5.5], [0, 1], ENTRANCE_N, VIA_N, 0.75, 2.2, { via: [], spot: [-18.5, -3.3] }),
  makeSeat([-18.5, -5.5], [0, 1], ENTRANCE_N, VIA_N, 0.75, 2.2, { via: [[-19.3, -3.0]], spot: [-19.3, -1.5] }),
  makeSeat([-20.5, -5.5], [0, 1], ENTRANCE_N, VIA_N, 0.75, 2.2, { via: [[-20.5, 0.4]], spot: [-19.0, 0.4] }),
]

export const FOLIAGE_RE = /leaves|grass|bush|dandelion|bluet|daisy|cornflower|lichen/i

export interface SceneMember {
  uuid: string
  username: string
  skinUrl: string | null
  capeUrl: string | null
  model: 'default' | 'slim' | 'auto-detect'
  isLeader: boolean
  /** Player level badge in the nametag; omitted when unknown (party members until the backend shares it). */
  level?: number
  /** 0..1 progress to the next level, shown as a thin bar under the nametag (local player only). */
  levelProgress?: number
}

// The dog that visits the fire (CampfireDog.ts). Seconds of scene time, i.e. only while the hub is on screen.
export const DOG_FIRST_VISIT_S = 45     // first visit shortly after the scene opens, so it gets noticed
export const DOG_STAY_S = 300           // lies next to the player for 5 minutes
export const DOG_ABSENT_S = 300         // then stays away for 5 minutes

// Visits: now and then a seated character climbs down, walks to a standing spot at the fire, stands there for a while
// and climbs back onto the same seat (which stays reserved). Only one character visits at a time.
export const VISITS_ENABLED = true
export const VISIT_WAIT_MIN_S = 60      // sitting time before a visit (rolled per character, again after every visit)
export const VISIT_WAIT_MAX_S = 180
export const VISIT_STAND_MIN_S = 20     // time standing at the fire
export const VISIT_STAND_MAX_S = 40
export const VISIT_RETRY_S = 10         // when a visit is not allowed right now (someone else moving), try again after this

// The villager who now and then comes to trade with the local player (CampfireVillager.ts). Rare on purpose.
export const VILLAGER_ENABLED = true        // automatic visits; the manual test button works either way
export const VILLAGER_FIRST_MIN_S = 240     // first possible visit after the scene opens
export const VILLAGER_FIRST_MAX_S = 600
export const VILLAGER_WAIT_MIN_S = 900      // pause between visits (15 - 30 minutes of scene time)
export const VILLAGER_WAIT_MAX_S = 1800
export const VILLAGER_EXCHANGES_MIN = 3     // how many trade offers per visit
export const VILLAGER_EXCHANGES_MAX = 4

// Hub test actions for dev builds: Ctrl+<number> (or the buttons in the corner of the hub card) runs the action with
// that number. The order must match CampfireScene's runAction().
export const SCENE_ACTION_LABELS = [
  'Call villager',
  'Call dog',
  'Send dog away',
  'Visit (you)',
  'Visit (friend)',
  'Shooting star',
  'Test mail (adds a notification)',
]

/** Seasonal look of the scene. 'christmas' adds snow cover and falling snowflakes; 'default' is the plain night scene. */
export const THEME: 'default' | 'christmas' = 'christmas'

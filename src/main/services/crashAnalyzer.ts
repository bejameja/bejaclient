export interface CrashFinding {
  severity: 'fatal' | 'error' | 'warn'
  category: string
  raw: string
  humanReadable: string
  suggestion: string
}

interface CrashPattern {
  pattern: RegExp
  severity: CrashFinding['severity']
  category: string
  explain: (m: RegExpMatchArray) => { humanReadable: string; suggestion: string }
}

const PATTERNS: CrashPattern[] = [
  {
    pattern: /Mixin apply failed (.+?) -> (.+?)(?:\n|:)/,
    severity: 'fatal',
    category: 'Mixin Conflict',
    explain: m => ({
      humanReadable: `Mixin failed to apply to class "${m[2]}" from config "${m[1]}".`,
      suggestion: 'A mod is patching the same class as BejaClient or Sodium. Check for conflicting mods (OptiFine, Rubidium, Embeddium) and remove them.',
    }),
  },
  {
    // Only fire when class_NNN appears inside an actual exception/error line, not normal mixin-apply logs
    pattern: /(?:Exception|Error|FATAL|Caused by)[^\n]*class_(\d+)/,
    severity: 'error',
    category: 'Mapping Mismatch',
    explain: m => ({
      humanReadable: `Obfuscated class reference "class_${m[1]}" in an exception — likely a yarn/intermediary mapping mismatch.`,
      suggestion: 'Ensure the Fabric Loader version matches the yarn mappings version in build.gradle. Run `./gradlew --stop` then rebuild.',
    }),
  },
  {
    pattern: /NullPointerException[\s\S]{0,200}?GameOptions/,
    severity: 'fatal',
    category: 'GameOptions NPE',
    explain: () => ({
      humanReadable: "Minecraft's GameOptions failed to initialize — options.txt is likely corrupt.",
      suggestion: 'Delete %APPDATA%\\.minecraft\\options.txt and let Minecraft recreate it.',
    }),
  },
  {
    pattern: /Cannot find .+? in the target class (.+?) for injection/,
    severity: 'fatal',
    category: 'Mixin Target Missing',
    explain: m => ({
      humanReadable: `Mixin injection target missing in class "${m[1]}".`,
      suggestion: 'The Minecraft version changed and a method was renamed. Update yarn mappings and re-check Mixin @At selectors.',
    }),
  },
  {
    pattern: /net\.fabricmc\.loader requires .+ version .+ of (.+?),/,
    severity: 'fatal',
    category: 'Dependency Version',
    explain: m => ({
      humanReadable: `Required dependency "${m[1]}" has an incompatible version.`,
      suggestion: `Update or remove "${m[1]}" to match the version required by Fabric Loader / Sodium.`,
    }),
  },
  {
    pattern: /java\.lang\.StackOverflowError/,
    severity: 'fatal',
    category: 'StackOverflow (Mixin Loop)',
    explain: () => ({
      humanReadable: 'Stack overflow — almost always caused by a Mixin that calls itself recursively.',
      suggestion: 'Check for @Inject methods that call the original method and trigger the same injection again. Use @Unique helpers or @Redirect instead.',
    }),
  },
  {
    pattern: /EXCEPTION_ACCESS_VIOLATION|opengl error|GL_INVALID_OPERATION/i,
    severity: 'fatal',
    category: 'GPU / OpenGL Crash',
    explain: () => ({
      humanReadable: 'OpenGL or GPU driver error.',
      suggestion: 'Update GPU drivers. If using Sodium, check that no conflicting renderer mods (OptiFabric, VulkanMod) are in mods/.',
    }),
  },
  {
    pattern: /Bootstrap JAR not found|beja-bootstrap.*?not found/i,
    severity: 'warn',
    category: 'BejaClient Bootstrap Missing',
    explain: () => ({
      humanReadable: 'BejaClient bootstrap JAR could not be located.',
      suggestion: 'Trigger a re-download in the launcher or verify %APPDATA%\\BejaClient\\beja-libs\\ contains beja-bootstrap-*.jar.',
    }),
  },
]

const ORDER: Record<CrashFinding['severity'], number> = { fatal: 0, error: 1, warn: 2 }

export function analyzeCrashLog(log: string): CrashFinding[] {
  const findings: CrashFinding[] = []
  const seen = new Set<string>()

  for (const { pattern, severity, category, explain } of PATTERNS) {
    const m = log.match(pattern)
    if (!m) continue
    const key = `${category}:${m[0].slice(0, 60)}`
    if (seen.has(key)) continue
    seen.add(key)
    const { humanReadable, suggestion } = explain(m)
    findings.push({ severity, category, raw: m[0].trim(), humanReadable, suggestion })
  }

  return findings.sort((a, b) => ORDER[a.severity] - ORDER[b.severity])
}

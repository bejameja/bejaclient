/**
 * electron-builder afterPack hook.
 * Runs AFTER the app is unpacked but BEFORE the installer is created,
 * so the icon gets embedded into BejaClient.exe before NSIS wraps it.
 *
 * We use the standalone `rcedit` npm package instead of electron-builder's
 * built-in winCodeSign bundle, which triggers an EPERM symlink error on
 * Windows machines without Developer Mode enabled.
 */
const path = require('path')
const { rcedit } = require('rcedit')

module.exports.afterPack = async (context) => {
  if (context.electronPlatformName !== 'win32') return

  const productName = context.packager.appInfo.productName  // "BejaClient"
  const version     = context.packager.appInfo.version       // "1.0.0"
  const exePath     = path.join(context.appOutDir, `${productName}.exe`)
  const iconPath    = path.resolve(__dirname, 'resources', 'icon.ico')

  console.log(`[afterPack] Embedding icon into ${exePath}`)

  await rcedit(exePath, {
    icon: iconPath,
    'version-string': {
      ProductName:      productName,
      FileDescription:  'BejaClient - Minecraft Launcher',
      OriginalFilename: `${productName}.exe`,
      CompanyName:      'BejaClient',
      LegalCopyright:   `Copyright (c) 2026 BejaClient`,
    },
    'product-version': `${version}.0`,
    'file-version':    `${version}.0`,
  })

  console.log(`[afterPack] ✓ Icon and version metadata embedded successfully`)
}
